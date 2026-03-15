import { useMemo } from "react";
import type { WorkerMonitorStats, RecentJob } from "@/lib/api/types";

export interface UtilizationData {
    active: number;
    idle: number;
    total: number;
    percent: number;
}

export type PressureLevel = "low" | "medium" | "high" | "critical";

export interface QueuePressureData {
    high: number;
    low: number;
    total: number;
    pressureLevel: PressureLevel;
}

export interface JobStatusCount {
    status: string;
    count: number;
    color: string;
    percent: number;
}

const STATUS_COLORS: Record<string, string> = {
    completed: "#10b981",
    failed: "#ef4444",
    processing: "#f59e0b",
    pending: "#94a3b8",
};

export const useWorkerUtilization = (stats: WorkerMonitorStats | undefined): UtilizationData => {
    return useMemo(() => {
        if (!stats) return { active: 0, idle: 0, total: 0, percent: 0 };
        const total = stats.total_workers;
        const percent = total > 0 ? Math.round((stats.active_workers / total) * 100) : 0;
        return { active: stats.active_workers, idle: stats.idle_workers, total, percent };
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
        if (!stats) return { high: 0, low: 0, total: 0, pressureLevel: "low" as PressureLevel };
        const total = stats.pending_high + stats.pending_low;
        return { high: stats.pending_high, low: stats.pending_low, total, pressureLevel: derivePressureLevel(total) };
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
