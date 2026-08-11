import { useMemo } from "react";

import type { WorkerMonitorStats, RecentJob } from "@/lib/api/types";

export interface UtilizationData {
    active: number | null;
    idle: number | null;
    percent: number | null;
    total: number | null;
    unavailable: boolean;
}

export type PressureLevel = "low" | "medium" | "high" | "critical";

export interface QueuePressureData {
    high: number | null;
    normal: number | null;
    low: number | null;
    pressureLevel: PressureLevel | "unavailable";
    total: number | null;
    unavailable: boolean;
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

export const useWorkerUtilization = (stats: WorkerMonitorStats | undefined): UtilizationData => {
    return useMemo(() => {
        const active = stats?.active_workers ?? null;
        const idle = stats?.idle_workers ?? null;
        const total = stats?.total_workers ?? null;
        const unavailable = active === null || idle === null || total === null;
        const percent = unavailable
            ? null
            : total > 0
                ? Math.round((active / total) * 100)
                : 0;
        return { active, idle, total, percent, unavailable };
    }, [stats]);
};

function derivePressureLevel(total: number): PressureLevel {
    if (total === 0) return "low";
    if (total <= 5) return "medium";
    if (total <= 15) return "high";
    return "critical";
}

export const useQueuePressure = (stats: WorkerMonitorStats | undefined): QueuePressureData => {
    return useMemo(() => {
        const high = stats?.pending_high ?? null;
        const normal = stats?.pending_normal ?? null;
        const low = stats?.pending_low ?? null;
        const unavailable = high === null || normal === null || low === null;
        const total = unavailable ? null : high + normal + low;
        return {
            high,
            normal,
            low,
            total,
            pressureLevel: unavailable || total === null ? "unavailable" : derivePressureLevel(total),
            unavailable,
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
