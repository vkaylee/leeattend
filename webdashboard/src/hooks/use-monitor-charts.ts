import { useMemo } from "react";
import type { WorkerMonitorStats, RecentJob } from "@/lib/api/types";

export interface UtilizationData {
    active: number;
    idle: number;
    total: number;
    percent: number;
}

export interface QueuePressureData {
    high: number;
    low: number;
    total: number;
}

export interface JobStatusCount {
    status: string;
    count: number;
    color: string;
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

export const useQueuePressure = (stats: WorkerMonitorStats | undefined): QueuePressureData => {
    return useMemo(() => {
        if (!stats) return { high: 0, low: 0, total: 0 };
        return { high: stats.pending_high, low: stats.pending_low, total: stats.pending_high + stats.pending_low };
    }, [stats]);
};

export const useJobStatusDistribution = (recentJobs: RecentJob[]): JobStatusCount[] => {
    return useMemo(() => {
        const counts: Record<string, number> = {};
        for (const job of recentJobs) {
            counts[job.status] = (counts[job.status] ?? 0) + 1;
        }
        return Object.entries(counts).map(([status, count]) => ({
            status,
            count,
            color: STATUS_COLORS[status] ?? "#94a3b8",
        }));
    }, [recentJobs]);
};
