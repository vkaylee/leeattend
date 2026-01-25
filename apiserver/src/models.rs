use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, sqlx::Type, strum_macros::Display,
)]
#[sqlx(type_name = "member_role_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, sqlx::Type, strum_macros::Display,
)]
#[sqlx(type_name = "leave_status_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LeaveStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, sqlx::Type, strum_macros::Display,
)]
#[sqlx(type_name = "adjustment_status_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, sqlx::Type, strum_macros::Display,
)]
#[sqlx(type_name = "log_source_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LogSource {
    Device,
    Import,
    Manual,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, sqlx::Type, strum_macros::Display,
)]
#[sqlx(type_name = "import_job_status_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ImportJobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, sqlx::Type, strum_macros::Display,
)]
#[sqlx(type_name = "leave_transaction_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LeaveTransactionType {
    Accrual,
    Usage,
    Adjustment,
    CarryOver,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceStats {
    pub total_employees: i64,
    pub present_today: i64,
    pub pending_leave_requests: i64,
    pub estimated_monthly_payroll: rust_decimal::Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmployeeSearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub q: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, sqlx::Type, strum_macros::Display)]
#[sqlx(type_name = "early_arrival_strategy_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EarlyArrivalStrategy {
    Allow,
    Block,
    Clamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRules {
    pub grace_period_minutes: i32,
    pub authorized_early_leave_minutes: i32,
    pub is_early_leave_paid: bool,
    pub early_arrival_strategy: EarlyArrivalStrategy,
    pub is_break_auto_deduct: bool,
    pub is_break_overtime: bool,
    pub allow_early_ot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "VARCHAR", rename_all = "UPPERCASE")]
