import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useRef, useState } from "react";

import type { WorkerMonitorStats } from "@/lib/api/types";

import { logError } from "@/lib/logger";

import { monitorKeys } from "./use-worker-monitor";

/** SSE base URL – mirrors the REST API_BASE_URL logic. */
const SSE_BASE = import.meta.env.DEV
  ? "/api"
  : ((import.meta.env.VITE_API_URL as string | undefined)?.replace(/\/$/, "") ?? "/api");
const SSE_URL = (SSE_BASE.endsWith("/v1") ? SSE_BASE : `${SSE_BASE}/v1`) + "/monitor/stream";

const MAX_RETRIES = 5;
const INITIAL_RETRY_DELAY_MS = 1_000;
const STALE_AFTER_MS = 15_000;
const STALE_CHECK_INTERVAL_MS = 5_000;

export type MonitorStreamStatus = "connecting" | "live" | "degraded" | "stale" | "error";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

const MONITOR_DATA_QUALITIES = new Set(["complete", "partial", "degraded", "unavailable"]);
const MONITOR_FRESHNESS_VALUES = new Set(["fresh", "aging", "stale", "unknown"]);
const MONITOR_SOURCE_STATUSES = new Set(["available", "degraded", "unavailable"]);
const NULLABLE_METRIC_FIELDS = [
  "active_workers",
  "completed_today",
  "failed_today",
  "idle_workers",
  "pending_high",
  "pending_low",
  "pending_normal",
  "pending_tasks",
  "total_workers",
] as const;

function isNullableNonNegativeNumber(value: unknown): boolean {
  return value === undefined || value === null || (
    typeof value === "number" &&
    Number.isFinite(value) &&
    value >= 0
  );
}

function isAllowedValue(value: unknown, allowed: Set<string>): value is string {
  return typeof value === "string" && allowed.has(value);
}

function isMonitorSourceHealth(value: unknown): boolean {
  if (!isRecord(value) || typeof value.status !== "string" || !MONITOR_SOURCE_STATUSES.has(value.status)) {
    return false;
  }
  return value.error_code === undefined || value.error_code === null || typeof value.error_code === "string";
}

function isMonitorMetric(value: unknown): boolean {
  if (!isRecord(value) || typeof value.unit !== "string" || typeof value.formula !== "string" || typeof value.source !== "string") return false;
  if (value.value !== undefined && value.value !== null && !isNullableNonNegativeNumber(value.value)) return false;
  if (value.observed_at !== undefined && (typeof value.observed_at !== "string" || !Number.isFinite(Date.parse(value.observed_at)))) return false;
  if (!isRecord(value.time_window)) return false;
  if (typeof value.time_window.start !== "string" || !Number.isFinite(Date.parse(value.time_window.start))) return false;
  if (typeof value.time_window.end !== "string" || !Number.isFinite(Date.parse(value.time_window.end))) return false;
  if (!isNullableNonNegativeNumber(value.time_window.duration_seconds)) return false;
  return value.unknown === undefined || value.unknown === null || (
    isRecord(value.unknown) && typeof value.unknown.code === "string" && typeof value.unknown.reason === "string"
  );
}

function isMetricSection(value: unknown, fields: string[]): boolean {
  return isRecord(value) && fields.every((field) => isMonitorMetric(value[field]));
}

