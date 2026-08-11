import { useMemo } from "react";

import type { WorkerMonitorStats, RecentJob } from "@/lib/api/types";

export interface UtilizationData {
    active: number | null;
    idle: number | null;
    percent: number | null;
    total: number | null;
    unavailable: boolean;
    thresholdsAvailable: boolean;
    scaleDownBelowPercent: number | null;
    warningAtPercent: number | null;
    criticalAtPercent: number | null;
}

export type PressureLevel = "low" | "medium" | "high" | "critical";

export interface QueuePressureData {
    high: number | null;
    normal: number | null;
    low: number | null;
    pressureLevel: PressureLevel | "unavailable";
    total: number | null;
    unavailable: boolean;
    thresholdsAvailable: boolean;
}

export interface JobStatusCount {
    color: string;
    count: number;
    percent: number;
    status: string;
}

const STATUS_COLORS: Record<string, string> = {
    completed: "#10b981",
    failed: "#ef4444",
    processing: "#f59e0b",
    pending: "#94a3b8",
};

function metricValue(
    metric: { value?: number | null } | undefined,
    legacyValue: number | null | undefined,
): number | null {
    if (metric !== undefined) return metric.value ?? null;
    return legacyValue ?? null;
}

export const useWorkerUtilization = (stats: WorkerMonitorStats | undefined): UtilizationData => {
    return useMemo(() => {
        const active = stats?.active_workers ?? null;
        const idle = stats?.idle_workers ?? null;
        const total = stats?.total_workers ?? null;
        const unavailable = active === null || idle === null || total === null;
        const thresholds = stats?.thresholds?.worker_utilization;
        const thresholdsAvailable = thresholds !== undefined && thresholds.source !== "unavailable";
        const percent = unavailable
            ? null
            : total > 0
                ? Math.round((active / total) * 100)
                : 0;
        return {
            active,
            idle,
            total,
            percent,
            unavailable,
            thresholdsAvailable,
            scaleDownBelowPercent: thresholds?.scale_down_below_percent ?? null,
            warningAtPercent: thresholds?.warning_at_percent ?? null,
            criticalAtPercent: thresholds?.critical_at_percent ?? null,
        };
    }, [stats]);
};

function derivePressureLevel(total: number, thresholds: NonNullable<WorkerMonitorStats["thresholds"]>["queue_pressure"]): PressureLevel {
    if (total < thresholds.medium_at) return "low";
    if (total < thresholds.high_at) return "medium";
    if (total < thresholds.critical_at) return "high";
    return "critical";
}

export const useQueuePressure = (stats: WorkerMonitorStats | undefined): QueuePressureData => {
    return useMemo(() => {
        const queueHealth = stats?.queue_health;
        const high = metricValue(queueHealth?.depth_high, stats?.pending_high);
        const normal = metricValue(queueHealth?.depth_normal, stats?.pending_normal);
        const low = metricValue(queueHealth?.depth_low, stats?.pending_low);
        const unavailable = high === null || normal === null || low === null;
        const total = unavailable ? null : high + normal + low;
        const thresholds = stats?.thresholds?.queue_pressure;
        const thresholdsAvailable = thresholds !== undefined && thresholds.source !== "unavailable";
        return {
            high,
            normal,
            low,
            total,
            pressureLevel: unavailable || total === null || !thresholdsAvailable
                ? "unavailable"
                : derivePressureLevel(total, thresholds),
            unavailable: unavailable || !thresholdsAvailable,
            thresholdsAvailable,
        };
    }, [stats]);
};

export const useJobStatusDistribution = (recentJobs: RecentJob[]): JobStatusCount[] => {
    return useMemo(() => {
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
    }, [recentJobs]);
};
