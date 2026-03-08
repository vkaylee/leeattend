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
    HrManager,
    ShiftManager,
    TeamLead,
    Member,
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
    OtHoursAsc,
    OtHoursDesc,
    TotalHoursAsc,
    TotalHoursDesc,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AccrualFrequency {
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
)]
#[sqlx(type_name = "recruitment_source_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RecruitmentSourceType {
    Direct,
    Referral,
    Agency,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_role_serde_roundtrip() {
        let roles = vec![
            MemberRole::Owner, MemberRole::Admin, MemberRole::HrManager,
            MemberRole::ShiftManager, MemberRole::TeamLead, MemberRole::Member,
            MemberRole::Employee,
        ];
        for role in roles {
            let json = serde_json::to_string(&role).unwrap();
            let back: MemberRole = serde_json::from_str(&json).unwrap();
            assert_eq!(role, back);
        }
    }

    #[test]
    fn test_member_role_display() {
        assert_eq!(MemberRole::Owner.to_string(), "Owner");
        assert_eq!(MemberRole::Admin.to_string(), "Admin");
        assert_eq!(MemberRole::HrManager.to_string(), "HrManager");
    }

    #[test]
    fn test_leave_status_serde_roundtrip() {
        let statuses = vec![
            LeaveStatus::Pending, LeaveStatus::Approved,
            LeaveStatus::Rejected, LeaveStatus::Cancelled,
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
    }

    #[test]
    fn test_adjustment_status_serde_roundtrip() {
        for status in [AdjustmentStatus::Pending, AdjustmentStatus::Approved, AdjustmentStatus::Rejected] {
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
        for status in [ImportJobStatus::Pending, ImportJobStatus::Processing, ImportJobStatus::Completed, ImportJobStatus::Failed] {
            let json = serde_json::to_string(&status).unwrap();
            let back: ImportJobStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, back);
        }
    }

    #[test]
    fn test_leave_transaction_type_serde() {
        for t in [LeaveTransactionType::Accrual, LeaveTransactionType::Usage, LeaveTransactionType::Adjustment, LeaveTransactionType::CarryOver] {
            let json = serde_json::to_string(&t).unwrap();
            let back: LeaveTransactionType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn test_early_arrival_strategy_serde() {
        for s in [EarlyArrivalStrategy::Allow, EarlyArrivalStrategy::Block, EarlyArrivalStrategy::Clamp] {
            let json = serde_json::to_string(&s).unwrap();
            let back: EarlyArrivalStrategy = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn test_working_condition_type_serde() {
        for wc in [WorkingConditionType::EarlyLeave, WorkingConditionType::LateIn, WorkingConditionType::Breastfeeding] {
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
            AssignmentHistoryType::Roster, AssignmentHistoryType::Pattern,
            AssignmentHistoryType::DepartmentPattern, AssignmentHistoryType::Fixed,
            AssignmentHistoryType::DepartmentFixed,
        ] {
            let json = serde_json::to_string(&t).unwrap();
            let back: AssignmentHistoryType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn test_ot_status_serde() {
        for s in [OtStatus::None, OtStatus::PendingEmployee, OtStatus::PendingManager, OtStatus::Approved, OtStatus::Rejected] {
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
            AttendanceReportSort::NameAsc, AttendanceReportSort::NameDesc,
            AttendanceReportSort::WorkHoursAsc, AttendanceReportSort::WorkHoursDesc,
            AttendanceReportSort::LeaveDaysAsc, AttendanceReportSort::LeaveDaysDesc,
            AttendanceReportSort::OtHoursAsc, AttendanceReportSort::OtHoursDesc,
            AttendanceReportSort::TotalHoursAsc, AttendanceReportSort::TotalHoursDesc,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let _back: AttendanceReportSort = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_accrual_frequency_serde() {
        for f in [AccrualFrequency::Monthly, AccrualFrequency::Quarterly, AccrualFrequency::Yearly] {
            let json = serde_json::to_string(&f).unwrap();
            let _back: AccrualFrequency = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_recruitment_source_type_serde() {
        for t in [RecruitmentSourceType::Direct, RecruitmentSourceType::Referral, RecruitmentSourceType::Agency] {
            let json = serde_json::to_string(&t).unwrap();
            let back: RecruitmentSourceType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn test_member_role_json_format() {
        // Verify snake_case serialization from #[serde(rename_all = "snake_case")]
        let json = serde_json::to_string(&MemberRole::HrManager).unwrap();
        assert_eq!(json, "\"hr_manager\"");
        let json = serde_json::to_string(&MemberRole::ShiftManager).unwrap();
        assert_eq!(json, "\"shift_manager\"");
        let json = serde_json::to_string(&MemberRole::TeamLead).unwrap();
        assert_eq!(json, "\"team_lead\"");
    }

    #[test]
    fn test_leave_status_json_format() {
        let json = serde_json::to_string(&LeaveStatus::Pending).unwrap();
        assert_eq!(json, "\"pending\"");
        let json = serde_json::to_string(&LeaveStatus::Approved).unwrap();
        assert_eq!(json, "\"approved\"");
    }
}
