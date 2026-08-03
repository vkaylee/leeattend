import { useCallback, useEffect, useRef } from "react";

import type { StreamStatus } from "@/types/report-stream";

import { clearAuthSession } from "@/lib/api/helpers";
import { ROUTES } from "@/lib/routes";
import { maybeNull } from "@/types/api-overrides";

import { useReconnectionBackoff } from "./use-reconnection-backoff";
import {
  buildWebSocketUrl,
  tryRefreshToken,
} from "./use-report-stream-handlers";
import { reduceSocketState } from "./use-report-stream-reducer";

// ── Types ─────────────────────────────────────────────────────────────────

interface UseReportStreamSocketProps {
  chunksRef: React.RefObject<unknown[]>;
  handleMessage: (event: MessageEvent) => void;
  jobId: string | null;
  maxRetries: number;
  onErrorRef: React.RefObject<((message: string) => void) | undefined>;
  retryCountRef: React.RefObject<number>;
  setChunks: (chunks: unknown[]) => void;
  setError: (error: string | null) => void;
  setProgress: (progress: number) => void;
  setStatus: (status: StreamStatus) => void;
  shouldReconnectRef: React.RefObject<boolean>;
  statusRef: React.RefObject<StreamStatus>;
  totalBatchesRef: React.RefObject<number>;
}

// ── Hook ──────────────────────────────────────────────────────────────────

/**
 * Manages the WebSocket lifecycle for report streaming: connect, disconnect,
 * ping keep-alive and auth-error recovery. Close-event decisions are delegated
 * to the pure `reduceSocketState` machine; reconnection backoff is delegated
 * to `useReconnectionBackoff`. Cleanup (close + abort) runs on unmount.
 */
export function useReportStreamSocket({
  jobId,
  maxRetries,
  setStatus,
  setError,
  setChunks,
  setProgress,
  handleMessage,
  onErrorRef,
  shouldReconnectRef,
  statusRef,
  retryCountRef,
  chunksRef,
  totalBatchesRef,
}: UseReportStreamSocketProps) {
  const socketRef = useRef<WebSocket | null>(null);
  const pingIntervalRef = useRef<NodeJS.Timeout | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  const connectRef = useRef<() => void>(() => {});

  // ── Auth-error recovery ──────────────────────────────────────────────

  const handleAuthError = useCallback(async () => {
    retryCountRef.current++;
    try {
      const refreshed = await tryRefreshToken();
      if (refreshed && shouldReconnectRef.current) {
        connectRef.current();
      } else {
        clearAuthSession();
        if (window.location.pathname !== ROUTES.LOGIN) window.location.href = ROUTES.LOGIN;
      }
    } catch {
      clearAuthSession();
      if (window.location.pathname !== ROUTES.LOGIN) window.location.href = ROUTES.LOGIN;
    }
  }, [retryCountRef, shouldReconnectRef]);

  // ── Reconnection scheduling ──────────────────────────────────────────

  const handleMaxRetriesExceeded = useCallback(() => {
    setStatus("error");
    setError("Connection lost. Max retries exceeded.");
    onErrorRef.current?.("Connection lost. Max retries exceeded.");
  }, [onErrorRef, setError, setStatus]);

  const attemptReconnect = useCallback(() => {
    connectRef.current();
  }, []);

  const { scheduleReconnect, reset } = useReconnectionBackoff({
    retryCountRef,
    shouldReconnectRef,
    onReconnect: attemptReconnect,
  });

  const handleClose = useCallback((event: CloseEvent, connectionTimeoutId: NodeJS.Timeout) => {
    clearTimeout(connectionTimeoutId);
    const decision = reduceSocketState(
      {
        status: statusRef.current,
        retryCount: retryCountRef.current,
        shouldReconnect: shouldReconnectRef.current,
        maxRetries,
      },
      { type: "close", code: event.code },
    );
    switch (decision.type) {
      case "auth_error":
        void handleAuthError();
        break;
      case "reconnect":
        scheduleReconnect();
        break;
      case "give_up":
        handleMaxRetriesExceeded();
        break;
      case "ignore":
        break;
    }
  }, [handleAuthError, handleMaxRetriesExceeded, maxRetries, retryCountRef, scheduleReconnect, shouldReconnectRef, statusRef]);

  // ── Connect ──────────────────────────────────────────────────────────

  const connect = useCallback(() => {
    if (!jobId) return;
    if (socketRef.current) {
      socketRef.current.close();
      socketRef.current = null;
    }
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
        reset();
        pingIntervalRef.current = setInterval(() => {
          if (socket.readyState === WebSocket.OPEN) socket.send(JSON.stringify({ type: "ping" }));
        }, 30000);
      };
      socket.onmessage = handleMessage;
      socket.onclose = (event) => {
        handleClose(event, connectionTimeoutId);
      };
    } catch {
      setStatus("error");
      setError("Failed to establish connection");
    }
  }, [handleClose, handleMessage, jobId, reset, setError, setStatus]);

  useEffect(() => {
    connectRef.current = connect;
  }, [connect]);

  // ── Disconnect ───────────────────────────────────────────────────────

  const disconnect = useCallback(() => {
    shouldReconnectRef.current = false;
    if (pingIntervalRef.current) {
      clearInterval(pingIntervalRef.current);
      pingIntervalRef.current = null;
    }
    if (socketRef.current) {
      socketRef.current.close();
      socketRef.current = null;
    }
  }, [shouldReconnectRef]);

  // ── Reconnect (manual) ───────────────────────────────────────────────

  const reconnect = useCallback(() => {
    reset();
    shouldReconnectRef.current = true;
    chunksRef.current = [];
    setChunks([]);
    setProgress(0);
    setError(null);
    connect();
  }, [chunksRef, connect, reset, setChunks, setError, setProgress, shouldReconnectRef]);

  // ── Lifecycle: connect on mount / jobId change, cleanup on unmount ───

  useEffect(() => {
    chunksRef.current = [];
    retryCountRef.current = 0;
    shouldReconnectRef.current = true;
    totalBatchesRef.current = 0;

    if (!jobId) return;

    if (maybeNull(shouldReconnectRef.current)) {
      connect();
    }

    return () => {
      disconnect();
    };
  }, [chunksRef, connect, disconnect, jobId, retryCountRef, shouldReconnectRef, totalBatchesRef]);

  return { connect, disconnect, reconnect };
}