function hasValidOperationalContract(value: Record<string, unknown>): boolean {
  const queueFields = [
    "dead_letter_depth", "depth_high", "depth_low", "depth_normal", "depth_total",
    "enqueue_rate_per_second", "estimated_drain_seconds", "growth_rate_per_second",
    "oldest_queued_age_seconds", "processing_rate_per_second", "queue_wait_seconds_p50", "queue_wait_seconds_p95",
  ];
  const reliabilityFields = [
    "consecutive_failures", "dead_letter_count", "execution_seconds_p50", "execution_seconds_p95",
    "execution_seconds_p99", "failure_rate", "lost_count", "retryable_count", "success_rate", "timeout_count",
  ];
  const queueValid = value.queue_health === undefined || isMetricSection(value.queue_health, queueFields);
  const reliabilityValid = value.reliability === undefined || isMetricSection(value.reliability, reliabilityFields);
  const thresholds = value.thresholds;
  const thresholdsValid = thresholds === undefined || (
    isRecord(thresholds) && typeof thresholds.source === "string" &&
    isRecord(thresholds.queue_pressure) &&
    isNullableNonNegativeNumber(thresholds.queue_pressure.medium_at) &&
    isNullableNonNegativeNumber(thresholds.queue_pressure.high_at) &&
    isNullableNonNegativeNumber(thresholds.queue_pressure.critical_at) &&
    isRecord(thresholds.worker_utilization) &&
    isNullableNonNegativeNumber(thresholds.worker_utilization.scale_down_below_percent) &&
    isNullableNonNegativeNumber(thresholds.worker_utilization.warning_at_percent) &&
    isNullableNonNegativeNumber(thresholds.worker_utilization.critical_at_percent)
  );
  return queueValid && reliabilityValid && thresholdsValid;
}

function isWorkerHeartbeat(value: unknown): boolean {
  if (!isRecord(value)) return false;
  return (
    typeof value.id === "string" &&
    typeof value.host === "string" &&
    (value.status === "IDLE" || value.status === "PROCESSING") &&
    (value.current_task_type === undefined || value.current_task_type === null || typeof value.current_task_type === "string") &&
    typeof value.last_seen_at === "string" &&
    Number.isFinite(Date.parse(value.last_seen_at)) &&
    (value.liveness === undefined || value.liveness === null || typeof value.liveness === "string" && ["live", "stale", "offline"].includes(value.liveness)) &&
    (value.heartbeat_age_seconds === undefined || value.heartbeat_age_seconds === null || (
      typeof value.heartbeat_age_seconds === "number" &&
      Number.isFinite(value.heartbeat_age_seconds) &&
      value.heartbeat_age_seconds >= 0
    ))
  );
}

function isMonitorSnapshot(value: unknown): value is WorkerMonitorStats {
  if (!isRecord(value) || !isRecord(value.quality)) return false;
  const quality = value.quality;
  const hasValidMetrics = NULLABLE_METRIC_FIELDS.every((field) => isNullableNonNegativeNumber(value[field]));
  return (
    typeof quality.snapshot_sequence === "string" &&
    isAllowedValue(quality.data_quality, MONITOR_DATA_QUALITIES) &&
    isAllowedValue(quality.freshness, MONITOR_FRESHNESS_VALUES) &&
    Array.isArray(quality.unknown_fields) &&
    quality.unknown_fields.every((field) => typeof field === "string") &&
    isMonitorSourceHealth(quality.redis) &&
    isMonitorSourceHealth(quality.durable_db) &&
    isMonitorSourceHealth(quality.worker_heartbeats) &&
    typeof value.observed_at === "string" &&
    Number.isFinite(Date.parse(value.observed_at)) &&
    typeof value.snapshot_age_seconds === "number" &&
    Number.isFinite(value.snapshot_age_seconds) &&
    value.snapshot_age_seconds >= 0 &&
    hasValidMetrics &&
    hasValidOperationalContract(value) &&
    Array.isArray(value.workers) &&
    value.workers.every(isWorkerHeartbeat) &&
    Array.isArray(value.recent_jobs) &&
    Array.isArray(value.job_type_stats)
  );
}

function compareSnapshotSequence(left: string, right: string): number {
  const leftTimestamp = Number.parseInt(left.split("-", 1)[0] ?? "", 10);
  const rightTimestamp = Number.parseInt(right.split("-", 1)[0] ?? "", 10);
  if (Number.isFinite(leftTimestamp) && Number.isFinite(rightTimestamp) && leftTimestamp !== rightTimestamp) {
    return leftTimestamp - rightTimestamp;
  }
  return left.localeCompare(right);
}

