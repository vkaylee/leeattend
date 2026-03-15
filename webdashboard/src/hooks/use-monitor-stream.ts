import { useEffect, useRef, useState, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type { WorkerMonitorStats } from "@/lib/api/types";
import { monitorKeys } from "./use-worker-monitor";

/** SSE base URL – mirrors the REST API_BASE_URL logic. */
const SSE_URL = (import.meta.env.DEV ? "/api" : (import.meta.env.VITE_API_URL?.replace(/\/$/, "") || "/api")) + "/monitor/stream";

/** Max consecutive connection errors before giving up (resets on success). */
const MAX_RETRIES = 5;

/** Initial retry delay in ms; doubles on each failure (capped at 30 s). */
const INITIAL_RETRY_DELAY_MS = 1_000;

/**
 * React hook that subscribes to the `/monitor/stream` SSE endpoint.
 *
 * On each `monitor-update` event the TanStack-Query cache for the
 * monitor overview is replaced in-place, so every component using
 * `useWorkerMonitor()` re-renders instantly – no polling needed.
 *
 * Falls back gracefully: if the SSE connection fails or the browser
 * doesn't support `EventSource`, `useWorkerMonitor` keeps its own
 * 2 s polling interval as a safety net.
 */
export function useMonitorStream() {
  const queryClient = useQueryClient();
  const esRef = useRef<EventSource | null>(null);
  const retryCount = useRef(0);
  const retryTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [connected, setConnected] = useState(false);

  const connect = useCallback(() => {
    function doConnect() {
      // Avoid duplicate connections
      if (esRef.current) return;

      const es = new EventSource(SSE_URL);
      esRef.current = es;

      es.addEventListener("monitor-update", (e: MessageEvent) => {
        try {
          const data: WorkerMonitorStats = JSON.parse(e.data);
          queryClient.setQueryData(monitorKeys.overview(), data);
          setConnected(true);
          retryCount.current = 0;           // reset on success
        } catch {
          // Malformed payload – ignore, wait for next tick
        }
      });

      es.addEventListener("monitor-error", (e: MessageEvent) => {
        console.warn("[SSE] server error event:", e.data);
      });

      es.onerror = () => {
        // EventSource fires "error" on connection loss; it auto-reconnects
        // after ~3 s by spec, but we layer our own retry logic on top to
        // cap attempts and avoid hammering a down server.
        setConnected(false);
        es.close();
        esRef.current = null;

        if (retryCount.current < MAX_RETRIES) {
          const delay = Math.min(
            INITIAL_RETRY_DELAY_MS * 2 ** retryCount.current,
            30_000,
          );
          retryCount.current += 1;
          retryTimer.current = setTimeout(doConnect, delay);
        }
      };
    }

    doConnect();
  }, [queryClient]);

  useEffect(() => {
    connect();

    return () => {
      esRef.current?.close();
      esRef.current = null;
      if (retryTimer.current) clearTimeout(retryTimer.current);
    };
  }, [connect]);

  return { connected };
}
