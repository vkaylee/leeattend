export interface DeviceStatus {
  device_id: string;
  status: "online" | "offline";
  last_sync?: string;
  error?: string;
  info?: {
    users_count: number;
    users_capacity?: number;
    logs_count: number;
    logs_capacity?: number;
    device_time: string;
    firmware: string;
    mac?: string;
    serial?: string;
    platform?: string;
    hw_name?: string;
    last_sync: string;
  };
}

export interface AgentStatus {
  agent_id: string;
  last_seen: string;
  is_online: boolean;
  devices: DeviceStatus[];
}

export interface Device {
  id: string;
  agent_id: string;
  name: string;
  ip: string;
  port: number;
  is_enabled: boolean;
  brand: string;
  last_status: string;
  created_at: string;
  workspace_id: string;
}

export interface AuthorizedAgent {
  id: string;
  name: string;
  token: string;
  is_trusted: boolean;
  last_ip?: string;
  last_seen_at: string;
  created_at: string;
  is_active: boolean;
  max_concurrent_workers: number;
  workspace_id: string;
}

export interface Workspace {
  id: string;
  name: string;
  work_unit_minutes: number;
  rounding_policy: string;
  date_format: string;
  time_format: string;
  timezone: string;
  theme_primary_color: string;
  default_language: string;
  pairing_algorithm: string;
  early_arrival_strategy: "allow" | "block" | "clamp";
  created_at: string;
}

export interface User {
  id: string;
  email: string;
  full_name?: string;
  avatar_url?: string;
  bio?: string;
  preferred_language?: string;
  created_at?: string;
}

export interface UpdateUserRequest {
  full_name?: string;
  avatar_url?: string;
  bio?: string;
}

export interface ChangePasswordRequest {
  old_password: string;
  new_password: string;
}

export interface RegisterRequest {
  email: string;
  password: string;
  full_name: string;
}

export interface WorkspaceMember {
  workspace_id: string;
  user_id: string;
  role: "owner" | "admin" | "member";
  email: string;
  full_name?: string;
  avatar_url?: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  user: User;
}

// --- Timekeeping Models ---

