import type { WorkerMonitorStats } from "@/lib/api/types";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

const MONITOR_DATA_QUALITIES = new Set(["complete", "partial", "degraded", "unavailable"]);
const MONITOR_FRESHNESS_VALUES = new Set(["fresh", "aging", "stale", "unknown"]);
const MONITOR_SOURCE_STATUSES = new Set(["available", "degraded", "unavailable"]);
const MONITOR_LIVENESS_VALUES = new Set(["live", "stale", "offline"]);
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
const QUEUE_METRIC_FIELDS = [
  "dead_letter_depth",
  "depth_high",
  "depth_low",
  "depth_normal",
  "depth_total",
  "enqueue_rate_per_second",
  "estimated_drain_seconds",
  "growth_rate_per_second",
  "oldest_queued_age_seconds",
  "processing_rate_per_second",
  "queue_wait_seconds_p50",
  "queue_wait_seconds_p95",
];
const RELIABILITY_METRIC_FIELDS = [
  "consecutive_failures",
  "dead_letter_count",
  "execution_seconds_p50",
  "execution_seconds_p95",
  "execution_seconds_p99",
  "failure_rate",
  "lost_count",
  "retryable_count",
  "success_rate",
  "timeout_count",
];

function isNullableNonNegativeNumber(value: unknown): boolean {
  if (value === undefined || value === null) return true;
  return typeof value === "number" && Number.isFinite(value) && value >= 0;
}

function isAllowedValue(value: unknown, allowed: Set<string>): value is string {
  return typeof value === "string" && allowed.has(value);
}

function isValidDate(value: unknown): value is string {
  return typeof value === "string" && Number.isFinite(Date.parse(value));
}

function isMonitorSourceHealth(value: unknown): boolean {
  if (!isRecord(value) || typeof value.status !== "string") return false;
  if (!MONITOR_SOURCE_STATUSES.has(value.status)) return false;
  return value.error_code === undefined || value.error_code === null || typeof value.error_code === "string";
}

function isValidMetricWindow(value: unknown): boolean {
  if (!isRecord(value)) return false;
  return isValidDate(value.start) && isValidDate(value.end) &&
    isNullableNonNegativeNumber(value.duration_seconds);
}

function isValidMetricValue(value: Record<string, unknown>): boolean {
  return value.value === undefined || value.value === null || isNullableNonNegativeNumber(value.value);
}

function isValidMetricUnknown(value: Record<string, unknown>): boolean {
  if (value.unknown === undefined || value.unknown === null) return true;
  if (!isRecord(value.unknown)) return false;
  return typeof value.unknown.code === "string" && typeof value.unknown.reason === "string";
}

function isMonitorMetric(value: unknown): boolean {
  if (!isRecord(value)) return false;
  if (typeof value.unit !== "string" || typeof value.formula !== "string" || typeof value.source !== "string") {
    return false;
  }
  if (!isValidMetricValue(value)) return false;
  if (value.observed_at !== undefined && !isValidDate(value.observed_at)) return false;
  return isValidMetricWindow(value.time_window) && isValidMetricUnknown(value);
}

function isMetricSection(value: unknown, fields: string[]): boolean {
  return isRecord(value) && fields.every((field) => isMonitorMetric(value[field]));
}

function isValidQueueThresholds(value: Record<string, unknown>): boolean {
  const queuePressure = value.queue_pressure;
  if (!isRecord(queuePressure)) return false;
  return isNullableNonNegativeNumber(queuePressure.medium_at) &&
    isNullableNonNegativeNumber(queuePressure.high_at) &&
    isNullableNonNegativeNumber(queuePressure.critical_at);
}

function isValidWorkerThresholds(value: Record<string, unknown>): boolean {
  const workerUtilization = value.worker_utilization;
  if (!isRecord(workerUtilization)) return false;
  return isNullableNonNegativeNumber(workerUtilization.scale_down_below_percent) &&
    isNullableNonNegativeNumber(workerUtilization.warning_at_percent) &&
    isNullableNonNegativeNumber(workerUtilization.critical_at_percent);
}

