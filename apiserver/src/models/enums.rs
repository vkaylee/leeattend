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
    strum_macros::EnumString,
    ToSchema,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
)]
#[diesel(sql_type = crate::schema::sql_types::ContractTypeEnum)]
#[sqlx(type_name = "contract_type_enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ContractType {
    Probation,
    Training,
    FixedTerm,
    Indefinite,
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
    strum_macros::EnumString,
    ToSchema,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
)]
#[diesel(sql_type = crate::schema::sql_types::ContractStatusEnum)]
#[sqlx(type_name = "contract_status_enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ContractStatus {
    Draft,
    Active,
    Expired,
    Terminated,
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
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
)]
#[diesel(sql_type = crate::schema::sql_types::LeaveStatusType)]
#[sqlx(type_name = "leave_status_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LeaveStatus {
    Pending,
    Approved,
    Rejected,
    Cancelled,
    PreApproved,
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
    strum_macros::EnumString,
    ToSchema,
    Default,
)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LeaveDurationType {
    #[default]
    FullDay,
    FirstHalf,
    SecondHalf,
    CustomHours,
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
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
)]
#[diesel(sql_type = crate::schema::sql_types::AdjustmentStatusType)]
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
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
)]
#[diesel(sql_type = crate::schema::sql_types::LogSourceType)]
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
#[strum(serialize_all = "snake_case")]
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
    strum_macros::EnumString,
    ToSchema,
)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ImportTargetType {
    Log,
    Adjustment,
    Employee,
    LeaveImport,
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
    Default,
)]
#[sqlx(type_name = "early_arrival_strategy_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EarlyArrivalStrategy {
    Allow,
    #[default]
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
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    sqlx::Type,
    strum_macros::Display,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
)]
#[diesel(sql_type = crate::schema::sql_types::OtStatusType)]
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
    OtHoursAsc,
    OtHoursDesc,
    TotalHoursAsc,
    TotalHoursDesc,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum AccrualFrequency {
    #[default]
    Monthly,
    Quarterly,
    Yearly,
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
    strum_macros::EnumString,
    ToSchema,
    Default,
)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TimekeepingUnit {
    #[default]
    DAY,
    SHIFT,
    HOUR,
    MINUTE,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, sqlx::Type, strum_macros::Display,
)]
#[sqlx(type_name = "recruitment_source_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RecruitmentSourceType {
    Direct,
    Referral,
    Agency,
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
    strum_macros::EnumString,
    ToSchema,
)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AssigneeType {
    Workspace,
    Department,
    Employee,
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
    strum_macros::EnumString,
    ToSchema,
)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PayrollCalculationMethod {
    Percentage,
    Fixed,
    Tiered,
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
    strum_macros::EnumString,
    ToSchema,
)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PayrollRuleCategory {
    StatutoryDeduction,
    Allowance,
    Bonus,
    Deduction,
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
    strum_macros::EnumString,
    ToSchema,
)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum RuleAppliesTo {
    Employee,
    Employer,
}

impl From<String> for AssigneeType {
    fn from(s: String) -> Self {
        std::str::FromStr::from_str(&s).unwrap_or(AssigneeType::Employee)
    }
}

impl From<String> for PayrollCalculationMethod {
    fn from(s: String) -> Self {
        std::str::FromStr::from_str(&s).unwrap_or(PayrollCalculationMethod::Fixed)
    }
}

impl From<String> for PayrollRuleCategory {
    fn from(s: String) -> Self {
        std::str::FromStr::from_str(&s).unwrap_or(PayrollRuleCategory::Allowance)
    }
}

impl From<String> for RuleAppliesTo {
    fn from(s: String) -> Self {
        std::str::FromStr::from_str(&s).unwrap_or(RuleAppliesTo::Employee)
    }
}

/// Determines whether a workspace role has global (workspace-wide) or
/// department-scoped visibility. Used to replace the hardcoded name check
/// for 'owner', 'admin', 'hr_manager', 'shift_manager' in MemberContext.
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    sqlx::Type,
    strum_macros::Display,
    strum_macros::EnumString,
    ToSchema,
)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum RoleScopeType {
    /// Role can manage all employees across the entire workspace.
    Global,
    /// Role is limited to employees within managed departments only.
    #[default]
    Scoped,
}

impl RoleScopeType {
    pub fn is_global(self) -> bool {
        self == Self::Global
    }
}

impl From<String> for RoleScopeType {
    fn from(s: String) -> Self {
        std::str::FromStr::from_str(&s).unwrap_or(RoleScopeType::Scoped)
    }
}

/// Platform-level roles that govern cross-workspace, infrastructure, and billing operations.
/// These are entirely separate from `workspace_roles` which are tenant-scoped.
///
/// Stored in the `system_roles` table and resolved via `SystemContext` extractor.
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    sqlx::Type,
    strum_macros::Display,
    strum_macros::EnumString,
    ToSchema,
)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SystemRoleType {
    /// Full platform control: create/suspend workspaces, assign system roles, manage infrastructure.
    PlatformAdmin,
    /// Read-only cross-workspace access for customer support and debugging.
    Support,
    /// Manage billing, subscription plans, and invoicing.
    BillingAdmin,
}

