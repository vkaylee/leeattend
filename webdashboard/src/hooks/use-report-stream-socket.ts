import { useEffect, useRef, useCallback } from "react";
import { clearAuthSession } from "@/lib/api/helpers";
import { ROUTES } from "@/lib/routes";

import {
  buildWebSocketUrl,
  getBackoffDelay,
  tryRefreshToken,
} from "./use-report-stream-utils";

interface UseReportStreamSocketProps {
  jobId: string | null;
  maxRetries: number;
  setStatus: (status: "idle" | "connecting" | "streaming" | "completed" | "error") => void;
  setError: (error: string | null) => void;
  handleMessage: (event: MessageEvent) => void;
  onErrorRef: React.MutableRefObject<((message: string) => void) | undefined>;
  shouldReconnectRef: React.MutableRefObject<boolean>;
  statusRef: React.MutableRefObject<"idle" | "connecting" | "streaming" | "completed" | "error">;
  retryCountRef: React.MutableRefObject<number>;
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
          setStatus("error"); setError("Connection lost. Max retries exceeded."); onErrorRef.current?.("Connection lost. Max retries exceeded.");
        }
      };
    } catch { setStatus("error"); setError("Failed to establish connection"); }
  }, [jobId, handleMessage, maxRetries, setError, setStatus, onErrorRef, retryCountRef, shouldReconnectRef, statusRef]);

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
