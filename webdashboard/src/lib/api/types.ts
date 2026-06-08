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
export type Workspace = components["schemas"]["Workspace"];
export type Employer = components["schemas"]["Employer"];
export type CreateEmployerRequest = components["schemas"]["CreateEmployerRequest"];
export type UpdateEmployerRequest = components["schemas"]["UpdateEmployerRequest"];

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

export type UpdateUserRequest = components["schemas"]["UpdateUserRequest"];
export type ChangePasswordRequest = components["schemas"]["ChangePasswordRequest"];
export type RegisterRequest = components["schemas"]["RegisterRequest"];

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

export type AuthResponse = components["schemas"]["AuthResponse"];
// --- Timekeeping Models ---
export type Employee = components["schemas"]["Employee"];
export type Employment = components["schemas"]["Employment"];
export type TimeEntry = components["schemas"]["TimeEntry"];
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
export type LeaveRequest = components["schemas"]["LeaveRequest"];

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

export type WorkspaceLeaveBalance = components["schemas"]["WorkspaceLeaveBalance"];
export type AttendanceAdjustmentRequest = components["schemas"]["AttendanceAdjustmentRequest"] & { requested_date?: string };

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

export type AuditLog = Omit<components["schemas"]["AuditLog"], "actor_id" | "target_user_id"> & {
  actor_id?: string;
  target_user_id?: string;
};

export type Holiday = components["schemas"]["Holiday"];

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
  shift_name?: string | null;
  shift_symbol?: string | null;
  status: string;
  std_work_units: number;
  unit_coefficient_hours: number;
  unit_type: 'DAY' | 'SHIFT' | 'HOUR' | 'MINUTE';
}

// --- Payroll and Shift Models ---
export type Shift = components["schemas"]["Shift"];
export type CreateShiftRequest = components["schemas"]["CreateShiftRequest"];
export type UpdateShiftRequest = components["schemas"]["UpdateShiftRequest"];
export type PayrollSettings = components["schemas"]["PayrollSettings"];
export type UpdatePayrollSettingsRequest = components["schemas"]["UpdatePayrollSettingsRequest"];

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

export type SalaryComponentDefinition = components["schemas"]["SalaryComponentDefinition"];
export type CreateSalaryComponentRequest = components["schemas"]["CreateSalaryComponentRequest"];
export type CreatePayrollRuleDefinitionRequest = components["schemas"]["CreatePayrollRuleDefinitionRequest"];
export type EmployeeSalaryComponent = components["schemas"]["EmployeeSalaryComponent"];
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

export type IncentiveEvaluationResult = components["schemas"]["IncentiveEvaluationResult"];
export type SalaryComponentWithDefinition = components["schemas"]["SalaryComponentWithDefinition"];
export type SalaryBaseGroup = components["schemas"]["SalaryBaseGroup"];
export type SalaryBaseGroupMember = components["schemas"]["SalaryBaseGroupMember"];
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

export type IncentiveTier = components["schemas"]["IncentiveTier"];
export type PayrollIncentivePolicy = components["schemas"]["PayrollIncentivePolicy"];
export type PayrollIncentivePolicyResponse = components["schemas"]["PayrollIncentivePolicyResponse"];
export type AssigneeType = components["schemas"]["AssigneeType"];
export type BatchCalculatePayrollRequest = components["schemas"]["BatchCalculatePayrollRequest"];
export type BatchCalculateResult = components["schemas"]["BatchCalculateResult"];
export type BatchCalculatePayrollResponse = components["schemas"]["BatchCalculatePayrollResponse"];
// --- Department Models ---
export type Department = components["schemas"]["Department"];
export type EmployeeDepartmentHistory = components["schemas"]["EmployeeDepartmentHistory"];
export type ShiftRoster = components["schemas"]["ShiftRoster"];
export type AttendanceImportJob = components["schemas"]["AttendanceImportJob"];

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

export type WorkerHeartbeat = components["schemas"]["WorkerHeartbeat"];

export interface MemberRolesResponse {
  extra: MemberExtraRole[];
  primary: string;
}

export type MemberExtraRole = components["schemas"]["MemberExtraRole"];
export type MemberExtraPermission = components["schemas"]["MemberExtraPermission"];

export interface EmploymentPeriod {
  created_at: string;
  employee_id: string;
  hire_date: string;
  id: string;
  is_rehire: boolean;
  termination_date?: string;
  updated_at: string;
}

