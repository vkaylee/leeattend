// Shared types for dashboard hooks.
// Split out from use-dashboard.ts to keep the hook file under max-lines.

/** A percentage rate plus its raw numerator/denominator. `value` is null when the denominator is zero. */
export interface RateQuantity {
  denominator: number;
  numerator: number;
  unit: string;
  value: string | null;
}

export interface OrganizationStats {
  absence_rate: RateQuantity;
  absent_today: number;
  attendance_rate: RateQuantity;
  avg_work_hours_today: number;
  early_departure_today: number;
  estimated_monthly_payroll: string;
  late_rate: RateQuantity;
  late_today: number;
  on_leave_today: number;
  pending_leave_requests: number;
  present_today: number;
  total_employees: number;
}

export interface AttendanceTrendPoint {
  absence_rate: RateQuantity;
  absent: number;
  attendance_rate: RateQuantity;
  date: string;
  eligible: number;
  late: number;
  late_rate: RateQuantity;
  on_leave: number;
  present: number;
}

export interface DepartmentAttendancePoint {
  absence_rate: RateQuantity;
  absent: number;
  attendance_rate: RateQuantity;
  department_id: string;
  department_name: string;
  late: number;
  late_rate: RateQuantity;
  on_leave: number;
  on_time: number;
  total_employees: number;
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
  delta_percentage_points: string | null;
  period_end: string;
  period_start: string;
  rate: TurnoverRate | null;
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
  average_headcount: number;
  by_department: TurnoverByDepartment[];
  by_employment_type: TurnoverByEmploymentType[];
  by_location: TurnoverByLocation[];
  by_manager: TurnoverByManager[];
  by_source: TurnoverBySource[];
  classification: TurnoverClassification[];
  cohort_retention: CohortRetentionRow[];
  comparison: TurnoverComparison | null;
  early_attrition: TurnoverRate;
  involuntary_count: number;
  monthly_trend: TurnoverTrendPoint[];
  period_end: string;
  period_start: string;
  regrettable_attrition: string | null;
  retention_rate: string | null;
  target: TurnoverTarget;
  tenure_distribution: TurnoverTenureBucket[];
  termination_reasons: TurnoverReasonRow[];
  total_terminated: number;
  turnover_rate: TurnoverRate;
  unknown_classification_count: number;
  voluntary_count: number;
  warnings: TurnoverWarning[];
}

/** One source row in the recruitment effectiveness ranking. */
export interface RecruitmentEffectivenessSource {
  cost_per_hire: string | null;
  cost_per_hire_available: boolean;
  hires: number;
  hires_ratio: string | null;
  retained_90d: number;
  retention_90d: string | null;
  source_id: string | null;
  source_name: string;
  source_type: string;
  turnover_in_range: number;
}

/** One stage of the honest conversion funnel. `count` null = no data source. */
export interface RecruitmentFunnelStage {
  conversion_rate: string | null;
  count: number | null;
  data_source: string | null;
  stage: string;
}

/** Overview KPIs for the recruitment effectiveness dashboard. */
export interface RecruitmentEffectivenessOverview {
  average_cost_per_hire: string | null;
  comparison_hires: number | null;
  comparison_hires_delta: number | null;
  cost_available: boolean;
  hires: number;
  retained_90d: number;
  retention_90d: string | null;
  turnover_in_range: number;
}

/** One month bucket in the monthly hires trend. */
export interface RecruitmentTrendPoint {
  hires: number;
  month: string;
}

/** One unavailable metric — a self-describing data-availability gap. */
export interface RecruitmentDataGap {
  metric: string;
  reason: string;
}

/** Combined recruitment effectiveness analytics response. */
export interface RecruitmentEffectivenessAnalytics {
  by_source: RecruitmentEffectivenessSource[];
  data_gaps: RecruitmentDataGap[];
  funnel: RecruitmentFunnelStage[];
  hires_trend: RecruitmentTrendPoint[];
  overview: RecruitmentEffectivenessOverview;
  period_end: string;
  period_start: string;
  total_hires: number;
  warnings: TurnoverWarning[];
}

/** One department row in the overtime-breakdown endpoint. */
export interface OvertimeBreakdownRow {
  approved_ot_hours: number;
  cost_available: boolean;
  department_id: string;
  department_name: string;
  overtime_cost: string | null;
  system_ot_hours: number;
  unapproved_ot_hours: number;
}

/** Envelope returned by `GET /api/v1/dashboard/overtime-breakdown`. */
export interface OvertimeBreakdownResponse {
  contract_version: "overtime-breakdown.v2";
  generated_at: string;
  organization: {
    timezone: string;
    currency: string;
  };
  rows: OvertimeBreakdownRow[];
}