function isValidThresholds(value: unknown): boolean {
  if (!isRecord(value) || typeof value.source !== "string") return false;
  return isValidQueueThresholds(value) && isValidWorkerThresholds(value);
}

function hasValidOperationalContract(value: Record<string, unknown>): boolean {
  return isMetricSection(value.queue_health, QUEUE_METRIC_FIELDS) &&
    isMetricSection(value.reliability, RELIABILITY_METRIC_FIELDS) &&
    isValidThresholds(value.thresholds);
}

function isValidHeartbeatStatus(value: Record<string, unknown>): boolean {
  return value.status === "IDLE" || value.status === "PROCESSING";
}

function isValidHeartbeatLiveness(value: Record<string, unknown>): boolean {
  return value.liveness === undefined || value.liveness === null ||
    isAllowedValue(value.liveness, MONITOR_LIVENESS_VALUES);
}

function isValidHeartbeatAge(value: Record<string, unknown>): boolean {
  return value.heartbeat_age_seconds === undefined || value.heartbeat_age_seconds === null ||
    isNullableNonNegativeNumber(value.heartbeat_age_seconds);
}

function isWorkerHeartbeat(value: unknown): boolean {
  if (!isRecord(value)) return false;
  return typeof value.id === "string" &&
    typeof value.host === "string" &&
    isValidHeartbeatStatus(value) &&
    (value.current_task_type === undefined || value.current_task_type === null || typeof value.current_task_type === "string") &&
    isValidDate(value.last_seen_at) &&
    isValidHeartbeatLiveness(value) &&
    isValidHeartbeatAge(value);
}

function isValidQuality(value: Record<string, unknown>): boolean {
  return isAllowedValue(value.data_quality, MONITOR_DATA_QUALITIES) &&
    isAllowedValue(value.freshness, MONITOR_FRESHNESS_VALUES) &&
    Array.isArray(value.unknown_fields) &&
    value.unknown_fields.every((field) => typeof field === "string") &&
    isMonitorSourceHealth(value.redis) &&
    isMonitorSourceHealth(value.durable_db) &&
    isMonitorSourceHealth(value.worker_heartbeats);
}

function isValidSnapshotMetadata(value: Record<string, unknown>): boolean {
  return isValidDate(value.observed_at) &&
    typeof value.snapshot_age_seconds === "number" &&
    Number.isFinite(value.snapshot_age_seconds) &&
    value.snapshot_age_seconds >= 0;
}

function isValidSnapshotCollections(value: Record<string, unknown>): boolean {
  return Array.isArray(value.workers) &&
    value.workers.every(isWorkerHeartbeat) &&
    Array.isArray(value.recent_jobs) &&
    Array.isArray(value.job_type_stats);
}

export function isMonitorSnapshot(value: unknown): value is WorkerMonitorStats {
  if (!isRecord(value) || !isRecord(value.quality)) return false;
  return typeof value.quality.snapshot_sequence === "string" &&
    isValidQuality(value.quality) &&
    isValidSnapshotMetadata(value) &&
    NULLABLE_METRIC_FIELDS.every((field) => isNullableNonNegativeNumber(value[field])) &&
    hasValidOperationalContract(value) &&
    isValidSnapshotCollections(value);
}

export function compareSnapshotSequence(left: string, right: string): number {
  const leftTimestamp = Number.parseInt(left.split("-", 1)[0] ?? "", 10);
  const rightTimestamp = Number.parseInt(right.split("-", 1)[0] ?? "", 10);
  if (Number.isFinite(leftTimestamp) && Number.isFinite(rightTimestamp) && leftTimestamp !== rightTimestamp) {
    return leftTimestamp - rightTimestamp;
  }
  return left.localeCompare(right);
}

export function readMonitorErrorCode(data: unknown): string {
  if (isRecord(data) && typeof data.code === "string" && data.code.length <= 100) {
    return data.code;
  }
  return "MONITOR_STREAM_ERROR";
}
