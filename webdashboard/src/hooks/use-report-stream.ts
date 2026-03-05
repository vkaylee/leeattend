"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import type {
  StreamMessage,
  StreamStatus,
  ChunkMessage,
} from "@/types/report-stream";
import {
  getAccessToken,
  getRefreshToken,
  setAccessToken,
  setRefreshToken,
  clearAuthSession,
} from "@/lib/api/helpers";
import { API_BASE_URL } from "@/lib/api/config";

interface UseReportStreamOptions {
  jobId: string | null;
  onComplete?: (data: unknown[]) => void;
  onError?: (message: string) => void;
  maxRetries?: number;
}

interface UseReportStreamResult {
  status: StreamStatus;
  progress: number; // 0-100
  chunks: unknown[];
  error: string | null;
  reconnect: () => void;
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
      const data = await response.json();
      setAccessToken(data.access_token);
      setRefreshToken(data.refresh_token);
      console.debug("[ReportStream] Token refreshed successfully");
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
  const chunksRef = useRef<unknown[]>([]);
  const retryCountRef = useRef(0);
  const shouldReconnectRef = useRef(true);
  const totalBatchesRef = useRef(0);
  const statusRef = useRef<StreamStatus>("idle");

  // Store callbacks in refs to avoid dependency issues
  const onCompleteRef = useRef(onComplete);
  const onErrorRef = useRef(onError);
  onCompleteRef.current = onComplete;
  onErrorRef.current = onError;
  // Keep statusRef in sync so onclose handler reads latest value
  statusRef.current = status;

  /**
   * Handle incoming WebSocket message
   */
  const handleMessage = useCallback((event: MessageEvent) => {
    try {
      const message: StreamMessage = JSON.parse(event.data);

      switch (message.type) {
        case "CHUNK": {
          const chunkMsg = message as ChunkMessage;

          // Update progress (data is no longer sent in CHUNK messages)
          totalBatchesRef.current = chunkMsg.total_batches;
          const newProgress =
            ((chunkMsg.batch_index + 1) / chunkMsg.total_batches) * 100;
          setProgress(Math.min(newProgress, 100));
          setStatus("streaming");

          console.debug(
            `[ReportStream] Received chunk ${chunkMsg.batch_index + 1}/${chunkMsg.total_batches}`
          );
          break;
        }

        case "DONE": {
          setStatus("completed");
          setProgress(100);
          shouldReconnectRef.current = false;
          // Update chunks state for consumers, then clear ref to avoid double-buffering
          const finalChunks = [...chunksRef.current];
          chunksRef.current = [];
          setChunks(finalChunks);
          console.debug(
            `[ReportStream] Complete: ${message.total_records} records`
          );
          onCompleteRef.current?.(finalChunks);
          break;
        }

        case "ERROR": {
          setStatus("error");
          setError(message.message);
          shouldReconnectRef.current = false;
          console.debug(`[ReportStream] Error: ${message.message}`);
          onErrorRef.current?.(message.message);
          break;
        }
      }
    } catch (err) {
      console.error("[ReportStream] Failed to parse message:", err);
    }
  }, []);

  /**
   * Connect to WebSocket
   */
  const connect = useCallback(() => {
    if (!jobId) return;

    // Clean up existing connection
    if (socketRef.current) {
      socketRef.current.close();
      socketRef.current = null;
    }

    const url = buildWebSocketUrl(jobId);
    console.debug(`[ReportStream] Connecting to ${url}`);

    setStatus("connecting");
    setError(null);

    try {
      const socket = new WebSocket(url);
      socketRef.current = socket;

      socket.onopen = () => {
        console.debug("[ReportStream] Connected");
        setStatus("streaming");
        retryCountRef.current = 0; // Reset retry count on success
      };

      socket.onmessage = handleMessage;

      socket.onerror = (event) => {
        console.debug("[ReportStream] WebSocket error:", event);
      };

      socket.onclose = (event) => {
        console.debug(
          `[ReportStream] Connection closed: code=${event.code}, reason=${event.reason}`
        );

        // Handle auth failure: code 1006 on first attempt = HTTP 401 (upgrade rejected)
        // Browser doesn't expose HTTP status on WS upgrade failure, but 1006 = abnormal close
        if (
          event.code === 1006 &&
          retryCountRef.current === 0 &&
          statusRef.current === "connecting"
        ) {
          console.debug("[ReportStream] Possible auth failure, attempting token refresh");
          retryCountRef.current++;
          tryRefreshToken().then((refreshed) => {
            if (refreshed && shouldReconnectRef.current) {
              console.debug("[ReportStream] Token refreshed, reconnecting");
              // eslint-disable-next-line react-hooks/immutability
              connect();
            } else {
              // Refresh failed — session expired, redirect to login
              console.warn("[ReportStream] Session expired, redirecting to login");
              clearAuthSession();
              if (window.location.pathname !== "/login") {
                window.location.href = "/login";
              }
            }
          });
          return;
        }

        // Only reconnect if not intentionally closed and not completed/errored
        if (
          shouldReconnectRef.current &&
          retryCountRef.current < maxRetries &&
          statusRef.current !== "completed" &&
          statusRef.current !== "error"
        ) {
          const delay = getBackoffDelay(retryCountRef.current);
          console.debug(
            `[ReportStream] Reconnecting in ${delay}ms (attempt ${retryCountRef.current + 1}/${maxRetries})`
          );

          retryCountRef.current++;
          setTimeout(() => {
            if (shouldReconnectRef.current) {
              connect();
            }
          }, delay);
        } else if (
          retryCountRef.current >= maxRetries &&
          statusRef.current !== "completed"
        ) {
          setStatus("error");
          setError("Connection lost. Max retries exceeded.");
          onErrorRef.current?.("Connection lost. Max retries exceeded.");
        }
      };
    } catch (err) {
      console.error("[ReportStream] Failed to create WebSocket:", err);
      setStatus("error");
      setError("Failed to establish connection");
    }
  }, [jobId, handleMessage, maxRetries]);

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

  // Connect when jobId changes
  useEffect(() => {
    if (!jobId) {
      // Reset state when jobId is cleared
      setStatus("idle");
      setProgress(0);
      setError(null);
      setChunks([]);
      chunksRef.current = [];
      retryCountRef.current = 0;
      return;
    }

    // Reset for new job
    chunksRef.current = [];
    setChunks([]);
    retryCountRef.current = 0;
    shouldReconnectRef.current = true;
    totalBatchesRef.current = 0;

    connect();

    // Cleanup on unmount or jobId change
    return () => {
      shouldReconnectRef.current = false;
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
