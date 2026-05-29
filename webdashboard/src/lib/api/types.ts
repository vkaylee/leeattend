import { components } from "@/types/api-generated";

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

export type DeviceStatus = components["schemas"]["DeviceStatus"];
export type AgentStatus = components["schemas"]["AgentStatus"];
export type Device = components["schemas"]["Device"];
export type AuthorizedAgent = components["schemas"]["AuthorizedAgent"];
export type Workspace = any; // fallback for ui

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

export type UpdateUserRequest = any; // fallback for ui
export type ChangePasswordRequest = any; // fallback for ui
export type RegisterRequest = any; // fallback for ui

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

export type WorkspaceRole = any; // fallback for ui
export type AuthResponse = any; // fallback for ui
// --- Timekeeping Models ---
export type Employee = any; // fallback for ui
export type TimeEntry = any; // fallback for ui
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
export type LeaveRequest = any; // fallback for ui
export type EmployeeLeaveBalance = any; // fallback for ui
export type WorkspaceLeaveBalance = any; // fallback for ui
export type AttendanceAdjustmentRequest = any; // fallback for ui

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

export type AuditLog = any; // fallback for ui
export type Holiday = any; // fallback for ui

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
export type Shift = any; // fallback for ui
export type PayrollSettings = any; // fallback for ui

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

export type SalaryComponentDefinition = any; // fallback for ui
export type CreateSalaryComponentRequest = any; // fallback for ui
export type EmployeeSalaryComponent = any; // fallback for ui

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

export type IncentiveEvaluationResult = any; // fallback for ui
export type SalaryComponentWithDefinition = any; // fallback for ui
export type SalaryBaseGroup = any; // fallback for ui
export type SalaryBaseGroupMember = any; // fallback for ui
export type Payslip = any; // fallback for ui

export interface RuleCondition {
  metric: string;
  operator: string;
  value: number;
}

export type IncentiveTier = any; // fallback for ui
export type PayrollIncentivePolicy = any; // fallback for ui
export type BatchCalculatePayrollRequest = any; // fallback for ui
export type BatchCalculateResult = any; // fallback for ui
export type BatchCalculatePayrollResponse = any; // fallback for ui
// --- Department Models ---
export type Department = any; // fallback for ui
export type EmployeeDepartmentHistory = any; // fallback for ui
export type ShiftRoster = any; // fallback for ui
export type AttendanceImportJob = any; // fallback for ui

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

export type WorkerHeartbeat = any; // fallback for ui

export interface MemberRolesResponse {
  extra: MemberExtraRole[];
  primary: string;
}

export type MemberExtraRole = any; // fallback for ui
export type MemberExtraPermission = any; // fallback for ui

export interface EmploymentPeriod {
  created_at: string;
  employee_id: string;
  hire_date: string;
  id: string;
  is_rehire: boolean;
  termination_date?: string;
  updated_at: string;
}

export type LeaveTypeConfig = any; // fallback for ui
export type LeaveType = any; // fallback for ui

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

export type SeniorityBonus = any; // fallback for ui
export type LeavePolicyConfig = any; // fallback for ui
export type ScheduleTrace = any; // fallback for ui
export type EffectiveSchedule = any; // fallback for ui
export type EmployeeSchedule = any; // fallback for ui
export type AssignmentHistory = any; // fallback for ui
export type LeaveHistoryItem = any; // fallback for ui
// --- Assignment (Secondment) Models ---

export enum AssignmentStatus {
  Scheduled = "scheduled",
  Active = "active",
  Completed = "completed",
  Cancelled = "cancelled",
}

export type EmployeeAssignment = any; // fallback for ui
export type CreateEmployeeAssignmentRequest = any; // fallback for ui
export type UpdateEmployeeAssignmentRequest = any; // fallback for ui
// --- Employee Self-Service (ESS) Types ---
export type EmployeeProfile = any; // fallback for ui
export type EmployeeEmergencyContact = any; // fallback for ui
export type EmployeeDocument = any; // fallback for ui
export type MyProfileResponse = any; // fallback for ui

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

export type CreateEmergencyContactRequest = any; // fallback for ui
// --- Working Pattern Models ---
export type WorkingPattern = any; // fallback for ui
export type WorkingPatternDetail = any; // fallback for ui
export type EmployeeWorkingPattern = any; // fallback for ui
export type WorkingPatternDetailRequest = any; // fallback for ui
export type CreateWorkingPatternRequest = any; // fallback for ui

export interface UpdateWorkingPatternRequest {

  cycle_length_days?: number;

  description?: string;

  details: WorkingPatternDetailRequest[];

  name: string;

  pattern_type?: string;

}

export type AssignPatternRequest = any; // fallback for ui
export type BulkAssignResponse = any; // fallback for ui
export type BulkAssignPatternRequest = any; // fallback for ui
export type WorkingCondition = any; // fallback for ui
// --- Payroll Models ---
export type OvertimePolicy = any; // fallback for ui
export type OvertimeRule = any; // fallback for ui
export type LeavePolicyAssignmentDetail = any; // fallback for ui
export type OtStatus = "none" | "pending_employee" | "pending_manager" | "approved" | "rejected";
export type AttendanceDailySummary = any; // fallback for ui
// --- Payroll Rules ---
export type PayrollRuleDefinition = any; // fallback for ui
export type PayrollRuleTier = any; // fallback for ui
export type PayrollRuleAssignment = any; // fallback for ui
// --- Employment Phases ---
export type EmploymentPhaseDefinition = any; // fallback for ui
export type EmployeePhase = any; // fallback for ui
// --- Workspace Config (EAV) ---
export type WorkspaceConfigEntry = any; // fallback for ui

export interface CreateConfigEntryRequest {
  config_key: string;
  effective_from: string;
  effective_to?: string | null;
  scope_id?: string | null;
  scope_type: "workspace" | "department" | "employee";
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

export type ApprovalWorkflow = any; // fallback for ui
export type CreateWorkflowRequest = any; // fallback for ui
export type ApprovalActionLog = any; // fallback for ui
export type ProcessApprovalRequest = any; // fallback for ui
// --- Overtime Requests ---
export type OtRequestStatus = "draft" | "pending_approval" | "approved" | "partially_accepted" | "active" | "completed" | "cancelled" | "rejected";
export type ConsentStatus = "pending" | "accepted" | "declined";
export type OvertimeRequestEmployee = any; // fallback for ui
export type OvertimeRequest = any; // fallback for ui
export type OvertimeRequestDetail = any; // fallback for ui
export type CreateOvertimeRequestPayload = any; // fallback for ui
