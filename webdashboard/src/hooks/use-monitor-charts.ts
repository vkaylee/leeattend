import { useMemo } from "react";

import type { WorkerMonitorStats, RecentJob } from "@/lib/api/types";

import {
  deriveJobStatusDistribution,
  deriveQueuePressure,
  deriveWorkerUtilization,
} from "./monitor-chart-helpers";

export interface UtilizationData {
  active: number | null;
  criticalAtPercent: number | null;
  idle: number | null;
  percent: number | null;
  scaleDownBelowPercent: number | null;
  thresholdsAvailable: boolean;
  total: number | null;
  unavailable: boolean;
  warningAtPercent: number | null;
}

export type PressureLevel = "low" | "medium" | "high" | "critical";

export interface QueuePressureData {
  high: number | null;
  low: number | null;
  normal: number | null;
  pressureLevel: PressureLevel | "unavailable";
  thresholdsAvailable: boolean;
  total: number | null;
  unavailable: boolean;
}

export interface JobStatusCount {
  color: string;
  count: number;
  percent: number;
  status: string;
}

export const useWorkerUtilization = (stats: WorkerMonitorStats | undefined): UtilizationData =>
  useMemo(() => deriveWorkerUtilization(stats), [stats]);

export const useQueuePressure = (stats: WorkerMonitorStats | undefined): QueuePressureData =>
  useMemo(() => deriveQueuePressure(stats), [stats]);

export const useJobStatusDistribution = (recentJobs: RecentJob[]): JobStatusCount[] =>
  useMemo(() => deriveJobStatusDistribution(recentJobs), [recentJobs]);
