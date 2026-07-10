import { components } from "@/types/api-generated";

// Pure 1:1 schema aliases moved to ./api-generated-reexports.
// Types.ts now only carries overrides (Omit/&), hand-written
// interfaces, and schema gaps (Group B/C).
// Anchor aliases: Group A names referenced by overrides below.
// Kept here (not in the barrel) so they can be used in local scope.
export type Department = components["schemas"]["Department"];
export type Employee = components["schemas"]["Employee"];
export type EmployeeProfile = components["schemas"]["EmployeeProfile"];
export type EmployeeSalaryComponent = components["schemas"]["EmployeeSalaryComponent"];
export type Employment = components["schemas"]["Employment"];
export type IncentiveEvaluationResult = components["schemas"]["IncentiveEvaluationResult"];
export type MemberExtraRole = components["schemas"]["MemberExtraRole"];
export type SalaryComponentWithDefinition = components["schemas"]["SalaryComponentWithDefinition"];
export type Shift = components["schemas"]["Shift"];
export type TimeEntry = components["schemas"]["TimeEntry"];
export type WorkerHeartbeat = components["schemas"]["WorkerHeartbeat"];
export type WorkingPatternDetailRequest = components["schemas"]["WorkingPatternDetailRequest"];
export type Workspace = components["schemas"]["Workspace"];

export * from "./api-generated-reexports";

export interface DeviceInfo {
  device_time: string;
  firmware?: string;
  hw_name?: string;
  last_info_update?: string;
  last_sync?: string;
  logs_capacity?: number;
  logs_count: number;
  mac?: string;
  platform?: string;
  serial?: string;
  users_capacity?: number;
  users_count: number;
}


export interface User {
  avatar_url?: string | null;
  bio?: string | null;
  created_at?: string;
  email: string;
  full_name?: string | null;
  id: string;
  preferred_language?: string | null;
  system_roles?: string[];
}


export interface WorkspaceMember {
  avatar_url?: string | null;
  email: string;
  extra_role_ids?: string[];
  full_name?: string | null;
  role_id: string;
  user_id: string;
  workspace_id: string;
}

export enum RoleScopeType {
  Global = 'global',
  Scoped = 'scoped',
}

export type WorkspaceRole = Omit<components["schemas"]["WorkspaceRole"], "description"> & {
  description?: string | null;
};

// --- Timekeeping Models ---
export type AttendanceEntry = TimeEntry;

export interface RawAttendanceLog {
  created_at: string;
  device_id?: string;
  device_name?: string;
  employee_full_name?: string;
  employee_id?: string;
  employee_id_number?: string;
  employment_id: string;
 
  id: string;
  punch_time: string;
  raw_data?: unknown;
  source: string;
  workspace_id: string;
}

export type LeaveDurationType = 'full_day' | 'first_half' | 'second_half' | 'custom_hours';

export interface EmployeeLeaveBalance {
  accrued_amount: number;
  carried_over: number;
  entitlement: number;
  is_paid?: boolean;
  leave_type?: string;
  leave_type_code?: string;
  leave_type_id?: string;
  leave_type_name?: string;
  remaining: number;
  used: number;
}

export type AttendanceAdjustmentRequest = Omit<components["schemas"]["AttendanceAdjustmentRequest"], "requested_punch_in_time" | "requested_punch_out_time"> & { 
  requested_date?: string;
  requested_punch_time: string;
  requested_punch_in_time?: string;
  requested_punch_out_time?: string | null;
  is_break_overtime?: boolean;
};

export interface PaginatedResponse<T> {
  data: T[];
  limit: number;
  page: number;
  total: number;
}

export type AttendanceReportSort =
  | "name_asc"
  | "name_desc"
  | "work_hours_asc"
  | "work_hours_desc"
  | "leave_days_asc"
  | "leave_days_desc"
  | "ot_hours_asc"
  | "ot_hours_desc"
  | "total_hours_asc"
  | "total_hours_desc";

export interface AttendanceSummary {
  computed_ot_hours: number;
  computed_std_hours: number;
  employee_id: string;
  employee_id_number?: string;
  employee_name: string;
  employer_id?: string;
  employer_name?: string;
  employment_id: string;
  total_leave_days: number;
  total_ot_units: number;
  total_std_units: number;
  unit_coefficient_hours: number;
  unit_type: 'DAY' | 'SHIFT' | 'HOUR' | 'MINUTE';
}

export type AuditLog = Omit<components["schemas"]["AuditLog"], "actor_id" | "target_user_id"> & {
  actor_id?: string;
  target_user_id?: string;
};


