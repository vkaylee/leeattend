use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::{AdjustmentStatus, ImportJobStatus, LogSource, OtStatus};

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
    pub total_std_units: i32,
    pub total_ot_units: i32,
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
    pub status: String,
    #[serde(default)]
    pub is_holiday: bool,
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
    pub target_type: String, // "adjustment" or "log"
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
