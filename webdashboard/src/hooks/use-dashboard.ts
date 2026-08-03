import { useQuery } from "@tanstack/react-query";

import { apiFetch } from "@/lib/api";

import {
  type AttendanceTrendPoint,
  type DepartmentAttendancePoint,
  type LatenessByDay,
  type LeaveTypeBreakdown,
  type OvertimeTrendPoint,
  type PeakTimeSlot,
  type RecruitmentSourceBreakdown,
  type TurnoverAnalytics,
  type UnexcusedAbsencesPoint,
  type WorkspaceStats,
} from "./use-dashboard-types";
import { useWorkspace } from "./use-workspace";

export type {
  AttendanceTrendPoint,
  DepartmentAttendancePoint,
  LatenessByDay,
  LeaveTypeBreakdown,
  OvertimeTrendPoint,
  PeakTimeSlot,
  RecruitmentSourceBreakdown,
  TurnoverAnalytics,
  UnexcusedAbsencesPoint,
  WorkspaceStats,
} from "./use-dashboard-types";

// ── Factory: reusable useQuery wrapper with workspace guard ────────────────

interface DashboardQueryOptions {
  /** Unique cache key segment (e.g. "workspace-stats"). */
  key: string;
  /** Lazily builds the API path (captures hook args in closure). */
  getPath: () => string;
  /** Extra values appended to the query key (e.g. range, dates). */
  extraQueryKeyParams?: unknown[];
  /** Additional enabled condition ANDed with the workspace guard. */
  extraEnabled?: boolean;
}

function useDashboardQuery<T>(options: DashboardQueryOptions) {
  const { workspace } = useWorkspace();

  return useQuery<T>({
    queryKey: [options.key, workspace?.id, ...(options.extraQueryKeyParams ?? [])],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(options.getPath());

      if (!response.ok) {
        throw new Error(`Failed to fetch ${options.key}`);
      }

      return response.json() as Promise<T>;
    },
    enabled: !!workspace && (options.extraEnabled ?? true),
  });
}

// ── Public hooks (one-liners) ─────────────────────────────────────────────

export const useDashboardStats = () =>
  useDashboardQuery<WorkspaceStats>({
    key: "workspace-stats",
    getPath: () => "/v1/dashboard/workspace-stats",
  });

export const useAttendanceTrend = () =>
  useDashboardQuery<AttendanceTrendPoint[]>({
    key: "attendance-trend",
    getPath: () => "/v1/dashboard/attendance-trend",
  });

export function useDepartmentAttendance(range: "today" | "week" = "today") {
  return useDashboardQuery<DepartmentAttendancePoint[]>({
    key: "department-attendance",
    getPath: () => `/v1/dashboard/department-attendance?range=${range}`,
    extraQueryKeyParams: [range],
  });
}

export function useUnexcusedAbsences(
  startDate?: string,
  endDate?: string,
  groupBy?: "department" | "employee",
) {
  return useDashboardQuery<UnexcusedAbsencesPoint[]>({
    key: "unexcused-absences",
    getPath: () => {
      const params = new URLSearchParams();
      if (startDate) params.set("start_date", startDate);
      if (endDate) params.set("end_date", endDate);
      if (groupBy) params.set("group_by", groupBy);
      const qs = params.toString();
      return `/v1/dashboard/unexcused-absences${qs ? `?${qs}` : ""}`;
    },
    extraQueryKeyParams: [startDate, endDate, groupBy],
  });
}

export const useOvertimeTrend = () =>
  useDashboardQuery<OvertimeTrendPoint[]>({
    key: "overtime-trend",
    getPath: () => "/v1/dashboard/overtime-trend",
  });

export const usePeakTimes = () =>
  useDashboardQuery<PeakTimeSlot[]>({
    key: "peak-times",
    getPath: () => "/v1/dashboard/peak-times",
  });

export const useLeaveTypesBreakdown = () =>
  useDashboardQuery<LeaveTypeBreakdown[]>({
    key: "leave-types-breakdown",
    getPath: () => "/v1/dashboard/leave-types",
  });

export const useLatenessByDay = () =>
  useDashboardQuery<LatenessByDay[]>({
    key: "lateness-by-day",
    getPath: () => "/v1/dashboard/lateness-by-day",
  });

export const useRecruitmentSourceBreakdown = () =>
  useDashboardQuery<RecruitmentSourceBreakdown[]>({
    key: "recruitment-sources-breakdown",
    getPath: () => "/v1/dashboard/recruitment-sources",
  });

export function useTurnoverAnalytics(startDate: string, endDate: string) {
  return useDashboardQuery<TurnoverAnalytics>({
    key: "turnover-analytics",
    getPath: () =>
      `/v1/dashboard/turnover-analytics?start=${startDate}&end=${endDate}`,
    extraQueryKeyParams: [startDate, endDate],
    extraEnabled: !!startDate && !!endDate,
  });
}
