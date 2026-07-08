// Shared types for dashboard hooks.
// Split out from use-dashboard.ts to keep the hook file under max-lines.

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

export interface DepartmentAttendancePoint {
  absent: number;
  department_id: string;
  department_name: string;
  late: number;
  on_leave: number;
  on_time: number;
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

export interface OvertimeTrendPoint {
  approved_ot_hours: number;
  date: string;
  system_ot_hours: number;
}

export interface PeakTimeSlot {
  check_in: number;
  check_out: number;
  time_slot: string;
}

export interface LeaveTypeBreakdown {
  count: number;
  leave_type: string;
}

export interface LatenessByDay {
  avg_late_minutes: number;
  day: string;
  late_count: number;
}

export interface RecruitmentSourceBreakdown {
  count: number;
  source_name: string;
  source_type: string;
}

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
