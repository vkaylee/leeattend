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

export interface DeviceStatus {
  device_id: string;
  error?: string;
  info?: DeviceInfo;
  last_sync?: string;
  status: "online" | "offline";
}

export interface AgentStatus {
  agent_id: string;
  devices: DeviceStatus[];
  is_online: boolean;
  last_seen: string;
}

export interface Device {
  agent_id: string;
  brand: string;
  created_at: string;
  id: string;
  ip: string;
  is_enabled: boolean;
  last_status: string;
  name: string;
  password?: string;
  port: number;
  timezone: string;
  workspace_id: string;
}

export interface AuthorizedAgent {
  created_at: string;
  id: string;
  is_active: boolean;
  is_trusted: boolean;
  last_ip?: string;
  last_seen_at: string;
  max_concurrent_workers: number;
  name: string;
  retention_days: number;
  success_cooldown_secs: number;
  token: string;
  workspace_id: string;
}

export interface Workspace {
  created_at: string;
  date_format: string;
  default_language: string;
  id: string;
  logo_url?: string;
  name: string;
  rounding_policy: string;
  tagline?: string;
  theme_primary_color: string;
  time_format: string;
  timezone: string;
  work_unit_minutes: number;
}

export interface User {
  avatar_url?: string;
  bio?: string;
  created_at?: string;
  email: string;
  full_name?: string;
  id: string;
  preferred_language?: string;
  system_roles?: string[];
}

export interface UpdateUserRequest {
  avatar_url?: string;
  bio?: string;
  full_name?: string;
}

export interface ChangePasswordRequest {
  new_password: string;
  old_password: string;
}

export interface RegisterRequest {
  email: string;
  full_name: string;
  password: string;
}

export interface WorkspaceMember {
  avatar_url?: string;
  email: string;
  extra_role_ids?: string[];
  full_name?: string;
  role_id: string;
  user_id: string;
  workspace_id: string;
}

export enum RoleScopeType {
  Global = 'global',
  Scoped = 'scoped',
}

export interface WorkspaceRole {
  color: string;
  created_at: string;
  description?: string;
  id: string;
  is_system: boolean;
  name: string;
  scope_type: RoleScopeType;
  updated_at: string;
  workspace_id?: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  user: User;
}

// --- Timekeeping Models ---

export interface Employee {
  created_at: string;
  department_name?: string;
  employee_id_number: string;
  full_name: string;
  hire_date: string;
  id: string;
  is_active: boolean;
  original_hire_date: string;
  termination_date?: string;
  updated_at: string;
  user_id?: string;
  workspace_id: string;
}

export interface TimeEntry {
  created_at: string;
  duration_minutes?: number;
  employee_full_name?: string;
  employee_id: string;
  employee_id_number?: string;
  entry_type: string;
  generation_metadata?: unknown;
  id: string;
 
  is_break_overtime: boolean;
  note?: string;
  punch_in_time: string;
  punch_out_time?: string;
  updated_at: string;
}

export type AttendanceEntry = TimeEntry;

export interface RawAttendanceLog {
  created_at: string;
  device_id?: string;
  device_name?: string;
  employee_full_name?: string;
  employee_id: string;
  employee_id_number?: string;
 
  id: string;
  punch_time: string;
  raw_data?: unknown;
  source: string;
  workspace_id: string;
}

export type LeaveDurationType = 'full_day' | 'first_half' | 'second_half' | 'custom_hours';

export interface LeaveRequest {
  approved_by_user_id?: string;
  created_at: string;
  duration_type: LeaveDurationType;
  employee_id: string;
  end_date: string;
  end_time?: string | null;
  id: string;
  leave_type_id: string;
  reason?: string;
  requested_by_user_id?: string;
  start_date: string;
  start_time?: string | null;
  status: string;
  updated_at: string;
}

export interface EmployeeLeaveBalance {
  accrued_amount: number;
  carried_over: number;
  entitlement: number;
  is_paid: boolean;
  leave_type_code: string;
  leave_type_id: string;
  leave_type_name: string;
  remaining: number;
  used: number;
}

