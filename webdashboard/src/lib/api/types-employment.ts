export interface EmploymentPeriod {
  created_at: string;
  employee_id: string;
  hire_date: string;
  id: string;
  is_rehire: boolean;
  termination_date?: string;
  updated_at: string;
}

export const AssignmentStatus = {
  Scheduled: "scheduled",
  Active: "active",
  Completed: "completed",
  Cancelled: "cancelled",
} as const;
export type AssignmentStatus = typeof AssignmentStatus[keyof typeof AssignmentStatus];

export interface EmployeeAssignment {
  created_at: string;
  employment_id: string;
  end_date?: string | null;
  home_location_id: string;
  host_location_id: string;
  id: string;
  reason?: string | null;
  start_date: string;
  status: AssignmentStatus;
  updated_at: string;
}

export interface UpdateEmployeeAssignmentRequest {
  end_date?: string | null;
  host_location_id?: string | null;
  reason?: string | null;
  status?: AssignmentStatus | null;
}

export interface ScheduleTrace {
  priority: number;
  reason?: string | null;
  rule_id?: string | null;
  rule_name?: string;
  status: string;
}

export interface EffectiveSchedule {
  allow_early_ot?: boolean | null;
  break_duration_minutes?: number | null;
  break_end?: string | null;
  break_is_overtime?: boolean | null;
  break_start?: string | null;
  closed?: boolean;
  date: string;
  early_punch_in_grace_period_minutes?: number | null;
  end_time?: string | null;
  holiday_name?: string | null;
  is_break_auto_deduct?: boolean | null;
  is_holiday?: boolean;
  is_holiday_paid?: boolean;
  max_early_ot_minutes?: number | null;
  provisional?: boolean;
  published_at?: string | null;
  revision_id?: string | null;
  revision_number?: number | null;
  schedule_status?: string | null;
  shift_id?: string | null;
  shift_name?: string | null;
  shift_symbol?: string | null;
  source: string;
  start_time?: string | null;
  timezone?: string | null;
  traces?: ScheduleTrace[];
}

export interface ScheduleCoverage {
  complete: boolean;
  expected_days: number;
  missing_official_days: number;
  published_off_days: number;
  published_work_days: number;
}

export interface EmployeeSchedule {
  coverage?: ScheduleCoverage;
  department_name?: string | null;
  employee_id?: string;
  employee_id_number?: string;
  employment_id: string;
  full_name: string;
  schedules: EffectiveSchedule[];
}

export interface WorkingPatternDetailRequest {
  schedule_day: number;
  shift_id?: string | null;
}

export interface WorkingPattern {
  created_at: string;
  cycle_length_days?: number | null;
  description?: string;
  id: string;
  is_active: boolean;
  name: string;
  pattern_type: string;
  updated_at: string;
}

export interface UpdateWorkingPatternRequest {
  cycle_length_days?: number;
  description?: string;
  details: WorkingPatternDetailRequest[];
  name: string;
  pattern_type?: string;
}

export interface WorkingCondition {
  condition_type_id?: string | null;
  employment_id: string;
  end_date?: string | null;
  id: string;
  is_paid: boolean;
  minute_allowance: number;
  start_date: string;
  type: string;
  type_?: string;
}

export interface EmploymentPhaseRuleConfig {
  duration_days?: number;
}

export interface EmploymentPhaseDefinition {
  created_at: string;
  description?: string | null;
  display_order: number;
  id: string;
  is_active: boolean;
  name: string;
  phase_type: string;
  rule_config?: EmploymentPhaseRuleConfig;
  updated_at: string;
}

export interface EmployeePhase {
  completed_at?: string | null;
  created_at: string;
  employment_id: string;
  end_date: string;
  evaluated_by?: string | null;
  evaluation_notes?: string | null;
  id: string;
  override_config?: EmploymentPhaseRuleConfig | Record<string, string | number | boolean | null> | null;
  phase_definition_id: string;
  rule_config?: EmploymentPhaseRuleConfig;
  start_date: string;
  status: string;
  updated_at: string;
}