export interface DailyAttendanceReport {
  actual_punch_in: string | null;
  actual_punch_out: string | null;
  break_duration_minutes: number | null;
  break_end: string | null;
  break_start: string | null;
  computed_ot_hours: number;
  computed_std_hours: number;
  date: string;
  employee_id: string;
  employee_name: string;
  employer_name?: string | null;
  employment_id: string;
  generation_metadata?: unknown;
  holiday_name?: string | null;
  interval_id?: string | null;
  interval_name?: string | null;
  is_break_auto_deduct: boolean | null;
  is_holiday: boolean;
  is_leave_paid?: boolean | null;
  is_unscheduled?: boolean;
  ot_claim_status?: string | null;
  ot_work_units: number;
  scheduled_end: string | null;
  scheduled_start: string | null;
  grace_period_minutes?: number | null;
  segment_count?: number | null;
  // Segment fields (segment-centric report path)
  segment_id?: string | null;
  segment_type?: string | null;
  shift_id?: string | null;
  shift_name?: string | null;
  shift_symbol?: string | null;
  sort_order?: number | null;
  status: string;
  std_work_units: number;
  unit_coefficient_hours: number;
  unit_type: 'DAY' | 'SHIFT' | 'HOUR' | 'MINUTE';
}

// --- Payroll and Shift Models ---

export interface ConfigRecord {
  created_at?: string;
  effective_date: string;
  hours_per_day: number;
  id?: string;
  is_virtual?: boolean;
  max_ot_hours_per_day?: number | null;
  max_ot_hours_per_month?: number | null;
  std_working_days: number;
}

export type EmployeeSalaryComponentDetail = EmployeeSalaryComponent & {
  name: string;
  code: string;
};

export interface PayrollSegment {
  actual_work_days: string | number;
  amount: string | number;
  applied_base_salary: string | number;
  applied_hours_per_day: string | number;
  applied_std_days: string | number;
  department_id?: string | null;
  end_date: string;
  id: string;
  paid_work_days: string | number;
  reason: string;
  start_date: string;
  std_days_source?: string | null;
}

export type Payslip = components["schemas"]["Payslip"] & { 
  incentives?: IncentiveEvaluationResult[];
  segments?: PayrollSegment[];
  salary_components?: SalaryComponentWithDefinition[];
};



export interface RuleCondition {
  metric: string;
  operator: string;
  value: number;
}

// --- Department Models ---

export interface ReportExportJobProgress {
  completed_batches: number;
  failed_batches: number;
  percentage: number;
  total_batches: number;
}

export interface ReportExportJob {
  completed_at?: string;
  created_at: string;
  error?: string;
  job_id: string;
  progress: ReportExportJobProgress;
  result_url?: string;
  status: "pending" | "processing" | "completed" | "failed";
}

export interface CreateReportExportRequest {
  employee_ids?: string[];
  end_date: string;
  report_type: "summary" | "detail";
  start_date: string;
}

export interface CreateReportExportResponse {
  estimated_seconds: number;
  job_id: string;
  status: string;
  status_url: string;
  total_batches: number;
}

export interface WorkerMonitorStats {
  active_workers: number;
  completed_today: number;
  failed_today: number;
  idle_workers: number;
  job_type_stats: JobTypeStats[];
  pending_high: number;
  pending_low: number;
  pending_tasks: number;
  recent_jobs: RecentJob[];
  total_workers: number;
  workers: WorkerHeartbeat[];
}

export interface JobTypeStats {
  completed: number;
  failed: number;
  pending: number;
  processing: number;
  task_type: string;
  total: number;
}

export interface MonitorJobsResponse {
  jobs: RecentJob[];
  next_cursor?: number | null;
}

export interface RecentJob {
  completed_at?: string | null;
  completed_batches: number;
  created_at: string;
  error_message?: string | null;
  failed_batches: number;
  job_id: string;
  payload?: unknown;
  report_type?: string | null;
  requester_id?: string | null;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  task_type?: string | null;
  total_batches: number;
  workspace_id: string;
}


export interface MemberRolesResponse {
  extra: MemberExtraRole[];
  primary: string;
}


export interface EmploymentPeriod {
  created_at: string;
  employee_id: string;
  hire_date: string;
  id: string;
  is_rehire: boolean;
  termination_date?: string;
  updated_at: string;
}


export interface LeavePolicy {
  created_at: string;
  description?: string;
  id: string;
  leave_type_id: string;
 