export interface Employee {
  id: string;
  workspace_id: string;
  user_id?: string;
  full_name: string;
  employee_id_number: string;
  department_name?: string;
  hire_date: string;
  termination_date?: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface TimeEntry {
  id: string;
  employee_id: string;
  punch_in_time: string;
  punch_out_time?: string;
  duration_minutes?: number;
  is_break_overtime: boolean;
  entry_type: string;
  note?: string;
  generation_metadata?: any;
  created_at: string;
  updated_at: string;
  employee_full_name?: string;
  employee_id_number?: string;
}

export interface RawAttendanceLog {
  id: string;
  workspace_id: string;
  employee_id: string;
  punch_time: string;
  source: string;
  device_id?: string;
  raw_data?: any;
  created_at: string;
  employee_full_name?: string;
  employee_id_number?: string;
  device_name?: string;
}

export interface LeaveRequest {
  id: string;
  employee_id: string;
  leave_type_id: string;
  start_date: string;
  end_date: string;
  reason?: string;
  status: string;
  requested_by_user_id?: string;
  approved_by_user_id?: string;
  created_at: string;
  updated_at: string;
}

export interface EmployeeLeaveBalance {
  leave_type_id: string;
  leave_type_code: string;
  leave_type_name: string;
  is_paid: boolean;
  entitlement: number;
  accrued_amount: number;
  carried_over: number;
  used: number;
  remaining: number;
}

export interface WorkspaceLeaveBalance {
  employee_id: string;
  employee_name: string;
  leave_type_id: string;
  leave_type_code: string;
  leave_type_name: string;
  is_paid: boolean;
  entitlement: number;
  accrued_amount: number;
  carried_over: number;
  used: number;
  remaining: number;
  policy_name?: string;
  assignment_type?: string;
}

export interface AttendanceAdjustmentRequest {
  id: string;
  employee_id: string;
  time_entry_id?: string;
  requested_punch_in_time: string;
  requested_punch_out_time?: string;
  is_break_overtime: boolean;
  reason?: string;
  status: string;
  requested_by_user_id?: string;
  approved_by_user_id?: string;
  created_at: string;
  updated_at: string;
  requested_date: string;
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  limit: number;
}

export type AttendanceReportSort =
  | "name_asc"
  | "name_desc"
  | "work_hours_asc"
  | "work_hours_desc"
  | "leave_days_asc"
  | "leave_days_desc";

export interface AttendanceSummary {
  employee_id: string;
  employee_name: string;
  total_work_units: number;
  total_leave_days: number;
}

export interface DailyAttendanceReport {
  date: string;
  employee_id: string;
  employee_name: string;
  scheduled_start: string | null;
  scheduled_end: string | null;
  break_duration_minutes: number | null;
  break_start: string | null;
  break_end: string | null;
  is_break_auto_deduct: boolean | null;
  actual_punch_in: string | null;
  actual_punch_out: string | null;
  std_work_units: number;
  ot_work_units: number;
  ot_claim_status?: string | null;
  status: string;
  generation_metadata?: any;
}

// --- Payroll and Shift Models ---

export interface Shift {
  id: string;
  workspace_id: string;
  name: string;
  start_time: string;
  end_time: string;
  break_duration_minutes: number;
  break_start?: string | null;
  break_end?: string | null;
  is_break_auto_deduct?: boolean;
  is_break_overtime?: boolean;
  is_correction?: boolean;
  effective_date?: string;
  early_punch_in_grace_period_minutes?: number;
  created_at: string;
  updated_at: string;
}

export interface PayrollSettings {
  workspace_id: string;
  std_working_day_mode: "fixed" | "actual_month";
  fixed_std_days: number;
  working_days_per_week: number;
  hours_per_day: number;
  effective_date: string;
  updated_at: string;
}

export interface SalaryComponentDefinition {
  id: string;
  workspace_id: string;
  name: string;
  code: string;
  is_base_for_ot: boolean;
  is_taxable: boolean;
}

export interface EmployeeSalaryComponent {
  id: string;
  employee_id: string;
  component_definition_id: string;
  amount: string;
  effective_date: string;
  name?: string;
  code?: string;
  is_base_for_ot?: boolean;
  is_taxable?: boolean;
}

export interface PayrollSegment {
  id: string;
  start_date: string;
  end_date: string;
  department_id?: string;
  applied_base_salary: number;
  applied_std_days: number;
  applied_hours_per_day: number;
  actual_work_days: number;
  paid_work_days: number;
  amount: number;
  reason: string;
  std_days_source?: "individual" | "department" | "workspace" | "system";
}

export interface IncentiveEvaluationResult {
  policy_name: string;
  amount: string;
  tier_id?: string;
}

export interface SalaryComponentDefinition {
  id: string;
  workspace_id: string;
  name: string;
  code: string;
  is_base_for_ot: boolean;
  is_taxable: boolean;
  created_at: string;
  updated_at: string;
}

export interface EmployeeSalaryComponent {
  id: string;
  employee_id: string;
  component_definition_id: string;
  amount: string;
  effective_date: string;
  created_at: string;
  updated_at: string;
}

export interface SalaryComponentWithDefinition {
  component: EmployeeSalaryComponent;
  definition: SalaryComponentDefinition;
}

export interface Payslip {
  id: string;
  employee_id: string;
  period_start: string;
  period_end: string;
  base_pay_amount: string;
  overtime_pay_amount: string;
  deductions_amount: string;
  net_pay_amount: string;
  standard_units: number;
  overtime_units: number;
  status: "draft" | "published" | "paid";
  segments?: PayrollSegment[];
  incentives?: IncentiveEvaluationResult[];
  salary_components?: SalaryComponentWithDefinition[];
  created_at: string;
  updated_at: string;
}

export interface IncentiveTier {
  id: string;
  policy_id: string;
  priority: number;
  max_late_early_count?: number;
  max_unpaid_leave_days?: number;
  max_sick_leave_days?: number;
  max_total_leave_days?: number;
  payout_amount: number;
}

export interface PayrollIncentivePolicy {
  id: string;
  workspace_id: string;
  name: string;
  base_amount: number;
  payout_type: "fixed" | "percentage";
  is_active: boolean;
  max_concurrent_workers: number;
  tiers?: IncentiveTier[];
  created_at: string;
}

export interface BatchCalculatePayrollRequest {
  employee_ids: string[];
  year: number;
  month: number;
}

export interface BatchCalculateResult {
  employee_id: string;
  status: "success" | "error";
  payslip?: Payslip;
  error_message?: string;
}

export interface BatchCalculatePayrollResponse {
  results: BatchCalculateResult[];
}

// --- Department Models ---

export interface Department {
  id: string;
  workspace_id: string;
  name: string;
  description?: string;
  manager_id?: string;
  default_working_pattern_id?: string;
  created_at: string;
  updated_at: string;
}

export interface EmployeeDepartmentHistory {
  id: string;
  employee_id: string;
  department_id: string;
  effective_date: string;
  notes?: string;
  created_at: string;
}

export interface ShiftRoster {
  id: string;
  employee_id: string;
  shift_id: string | null;
  date: string;
  status: string;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

export interface AttendanceImportJob {
  id: string;
  workspace_id: string;
  user_id: string;
  status: "processing" | "completed" | "failed" | "pending";
  total_rows: number;
  processed_rows: number;
  error_log?: string;
  created_at: string;
  updated_at: string;
}

export interface ReportExportJobProgress {
  total_batches: number;
  completed_batches: number;
  failed_batches: number;
  percentage: number;
}

export interface ReportExportJob {
  job_id: string;
  status: "pending" | "processing" | "completed" | "failed";
  progress: ReportExportJobProgress;
  result_url?: string;
  error?: string;
  created_at: string;
  completed_at?: string;
}

export interface CreateReportExportRequest {
  start_date: string;
  end_date: string;
  report_type: "summary" | "detail";
  employee_ids?: string[];
}

export interface CreateReportExportResponse {
  job_id: string;
  status: string;
  total_batches: number;
  estimated_seconds: number;
  status_url: string;
}

export interface WorkerMonitorStats {
  pending_tasks: number;
  total_workers: number;
  active_workers: number;
  idle_workers: number;
  completed_today: number;
  failed_today: number;
  workers: WorkerHeartbeat[];
}

export interface WorkerHeartbeat {
  id: string;
  host: string;
  status: "IDLE" | "PROCESSING";
  current_task_type: string | null;
  last_seen_at: string;
}

export interface MemberRolesResponse {
  primary: string;
  extra: MemberExtraRole[];
}

export interface MemberExtraRole {
  id: string;
  workspace_id: string;
  user_id: string;
  role: string;
  assigned_at: string;
  expires_at: string;
}

export interface EmploymentPeriod {
  id: string;
  employee_id: string;
  hire_date: string;
  termination_date?: string;
  is_rehire: boolean;
  created_at: string;
  updated_at: string;
}

export interface LeaveTypeConfig {
  requires_document?: boolean;
  allow_negative_balance?: boolean;
  negative_balance_limit?: number;
  approval_required?: boolean;
}

export interface LeaveType {
  id: string;
  workspace_id: string;
  code: string;
  name: string;
  is_paid: boolean;
  policy_config: LeaveTypeConfig;
  created_at: string;
}

export interface LeavePolicy {
  id: string;
  workspace_id: string;
  name: string;
  description?: string;
  leave_type_id: string;
  rule_config: LeavePolicyConfig;
  created_at: string;
  updated_at: string;
}

export type AccrualFrequency = "monthly" | "quarterly" | "yearly";

export interface SeniorityBonus {
  min_years: number;
  extra_days: number;
}

export interface LeaveTypeConfig {
  requires_document?: boolean;
  allow_negative_balance?: boolean;
  negative_balance_limit?: number;
  approval_required?: boolean;
}

export interface LeaveType {
  id: string;
  workspace_id: string;
  code: string;
  name: string;
  is_paid: boolean;
  policy_config: LeaveTypeConfig;
  created_at: string;
}

export interface LeavePolicyConfig {
  frequency: AccrualFrequency;
  accrual_amount: number;
  accrual_timing: "start_of_period" | "end_of_period";
  seniority_bonus: SeniorityBonus[];
  max_balance?: number;
  reset_month?: number;
  min_work_days_threshold?: number;
  carry_over_limit?: number;
}

export interface ScheduleTrace {
  rule_id?: string;
  rule_name: string;
  priority: number;
  status: "applied" | "overridden" | "conflict";
  reason?: string;
}

export interface EffectiveSchedule {
  date: string;
  shift_id: string | null;
  source: "roster" | "pattern" | "assignment_rule" | "department_pattern" | "fixed" | "department_fixed" | "none";
  shift_name: string | null;
  start_time: string | null;
  end_time: string | null;
  break_duration_minutes: number | null;
  break_start: string | null;
  break_end: string | null;
  is_break_auto_deduct: boolean | null;
  traces: ScheduleTrace[];
}

export interface EmployeeSchedule {
  employee_id: string;
  full_name: string;
  schedules: EffectiveSchedule[];
}

export interface AssignmentHistory {
  id: string;
  employee_id: string;
  employee_name: string;
  assignment_type: string;
  shift_name: string | null;
  date_range: string;
  created_at: string;
}

export interface LeaveHistoryItem {
  id: string;
  date: string;
  transaction_type: "accrual" | "usage" | "adjustment" | "carry_over";
  amount: number;
  reason?: string;
  reference_id?: string;
  created_at: string;
}

// --- Employee Self-Service (ESS) Types ---

export interface EmployeeProfile {
  id: string;
  employee_id: string;
  phone?: string;
  address?: string;
  date_of_birth?: string;
  gender?: string;
  national_id?: string;
  bank_name?: string;
  bank_account_number?: string;
  bank_branch?: string;
  created_at: string;
  updated_at: string;
}

export interface EmployeeEmergencyContact {
  id: string;
  employee_id: string;
  contact_name: string;
  relationship: string;
  phone: string;
  is_primary: boolean;
  created_at: string;
}

export interface EmployeeDocument {
  id: string;
  employee_id: string;
  workspace_id: string;
  document_type: string;
  file_name: string;
  file_path: string;
  file_size: number;
  mime_type: string;
  expires_at?: string;
  is_active: boolean;
  max_concurrent_workers: number;
  uploaded_by?: string;
  created_at: string;
}

export interface MyProfileResponse {
  employee: Employee;
  profile?: EmployeeProfile;
  emergency_contacts: EmployeeEmergencyContact[];
}

export interface UpdateProfileRequest {
  phone?: string;
  address?: string;
  date_of_birth?: string;
  gender?: string;
  national_id?: string;
  bank_name?: string;
  bank_account_number?: string;
  bank_branch?: string;
}

export interface CreateEmergencyContactRequest {