pub enum WorkingConditionType {
    EarlyLeave,
    LateIn,
    Breastfeeding,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkingCondition {
    pub id: Uuid,
    pub employee_id: Uuid,
    #[sqlx(rename = "type")]
    pub type_: String, // Keep as string for now to match DB, but we can parse to enum
    pub minute_allowance: i32,
    pub is_paid: bool,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateWorkingConditionRequest {
    pub type_: String,
    pub minute_allowance: i32,
    pub is_paid: bool,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
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
    pub pairing_algorithm: String,
    pub early_arrival_strategy: EarlyArrivalStrategy,
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
    pub pairing_algorithm: Option<String>,
    pub early_arrival_strategy: Option<EarlyArrivalStrategy>,
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
    pub created_at: DateTime<Utc>,
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
    pub role: MemberRole,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddMemberRequest {
    pub email: String,
    pub role: MemberRole,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct MemberExtraRole {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: MemberRole,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignRoleRequest {
    pub role: MemberRole,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct WorkspaceMemberDetail {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: MemberRole,
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
    pub full_name: String,
    pub employee_id_number: String,
    pub hire_date: NaiveDate, // Fetched from latest employment_period
    pub termination_date: Option<NaiveDate>, // Fetched from latest employment_period
    pub tags: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignmentRuleConditions {
    pub department_id: Option<Uuid>,
    pub employee_id_numbers: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct AssignmentRule {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub pattern_id: Uuid,
    pub name: String,
    pub priority: i32,
    pub conditions: sqlx::types::Json<AssignmentRuleConditions>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAssignmentRuleRequest {
    pub pattern_id: Uuid,
    pub name: String,
    pub priority: i32,
    pub conditions: AssignmentRuleConditions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAssignmentRuleRequest {
    pub pattern_id: Option<Uuid>,
    pub name: Option<String>,
    pub priority: Option<i32>,
    pub conditions: Option<AssignmentRuleConditions>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RehireEmployeeRequest {
    pub hire_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateEmployeeRequest {
    pub user_id: Option<Uuid>,
    pub full_name: String,
    pub employee_id_number: String,
    pub hire_date: NaiveDate,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateEmployeeRequest {
    pub user_id: Option<Uuid>,
    pub full_name: Option<String>,
    pub employee_id_number: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub termination_date: Option<NaiveDate>,
    pub tags: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

// --- Time Tracking Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct TimeEntry {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub workspace_id: Uuid,
    pub shift_id: Option<Uuid>,
    pub punch_in_time: DateTime<Utc>,
    pub punch_out_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub is_break_overtime: bool,
    pub entry_type: String,
    pub note: Option<String>,
    pub generation_metadata: Option<serde_json::Value>,
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
    pub leave_type_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
    pub status: LeaveStatus,
    pub requested_by_user_id: Option<Uuid>,
    pub approved_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response wrapper with leave type details for API responses
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeaveRequestWithType {
    #[serde(flatten)]
    pub request: LeaveRequest,
    pub leave_type_code: String,
    pub leave_type_name: String,
    pub is_paid: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitLeaveRequest {
    pub employee_id: Uuid,
    pub leave_type_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApproveLeaveRequest {
    pub status: LeaveStatus,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeaveBalance {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub leave_type_id: Uuid,
    pub year: i32,
    pub entitlement: i32,
    pub accrued_amount: rust_decimal::Decimal,
    pub carried_over: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmployeeLeaveBalance {
    pub leave_type_id: Uuid,
    pub leave_type_code: String,
    pub leave_type_name: String,
    pub is_paid: bool,
    pub entitlement: i32,
    pub accrued_amount: f64,
    pub carried_over: i32,
    pub used: i32,
    pub remaining: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct WorkspaceLeaveBalance {
    pub employee_id: Uuid,
    pub employee_name: String,
    pub leave_type_id: Uuid,
    pub leave_type_code: String,
    pub leave_type_name: String,
    pub is_paid: bool,
    pub entitlement: i32,
    pub accrued_amount: f64,
    pub carried_over: i32,
    pub used: i32,
    pub remaining: f64,
    pub policy_name: Option<String>,
    pub assignment_type: Option<String>,
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
    pub status: AdjustmentStatus,
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
    pub status: AdjustmentStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum AttendanceReportSort {
    NameAsc,
    NameDesc,
    WorkHoursAsc,
    WorkHoursDesc,
    LeaveDaysAsc,
    LeaveDaysDesc,
}

// --- Basic Reporting Models ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttendanceSummary {
    pub employee_id: Uuid,
    pub employee_name: String,
    pub total_work_units: i32,
    pub total_leave_days: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduleTrace {
    pub rule_id: Option<Uuid>,
    pub rule_name: String,
    pub priority: i32,
    pub status: String, // "applied", "overridden", "conflict"
    pub reason: Option<String>,
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
    pub work_units: i32,
    pub status: String,
    pub generation_metadata: Option<serde_json::Value>,
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
    pub allow_early_ot: bool,
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
    #[serde(default)]
    pub break_is_overtime: bool,
    #[serde(default)]
    pub early_punch_in_grace_period_minutes: i32,
    #[serde(default)]
    pub allow_early_ot: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShiftRequest {
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
    #[serde(default)]
    pub break_is_overtime: bool,
    #[serde(default)]
    pub early_punch_in_grace_period_minutes: i32,
    #[serde(default)]
    pub allow_early_ot: bool,
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
pub struct PayslipSegment {
    pub id: Uuid,
    pub payslip_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub department_id: Option<Uuid>,
    pub applied_base_salary: rust_decimal::Decimal,
    pub applied_std_days: rust_decimal::Decimal,
    pub applied_hours_per_day: rust_decimal::Decimal,
    pub actual_work_days: rust_decimal::Decimal,
    pub paid_work_days: rust_decimal::Decimal,
    pub amount: rust_decimal::Decimal,
    pub reason: Option<String>,
    pub std_days_source: Option<String>,
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
    pub standard_units: i32,
    pub overtime_units: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
pub enum PayoutType {
    Fixed,
    Percentage,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct PayrollIncentivePolicy {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub base_amount: f64,
    pub payout_type: String, // String to match DB, can be parsed to PayoutType
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct IncentiveTier {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub priority: i32,
    pub max_late_early_count: Option<i32>,
    pub max_unpaid_leave_days: Option<f64>,
    pub max_sick_leave_days: Option<f64>,
    pub max_total_leave_days: Option<f64>,
    pub payout_amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncentiveEvaluationResult {
    pub policy_name: String,
    pub amount: rust_decimal::Decimal,
    pub standard_amount: rust_decimal::Decimal,
    pub tier_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateIncentiveTierRequest {
    pub priority: i32,
    pub max_late_early_count: Option<i32>,
    pub max_unpaid_leave_days: Option<f64>,
    pub max_sick_leave_days: Option<f64>,
    pub max_total_leave_days: Option<f64>,
    pub payout_amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatePayrollIncentivePolicyRequest {
    pub name: String,
    pub base_amount: f64,
    pub payout_type: String,
    pub tiers: Vec<CreateIncentiveTierRequest>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PayrollIncentivePolicyResponse {
    #[serde(flatten)]
    pub policy: PayrollIncentivePolicy,
    pub tiers: Vec<IncentiveTier>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayslipResponse {
    #[serde(flatten)]
    pub payslip: Payslip,
    pub segments: Vec<PayslipSegment>,
    pub incentives: Vec<IncentiveEvaluationResult>,
    pub salary_components: Vec<SalaryComponentWithDefinition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchCalculatePayrollRequest {
    pub employee_ids: Vec<Uuid>,
    pub year: i32,
    pub month: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchCalculateResult {
    pub employee_id: Uuid,
    pub status: String, // "success" or "error"
    pub payslip: Option<Payslip>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchCalculatePayrollResponse {
    pub results: Vec<BatchCalculateResult>,
}

// --- Department Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Department {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub manager_id: Option<Uuid>,
    pub default_working_pattern_id: Option<Uuid>,
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
    pub default_working_pattern_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub manager_id: Option<Uuid>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::deserialize_option_option"
    )]
    pub default_working_pattern_id: Option<Option<Uuid>>,
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
pub struct WorkingPattern {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct WorkingPatternDetail {
    pub id: Uuid,
    pub pattern_id: Uuid,
    pub day_of_week: i32,
    pub shift_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct EmployeeWorkingPattern {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub pattern_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkingPatternDetailRequest {
    pub day_of_week: i32,
    pub shift_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateWorkingPatternRequest {
    pub name: String,
    pub description: Option<String>,
    pub details: Vec<WorkingPatternDetailRequest>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AssignPatternRequest {
    pub pattern_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BulkAssignPatternRequest {
    pub employee_ids: Vec<Uuid>,
    pub pattern_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkAssignResponse {
    pub success_count: usize,
    pub failed_employees: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct RawAttendanceLog {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub punch_time: DateTime<Utc>,
    pub source: LogSource,
    pub device_id: Option<String>,
    pub raw_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct AttendanceImportJob {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub status: ImportJobStatus,
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
    pub early_punch_in_grace_period_minutes: Option<i32>,
    pub allow_early_ot: Option<bool>,
    #[serde(default)]
    pub traces: Vec<ScheduleTrace>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmployeeSchedule {
    pub employee_id: Uuid,
    pub full_name: String,
    pub schedules: Vec<EffectiveSchedule>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "assignment_history_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AssignmentHistoryType {
    Roster,
    Pattern,
    DepartmentPattern,
    Fixed,
    DepartmentFixed,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct AssignmentHistory {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: String,
    pub assignment_type: AssignmentHistoryType,
    pub shift_name: Option<String>,
    pub date_range: String, // "2023-01-01" or "2023-01-01 -> 2023-02-01"
    pub created_at: DateTime<Utc>,
}

// --- Payroll Settings & Components Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct PayrollSettings {
    pub workspace_id: Uuid,
    pub std_working_day_mode: String,
    pub fixed_std_days: rust_decimal::Decimal,
    pub working_days_per_week: i32,
    pub hours_per_day: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct DepartmentPayrollConfig {
    pub id: Uuid,
    pub department_id: Uuid,
    pub std_working_days: rust_decimal::Decimal,
    pub hours_per_day: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct EmployeePayrollConfig {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub std_working_days: rust_decimal::Decimal,
    pub hours_per_day: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SalaryComponentDefinition {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub code: String,
    pub is_base_for_ot: bool,
    pub is_taxable: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmployeeSalaryComponent {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub component_definition_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePayrollSettingsRequest {
    pub std_working_day_mode: Option<String>,
    pub fixed_std_days: Option<rust_decimal::Decimal>,
    pub working_days_per_week: Option<i32>,
    pub hours_per_day: Option<rust_decimal::Decimal>,
    pub effective_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct SetDepartmentPayrollConfigRequest {
    pub std_working_days: rust_decimal::Decimal,
    pub hours_per_day: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct SetEmployeePayrollConfigRequest {
    pub std_working_days: rust_decimal::Decimal,
    pub hours_per_day: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct CreateSalaryComponentRequest {
    pub name: String,
    pub code: String,
    pub is_base_for_ot: bool,
    pub is_taxable: bool,
}

#[derive(Debug, Deserialize)]
pub struct AssignSalaryComponentRequest {
    pub component_definition_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SalaryComponentWithDefinition {
    pub component: EmployeeSalaryComponent,
    pub definition: SalaryComponentDefinition,
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
    pub months: i32,
    pub bonus_amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LeaveTypeConfig {
    #[serde(default)]
    pub requires_document: bool,
    #[serde(default)]
    pub allow_negative_balance: bool,
    #[serde(default)]
    pub negative_balance_limit: Option<f64>,
    #[serde(default)]
    pub approval_required: bool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeaveType {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub code: String,
    pub name: String,
    pub is_paid: bool,
    pub policy_config: sqlx::types::Json<LeaveTypeConfig>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeavePolicyConfig {
    pub frequency: AccrualFrequency,
    pub accrual_amount: f64,
    pub accrual_timing: String, // "start_of_period" or "end_of_period"
    #[serde(default)]
    pub seniority_bonus: Vec<SeniorityBonus>,
    pub carry_over_limit: Option<i32>,
    pub max_balance: Option<f64>,
    pub reset_month: Option<i32>,
    pub min_work_days_threshold: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeavePolicyDefinition {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub leave_type_id: Uuid,
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
    pub leave_type_id: Uuid,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateLeavePolicyRequest {
    pub name: String,
    pub description: Option<String>,
    pub leave_type_id: Uuid,
    pub rule_config: LeavePolicyConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateLeavePolicyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub leave_type_id: Option<Uuid>,
    pub rule_config: Option<LeavePolicyConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignLeavePolicyRequest {
    pub assignee_type: String,
    #[serde(default)]
    pub assignee_id: Option<Uuid>,
    pub policy_id: Uuid,
    pub effective_from: NaiveDate,
    #[serde(default)]
    pub effective_to: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct LeaveHistoryItem {
    pub id: Uuid,
    pub date: NaiveDate,
    pub transaction_type: LeaveTransactionType,
    pub amount: f64,
    pub reason: Option<String>,
    pub reference_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdjustBalanceRequest {
    pub leave_type_id: Uuid,
    pub amount: f64, // Positive to add, Negative to deduct
    pub reason: String,
    pub transaction_date: NaiveDate,
}

// --- Employee Self-Service (ESS) Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct EmployeeProfile {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub national_id: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_branch: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateEmployeeProfileRequest {
    pub phone: Option<String>,
    pub address: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub national_id: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateEmployeeProfileRequest {
    pub phone: Option<String>,
    pub address: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub national_id: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct EmployeeEmergencyContact {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub contact_name: String,
    pub relationship: String,
    pub phone: String,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateEmergencyContactRequest {
    pub contact_name: String,
    pub relationship: String,
    pub phone: String,
    pub is_primary: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct EmployeeDocument {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub workspace_id: Uuid,
    pub document_type: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i32,
    pub mime_type: String,
    pub expires_at: Option<NaiveDate>,
    pub is_active: bool,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadDocumentRequest {
    pub document_type: String,
    pub expires_at: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyProfileResponse {
    pub employee: Employee,
    pub profile: Option<EmployeeProfile>,
    pub emergency_contacts: Vec<EmployeeEmergencyContact>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
#[derive(Default)]
pub struct OvertimePolicy {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct OvertimeRule {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub priority: i32,
    pub multiplier: rust_decimal::Decimal,
    pub time_start: Option<NaiveTime>,
    pub time_end: Option<NaiveTime>,
    pub days_of_week: i32,
    pub is_holiday: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct OvertimePolicyAssignment {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub assignee_type: String,
    pub assignee_id: Option<Uuid>,
    pub policy_id: Uuid,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub created_at: Option<DateTime<Utc>>,
}
