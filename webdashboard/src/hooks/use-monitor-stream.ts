import type { QueryClient } from "@tanstack/react-query";
import { useQueryClient } from "@tanstack/react-query";
import type { MutableRefObject } from "react";
import { useCallback, useEffect, useRef, useState } from "react";

import type { WorkerMonitorStats } from "@/lib/api/types";
import { logError } from "@/lib/logger";

import { monitorKeys } from "./use-worker-monitor";
import {
  compareSnapshotSequence,
  isMonitorSnapshot,
  readMonitorErrorCode,
} from "./use-monitor-stream-validation";

/** SSE base URL – mirrors the REST API_BASE_URL logic. */
const SSE_BASE = import.meta.env.DEV
  ? "/api"
  : ((import.meta.env.VITE_API_URL as string | undefined)?.replace(/\/$/, "") ?? "/api");
const SSE_URL = (SSE_BASE.endsWith("/v1") ? SSE_BASE : `${SSE_BASE}/v1`) + "/monitor/stream";

const MAX_RETRIES = 5;
const INITIAL_RETRY_DELAY_MS = 1_000;
const MAX_RETRY_DELAY_MS = 30_000;
const STALE_AFTER_MS = 15_000;
const STALE_CHECK_INTERVAL_MS = 5_000;

export type MonitorStreamStatus = "connecting" | "live" | "degraded" | "stale" | "error";

interface StreamState {
  queryClient: QueryClient;
  lastEventAt: MutableRefObject<number | null>;
  lastSequence: MutableRefObject<string | null>;
  retryCount: MutableRefObject<number>;
  setErrorCode: (value: string | null) => void;
  setIsFresh: (value: boolean) => void;
  setLastUpdateAt: (value: number | null) => void;
  setStatus: (value: MonitorStreamStatus) => void;
}

function markStreamIssue(
  state: StreamState,
  status: MonitorStreamStatus,
  errorCode: string,
): void {
  state.setStatus(status);
  state.setIsFresh(false);
  state.setErrorCode(errorCode);
}

function isSnapshotFresh(snapshot: WorkerMonitorStats): boolean {
  return snapshot.snapshot_age_seconds <= STALE_AFTER_MS / 1_000 && snapshot.quality.freshness === "fresh";
}

function handleMonitorUpdate(
  event: MessageEvent,
  state: StreamState,
): void {
  try {
    const parsed: unknown = JSON.parse(event.data as string);
    if (!isMonitorSnapshot(parsed)) {
      markStreamIssue(state, "degraded", "MONITOR_MALFORMED_PAYLOAD");
      return;
    }

    const sequence = parsed.quality.snapshot_sequence;
    if (state.lastSequence.current && compareSnapshotSequence(sequence, state.lastSequence.current) <= 0) {
      markStreamIssue(state, "degraded", "MONITOR_OUT_OF_ORDER");
      return;
    }

    state.lastSequence.current = sequence;
    const receivedAt = Date.now();
    state.lastEventAt.current = receivedAt;
    state.setLastUpdateAt(receivedAt);
    const snapshotIsFresh = isSnapshotFresh(parsed);
    state.setIsFresh(snapshotIsFresh);
    state.setStatus(snapshotIsFresh && parsed.quality.data_quality === "complete"
      ? "live"
      : snapshotIsFresh
        ? "degraded"
        : "stale");
    state.setErrorCode(null);
    state.queryClient.setQueryData(monitorKeys.overview(), parsed);
    state.retryCount.current = 0;
  } catch (error: unknown) {
    markStreamIssue(state, "degraded", "MONITOR_MALFORMED_PAYLOAD");
    logError(error, { component: "useMonitorStream" });
  }
}

function parseMonitorErrorPayload(data: unknown): unknown {
  if (typeof data !== "string") return data;
  try {
    return JSON.parse(data) as unknown;
  } catch {
    return data;
  }
}

function getRetryDelay(retryCount: number): number {
  return Math.min(INITIAL_RETRY_DELAY_MS * 2 ** retryCount, MAX_RETRY_DELAY_MS);
}

/**
 * Subscribe to monitor SSE updates while keeping transport, freshness, and
 * source quality separate. A connected EventSource is not itself proof that
 * a fresh snapshot has arrived.
 */
export function useMonitorStream() {
  const queryClient = useQueryClient();
  const esRef = useRef<EventSource | null>(null);
  const retryCount = useRef(0);
  const retryTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const connectRef = useRef<() => void>(() => undefined);
  const lastSequence = useRef<string | null>(null);
  const lastEventAt = useRef<number | null>(null);
  const [connected, setConnected] = useState(false);
  const [status, setStatus] = useState<MonitorStreamStatus>("connecting");
  const [isFresh, setIsFresh] = useState(false);
  const [lastUpdateAt, setLastUpdateAt] = useState<number | null>(null);
  const [errorCode, setErrorCode] = useState<string | null>(null);

  const connect = useCallback(() => {
    if (esRef.current) return;

    setStatus("connecting");
    const eventSource = new EventSource(SSE_URL, { withCredentials: true });
    esRef.current = eventSource;
    const state: StreamState = {
      lastEventAt,
      lastSequence,
      queryClient,
      retryCount,
      setErrorCode,
      setIsFresh,
      setLastUpdateAt,
      setStatus,
    };

    eventSource.onopen = () => {
      setConnected(true);
      if (!lastEventAt.current) setStatus("connecting");
    };
    eventSource.addEventListener("monitor-update", (event: MessageEvent) => handleMonitorUpdate(event, state));
    eventSource.addEventListener("monitor-error", (event: MessageEvent) => {
      markStreamIssue(state, "degraded", readMonitorErrorCode(parseMonitorErrorPayload(event.data)));
    });
    eventSource.onerror = () => {
      setConnected(false);
      markStreamIssue(state, "error", "MONITOR_STREAM_DISCONNECTED");
      eventSource.close();
      esRef.current = null;

      if (retryCount.current >= MAX_RETRIES) return;
      retryTimer.current = setTimeout(() => { connectRef.current(); }, getRetryDelay(retryCount.current));
      retryCount.current += 1;
    };
  }, [queryClient]);
  useEffect(() => {
    connectRef.current = connect;
    connect();
    const staleTimer = setInterval(() => {
      const eventAt = lastEventAt.current;
      if (eventAt && Date.now() - eventAt > STALE_AFTER_MS) {
        markStreamIssue(
          { lastEventAt, lastSequence, queryClient, retryCount, setErrorCode, setIsFresh, setLastUpdateAt, setStatus },
          "stale",
          "MONITOR_STREAM_STALE",
        );
      }
    }, STALE_CHECK_INTERVAL_MS);

    return () => {
      esRef.current?.close();
      esRef.current = null;
      if (retryTimer.current) clearTimeout(retryTimer.current);
      clearInterval(staleTimer);
    };
  }, [connect, queryClient]);

  return { connected, errorCode, isFresh, lastUpdateAt, status };
}