  contact_name: string;

  relationship: string;

  phone: string;

  is_primary?: boolean;

}



// --- Working Pattern Models ---



export interface WorkingPattern {

  id: string;

  workspace_id: string;

  name: string;

  description?: string;

  is_active: boolean;
  max_concurrent_workers: number;

  created_at: string;

  updated_at: string;

}



export interface WorkingPatternDetail {

  id: string;

  pattern_id: string;

  day_of_week: number; // 0=Sunday, 1=Monday...

  shift_id: string | null;

}



export interface EmployeeWorkingPattern {

  id: string;

  workspace_id: string;

  employee_id: string;

  pattern_id: string;

  start_date: string;

  end_date: string | null;

  created_at: string;

  updated_at: string;

}



export interface WorkingPatternDetailRequest {

  day_of_week: number;

  shift_id: string | null;

}



export interface CreateWorkingPatternRequest {

  name: string;

  description?: string;

  details: WorkingPatternDetailRequest[];

}



export interface AssignPatternRequest {
  pattern_id: string;
  start_date: string;
  end_date?: string | null;
}

export interface BulkAssignResponse {
  success_count: number;
  failed_employees: string[];
}

export interface BulkAssignPatternRequest {
  employee_ids: string[];
  pattern_id: string;
  start_date: string;
  end_date?: string | null;
}

export interface WorkingCondition {
  id: string;
  employee_id: string;
  type: string;
  minute_allowance: number;
  is_paid: boolean;
  start_date: string;
  end_date?: string;
}





// --- Payroll Models ---

export interface OvertimePolicy {
  id: string;
  workspace_id: string;
  name: string;
  description?: string;
  created_at?: string;
  updated_at?: string;
}

export interface OvertimeRule {
  id: string;
  policy_id: string;
  priority: number;
  multiplier: number;
  time_start?: string;
  time_end?: string;
  days_of_week: number;
  is_holiday: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface LeavePolicyAssignmentDetail {
  id: string;
  workspace_id: string;
  assignee_type: string;
  assignee_id: string | null;
  policy_id: string;
  effective_from: string;
  effective_to: string | null;
  created_at: string;
  policy_name: string;
  leave_type_name: string;
}


export type OtStatus = "none" | "pending_employee" | "pending_manager" | "approved" | "rejected";

export interface AttendanceDailySummary {
  id: string;
  workspace_id: string;
  employee_id: string;
  date: string;
  actual_work_minutes: number;
  system_detected_ot_minutes: number;
  claimed_ot_minutes: number;
  approved_ot_minutes: number;
  ot_status: OtStatus;
  ot_reason?: string;
  manager_note?: string;
  claim_entered_by_user_id?: string;
  claim_entered_at?: string;
  paper_form_photo_url?: string;
  entered_via?: "self_service" | "paper_form";
  created_at: string;
  updated_at: string;
}
