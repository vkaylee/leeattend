import { useQuery } from "@tanstack/react-query";

import { apiFetch } from "@/lib/api";

import { useWorkspace } from "./use-workspace";

export interface WorkspaceStats {
  absent_today: number;
  avg_work_hours_today: number;
  early_departure_today: number;
  estimated_monthly_payroll: string;
  late_today: number;
  on_leave_today: number;
  pending_leave_requests: number;
  present_today: number;
  total_employees: number;
}

export interface AttendanceTrendPoint {
  absent: number;
  date: string;
  late: number;
  on_leave: number;
  present: number;
}

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

export interface DepartmentAttendancePoint {
  absent: number;
  department_id: string;
  department_name: string;
  late: number;
  on_leave: number;
  on_time: number;
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

export interface UnexcusedAbsencesPoint {
  consecutive_count: number;
  department_id: string;
  department_name: string;
  employee_id?: string;
  employee_name?: string;
  total_absent_count: number;
  total_leave_count: number;
  unexcused_count: number;
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

export interface OvertimeTrendPoint {
  approved_ot_hours: number;
  date: string;
  system_ot_hours: number;
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

export interface PeakTimeSlot {
  check_in: number;
  check_out: number;
  time_slot: string; // e.g. "07:30-08:00"
}

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

export interface LeaveTypeBreakdown {
  count: number;
  leave_type: string; // e.g. "Sick", "Annual", "Maternity"
}

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

export interface LatenessByDay {
  avg_late_minutes: number;
  day: string; // e.g. "Mon", "Tue", ...
  late_count: number;
}

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

export interface RecruitmentSourceBreakdown {
  count: number;
  source_name: string;
  source_type: string;
}

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

export interface TurnoverTenureBucket {
  bucket: string;
  count: number;
}

export interface TurnoverBySource {
  count: number;
  source_name: string;
  source_type: string;
}

export interface TurnoverByDepartment {
  count: number;
  department_name: string;
}

export interface TurnoverAnalytics {
  by_department: TurnoverByDepartment[];
  by_source: TurnoverBySource[];
  period_end: string;
  period_start: string;
  tenure_distribution: TurnoverTenureBucket[];
  total_terminated: number;
}

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
