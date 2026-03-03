import { useQuery } from "@tanstack/react-query";
import { useWorkspace } from "./use-workspace";
import { apiFetch } from "@/lib/api";

export interface WorkspaceStats {
  total_employees: number;
  present_today: number;
  pending_leave_requests: number;
  estimated_monthly_payroll: string;
  late_today: number;
  absent_today: number;
  on_leave_today: number;
  early_departure_today: number;
  avg_work_hours_today: number;
}

export interface AttendanceTrendPoint {
  date: string;
  present: number;
  late: number;
  absent: number;
  on_leave: number;
}

export function useDashboardStats() {
  const { workspace } = useWorkspace();

  return useQuery<WorkspaceStats>({
    queryKey: ["workspace-stats", workspace?.id],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(`/dashboard/workspace-stats`);

      if (!response.ok) {
        throw new Error("Failed to fetch dashboard stats");
      }

      return response.json();
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

      const response = await apiFetch(`/dashboard/attendance-trend`);

      if (!response.ok) {
        throw new Error("Failed to fetch attendance trend");
      }

      return response.json();
    },
    enabled: !!workspace,
  });
}

export interface DepartmentAttendancePoint {
  department_id: string;
  department_name: string;
  on_time: number;
  late: number;
  absent: number;
  on_leave: number;
}

export function useDepartmentAttendance(range: "today" | "week" = "today") {
  const { workspace } = useWorkspace();

  return useQuery<DepartmentAttendancePoint[]>({
    queryKey: ["department-attendance", workspace?.id, range],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(
        `/dashboard/department-attendance?range=${range}`
      );

      if (!response.ok) {
        throw new Error("Failed to fetch department attendance");
      }

      return response.json();
    },
    enabled: !!workspace,
  });
}

export interface UnexcusedAbsencesPoint {
  department_id: string;
  department_name: string;
  unexcused_count: number;
  consecutive_count: number;
  total_absent_count: number;
  total_leave_count: number;
}

export function useUnexcusedAbsences(startDate?: string, endDate?: string) {
  const { workspace } = useWorkspace();

  return useQuery<UnexcusedAbsencesPoint[]>({
    queryKey: ["unexcused-absences", workspace?.id, startDate, endDate],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const params = new URLSearchParams();
      if (startDate) params.set("start_date", startDate);
      if (endDate) params.set("end_date", endDate);
      const qs = params.toString();
      const response = await apiFetch(`/dashboard/unexcused-absences${qs ? `?${qs}` : ""}`);

      if (!response.ok) {
        throw new Error("Failed to fetch unexcused absences");
      }

      return response.json();
    },
    enabled: !!workspace,
  });
}

export interface OvertimeTrendPoint {
  date: string;
  system_ot_hours: number;
  approved_ot_hours: number;
}

export function useOvertimeTrend() {
  const { workspace } = useWorkspace();

  return useQuery<OvertimeTrendPoint[]>({
    queryKey: ["overtime-trend", workspace?.id],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(`/dashboard/overtime-trend`);

      if (!response.ok) {
        throw new Error("Failed to fetch overtime trend");
      }

      return response.json();
    },
    enabled: !!workspace,
  });
}

// ── Peak Check-in/Check-out Times ──────────────────────────

export interface PeakTimeSlot {
  time_slot: string; // e.g. "07:30-08:00"
  check_in: number;
  check_out: number;
}

export function usePeakTimes() {
  const { workspace } = useWorkspace();

  return useQuery<PeakTimeSlot[]>({
    queryKey: ["peak-times", workspace?.id],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(`/dashboard/peak-times`);

      if (!response.ok) {
        throw new Error("Failed to fetch peak times");
      }

      return response.json();
    },
    enabled: !!workspace,
  });
}

// ── Leave Types Breakdown ──────────────────────────────────

export interface LeaveTypeBreakdown {
  leave_type: string; // e.g. "Sick", "Annual", "Maternity"
  count: number;
}

export function useLeaveTypesBreakdown() {
  const { workspace } = useWorkspace();

  return useQuery<LeaveTypeBreakdown[]>({
    queryKey: ["leave-types-breakdown", workspace?.id],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(`/dashboard/leave-types`);

      if (!response.ok) {
        throw new Error("Failed to fetch leave types breakdown");
      }

      return response.json();
    },
    enabled: !!workspace,
  });
}

// ── Lateness by Day of Week ────────────────────────────────

export interface LatenessByDay {
  day: string; // e.g. "Mon", "Tue", ...
  late_count: number;
  avg_late_minutes: number;
}

export function useLatenessByDay() {
  const { workspace } = useWorkspace();

  return useQuery<LatenessByDay[]>({
    queryKey: ["lateness-by-day", workspace?.id],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(`/dashboard/lateness-by-day`);

      if (!response.ok) {
        throw new Error("Failed to fetch lateness by day");
      }

      return response.json();
    },
    enabled: !!workspace,
  });
}

// ── Recruitment Source Breakdown ──────────────────────────────────

export interface RecruitmentSourceBreakdown {
  source_type: string;
  source_name: string;
  count: number;
}

export function useRecruitmentSourceBreakdown() {
  const { workspace } = useWorkspace();

  return useQuery<RecruitmentSourceBreakdown[]>({
    queryKey: ["recruitment-sources-breakdown", workspace?.id],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(`/dashboard/recruitment-sources`);

      if (!response.ok) {
        throw new Error("Failed to fetch recruitment source breakdown");
      }

      return response.json();
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
  source_type: string;
  source_name: string;
  count: number;
}

export interface TurnoverByDepartment {
  department_name: string;
  count: number;
}

export interface TurnoverAnalytics {
  tenure_distribution: TurnoverTenureBucket[];
  by_source: TurnoverBySource[];
  by_department: TurnoverByDepartment[];
  total_terminated: number;
  period_start: string;
  period_end: string;
}

export function useTurnoverAnalytics(startDate: string, endDate: string) {
  const { workspace } = useWorkspace();

  return useQuery<TurnoverAnalytics>({
    queryKey: ["turnover-analytics", workspace?.id, startDate, endDate],
    queryFn: async () => {
      if (!workspace) throw new Error("No workspace selected");

      const response = await apiFetch(
        `/dashboard/turnover-analytics?start=${startDate}&end=${endDate}`
      );

      if (!response.ok) {
        throw new Error("Failed to fetch turnover analytics");
      }

      return response.json();
    },
    enabled: !!workspace && !!startDate && !!endDate,
  });
}
