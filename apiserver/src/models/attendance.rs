use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::{
    AdjustmentStatus, EarlyArrivalStrategy, ImportJobStatus, ImportTargetType, LogSource, OtStatus,
    TimekeepingUnit,
};
use crate::services::attendance_utils::AttendanceStatus;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttendanceSummary {
    pub employee_id: Uuid,
    pub employee_name: String,
    #[serde(default)]
    pub employee_id_number: String,
    pub total_std_units: i32,
    pub total_ot_units: i32,
    pub total_leave_days: i64,
    #[serde(default)]
    pub unit_type: TimekeepingUnit,
    #[serde(default)]
    pub unit_coefficient_hours: f64,
    #[serde(default)]
    pub computed_std_hours: f64,
    #[serde(default)]
    pub computed_ot_hours: f64,
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
    pub employee_id_number: String,
    pub shift_symbol: Option<String>,
    pub scheduled_start: Option<NaiveTime>,
    pub scheduled_end: Option<NaiveTime>,
    pub break_duration_minutes: Option<i32>,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_break_auto_deduct: Option<bool>,
    pub actual_punch_in: Option<DateTime<Utc>>,
    pub actual_punch_out: Option<DateTime<Utc>>,
    pub std_work_units: i32,
    pub ot_work_units: i32,
    #[serde(default)]
    pub ot_claim_status: Option<String>,
    #[serde(default)]
    pub unit_type: TimekeepingUnit,
    #[serde(default)]
    pub unit_coefficient_hours: f64,
    #[serde(default)]
    pub computed_std_hours: f64,
    #[serde(default)]
    pub computed_ot_hours: f64,
    pub status: AttendanceStatus,
    #[serde(default)]
    pub is_holiday: bool,
    #[serde(default)]
    pub is_unscheduled: bool,
    pub holiday_name: Option<String>,
    pub generation_metadata: Option<serde_json::Value>,
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
    pub punch: i32,
    pub status: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct RawAttendanceLogWithEmployee {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub punch_time: DateTime<Utc>,
    pub source: LogSource,
    pub device_id: Option<String>,
    pub raw_data: Option<serde_json::Value>,
    pub punch: i32,
    pub status: i32,
    pub created_at: DateTime<Utc>,
    pub employee_full_name: String,
    pub employee_id_number: String,
    pub device_name: Option<String>,
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
    pub target_type: ImportTargetType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct AttendanceDailySummary {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub date: NaiveDate,
    pub actual_work_minutes: i32,
    pub system_detected_ot_minutes: i32,
    pub claimed_ot_minutes: i32,
    pub approved_ot_minutes: i32,
    pub ot_status: OtStatus,
    pub ot_reason: Option<String>,
    pub manager_note: Option<String>,
    pub claim_entered_by_user_id: Option<Uuid>,
    pub claim_entered_at: Option<DateTime<Utc>>,
    pub paper_form_photo_url: Option<String>,
    pub entered_via: Option<String>,
    pub is_holiday: bool,
    pub holiday_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Bulk OT Upload Models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BulkOtEntryError {
    pub line_number: usize,
    pub employee_id: String,
    pub error_message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BulkOtUploadResult {
    pub total_rows: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub errors: Vec<BulkOtEntryError>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct AttendancePolicy {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct AttendancePolicyVersion {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub rules: sqlx::types::Json<AttendanceRules>,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttendanceRules {
    pub work_unit_minutes: i32,
    pub rounding_policy: String,
    #[serde(default)]
    pub early_arrival_strategy: EarlyArrivalStrategy,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct AttendancePolicyAssignment {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub target_type: String, // "GROUP" or "USER"
    pub target_id: Uuid,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_attendance_summary_backward_compatibility() {
        // Old payload missing the multi-unit fields
        let old_json = json!({
            "employee_id": Uuid::new_v4(),
            "employee_name": "Test User",
            "employee_id_number": "EMP001",
            "total_std_units": 160,
            "total_ot_units": 20,
            "total_leave_days": 2
        });

        let summary: AttendanceSummary = serde_json::from_value(old_json).unwrap();

        // Ensure defaults are applied correctly
        assert_eq!(summary.total_std_units, 160);
        assert!(matches!(summary.unit_type, TimekeepingUnit::DAY));
        assert_eq!(summary.unit_coefficient_hours, 0.0);
        assert_eq!(summary.computed_std_hours, 0.0);
        assert_eq!(summary.computed_ot_hours, 0.0);
    }

    #[test]
    fn test_attendance_summary_multi_unit_serialization() {
        let summary = AttendanceSummary {
            employee_id: Uuid::new_v4(),
            employee_name: "Test User".to_string(),
            employee_id_number: "EMP001".to_string(),
            total_std_units: 160,
            total_ot_units: 20,
            total_leave_days: 2,
            unit_type: TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 1280.0,
            computed_ot_hours: 160.0,
        };

        let serialized = serde_json::to_string(&summary).unwrap();

        let deserialized: AttendanceSummary = serde_json::from_str(&serialized).unwrap();
        // Use a small epsilon for float comparison to avoid clippy::float_cmp
        assert!((deserialized.computed_ot_hours - 160.0).abs() < f64::EPSILON);
        assert!((deserialized.unit_coefficient_hours - 8.0).abs() < f64::EPSILON);
    }
}