function readMonitorErrorCode(data: unknown): string {
  if (isRecord(data) && typeof data.code === "string" && data.code.length <= 100) {
    return data.code;
  }
  return "MONITOR_STREAM_ERROR";
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
  const lastSequence = useRef<string | null>(null);
  const lastEventAt = useRef<number | null>(null);
  const [connected, setConnected] = useState(false);
  const [status, setStatus] = useState<MonitorStreamStatus>("connecting");
  const [isFresh, setIsFresh] = useState(false);
  const [lastUpdateAt, setLastUpdateAt] = useState<number | null>(null);
  const [errorCode, setErrorCode] = useState<string | null>(null);

  const connect = useCallback(() => {
    function doConnect() {
      if (esRef.current) return;

      setStatus("connecting");
      const es = new EventSource(SSE_URL, { withCredentials: true });
      esRef.current = es;

      es.onopen = () => {
        setConnected(true);
        if (!lastEventAt.current) setStatus("connecting");
      };

      es.addEventListener("monitor-update", (event: MessageEvent) => {
        try {
          const parsed: unknown = JSON.parse(event.data as string);
          if (!isMonitorSnapshot(parsed)) {
            setStatus("degraded");
            setIsFresh(false);
            setErrorCode("MONITOR_MALFORMED_PAYLOAD");
            return;
          }

          const sequence = parsed.quality.snapshot_sequence;
          if (lastSequence.current && compareSnapshotSequence(sequence, lastSequence.current) <= 0) {
            setStatus("degraded");
            setErrorCode("MONITOR_OUT_OF_ORDER");
            return;
          }

          lastSequence.current = sequence;
          const receivedAt = Date.now();
          lastEventAt.current = receivedAt;
          setLastUpdateAt(receivedAt);
          const snapshotIsFresh =
            parsed.snapshot_age_seconds <= STALE_AFTER_MS / 1_000 &&
            parsed.quality.freshness === "fresh";
          setIsFresh(snapshotIsFresh);
          setStatus(
            snapshotIsFresh && parsed.quality.data_quality === "complete"
              ? "live"
              : snapshotIsFresh
                ? "degraded"
                : "stale",
          );
          setErrorCode(null);
          queryClient.setQueryData(monitorKeys.overview(), parsed);
          retryCount.current = 0;
        } catch (error: unknown) {
          setStatus("degraded");
          setIsFresh(false);
          setErrorCode("MONITOR_MALFORMED_PAYLOAD");
          logError(error, { component: "useMonitorStream" });
        }
      });

      es.addEventListener("monitor-error", (event: MessageEvent) => {
        let payload: unknown = event.data;
        if (typeof event.data === "string") {
          try {
            payload = JSON.parse(event.data) as unknown;
          } catch {
            payload = event.data;
          }
        }
        setStatus("degraded");
        setIsFresh(false);
        setErrorCode(readMonitorErrorCode(payload));
      });

      es.onerror = () => {
        setConnected(false);
        setIsFresh(false);
        setStatus("error");
        setErrorCode("MONITOR_STREAM_DISCONNECTED");
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

    const staleTimer = setInterval(() => {
      const eventAt = lastEventAt.current;
      if (eventAt && Date.now() - eventAt > STALE_AFTER_MS) {
        setIsFresh(false);
        setStatus("stale");
        setErrorCode("MONITOR_STREAM_STALE");
      }
    }, STALE_CHECK_INTERVAL_MS);

    return () => {
      esRef.current?.close();
      esRef.current = null;
      if (retryTimer.current) clearTimeout(retryTimer.current);
      clearInterval(staleTimer);
    };
  }, [connect]);

  return { connected, errorCode, isFresh, lastUpdateAt, status };
}
