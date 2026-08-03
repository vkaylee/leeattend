import { useCallback, useEffect, useRef } from "react";

import { getBackoffDelay } from "./use-report-stream-handlers";

interface UseReconnectionBackoffOptions {
  onReconnect: () => void;
  retryCountRef: React.RefObject<number>;
  shouldReconnectRef: React.RefObject<boolean>;
}

/**
 * Owns the exponential-backoff side of WebSocket reconnection:
 * delay computation, retry counting, timer scheduling and cleanup.
 * The decision to reconnect is made by `reduceSocketState`; this hook
 * executes it.
 */
export function useReconnectionBackoff({
  retryCountRef,
  shouldReconnectRef,
  onReconnect,
}: UseReconnectionBackoffOptions) {
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const scheduleReconnect = useCallback(() => {
    const delay = getBackoffDelay(retryCountRef.current);
    retryCountRef.current += 1;
    timerRef.current = setTimeout(() => {
      if (shouldReconnectRef.current) onReconnect();
    }, delay);
  }, [onReconnect, retryCountRef, shouldReconnectRef]);

  const reset = useCallback(() => {
    retryCountRef.current = 0;
  }, [retryCountRef]);

  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  return { scheduleReconnect, reset };
}
