use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceStats {
    pub total_employees: i64,
    pub present_today: i64,
    pub pending_leave_requests: i64,
    pub estimated_monthly_payroll: rust_decimal::Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenericResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceStatus {
    pub device_id: String,
    pub status: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentStatus {
    pub agent_id: String,
    pub last_seen: DateTime<Utc>,
    pub is_online: bool,
    pub devices: Vec<DeviceStatus>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct AuthorizedAgent {
    pub id: String,
    pub name: String,
    pub token: String,
    pub is_trusted: bool,
    pub last_ip: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub is_active: bool,
    pub workspace_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAuthorizedAgentRequest {
    pub name: Option<String>,
    pub token: Option<String>,
    pub is_trusted: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAuthorizedAgentRequest {
    pub id: String,
    pub name: String,
    pub token: String,
    pub workspace_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrustAgentRequest {
    pub is_trusted: bool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Device {
    pub id: Uuid,
    pub agent_id: String,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub is_enabled: bool,
    pub last_status: String,
    pub created_at: DateTime<Utc>,
    pub workspace_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeviceRequest {
    pub agent_id: String,
    pub name: String,
    pub ip: String,
    pub port: Option<i32>,
    pub workspace_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeviceRequest {
    pub name: Option<String>,
    pub ip: Option<String>,
    pub port: Option<i32>,
    pub is_enabled: Option<bool>,
}

// --- Workspace Management Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub work_unit_minutes: i32,
    pub rounding_policy: String,
    pub date_format: String,
    pub time_format: String,
    pub timezone: String,
    pub theme_primary_color: String,
    pub default_language: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateWorkspaceRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
    pub work_unit_minutes: Option<i32>,
    pub rounding_policy: Option<String>,
    pub date_format: Option<String>,
    pub time_format: Option<String>,
    pub timezone: Option<String>,
    pub theme_primary_color: Option<String>,
    pub default_language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub preferred_language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub preferred_language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateUserRequest {
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub preferred_language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct WorkspaceMember {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddMemberRequest {
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct MemberExtraRole {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignRoleRequest {
    pub role: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct WorkspaceMemberDetail {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
}

// --- Employee Management Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct EmploymentPeriod {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub hire_date: NaiveDate,
    pub termination_date: Option<NaiveDate>,
    pub is_rehire: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Employee {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    pub employee_id_number: String,
    pub hire_date: NaiveDate, // Fetched from latest employment_period
    pub termination_date: Option<NaiveDate>, // Fetched from latest employment_period
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RehireEmployeeRequest {
    pub hire_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateEmployeeRequest {
    pub user_id: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    pub employee_id_number: String,
    pub hire_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateEmployeeRequest {
    pub user_id: Option<Uuid>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub employee_id_number: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub termination_date: Option<NaiveDate>,
    pub is_active: Option<bool>,
}

// --- Time Tracking Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct TimeEntry {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub shift_id: Option<Uuid>,
    pub punch_in_time: DateTime<Utc>,
    pub punch_out_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub is_break_overtime: bool,
    pub entry_type: String,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PunchInRequest {
    pub employee_id: Uuid,
    pub entry_type: Option<String>,
    pub is_break_overtime: Option<bool>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PunchOutRequest {
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateTimeEntryRequest {
    pub punch_in_time: Option<DateTime<Utc>>,
    pub punch_out_time: Option<DateTime<Utc>>,
    pub entry_type: Option<String>,
    pub note: Option<String>,
}

// --- Leave Management Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeaveRequest {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub leave_type: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
    pub status: String,
    pub requested_by_user_id: Option<Uuid>,
    pub approved_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitLeaveRequest {
    pub employee_id: Uuid,
    pub leave_type: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApproveLeaveRequest {
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeaveBalance {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub leave_type: String,
    pub year: i32,
    pub entitlement: i32,
    pub carried_over: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmployeeLeaveBalance {
    pub leave_type: String,
    pub entitlement: i32,
    pub carried_over: i32,
    pub used: i32,
    pub remaining: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeaveBalanceRequest {
    pub entitlement: Option<i32>,
    pub carried_over: Option<i32>,
}

// --- Attendance Adjustment Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct AttendanceAdjustmentRequest {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub time_entry_id: Option<Uuid>,
    pub requested_punch_in_time: DateTime<Utc>,
    pub requested_punch_out_time: Option<DateTime<Utc>>,
    pub is_break_overtime: bool,
    pub reason: Option<String>,
    pub status: String,
    pub requested_by_user_id: Option<Uuid>,
    pub approved_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAttendanceAdjustmentRequest {
    pub employee_id: Uuid,
    pub time_entry_id: Option<Uuid>,
    pub requested_punch_in_time: DateTime<Utc>,
    pub requested_punch_out_time: Option<DateTime<Utc>>,
    pub is_break_overtime: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApproveAttendanceAdjustmentRequest {
    pub status: String,
}

// --- Basic Reporting Models ---

#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct AttendanceSummary {
    pub employee_id: Uuid,

    pub employee_name: String,

    pub total_hours: f64,

    pub total_leave_days: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DailyAttendanceReport {
    pub date: NaiveDate,
    pub employee_id: Uuid,
    pub employee_name: String,
    pub scheduled_start: Option<NaiveTime>,
    pub scheduled_end: Option<NaiveTime>,
    pub break_duration_minutes: Option<i32>,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_break_auto_deduct: Option<bool>,
    pub actual_punch_in: Option<DateTime<Utc>>,
    pub actual_punch_out: Option<DateTime<Utc>>,
    pub total_minutes: i32,
    pub status: String,
}

// --- Payroll and Shift Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct EmployeeShift {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub shift_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AssignShiftRequest {
    pub employee_id: Uuid,
    pub shift_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Shift {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub start_time: chrono::NaiveTime,
    pub end_time: chrono::NaiveTime,
    pub break_duration_minutes: i32,
    pub break_start: Option<chrono::NaiveTime>,
    pub break_end: Option<chrono::NaiveTime>,
    pub is_break_auto_deduct: bool,
    pub break_is_overtime: bool,
    pub early_punch_in_grace_period_minutes: i32,
    pub is_active: bool,
    pub parent_shift_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShiftRequest {
    pub name: String,
    pub start_time: chrono::NaiveTime,
    pub end_time: chrono::NaiveTime,
    pub break_duration_minutes: i32,
    #[serde(default)]
    pub break_start: Option<chrono::NaiveTime>,
    #[serde(default)]
    pub break_end: Option<chrono::NaiveTime>,
    #[serde(default = "default_true")]
    pub is_break_auto_deduct: bool,
    pub break_is_overtime: bool,
    pub early_punch_in_grace_period_minutes: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShiftRequest {
    pub name: String,
    pub start_time: chrono::NaiveTime,
    pub end_time: chrono::NaiveTime,
    pub break_duration_minutes: i32,
    pub break_start: Option<chrono::NaiveTime>,
    pub break_end: Option<chrono::NaiveTime>,
    pub is_break_auto_deduct: bool,
    pub break_is_overtime: bool,
    pub early_punch_in_grace_period_minutes: i32,
    /// If true, update the current record (Correction). 
    /// If false (default), create a new version if time-related fields changed.
    #[serde(default)]
    pub is_correction: bool,
    /// Effective date for the new version. If not provided, defaults to today.
    pub effective_date: Option<NaiveDate>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct SalaryHistory {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub base_salary: rust_decimal::Decimal,
    pub salary_type: String,
    pub currency: String,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSalaryHistoryRequest {
    pub base_salary: rust_decimal::Decimal,
    pub salary_type: String,
    pub currency: String,
    pub effective_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Payslip {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub base_pay_amount: rust_decimal::Decimal,
    pub overtime_pay_amount: rust_decimal::Decimal,
    pub deductions_amount: rust_decimal::Decimal,
    pub net_pay_amount: rust_decimal::Decimal,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// --- Department Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Department {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub manager_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ShiftRoster {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub shift_id: Option<Uuid>,
    pub date: NaiveDate,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRosterRequest {
    pub employee_id: Uuid,
    pub shift_id: Option<Uuid>,
    pub date: NaiveDate,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RosterItem {
    pub date: NaiveDate,
    pub shift_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BulkRosterRequest {
    pub items: Vec<RosterItem>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDepartmentRequest {
    pub name: String,
    pub description: Option<String>,
    pub manager_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub manager_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct EmployeeDepartmentHistory {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub department_id: Uuid,
    pub effective_date: NaiveDate,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TransferEmployeeRequest {
    pub department_id: Uuid,
    pub effective_date: NaiveDate,
    pub notes: Option<String>,
}

// --- Schedule & Roster Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct RawAttendanceLog {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub punch_time: DateTime<Utc>,
    pub source: String,
    pub device_id: Option<String>,
    pub raw_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct AttendanceImportJob {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub total_rows: i32,
    pub processed_rows: i32,
    pub file_path: Option<String>,
    pub error_log: Option<String>,
    pub target_type: String, // "adjustment" or "log"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EffectiveSchedule {
    pub date: NaiveDate,
    pub shift_id: Option<Uuid>, // None means Day Off
    pub source: String,         // "roster", "fixed", "department", "none"
    pub shift_name: Option<String>,
    pub start_time: Option<chrono::NaiveTime>,
    pub end_time: Option<chrono::NaiveTime>,
    pub break_duration_minutes: Option<i32>,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_break_auto_deduct: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmployeeSchedule {
    pub employee_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub schedules: Vec<EffectiveSchedule>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct AssignmentHistory {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: String,
    pub assignment_type: String, // "roster", "fixed"
    pub shift_name: Option<String>,
    pub date_range: String,      // "2023-01-01" or "2023-01-01 -> 2023-02-01"
    pub created_at: DateTime<Utc>,
}

// --- Dynamic Leave Policy Models ---

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AccrualFrequency {
    Monthly,
    Quarterly,
    Yearly,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeniorityBonus {
    pub min_years: i32,
    pub extra_days: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeavePolicyConfig {
    pub frequency: AccrualFrequency,
    pub accrual_amount: f64,
    pub accrual_timing: String, // "start_of_period" or "end_of_period"
    #[serde(default)]
    pub seniority_bonus: Vec<SeniorityBonus>,
    pub max_balance: Option<f64>,
    pub reset_month: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeavePolicyDefinition {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub rule_config: sqlx::types::Json<LeavePolicyConfig>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeavePolicyAssignment {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub assignee_type: String,
    pub assignee_id: Option<Uuid>,
    pub policy_id: Uuid,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeaveAccrualLog {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub policy_assignment_id: Option<Uuid>,
    pub accrual_date: NaiveDate,
    pub amount: rust_decimal::Decimal,
    pub leave_type: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateLeavePolicyRequest {
    pub name: String,
    pub description: Option<String>,
    pub rule_config: LeavePolicyConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignLeavePolicyRequest {
    pub assignee_type: String,
    pub assignee_id: Option<Uuid>,
    pub policy_id: Uuid,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
}
