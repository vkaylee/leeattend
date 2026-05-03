"use client";

import { useEffect, useRef, useState, useCallback } from "react";

import type {
  StreamMessage,
  StreamStatus,
} from "@/types/report-stream";

import { API_BASE_URL } from "@/lib/api/config";
import {
  getAccessToken,
  getRefreshToken,
  setAccessToken,
  setRefreshToken,
  clearAuthSession,
} from "@/lib/api/helpers";
import { ROUTES } from "@/lib/routes";

interface UseReportStreamOptions {
  jobId: string | null;
  maxRetries?: number;
  onComplete?: (data: unknown[]) => void;
  onError?: (message: string) => void;
}

export interface UseReportStreamResult {
  chunks: unknown[];
  error: string | null;
  progress: number; // 0-100
  reconnect: () => void;
  status: StreamStatus;
}

/**
 * Build WebSocket URL for report streaming
 * Uses wss: for HTTPS, ws: for HTTP
 * Includes JWT token as query param (required by ws_auth_middleware)
 */
function buildWebSocketUrl(jobId: string): string {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const host = window.location.host;
  const token = getAccessToken();
  const params = token ? `?token=${encodeURIComponent(token)}` : "";
  return `${protocol}//${host}/api/ws/reports/${jobId}${params}`;
}

/**
 * Calculate exponential backoff delay
 * Returns delays: 1s, 2s, 4s for retries 0, 1, 2
 */
function getBackoffDelay(retryCount: number): number {
  return Math.pow(2, retryCount) * 1000;
}

/**
 * Attempt to refresh JWT token (same logic as apiFetch 401 handler)
 * Returns true if refresh succeeded, false otherwise
 */
async function tryRefreshToken(): Promise<boolean> {
  const refreshToken = getRefreshToken();
  if (!refreshToken) return false;

  try {
    const response = await fetch(`${API_BASE_URL}/auth/refresh`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ refresh_token: refreshToken }),
    });

    if (response.ok) {
      const data = await response.json() as { access_token: string; refresh_token: string };
      setAccessToken(data.access_token);
      setRefreshToken(data.refresh_token);
      
      return true;
    }
  } catch (e) {
    console.error("[ReportStream] Token refresh failed", e);
  }

  return false;
}

/**
 * Hook for WebSocket-based report streaming
 * Connects to the report WebSocket endpoint and accumulates chunks
 */