export interface WorkspaceLeaveBalance {
  accrued_amount: number;
  assignment_type?: string;
  carried_over: number;
  employee_id: string;
  employee_name: string;
  entitlement: number;
  is_paid: boolean;
  leave_type_code: string;
  leave_type_id: string;
  leave_type_name: string;
  policy_name?: string;
  remaining: number;
  used: number;
}

export interface AttendanceAdjustmentRequest {
  approved_by_user_id?: string;
  created_at: string;
  employee_id: string;
  id: string;
  is_break_overtime: boolean;
  reason?: string;
  requested_by_user_id?: string;
  requested_date: string;
  requested_punch_in_time: string;
  requested_punch_out_time?: string;
  status: string;
  time_entry_id?: string;
  updated_at: string;
}

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
  total_leave_days: number;
  total_ot_units: number;
  total_std_units: number;
  unit_coefficient_hours: number;
  unit_type: 'DAY' | 'SHIFT' | 'HOUR' | 'MINUTE';
}

export interface AuditLog {
  action: string;
  actor_id?: string;
  created_at: string;
  details?: Record<string, unknown>;
  id: string;
  ip_address?: string;
 
  target_resource_id?: string;
  target_user_id?: string;
  user_agent?: string;
  workspace_id: string;
}

export interface Holiday {
  created_at: string;
  date: string;
  description?: string;
  holiday_type: string;
  id: string;
  is_credit_on_work: boolean;
  is_paid: boolean;
  leave_type_id?: string | null;
  name: string;
  ot_multiplier?: number;
  updated_at: string;
  work_threshold_percent?: number | null;
  workspace_id: string;
}

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
  generation_metadata?: unknown;
  holiday_name?: string | null;
  is_break_auto_deduct: boolean | null;
  is_holiday: boolean;
  is_unscheduled?: boolean;
  ot_claim_status?: string | null;
  ot_work_units: number;
  scheduled_end: string | null;
  scheduled_start: string | null;
  status: string;
  std_work_units: number;
  unit_coefficient_hours: number;
  unit_type: 'DAY' | 'SHIFT' | 'HOUR' | 'MINUTE';
}

// --- Payroll and Shift Models ---

export interface Shift {
  break_duration_minutes: number;
  break_end?: string | null;
  break_start?: string | null;
  created_at: string;
  early_punch_in_grace_period_minutes?: number;
  effective_date?: string;
  end_time: string;
  id: string;
  is_break_auto_deduct?: boolean;
  is_break_overtime?: boolean;
  is_correction?: boolean;
  name: string;
  report_symbol?: string | null;
  start_time: string;
  updated_at: string;
  workspace_id: string;
}

export interface PayrollSettings {
  effective_date: string;
  fixed_std_days: number;
  hours_per_day: number;
  max_ot_hours_per_day?: number | null;
  max_ot_hours_per_month?: number | null;
  std_working_day_mode: "fixed" | "actual_month";
  updated_at: string;
  working_days_per_week: number;
  workspace_id: string;
}

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

 
export interface SalaryComponentDefinition {
  code: string;
  created_at: string;
  id: string;
  name: string;
  updated_at: string;
  workspace_id: string;
}

export interface CreateSalaryComponentRequest {
  base_group_ids?: string[];
  code: string;
  name: string;
}

export interface EmployeeSalaryComponent {
  amount: string;
  code?: string;
  component_definition_id: string;
  created_at: string;
  effective_date: string;
  employee_id: string;
  id: string;
  name?: string;
  updated_at: string;
}

export interface PayrollSegment {
  actual_work_days: number;
  amount: number;
  applied_base_salary: number;
  applied_hours_per_day: number;
  applied_std_days: number;
  department_id?: string;
  end_date: string;
  id: string;
  paid_work_days: number;
  reason: string;
  start_date: string;
  std_days_source?: "individual" | "department" | "workspace" | "system";
}

export interface IncentiveEvaluationResult {
  amount: string;
 
  policy_name: string;
  tier_id?: string;
}



export interface SalaryComponentWithDefinition {
  component?: EmployeeSalaryComponent;
  definition?: SalaryComponentDefinition;
}

 
export interface SalaryBaseGroup {
  created_at: string;
  description: string | null;
  id: string;
  is_system: boolean;
  name: string;
  slug: string;
  updated_at: string;
  workspace_id: string;
}

