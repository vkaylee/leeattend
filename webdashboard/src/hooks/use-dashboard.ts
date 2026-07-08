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

export function useDashboardStats() {
  const { workspace } = useWorkspace();

  return useQuery<WorkspaceStats>({
    queryKey: ["workspace-stats", workspace?.id],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(`/v1/dashboard/workspace-stats`);

      if (!response.ok) {
        throw new Error("Failed to fetch dashboard stats");
      }

      return response.json() as Promise<WorkspaceStats>;
    },
    enabled: !!workspace,
  });
}

export function useAttendanceTrend() {
  const { workspace } = useWorkspace();

  return useQuery<AttendanceTrendPoint[]>({
    queryKey: ["attendance-trend", workspace?.id],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(`/v1/dashboard/attendance-trend`);

      if (!response.ok) {
        throw new Error("Failed to fetch attendance trend");
      }

      return response.json() as Promise<AttendanceTrendPoint[]>;
    },
    enabled: !!workspace,
  });
}

export function useDepartmentAttendance(range: "today" | "week" = "today") {
  const { workspace } = useWorkspace();

  return useQuery<DepartmentAttendancePoint[]>({
    queryKey: ["department-attendance", workspace?.id, range],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(
        `/v1/dashboard/department-attendance?range=${range}`
      );

      if (!response.ok) {
        throw new Error("Failed to fetch department attendance");
      }

      return response.json() as Promise<DepartmentAttendancePoint[]>;
    },
    enabled: !!workspace,
  });
}

export function useUnexcusedAbsences(startDate?: string, endDate?: string, groupBy?: "department" | "employee") {
  const { workspace } = useWorkspace();

  return useQuery<UnexcusedAbsencesPoint[]>({
    queryKey: ["unexcused-absences", workspace?.id, startDate, endDate, groupBy],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const params = new URLSearchParams();
      if (startDate) params.set("start_date", startDate);
      if (endDate) params.set("end_date", endDate);
      if (groupBy) params.set("group_by", groupBy);
      const qs = params.toString();
      const response = await apiFetch(`/v1/dashboard/unexcused-absences${qs ? `?${qs}` : ""}`);

      if (!response.ok) {
        throw new Error("Failed to fetch unexcused absences");
      }

      return response.json() as Promise<UnexcusedAbsencesPoint[]>;
    },
    enabled: !!workspace,
  });
}

export function useOvertimeTrend() {
  const { workspace } = useWorkspace();

  return useQuery<OvertimeTrendPoint[]>({
    queryKey: ["overtime-trend", workspace?.id],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(`/v1/dashboard/overtime-trend`);

      if (!response.ok) {
        throw new Error("Failed to fetch overtime trend");
      }

      return response.json() as Promise<OvertimeTrendPoint[]>;
    },
    enabled: !!workspace,
  });
}

// ── Peak Check-in/Check-out Times ──────────────────────────

export function usePeakTimes() {
  const { workspace } = useWorkspace();

  return useQuery<PeakTimeSlot[]>({
    queryKey: ["peak-times", workspace?.id],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(`/v1/dashboard/peak-times`);

      if (!response.ok) {
        throw new Error("Failed to fetch peak times");
      }

      return response.json() as Promise<PeakTimeSlot[]>;
    },
    enabled: !!workspace,
  });
}

// ── Leave Types Breakdown ──────────────────────────────────

export function useLeaveTypesBreakdown() {
  const { workspace } = useWorkspace();

  return useQuery<LeaveTypeBreakdown[]>({
    queryKey: ["leave-types-breakdown", workspace?.id],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(`/v1/dashboard/leave-types`);

      if (!response.ok) {
        throw new Error("Failed to fetch leave types breakdown");
      }

      return response.json() as Promise<LeaveTypeBreakdown[]>;
    },
    enabled: !!workspace,
  });
}

// ── Lateness by Day of Week ────────────────────────────────

export function useLatenessByDay() {
  const { workspace } = useWorkspace();

  return useQuery<LatenessByDay[]>({
    queryKey: ["lateness-by-day", workspace?.id],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(`/v1/dashboard/lateness-by-day`);

      if (!response.ok) {
        throw new Error("Failed to fetch lateness by day");
      }

      return response.json() as Promise<LatenessByDay[]>;
    },
    enabled: !!workspace,
  });
}

// ── Recruitment Source Breakdown ──────────────────────────────────

export function useRecruitmentSourceBreakdown() {
  const { workspace } = useWorkspace();

  return useQuery<RecruitmentSourceBreakdown[]>({
    queryKey: ["recruitment-sources-breakdown", workspace?.id],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(`/v1/dashboard/recruitment-sources`);

      if (!response.ok) {
        throw new Error("Failed to fetch recruitment source breakdown");
      }

      return response.json() as Promise<RecruitmentSourceBreakdown[]>;
    },
    enabled: !!workspace,
  });
}

// ── Turnover Analytics ─────────────────────────────────────────────

export function useTurnoverAnalytics(startDate: string, endDate: string) {
  const { workspace } = useWorkspace();

  return useQuery<TurnoverAnalytics>({
    queryKey: ["turnover-analytics", workspace?.id, startDate, endDate],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(
        `/v1/dashboard/turnover-analytics?start=${startDate}&end=${endDate}`
      );

      if (!response.ok) {
        throw new Error("Failed to fetch turnover analytics");
      }

      return response.json() as Promise<TurnoverAnalytics>;
    },
    enabled: !!workspace && !!startDate && !!endDate,
  });
}