export function useReportStream({
  jobId,
  onComplete,
  onError,
  maxRetries = 3,
}: UseReportStreamOptions): UseReportStreamResult {
  const [status, setStatus] = useState<StreamStatus>("idle");
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  // Expose chunks via state for consumers that need reactivity
  const [chunks, setChunks] = useState<unknown[]>([]);

  // Use refs to avoid re-renders on each chunk and prevent stale closures
  const socketRef = useRef<WebSocket | null>(null);
  const pingIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const chunksRef = useRef<unknown[]>([]);
  const retryCountRef = useRef(0);
  const shouldReconnectRef = useRef(true);
  const totalBatchesRef = useRef(0);
  const statusRef = useRef<StreamStatus>("idle");

  const onCompleteRef = useRef(onComplete);
  const onErrorRef = useRef(onError);
  
  useEffect(() => {
    onCompleteRef.current = onComplete;
    onErrorRef.current = onError;
    statusRef.current = status;
  });

  const handleMessage = useCallback((event: MessageEvent) => {
    try {
      const message = JSON.parse(event.data as string) as StreamMessage;
      switch (message.type) {
        case "CHUNK": {
          totalBatchesRef.current = message.total_batches;
          const newProgress = ((message.batch_index + 1) / message.total_batches) * 100;
          setProgress(Math.min(newProgress, 100));
          setStatus("streaming");
          break;
        }
        case "DONE": {
          setStatus("completed");
          setProgress(100);
          shouldReconnectRef.current = false;
          const finalChunks = [...chunksRef.current];
          chunksRef.current = [];
          setChunks(finalChunks);
          onCompleteRef.current?.(finalChunks);
          break;
        }
        case "ERROR": {
          setStatus("error");
          setError(message.message);
          shouldReconnectRef.current = false;
          onErrorRef.current?.(message.message);
          break;
        }
      }
    } catch (error_) {
      console.error("[ReportStream] Failed to parse message:", error_);
    }
  }, []);

  /**
   * Connect to WebSocket
   */
  const connectRef = useRef<() => void>(() => {});

  const connect = useCallback(() => {
    if (!jobId) return;
    if (socketRef.current) { socketRef.current.close(); socketRef.current = null; }
    const url = buildWebSocketUrl(jobId);
    setStatus("connecting");
    setError(null);
    try {
      const socket = new WebSocket(url);
      socketRef.current = socket;
      const connectionTimeoutId = setTimeout(() => {
        if (socket.readyState === WebSocket.CONNECTING) socket.close();
      }, 3000);
      socket.onopen = () => {
        clearTimeout(connectionTimeoutId);
        setStatus("streaming");
        retryCountRef.current = 0;
        pingIntervalRef.current = setInterval(() => {
          if (socket.readyState === WebSocket.OPEN) socket.send(JSON.stringify({ type: "ping" }));
        }, 30000);
      };
      socket.onmessage = handleMessage;
      socket.onclose = (event) => {
        clearTimeout(connectionTimeoutId);
        if (event.code === 1006 && retryCountRef.current === 0 && statusRef.current === "connecting") {
          retryCountRef.current++;
          void tryRefreshToken().then((ref) => {
            if (ref && shouldReconnectRef.current) connectRef.current();
            else { clearAuthSession(); if (window.location.pathname !== ROUTES.LOGIN) window.location.href = ROUTES.LOGIN; }
          });
          return;
        }
        if (shouldReconnectRef.current && retryCountRef.current < maxRetries && statusRef.current !== "completed" && statusRef.current !== "error") {
          const delay = getBackoffDelay(retryCountRef.current);
          retryCountRef.current++;
          setTimeout(() => { if (shouldReconnectRef.current) connectRef.current(); }, delay);
        } else if (retryCountRef.current >= maxRetries && statusRef.current !== "completed") {
          setStatus("error");
          setError("Connection lost. Max retries exceeded.");
          onErrorRef.current?.("Connection lost. Max retries exceeded.");
        }
      };
    } catch (err) { setStatus("error"); setError("Failed to establish connection"); }
  }, [jobId, handleMessage, maxRetries]);

  useEffect(() => {
    connectRef.current = connect;
  }, [connect]);

  /**
   * Manual reconnect function exposed to consumers
   */
  const reconnect = useCallback(() => {
    retryCountRef.current = 0;
    shouldReconnectRef.current = true;
    chunksRef.current = [];
    setChunks([]);
    setProgress(0);
    setError(null);
    connect();
  }, [connect]);

  const [previousJobId, setPreviousJobId] = useState(jobId);
  if (jobId !== previousJobId) {
    setPreviousJobId(jobId);
    if (!jobId) {
      setStatus("idle");
      setProgress(0);
      setError(null);
      setChunks([]);
    } else {
      setChunks([]);
    }
  }

  // Connect when jobId changes
  useEffect(() => {
    chunksRef.current = [];
    retryCountRef.current = 0;
    shouldReconnectRef.current = true;
    totalBatchesRef.current = 0;

    if (!jobId) return;

    void Promise.resolve().then(() => {
      if (shouldReconnectRef.current) {
        connect();
      }
    });

    // Cleanup on unmount or jobId change
    return () => {
      shouldReconnectRef.current = false;
      if (pingIntervalRef.current) {
        clearInterval(pingIntervalRef.current);
        pingIntervalRef.current = null;
      }
      if (socketRef.current) {
        socketRef.current.close();
        socketRef.current = null;
      }
    };
  }, [jobId, connect]);

  return {
    status,
    progress,
    chunks,
    error,
    reconnect,
  };
}