export interface SalaryBaseGroupMember {
  base_group_id: string;
  component_definition_id: string;
  created_at: string;
}

export interface Payslip {
  base_pay_amount: string;
  created_at: string;
  deductions_amount: string;
  employee_id: string;
  holiday_work_premium_amount: string;
  id: string;
  incentives?: IncentiveEvaluationResult[];
  net_pay_amount: string;
  overtime_pay_amount: string;
  overtime_units: number;
  period_end: string;
  period_start: string;
  salary_components?: SalaryComponentWithDefinition[];
  segments?: PayrollSegment[];
  standard_units: number;
  status: "draft" | "published" | "paid";
  updated_at: string;
}

export interface RuleCondition {
  metric: string;
  operator: string;
  value: number;
}

export interface IncentiveTier {
  id: string;
  payout_amount: number;
  policy_id: string;
  priority: number;
  rule_config?: Record<string, unknown> | RuleCondition[];
}

export interface PayrollIncentivePolicy {
  base_amount: number;
  created_at: string;
  effective_from: string;
  effective_to?: string;
  id: string;
  is_active: boolean;
  max_concurrent_workers: number;
  name: string;
  payout_type: "fixed" | "percentage";
  tiers?: IncentiveTier[];
  workspace_id: string;
}

export interface BatchCalculatePayrollRequest {
  employee_ids: string[];
  month: number;
  year: number;
}

export interface BatchCalculateResult {
  employee_id: string;
  error_message?: string;
  payslip?: Payslip;
  status: "success" | "error";
}

export interface BatchCalculatePayrollResponse {
  results: BatchCalculateResult[];
}

// --- Department Models ---

export interface Department {
  active_employee_count: number;
  created_at: string;
  default_working_pattern_id?: string;
  description?: string;
  id: string;
  manager_id?: string;
  name: string;
  updated_at: string;
  workspace_id: string;
}

export interface EmployeeDepartmentHistory {
  created_at: string;
  department_id: string;
  effective_date: string;
  employee_id: string;
  id: string;
  notes?: string;
}

export interface ShiftRoster {
  created_at: string;
  date: string;
  employee_id: string;
  id: string;
  notes: string | null;
  shift_id: string | null;
  status: string;
  updated_at: string;
}

export interface AttendanceImportJob {
  created_at: string;
  error_log?: string;
  id: string;
  processed_rows: number;
  status: "processing" | "completed" | "failed" | "pending";
  total_rows: number;
  updated_at: string;
  user_id: string;
  workspace_id: string;
}

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
  pending_high: number;
  pending_low: number;
  pending_tasks: number;
  recent_jobs: RecentJob[];
  total_workers: number;
  workers: WorkerHeartbeat[];
}

export interface RecentJob {
  completed_at: string | null;
  completed_batches: number;
  created_at: string;
  error_message: string | null;
  failed_batches: number;
  job_id: string;
  report_type: string;
  requester_id: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  total_batches: number;
  workspace_id: string;
}

export interface WorkerHeartbeat {
  current_task_type: string | null;
  host: string;
  id: string;
  last_seen_at: string;
  status: "IDLE" | "PROCESSING";
}

export interface MemberRolesResponse {
  extra: MemberExtraRole[];
  primary: string;
}

export interface MemberExtraRole {
  assigned_at: string;
  expires_at: string;
  id: string;
  role_id: string;
  user_id: string;
  workspace_id: string;
}

export interface MemberExtraPermission {
  assigned_at: string | null;
  assigned_by: string | null;
  expires_at: string;
  id: string;
  permission: string;
  user_id: string;
  workspace_id: string;
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

export interface LeaveTypeConfig {
  allow_negative_balance?: boolean;
  approval_required?: boolean;
  negative_balance_limit?: number;
  requires_document?: boolean;
}

export interface LeaveType {
  code: string;
  created_at: string;
  id: string;
  is_paid: boolean;
  name: string;
  policy_config?: LeaveTypeConfig;
  workspace_id: string;
 
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

export interface SeniorityBonus {
  extra_days: number;
  min_years: number;
}



export interface LeavePolicyConfig {
  accrual_amount: number;
  accrual_timing: "start_of_period" | "end_of_period";
  carry_over_limit?: number;
  frequency: AccrualFrequency;
  max_balance?: number;
  min_work_days_threshold?: number;
  reset_month?: number;
  seniority_bonus: SeniorityBonus[];
}

export interface ScheduleTrace {
  priority: number;
 