export type LeaveTypeConfig = components["schemas"]["LeaveTypeConfig"];
export type LeaveType = components["schemas"]["LeaveType"];

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
export type ScheduleTrace = components["schemas"]["ScheduleTrace"];
export type EffectiveSchedule = components["schemas"]["EffectiveSchedule"];
export type EmployeeSchedule = Omit<components["schemas"]["EmployeeSchedule"], "employee_id_number"> & {
  employee_id_number?: string;
};
export type AssignmentHistory = components["schemas"]["AssignmentHistory"];
export type LeaveHistoryItem = components["schemas"]["LeaveHistoryItem"];
// --- Assignment (Secondment) Models ---

export const AssignmentStatus = {
  Scheduled: "scheduled",
  Active: "active",
  Completed: "completed",
  Cancelled: "cancelled",
} as const;
export type AssignmentStatus = typeof AssignmentStatus[keyof typeof AssignmentStatus];

export type EmployeeAssignment = Omit<components["schemas"]["EmployeeAssignment"], "status"> & { status: AssignmentStatus };
export type CreateEmployeeAssignmentRequest = components["schemas"]["CreateEmployeeAssignmentRequest"];
export type UpdateEmployeeAssignmentRequest = components["schemas"]["UpdateEmployeeAssignmentRequest"] & { host_location_id?: string | null };
// --- Employee Self-Service (ESS) Types ---
export type EmployeeProfile = components["schemas"]["EmployeeProfile"];
export type EmployeeEmergencyContact = components["schemas"]["EmployeeEmergencyContact"];
export type EmployeeDocument = components["schemas"]["EmployeeDocument"];
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

export type CreateEmergencyContactRequest = components["schemas"]["CreateEmergencyContactRequest"];
// --- Working Pattern Models ---
export type WorkingPattern = Omit<components["schemas"]["WorkingPattern"], "description"> & {
  description?: string;
};
export type WorkingPatternDetail = components["schemas"]["WorkingPatternDetail"];
export type EmployeeWorkingPattern = components["schemas"]["EmployeeWorkingPattern"];
export type WorkingPatternDetailRequest = components["schemas"]["WorkingPatternDetailRequest"];
export type CreateWorkingPatternRequest = components["schemas"]["CreateWorkingPatternRequest"];

export interface UpdateWorkingPatternRequest {

  cycle_length_days?: number;

  description?: string;

  details: WorkingPatternDetailRequest[];

  name: string;

  pattern_type?: string;

}

export type AssignPatternRequest = components["schemas"]["AssignPatternRequest"];
export type BulkAssignResponse = components["schemas"]["BulkAssignResponse"];
export type BulkAssignPatternRequest = components["schemas"]["BulkAssignPatternRequest"];
export type WorkingCondition = Omit<components["schemas"]["WorkingCondition"], "type_"> & { type: string };
// --- Payroll Models ---
export type OvertimePolicy = components["schemas"]["OvertimePolicy"];
export type OvertimeRule = components["schemas"]["OvertimeRule"];
export type LeavePolicyAssignmentDetail = Omit<components["schemas"]["LeavePolicyAssignmentDetail"], "effective_to"> & {
  effective_to: string | null;
};
export type OtStatus = "none" | "pending_employee" | "pending_manager" | "approved" | "rejected";
export type AttendanceDailySummary = Omit<components["schemas"]["AttendanceDailySummary"], "entered_via" | "ot_reason"> & {
  entered_via?: string | null;
  ot_reason?: string | null;
};
// --- Payroll Rules ---
export type PayrollRuleDefinition = components["schemas"]["PayrollRuleDefinition"];
export type PayrollRuleTier = components["schemas"]["PayrollRuleTier"];
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
export type WorkspaceConfigEntry = components["schemas"]["WorkspaceConfigEntry"];

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
export type CreateWorkflowRequest = components["schemas"]["CreateWorkflowRequest"];
export type ApprovalActionLog = components["schemas"]["ApprovalActionLog"];
export type ProcessApprovalRequest = components["schemas"]["ProcessApprovalRequest"];
// --- Overtime Requests ---
export type OtRequestStatus = "draft" | "pending_approval" | "approved" | "partially_accepted" | "active" | "completed" | "cancelled" | "rejected";
export type ConsentStatus = "pending" | "accepted" | "declined";
export type OvertimeRequestEmployee = components["schemas"]["OvertimeRequestEmployee"];
export type OvertimeRequest = components["schemas"]["OvertimeRequest"];
export type OvertimeRequestDetail = components["schemas"]["OvertimeRequestDetail"];
export type CreateOvertimeRequestPayload = components["schemas"]["CreateOvertimeRequestPayload"];
export type BehalfAssignment = components["schemas"]["BehalfAssignment"];
export type CreateBehalfAssignmentRequest = components["schemas"]["CreateBehalfAssignmentRequest"];
