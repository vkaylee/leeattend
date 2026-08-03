import { useEffect, useRef, useCallback } from "react";

import { clearAuthSession } from "@/lib/api/helpers";
import { ROUTES } from "@/lib/routes";
import { maybeNull } from "@/types/api-overrides";

import {
  buildWebSocketUrl,
  getBackoffDelay,
  tryRefreshToken,
} from "./use-report-stream-handlers";

// ── Types ─────────────────────────────────────────────────────────────────

type StreamStatus = "idle" | "connecting" | "streaming" | "completed" | "error";

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
 * Single-responsibility hook: manages the full WebSocket lifecycle for
 * report streaming — connect, disconnect, reconnect, ping keep-alive,
 * auth-error recovery, exponential backoff, and max-retry enforcement.
 *
 * Cleanup (close + abort) is performed in the useEffect return.
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
      const ref = await tryRefreshToken();
      if (ref && shouldReconnectRef.current) connectRef.current();
      else {
        clearAuthSession();
        if (window.location.pathname !== ROUTES.LOGIN) window.location.href = ROUTES.LOGIN;
      }
    } catch {
      clearAuthSession();
      if (window.location.pathname !== ROUTES.LOGIN) window.location.href = ROUTES.LOGIN;
    }
  }, [retryCountRef, shouldReconnectRef]);

  // ── Reconnection scheduling ──────────────────────────────────────────

  const scheduleReconnect = useCallback(() => {
    const delay = getBackoffDelay(retryCountRef.current);
    retryCountRef.current += 1;
    setTimeout(() => { if (shouldReconnectRef.current) connectRef.current(); }, delay);
  }, [retryCountRef, shouldReconnectRef]);

  const handleMaxRetriesExceeded = useCallback(() => {
    setStatus("error");
    setError("Connection lost. Max retries exceeded.");
    onErrorRef.current?.("Connection lost. Max retries exceeded.");
  }, [setError, setStatus, onErrorRef]);

  // ── Close handler ────────────────────────────────────────────────────

  const handleClose = useCallback((event: CloseEvent, connectionTimeoutId: NodeJS.Timeout) => {
    clearTimeout(connectionTimeoutId);

    const isAuthErr = event.code === 1006 && retryCountRef.current === 0 && statusRef.current === "connecting";
    if (isAuthErr) {
      void handleAuthError();
      return;
    }

    const isTerminal = statusRef.current === "completed" || statusRef.current === "error";
    if (isTerminal) return;

    if (shouldReconnectRef.current && retryCountRef.current < maxRetries) {
      scheduleReconnect();
    } else if (retryCountRef.current >= maxRetries) {
      handleMaxRetriesExceeded();
    }
  }, [maxRetries, retryCountRef, shouldReconnectRef, statusRef, handleAuthError, scheduleReconnect, handleMaxRetriesExceeded]);

  // ── Connect ──────────────────────────────────────────────────────────

  const connect = useCallback(() => {
    if (!jobId) return;
    if (socketRef.current) { socketRef.current.close(); socketRef.current = null; }
    const url = buildWebSocketUrl(jobId);
    setStatus("connecting"); setError(null);
    try {
      const socket = new WebSocket(url);
      socketRef.current = socket;
      const connectionTimeoutId = setTimeout(() => {
        if (socket.readyState === WebSocket.CONNECTING) socket.close();
      }, 3000);
      socket.onopen = () => {
        clearTimeout(connectionTimeoutId); setStatus("streaming"); retryCountRef.current = 0;
        pingIntervalRef.current = setInterval(() => {
          if (socket.readyState === WebSocket.OPEN) socket.send(JSON.stringify({ type: "ping" }));
        }, 30000);
      };
      socket.onmessage = handleMessage;
      socket.onclose = (event) => { handleClose(event, connectionTimeoutId); };
    } catch { setStatus("error"); setError("Failed to establish connection"); }
  }, [jobId, handleMessage, setError, setStatus, retryCountRef, handleClose]);

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
    retryCountRef.current = 0;
    shouldReconnectRef.current = true;
    chunksRef.current = [];
    setChunks([]);
    setProgress(0);
    setError(null);
    connect();
  }, [connect, chunksRef, setChunks, setError, setProgress, retryCountRef, shouldReconnectRef]);

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
  }, [jobId, connect, disconnect, chunksRef, retryCountRef, shouldReconnectRef, totalBatchesRef]);

  return { connect, disconnect, reconnect };
}