  name: string;
  rule_config: LeavePolicyConfig;
  updated_at: string;
  workspace_id: string;
}

export type AccrualFrequency = "monthly" | "quarterly" | "yearly";

export interface ProcessResponse {
  logs_deleted: number;
  logs_processed: number;
}

export * from "./contract-types";

export type SeniorityBonus = Omit<components["schemas"]["SeniorityBonus"], "bonus_amount" | "months"> & {
  bonus_amount?: number;
  months?: number;
  min_years?: number;
  extra_days?: number;
};

export type LeavePolicyConfig = Omit<components["schemas"]["LeavePolicyConfig"], "seniority_bonus"> & {
  seniority_bonus?: SeniorityBonus[];
};
export type EmployeeSchedule = Omit<components["schemas"]["EmployeeSchedule"], "employee_id_number"> & {
  employee_id_number?: string;
  employee_id?: string;
};
// --- Assignment (Secondment) Models ---

export const AssignmentStatus = {
  Scheduled: "scheduled",
  Active: "active",
  Completed: "completed",
  Cancelled: "cancelled",
} as const;
export type AssignmentStatus = typeof AssignmentStatus[keyof typeof AssignmentStatus];

export type EmployeeAssignment = Omit<components["schemas"]["EmployeeAssignment"], "status"> & { status: AssignmentStatus };
export type UpdateEmployeeAssignmentRequest = components["schemas"]["UpdateEmployeeAssignmentRequest"] & { host_location_id?: string | null };
// --- Employee Self-Service (ESS) Types ---
export type MyProfileResponse = Omit<components["schemas"]["MyProfileResponse"], "profile"> & {
  profile?: components["schemas"]["EmployeeProfile"] | null;
};

export interface UpdateProfileRequest {
  address?: string;
  bank_account_number?: string;
  bank_branch?: string;
  bank_name?: string;
  date_of_birth?: string;
  gender?: string;
  national_id?: string;
  phone?: string;
}

// --- Working Pattern Models ---
export type WorkingPattern = Omit<components["schemas"]["WorkingPattern"], "description"> & {
  description?: string;
};

export interface UpdateWorkingPatternRequest {

  cycle_length_days?: number;

  description?: string;

  details: WorkingPatternDetailRequest[];

  name: string;

  pattern_type?: string;

}

export type WorkingCondition = Omit<components["schemas"]["WorkingCondition"], "type_"> & { type: string };
// --- Payroll Models ---
export type LeavePolicyAssignmentDetail = Omit<components["schemas"]["LeavePolicyAssignmentDetail"], "effective_to"> & {
  effective_to: string | null;
};
export type OtStatus = "none" | "pending_employee" | "pending_manager" | "approved" | "rejected";
export type AttendanceDailySummary = Omit<components["schemas"]["AttendanceDailySummary"], "entered_via" | "ot_reason"> & {
  entered_via?: string | null;
  ot_reason?: string | null;
};
// --- Payroll Rules ---
export type PayrollRuleAssignment = Omit<components["schemas"]["PayrollRuleAssignment"], "override_config"> & {
  override_config?: unknown;
};
// --- Employment Phases ---
export interface EmploymentPhaseRuleConfig {
  duration_days?: number;
}
export type EmploymentPhaseDefinition = Omit<components["schemas"]["EmploymentPhaseDefinition"], "rule_config"> & {
  rule_config?: EmploymentPhaseRuleConfig;
};
export type EmployeePhase = Omit<components["schemas"]["EmployeePhase"], "rule_config"> & {
  rule_config?: EmploymentPhaseRuleConfig;
};
// --- Workspace Config (EAV) ---

export interface CreateConfigEntryRequest {
  config_key: string;
  effective_from: string;
  effective_to?: string | null;
  scope_id?: string | null;
  scope_type: "workspace" | "employer" | "department" | "employee";
  value_bool?: boolean | null;
  value_decimal?: number | null;
  value_int?: number | null;
  value_json?: Record<string, unknown> | null;
  value_text?: string | null;
}

// --- Approval Workflows ---
export type ApproverType = "department_manager" | "role" | "permission" | "specific_user";

export interface ApprovalWorkflowStep {
  action_required: "approve" | "review" | "acknowledge";
  approver_config: Record<string, unknown>;
  approver_type: ApproverType;
  can_skip: boolean;
  created_at: string;
  id: string;
  step_name: string;
  step_order: number;
  workflow_id: string;
}

