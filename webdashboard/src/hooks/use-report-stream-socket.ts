import { useEffect, useRef, useCallback } from "react";

import { clearAuthSession } from "@/lib/api/helpers";
import { ROUTES } from "@/lib/routes";

import {
  buildWebSocketUrl,
  getBackoffDelay,
  tryRefreshToken,
} from "./use-report-stream-utils";

interface UseReportStreamSocketProps {
  handleMessage: (event: MessageEvent) => void;
  jobId: string | null;
  maxRetries: number;
  onErrorRef: React.RefObject<((message: string) => void) | undefined>;
  retryCountRef: React.RefObject<number>;
  setError: (error: string | null) => void;
  setStatus: (status: "idle" | "connecting" | "streaming" | "completed" | "error") => void;
  shouldReconnectRef: React.RefObject<boolean>;
  statusRef: React.RefObject<"idle" | "connecting" | "streaming" | "completed" | "error">;
}

export function useReportStreamSocket({
  jobId,
  maxRetries,
  setStatus,
  setError,
  handleMessage,
  onErrorRef,
  shouldReconnectRef,
  statusRef,
  retryCountRef,
}: UseReportStreamSocketProps) {
  const socketRef = useRef<WebSocket | null>(null);
  const pingIntervalRef = useRef<NodeJS.Timeout | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  const connectRef = useRef<() => void>(() => {});

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

  const handleClose = useCallback((event: CloseEvent, connectionTimeoutId: NodeJS.Timeout) => {
    clearTimeout(connectionTimeoutId);
    
    const isAuthErr = event.code === 1006 && retryCountRef.current === 0 && statusRef.current === "connecting";
    if (isAuthErr) {
      handleAuthError();
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

  return { connect, disconnect };
}
