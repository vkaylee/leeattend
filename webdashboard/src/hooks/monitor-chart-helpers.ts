import type { WorkerMonitorStats, RecentJob } from "@/lib/api/types";

import type { JobStatusCount, PressureLevel, QueuePressureData, UtilizationData } from "./use-monitor-charts";

type QueueThresholds = NonNullable<NonNullable<WorkerMonitorStats["thresholds"]>["queue_pressure"]>;

const STATUS_COLORS: Record<string, string> = {
  completed: "#10b981",
  failed: "#ef4444",
  processing: "#f59e0b",
  pending: "#94a3b8",
};

type Metric = { value?: number | null } | undefined;

function metricValue(metric: Metric, legacyValue: number | null | undefined): number | null {
  return metric !== undefined ? metric.value ?? null : legacyValue ?? null;
}

export function deriveWorkerUtilization(stats: WorkerMonitorStats | undefined): UtilizationData {
  const values = workerValues(stats);
  const thresholds = stats?.thresholds.worker_utilization;
  return {
    ...values,
    thresholdsAvailable: thresholds !== undefined && thresholds.source !== "unavailable",
    scaleDownBelowPercent: thresholds?.scale_down_below_percent ?? null,
    warningAtPercent: thresholds?.warning_at_percent ?? null,
    criticalAtPercent: thresholds?.critical_at_percent ?? null,
  };
}

function workerValues(stats: WorkerMonitorStats | undefined): Pick<UtilizationData, "active" | "idle" | "total" | "percent" | "unavailable"> {
  const active = stats?.active_workers ?? null;
  const idle = stats?.idle_workers ?? null;
  const total = stats?.total_workers ?? null;
  const unavailable = hasMissingWorkerValue(active, idle, total);
  return { active, idle, total, unavailable, percent: utilizationPercent(active, total, unavailable) };
}

function hasMissingWorkerValue(active: number | null, idle: number | null, total: number | null): boolean {
  return [active, idle, total].includes(null);
}

function utilizationPercent(active: number | null, total: number | null, unavailable: boolean): number | null {
  if (unavailable || active === null || total === null) return null;
  return total > 0 ? Math.round((active / total) * 100) : 0;
}

function sumQueueDepth(high: number | null, normal: number | null, low: number | null): number | null {
  return high === null || normal === null || low === null ? null : high + normal + low;
}

function derivePressureLevel(total: number, thresholds: NonNullable<WorkerMonitorStats["thresholds"]>["queue_pressure"]): PressureLevel {
  if (total < thresholds.medium_at) return "low";
  if (total < thresholds.high_at) return "medium";
  if (total < thresholds.critical_at) return "high";
  return "critical";
}

export function deriveQueuePressure(stats: WorkerMonitorStats | undefined): QueuePressureData {
  const depths = queueDepths(stats);
  const thresholds = stats?.thresholds.queue_pressure;
  const thresholdsAvailable = hasAvailableThresholds(thresholds);
  const pressureLevel = getPressureLevel(depths.total, thresholds);

  return {
    ...depths,
    pressureLevel,
    unavailable: pressureLevel === "unavailable",
    thresholdsAvailable,
  };
}

function getPressureLevel(total: number | null, thresholds: QueueThresholds | undefined): QueuePressureData["pressureLevel"] {
  if (total === null || thresholds === undefined || thresholds.source === "unavailable") return "unavailable";
  return derivePressureLevel(total, thresholds);
}

function queueDepths(stats: WorkerMonitorStats | undefined): Pick<QueuePressureData, "high" | "normal" | "low" | "total"> {
  const queueHealth = stats?.queue_health;
  const high = metricValue(queueHealth?.depth_high, stats?.pending_high);
  const normal = metricValue(queueHealth?.depth_normal, stats?.pending_normal);
  const low = metricValue(queueHealth?.depth_low, stats?.pending_low);
  return { high, normal, low, total: sumQueueDepth(high, normal, low) };
}

function hasAvailableThresholds(thresholds: QueueThresholds | undefined): boolean {
  return thresholds !== undefined && thresholds.source !== "unavailable";
}

export function deriveJobStatusDistribution(recentJobs: RecentJob[]): JobStatusCount[] {
  const counts: Record<string, number> = {};
  for (const job of recentJobs) {
    counts[job.status] = (counts[job.status] ?? 0) + 1;
  }
  const totalJobs = recentJobs.length || 1;
  return Object.entries(counts).map(([status, count]) => ({
    status,
    count,
    color: STATUS_COLORS[status] ?? "#94a3b8",
    percent: Math.round((count / totalJobs) * 100),
  }));
}