export type ApprovalWorkflow = components["schemas"]["ApprovalWorkflow"] & {
  steps?: ApprovalWorkflowStep[];
};
// --- Overtime Requests ---
export type OtRequestStatus = "draft" | "pending_approval" | "approved" | "partially_accepted" | "active" | "completed" | "cancelled" | "rejected";
export type ConsentStatus = "pending" | "accepted" | "declined";

// --- Custom Fields Models ---

// --- Lifecycle Onboarding/Offboarding Models ---

// ── Work Segments ──────────────────────────────────────────────────────────

export interface WorkSegment {
  break_config_override?: Record<string, unknown> | null;
  compensation_rate: number;
  created_at: string;
  date: string;
  employment_id: string;
  gap_compensation_before: 'unpaid' | 'standby' | 'break';
  gap_compensation_rate_before: number;
  generation_metadata: Record<string, unknown>;
  id: string;
  interval_id?: string | null;
  notes?: string | null;
  ot_config_override?: Record<string, unknown> | null;
  scheduled_end: string;
  scheduled_start: string;
  segment_status: 'scheduled' | 'in_progress' | 'completed' | 'cancelled';
  segment_type: 'work' | 'standby' | 'on_call' | 'training' | 'travel';
  shift_id: string;
  sort_order: number;
  source: 'roster' | 'pattern' | 'rule' | 'fixed' | 'manual' | 'auto';
  source_detail: Record<string, unknown>;
  updated_at: string;
  workspace_id: string;
}

export interface CreateWorkSegmentRequest {
  break_config_override?: Record<string, unknown> | null;
  compensation_rate?: number;
  date: string;
  employment_id: string;
  gap_compensation_before?: string;
  gap_compensation_rate_before?: number;
  interval_id?: string | null;
  notes?: string | null;
  ot_config_override?: Record<string, unknown> | null;
  scheduled_end: string;
  scheduled_start: string;
  segment_status?: string;
  segment_type?: string;
  shift_id: string;
  sort_order?: number;
  source?: string;
  source_detail?: Record<string, unknown>;
}

export interface UpdateWorkSegmentRequest {
  break_config_override?: Record<string, unknown> | null;
  compensation_rate?: number | null;
  gap_compensation_before?: string | null;
  gap_compensation_rate_before?: number | null;
  interval_id?: string | null;
  notes?: string | null;
  ot_config_override?: Record<string, unknown> | null;
  scheduled_end?: string | null;
  scheduled_start?: string | null;
  segment_status?: string | null;
  segment_type?: string | null;
  shift_id?: string | null;
  sort_order?: number | null;
  source?: string | null;
  source_detail?: Record<string, unknown> | null;
}

export interface BulkCreateWorkSegmentsRequest {
  replace_existing?: boolean;
  segments: CreateWorkSegmentRequest[];
}

// ── Shift Default Intervals ─────────────────────────────────────────────────

export interface ShiftDefaultInterval {
  allow_early_ot: boolean;
  break_duration_minutes: number;
  break_end?: string | null;
  break_is_overtime: boolean;
  break_start?: string | null;
  created_at: string;
  early_punch_in_grace_period_minutes: number;
  end_time: string;
  id: string;
  interval_name?: string | null;
  is_break_auto_deduct: boolean;
  is_night_shift: boolean;
  max_early_ot_minutes?: number | null;
  ot_aggregation_mode: string;
  shift_id: string;
  sort_order: number;
  start_time: string;
  updated_at: string;
}

export interface CreateShiftIntervalRequest {
  allow_early_ot?: boolean;
  break_duration_minutes?: number;
  break_end?: string | null;
  break_is_overtime?: boolean;
  break_start?: string | null;
  early_punch_in_grace_period_minutes?: number;
  end_time: string;
  interval_name?: string | null;
  is_break_auto_deduct?: boolean;
  is_night_shift?: boolean;
  max_early_ot_minutes?: number | null;
  ot_aggregation_mode?: string;
  sort_order?: number;
  start_time: string;
}

export interface UpdateShiftIntervalRequest {
  allow_early_ot?: boolean;
  break_duration_minutes?: number;
  break_end?: string | null;
  break_is_overtime?: boolean;
  break_start?: string | null;
  early_punch_in_grace_period_minutes?: number;
  end_time?: string | null;
  interval_name?: string | null;
  is_break_auto_deduct?: boolean;
  is_night_shift?: boolean;
  max_early_ot_minutes?: number | null;
  ot_aggregation_mode?: string;
  sort_order?: number | null;
  start_time?: string | null;
}

// --- Termination Import Row Types (from openapi-generated schemas) ---