  reason?: string;
  rule_id?: string;
  rule_name: string;
  status: "applied" | "overridden" | "conflict";
}

export interface EffectiveSchedule {
 
  break_duration_minutes: number | null;
  break_end: string | null;
  break_start: string | null;
  date: string;
  end_time: string | null;
  holiday_name?: string | null;
  is_break_auto_deduct: boolean | null;
  is_holiday?: boolean;
  shift_id: string | null;
  shift_name: string | null;
  source: "roster" | "pattern" | "assignment_rule" | "department_pattern" | "fixed" | "department_fixed" | "none";
  start_time: string | null;
  traces: ScheduleTrace[];
}

export interface EmployeeSchedule {
  employee_id: string;
  full_name: string;
  schedules: EffectiveSchedule[];
}

export interface AssignmentHistory {
  assignment_type: string;
  created_at: string;
  date_range: string;
  employee_id: string;
  employee_name: string;
  id: string;
  shift_name: string | null;
}

export interface LeaveHistoryItem {
  amount: number;
  created_at: string;
  date: string;
  id: string;
  reason?: string;
  reference_id?: string;
  transaction_type: "accrual" | "usage" | "adjustment" | "carry_over";
}

// --- Employee Self-Service (ESS) Types ---

export interface EmployeeProfile {
  address?: string;
  bank_account_number?: string;
  bank_branch?: string;
  bank_name?: string;
  created_at: string;
  date_of_birth?: string;
  employee_id: string;
  gender?: string;
  id: string;
  national_id?: string;
  phone?: string;
  updated_at: string;
}

export interface EmployeeEmergencyContact {
  contact_name: string;
  created_at: string;
  employee_id: string;
  id: string;
  is_primary: boolean;
  phone: string;
  relationship: string;
}

export interface EmployeeDocument {
  created_at: string;
  document_type: string;
  employee_id: string;
  expires_at?: string;
  file_name: string;
  file_path: string;
  file_size: number;
  id: string;
  is_active: boolean;
  max_concurrent_workers: number;
  mime_type: string;
  uploaded_by?: string;
  workspace_id: string;
}

export interface MyProfileResponse {
  emergency_contacts: EmployeeEmergencyContact[];
  employee: Employee;
  profile?: EmployeeProfile;
}

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

export interface CreateEmergencyContactRequest {

  contact_name: string;

  is_primary?: boolean;

  phone: string;

  relationship: string;

}



// --- Working Pattern Models ---



export interface WorkingPattern {

  created_at: string;

  description?: string;

  id: string;

  is_active: boolean;

  max_concurrent_workers: number;
  name: string;

  updated_at: string;

  workspace_id: string;

  /** 'weekly' (default) or 'rotation' for N-day cycles */
  pattern_type: string;

  /** Only set for rotation patterns. Number of days in the cycle. */
  cycle_length_days?: number | null;

}



export interface WorkingPatternDetail {

  /** For weekly: 0=Sunday..6=Saturday. For rotation: cycle day index 0..N-1 */
  schedule_day: number;

  id: string;

  pattern_id: string;

  shift_id: string | null;

}



export interface EmployeeWorkingPattern {

  created_at: string;

  employee_id: string;

  end_date: string | null;

  id: string;

  pattern_id: string;

  start_date: string;

  updated_at: string;

  workspace_id: string;

  /** Phase offset for rotation patterns. YYYY-MM-DD format. */
  anchor_date?: string | null;

}



export interface WorkingPatternDetailRequest {

  schedule_day: number;

  shift_id: string | null;

}



export interface CreateWorkingPatternRequest {

  description?: string;

  details: WorkingPatternDetailRequest[];

