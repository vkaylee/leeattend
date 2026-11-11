// Shared types for dashboard hooks.
// Split out from use-dashboard.ts to keep the hook file under max-lines.

/** A percentage rate plus its raw numerator/denominator. `value` is null when the denominator is zero. */
export interface RateQuantity {
  value: string | null;
  numerator: number;
  denominator: number;
  unit: string;
}

export interface OrganizationStats {
  absent_today: number;
  avg_work_hours_today: number;
  early_departure_today: number;
  estimated_monthly_payroll: string;
  late_today: number;
  on_leave_today: number;
  pending_leave_requests: number;
  present_today: number;
  total_employees: number;
  attendance_rate: RateQuantity;
  absence_rate: RateQuantity;
  late_rate: RateQuantity;
}

export interface AttendanceTrendPoint {
  absent: number;
  date: string;
  late: number;
  on_leave: number;
  present: number;
  eligible: number;
  attendance_rate: RateQuantity;
  absence_rate: RateQuantity;
  late_rate: RateQuantity;
}

export interface DepartmentAttendancePoint {
  absent: number;
  department_id: string;
  department_name: string;
  late: number;
  on_leave: number;
  on_time: number;
  total_employees: number;
  attendance_rate: RateQuantity;
  absence_rate: RateQuantity;
  late_rate: RateQuantity;
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

export interface TurnoverRate {
  denominator: number;
  numerator: number;
  unit: string;
  value: string | null;
}

export interface TurnoverComparison {
  basis: "none" | "previous_period" | "same_period_last_year" | "target" | "budget" | "benchmark";
  period_start: string;
  period_end: string;
  rate: TurnoverRate | null;
  delta_percentage_points: string | null;
}

export interface TurnoverWarning {
  code: string;
  message: string;
}

export interface TurnoverReasonRow {
  count: number;
  reason: string;
}

export interface TurnoverClassification {
  classification: string;
  count: number;
}

export interface TurnoverByEmploymentType {
  count: number;
  employment_type: string;
}

export interface TurnoverByLocation {
  count: number;
  location_name: string;
}

export interface TurnoverByManager {
  count: number;
  manager_name: string;
}

export interface TurnoverTrendPoint {
  headcount: number;
  leavers: number;
  month: string;
  turnover_rate: TurnoverRate;
}

export interface CohortRetentionRow {
  active_count: number;
  cohort_month: string;
  cohort_size: number;
  months_since_hire: number;
  retention_rate: string | null;
}

export interface TurnoverTarget {
  operator: string;
  source: string;
  value: string;
}

export interface TurnoverAnalytics {
  by_department: TurnoverByDepartment[];
  by_source: TurnoverBySource[];
  period_end: string;
  period_start: string;
  tenure_distribution: TurnoverTenureBucket[];
  total_terminated: number;
  turnover_rate: TurnoverRate;
  average_headcount: number;
  voluntary_count: number;
  involuntary_count: number;
  unknown_classification_count: number;
  retention_rate: string | null;
  early_attrition: TurnoverRate;
  regrettable_attrition: string | null;
  comparison: TurnoverComparison | null;
  target: TurnoverTarget;
  monthly_trend: TurnoverTrendPoint[];
  cohort_retention: CohortRetentionRow[];
  termination_reasons: TurnoverReasonRow[];
  classification: TurnoverClassification[];
  by_employment_type: TurnoverByEmploymentType[];
  by_location: TurnoverByLocation[];
  by_manager: TurnoverByManager[];
  warnings: TurnoverWarning[];
}

/** One source row in the recruitment effectiveness ranking. */
export interface RecruitmentEffectivenessSource {
  source_id: string | null;
  source_type: string;
  source_name: string;
  hires: number;
  hires_ratio: string | null;
  retained_90d: number;
  retention_90d: string | null;
  cost_per_hire: string | null;
  cost_per_hire_available: boolean;
  turnover_in_range: number;
}

/** One stage of the honest conversion funnel. `count` null = no data source. */
export interface RecruitmentFunnelStage {
  stage: string;
  count: number | null;
  data_source: string | null;
  conversion_rate: string | null;
}

/** Overview KPIs for the recruitment effectiveness dashboard. */
export interface RecruitmentEffectivenessOverview {
  hires: number;
  average_cost_per_hire: string | null;
  cost_available: boolean;
  retained_90d: number;
  retention_90d: string | null;
  turnover_in_range: number;
  comparison_hires: number | null;
  comparison_hires_delta: number | null;
}

/** One month bucket in the monthly hires trend. */
export interface RecruitmentTrendPoint {
  month: string;
  hires: number;
}

/** One unavailable metric — a self-describing data-availability gap. */
export interface RecruitmentDataGap {
  metric: string;
  reason: string;
}

/** Combined recruitment effectiveness analytics response. */
export interface RecruitmentEffectivenessAnalytics {
  period_start: string;
  period_end: string;
  total_hires: number;
  overview: RecruitmentEffectivenessOverview;
  funnel: RecruitmentFunnelStage[];
  by_source: RecruitmentEffectivenessSource[];
  hires_trend: RecruitmentTrendPoint[];
  data_gaps: RecruitmentDataGap[];
  warnings: TurnoverWarning[];
}

/** One department row in the overtime-breakdown endpoint. */
export interface OvertimeBreakdownRow {
  department_id: string;
  department_name: string;
  system_ot_hours: number;
  approved_ot_hours: number;
  unapproved_ot_hours: number;
  overtime_cost: string | null;
  cost_available: boolean;
}

/** Envelope returned by `GET /api/v1/dashboard/overtime-breakdown`. */
export interface OvertimeBreakdownResponse {
  contract_version: "overtime-breakdown.v2";
  organization: {
    timezone: string;
    currency: string;
  };
  rows: OvertimeBreakdownRow[];
  generated_at: string;
}
