use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    sqlx::Type,
    strum_macros::Display,
    ToSchema,
)]
#[sqlx(type_name = "member_role_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    sqlx::Type,
    strum_macros::Display,
    ToSchema,
)]
#[sqlx(type_name = "leave_status_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LeaveStatus {
    Pending,
    Approved,
    Rejected,
    Cancelled,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    sqlx::Type,
    strum_macros::Display,
    ToSchema,
)]
#[sqlx(type_name = "adjustment_status_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    sqlx::Type,
    strum_macros::Display,
    ToSchema,
)]
#[sqlx(type_name = "log_source_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LogSource {
    Device,
    Import,
    Manual,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    sqlx::Type,
    strum_macros::Display,
    ToSchema,
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
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    sqlx::Type,
    strum_macros::Display,
    ToSchema,
)]
#[sqlx(type_name = "leave_transaction_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LeaveTransactionType {
    Accrual,
    Usage,
    Adjustment,
    CarryOver,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    sqlx::Type,
    strum_macros::Display,
    ToSchema,
)]
#[sqlx(type_name = "early_arrival_strategy_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EarlyArrivalStrategy {
    Allow,
    Block,
    Clamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "VARCHAR", rename_all = "UPPERCASE")]
pub enum WorkingConditionType {
    EarlyLeave,
    LateIn,
    Breastfeeding,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
pub enum PayoutType {
    Fixed,
    Percentage,
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

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, sqlx::Type, strum_macros::Display,
)]
#[sqlx(type_name = "ot_status_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OtStatus {
    None,
    PendingEmployee,
    PendingManager,
    Approved,
    Rejected,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AccrualFrequency {
    Monthly,
    Quarterly,
    Yearly,
}