  name: string;

  /** 'weekly' (default) or 'rotation' */
  pattern_type?: string;

  /** Required for rotation patterns. Number of days in cycle (1-366). */
  cycle_length_days?: number;

}



export interface AssignPatternRequest {
  end_date?: string | null;
  pattern_id: string;
  start_date: string;
  /** Phase offset for rotation patterns. YYYY-MM-DD format. */
  anchor_date?: string | null;
}

export interface BulkAssignResponse {
  failed_employees: string[];
  success_count: number;
}

export interface BulkAssignPatternRequest {
  employee_ids: string[];
  end_date?: string | null;
  pattern_id: string;
  start_date: string;
  /** Phase offset for rotation patterns. YYYY-MM-DD format. */
  anchor_date?: string | null;
}

export interface WorkingCondition {
  employee_id: string;
  end_date?: string;
  id: string;
  is_paid: boolean;
  minute_allowance: number;
  start_date: string;
  type: string;
}





// --- Payroll Models ---

export interface OvertimePolicy {
  created_at?: string;
  description?: string;
  id: string;
  name: string;
  updated_at?: string;
  workspace_id: string;
}

export interface OvertimeRule {
  created_at?: string;
  days_of_week: number;
  id: string;
  is_holiday: boolean;
  multiplier: number;
  policy_id: string;
  priority: number;
  time_end?: string;
  time_start?: string;
  updated_at?: string;
}

export interface LeavePolicyAssignmentDetail {
  assignee_id: string | null;
  assignee_type: string;
  created_at: string;
  effective_from: string;
  effective_to: string | null;
  id: string;
  leave_type_name: string;
  policy_id: string;
  policy_name: string;
  workspace_id: string;
}


export type OtStatus = "none" | "pending_employee" | "pending_manager" | "approved" | "rejected";

export interface AttendanceDailySummary {
  actual_work_minutes: number;
  approved_ot_minutes: number;
  claim_entered_at?: string;
  claim_entered_by_user_id?: string;
  claimed_ot_minutes: number;
  created_at: string;
  date: string;
  employee_id: string;
  entered_via?: "self_service" | "paper_form";
  id: string;
  manager_note?: string;
  ot_reason?: string;
  ot_status: OtStatus;
  paper_form_photo_url?: string;
  system_detected_ot_minutes: number;
  updated_at: string;
  workspace_id: string;
}

// --- Payroll Rules ---

export interface PayrollRuleDefinition {
  applies_to: "all" | "department" | "employee";
  calculation_method: "percentage" | "fixed" | "tiered";
  category: "statutory_deduction" | "allowance" | "bonus" | "deduction";
  code: string;
  created_at: string;
  description?: string;
  display_order: number;
  id: string;
  is_mandatory: boolean;
  name: string;
  rule_config: Record<string, unknown>;
  workspace_id: string;
}

export interface PayrollRuleTier {
  fixed_amount?: number;
  id: string;
  max_amount?: number;
  min_amount: number;
  rate?: number;
  rule_definition_id: string;
  tier_order: number;
}

export interface PayrollRuleAssignment {
  assignee_id: string | null;
  assignee_type: "workspace" | "department" | "employee";
  created_at: string;
  effective_from: string;
  effective_to?: string | null;
  id: string;
  override_config?: Record<string, unknown> | null;
  rule_definition_id: string;
  workspace_id: string;
}

// --- Employment Phases ---

 
export interface EmploymentPhaseDefinition {
  created_at: string;
  description?: string;
  display_order: number;
  id: string;
  is_active: boolean;
  name: string;
  phase_type: "probation" | "internship" | "official" | "contract";
  rule_config?: {
    duration_days?: number;
    [key: string]: unknown;
  };
  workspace_id: string;
}

export interface EmployeePhase {
  completed_at?: string;
  created_at: string;
  employee_id: string;
  end_date: string;
  evaluated_by?: string;
  evaluation_notes?: string;
  id: string;
  override_config?: unknown;
  phase_definition_id: string;
  start_date: string;
  status: string; // "active", "completed", "terminated"
  updated_at: string;
  workspace_id: string;
}
 