impl SystemRoleType {
    /// Returns true if this role allows creation of new workspaces.
    pub fn can_create_workspace(self) -> bool {
        matches!(self, Self::PlatformAdmin)
    }

    /// Returns true if this role allows read access to infrastructure monitoring.
    pub fn can_monitor_infra(self) -> bool {
        matches!(self, Self::PlatformAdmin | Self::Support)
    }

    /// Returns true if this role allows billing operations.
    pub fn can_manage_billing(self) -> bool {
        matches!(self, Self::PlatformAdmin | Self::BillingAdmin)
    }
}

impl From<String> for SystemRoleType {
    fn from(s: String) -> Self {
        use std::str::FromStr;
        // Safe fallback: unknown strings are treated as Support (lowest privilege).
        // This prevents privilege escalation via corrupted DB values.
        Self::from_str(&s).unwrap_or(Self::Support)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leave_status_serde_roundtrip() {
        let statuses = vec![
            LeaveStatus::Pending,
            LeaveStatus::Approved,
            LeaveStatus::Rejected,
            LeaveStatus::Cancelled,
            LeaveStatus::PreApproved,
        ];
        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let back: LeaveStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, back);
        }
    }

    #[test]
    fn test_leave_status_display() {
        assert_eq!(LeaveStatus::Pending.to_string(), "Pending");
        assert_eq!(LeaveStatus::Approved.to_string(), "Approved");
        assert_eq!(LeaveStatus::PreApproved.to_string(), "PreApproved");
    }

    #[test]
    fn test_adjustment_status_serde_roundtrip() {
        for status in [
            AdjustmentStatus::Pending,
            AdjustmentStatus::Approved,
            AdjustmentStatus::Rejected,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: AdjustmentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, back);
        }
    }

    #[test]
    fn test_log_source_serde_roundtrip() {
        for src in [LogSource::Device, LogSource::Import, LogSource::Manual] {
            let json = serde_json::to_string(&src).unwrap();
            let back: LogSource = serde_json::from_str(&json).unwrap();
            assert_eq!(src, back);
        }
    }

    #[test]
    fn test_import_job_status_serde_roundtrip() {
        for status in [
            ImportJobStatus::Pending,
            ImportJobStatus::Processing,
            ImportJobStatus::Completed,
            ImportJobStatus::Failed,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: ImportJobStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, back);
        }
    }

    #[test]
    fn test_import_target_type_serde_roundtrip() {
        for target in [
            ImportTargetType::Log,
            ImportTargetType::Adjustment,
            ImportTargetType::Employee,
            ImportTargetType::LeaveImport,
        ] {
            let json = serde_json::to_string(&target).unwrap();
            let back: ImportTargetType = serde_json::from_str(&json).unwrap();
            assert_eq!(target, back);
        }
    }

    #[test]
    fn test_leave_transaction_type_serde() {
        for t in [
            LeaveTransactionType::Accrual,
            LeaveTransactionType::Usage,
            LeaveTransactionType::Adjustment,
            LeaveTransactionType::CarryOver,
        ] {
            let json = serde_json::to_string(&t).unwrap();
            let back: LeaveTransactionType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn test_early_arrival_strategy_serde() {
        for s in [
            EarlyArrivalStrategy::Allow,
            EarlyArrivalStrategy::Block,
            EarlyArrivalStrategy::Clamp,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let back: EarlyArrivalStrategy = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn test_working_condition_type_serde() {
        for wc in [
            WorkingConditionType::EarlyLeave,
            WorkingConditionType::LateIn,
            WorkingConditionType::Breastfeeding,
        ] {
            let json = serde_json::to_string(&wc).unwrap();
            let back: WorkingConditionType = serde_json::from_str(&json).unwrap();
            assert_eq!(wc, back);
        }
    }

    #[test]
    fn test_payout_type_serde() {
        for pt in [PayoutType::Fixed, PayoutType::Percentage] {
            let json = serde_json::to_string(&pt).unwrap();
            let back: PayoutType = serde_json::from_str(&json).unwrap();
            assert_eq!(pt, back);
        }
    }

    #[test]
    fn test_assignment_history_type_serde() {
        for t in [
            AssignmentHistoryType::Roster,
            AssignmentHistoryType::Pattern,
            AssignmentHistoryType::DepartmentPattern,
            AssignmentHistoryType::Fixed,
            AssignmentHistoryType::DepartmentFixed,
        ] {
            let json = serde_json::to_string(&t).unwrap();
            let back: AssignmentHistoryType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn test_ot_status_serde() {
        for s in [
            OtStatus::None,
            OtStatus::PendingEmployee,
            OtStatus::PendingManager,
            OtStatus::Approved,
            OtStatus::Rejected,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let back: OtStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn test_ot_status_display() {
        assert_eq!(OtStatus::None.to_string(), "None");
        assert_eq!(OtStatus::PendingEmployee.to_string(), "PendingEmployee");
        assert_eq!(OtStatus::Approved.to_string(), "Approved");
    }

    #[test]
    fn test_attendance_report_sort_serde() {
        for s in [
            AttendanceReportSort::NameAsc,
            AttendanceReportSort::NameDesc,
            AttendanceReportSort::WorkHoursAsc,
            AttendanceReportSort::WorkHoursDesc,
            AttendanceReportSort::LeaveDaysAsc,
            AttendanceReportSort::LeaveDaysDesc,
            AttendanceReportSort::OtHoursAsc,
            AttendanceReportSort::OtHoursDesc,
            AttendanceReportSort::TotalHoursAsc,
            AttendanceReportSort::TotalHoursDesc,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let _back: AttendanceReportSort = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_accrual_frequency_serde() {
        for f in [
            AccrualFrequency::Monthly,
            AccrualFrequency::Quarterly,
            AccrualFrequency::Yearly,
        ] {
            let json = serde_json::to_string(&f).unwrap();
            let _back: AccrualFrequency = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_timekeeping_unit_serde() {
        for u in [
            TimekeepingUnit::DAY,
            TimekeepingUnit::SHIFT,
            TimekeepingUnit::HOUR,
            TimekeepingUnit::MINUTE,
        ] {
            let json = serde_json::to_string(&u).unwrap();
            let back: TimekeepingUnit = serde_json::from_str(&json).unwrap();
            assert_eq!(u, back);
        }
    }

    #[test]
    fn test_recruitment_source_type_serde() {
        for t in [
            RecruitmentSourceType::Direct,
            RecruitmentSourceType::Referral,
            RecruitmentSourceType::Agency,
        ] {
            let json = serde_json::to_string(&t).unwrap();
            let back: RecruitmentSourceType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn test_role_scope_type_serde_roundtrip() {
        for s in [RoleScopeType::Global, RoleScopeType::Scoped] {
            let json = serde_json::to_string(&s).unwrap();
            let back: RoleScopeType = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn test_role_scope_type_json_format() {
        assert_eq!(
            serde_json::to_string(&RoleScopeType::Global).unwrap(),
            "\"global\""
        );
        assert_eq!(
            serde_json::to_string(&RoleScopeType::Scoped).unwrap(),
            "\"scoped\""
        );
    }

    #[test]
    fn test_role_scope_type_default_is_scoped() {
        assert_eq!(RoleScopeType::default(), RoleScopeType::Scoped);
    }

    #[test]
    fn test_role_scope_type_is_global() {
        assert!(RoleScopeType::Global.is_global());
        assert!(!RoleScopeType::Scoped.is_global());
    }

    #[test]
    fn test_role_scope_type_from_string() {
        let g = RoleScopeType::from("global".to_string());
        assert_eq!(g, RoleScopeType::Global);
        let s = RoleScopeType::from("scoped".to_string());
        assert_eq!(s, RoleScopeType::Scoped);
        // Unknown value falls back to Scoped (safe default)
        let unknown = RoleScopeType::from("unknown_value".to_string());
        assert_eq!(unknown, RoleScopeType::Scoped);
    }

    #[test]
    fn test_leave_status_json_format() {
        let json = serde_json::to_string(&LeaveStatus::Pending).unwrap();
        assert_eq!(json, "\"pending\"");
        let json = serde_json::to_string(&LeaveStatus::Approved).unwrap();
        assert_eq!(json, "\"approved\"");
        let json = serde_json::to_string(&LeaveStatus::PreApproved).unwrap();
        assert_eq!(json, "\"pre_approved\"");
    }

    #[test]
    fn test_leave_duration_type_serde_roundtrip() {
        for dt in [
            LeaveDurationType::FullDay,
            LeaveDurationType::FirstHalf,
            LeaveDurationType::SecondHalf,
            LeaveDurationType::CustomHours,
        ] {
            let json = serde_json::to_string(&dt).unwrap();
            let back: LeaveDurationType = serde_json::from_str(&json).unwrap();
            assert_eq!(dt, back);
        }
    }

    #[test]
    fn test_leave_duration_type_json_format() {
        assert_eq!(
            serde_json::to_string(&LeaveDurationType::FullDay).unwrap(),
            "\"full_day\""
        );
        assert_eq!(
            serde_json::to_string(&LeaveDurationType::FirstHalf).unwrap(),
            "\"first_half\""
        );
        assert_eq!(
            serde_json::to_string(&LeaveDurationType::SecondHalf).unwrap(),
            "\"second_half\""
        );
        assert_eq!(
            serde_json::to_string(&LeaveDurationType::CustomHours).unwrap(),
            "\"custom_hours\""
        );
    }

    #[test]
    fn test_leave_duration_type_default() {
        let dt = LeaveDurationType::default();
        assert_eq!(dt, LeaveDurationType::FullDay);
    }
}
