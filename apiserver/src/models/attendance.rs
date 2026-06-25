use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::{
    AdjustmentStatus, EarlyArrivalStrategy, ImportJobStatus, ImportTargetType, LogSource, OtStatus,
    TimekeepingUnit,
};
use crate::services::attendance_utils::AttendanceStatus;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct AttendanceAdjustmentRequest {
    id: Uuid,
    employment_id: Uuid,
    time_entry_id: Option<Uuid>,
    requested_punch_time: DateTime<Utc>,
    reason: Option<String>,
    status: AdjustmentStatus,
    requested_by_user_id: Option<Uuid>,
    approved_by_user_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    pub attachment_file_ids: Option<Vec<Option<Uuid>>>,
}

impl AttendanceAdjustmentRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        employment_id: Uuid,
        time_entry_id: Option<Uuid>,
        requested_punch_time: DateTime<Utc>,
        reason: Option<String>,
        status: AdjustmentStatus,
        requested_by_user_id: Option<Uuid>,
        approved_by_user_id: Option<Uuid>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        attachment_file_ids: Option<Vec<Option<Uuid>>>,
    ) -> Self {
        Self {
            id,
            employment_id,
            time_entry_id,
            requested_punch_time,
            reason,
            status,
            requested_by_user_id,
            approved_by_user_id,
            created_at,
            updated_at,
            attachment_file_ids,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn employment_id(&self) -> Uuid {
        self.employment_id
    }

    pub fn time_entry_id(&self) -> Option<Uuid> {
        self.time_entry_id
    }

    pub fn requested_punch_time(&self) -> DateTime<Utc> {
        self.requested_punch_time
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    pub fn status(&self) -> AdjustmentStatus {
        self.status
    }

    pub fn requested_by_user_id(&self) -> Option<Uuid> {
        self.requested_by_user_id
    }

    pub fn approved_by_user_id(&self) -> Option<Uuid> {
        self.approved_by_user_id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn attachment_file_ids(&self) -> Option<&[Option<Uuid>]> {
        self.attachment_file_ids.as_deref()
    }
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateAttendanceAdjustmentRequest {
    pub employment_id: Uuid,
    pub time_entry_id: Option<Uuid>,
    pub requested_punch_time: DateTime<Utc>,
    pub reason: Option<String>,
    pub attachment_file_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ApproveAttendanceAdjustmentRequest {
    pub status: AdjustmentStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct AttendanceSummary {
    pub employment_id: Uuid,
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
    #[serde(default)]
    pub employer_id: Option<Uuid>,
    #[serde(default)]
    pub employer_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
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
    pub employment_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: String,
    pub employee_id_number: String,
    pub shift_symbol: Option<String>,
    pub shift_name: Option<String>,
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
    #[serde(default)]
    pub is_leave_paid: Option<bool>,
    #[serde(default)]
    pub employer_name: Option<String>,
    // ── Segment fields (segment-centric report path) ──
    #[serde(default)]
    pub segment_id: Option<Uuid>,
    #[serde(default)]
    pub shift_id: Option<Uuid>,
    #[serde(default)]
    pub interval_id: Option<Uuid>,
    #[serde(default)]
    pub interval_name: Option<String>,
    #[serde(default)]
    pub segment_type: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i32>,
    #[serde(default)]
    pub segment_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawAttendanceLog {
    id: Uuid,
    workspace_id: Uuid,
    employment_id: Uuid,
    punch_time: DateTime<Utc>,
    source: LogSource,
    device_id: Option<String>,
    raw_data: Option<serde_json::Value>,
    punch: i32,
    status: i32,
    created_at: DateTime<Utc>,
}

impl RawAttendanceLog {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        employment_id: Uuid,
        punch_time: DateTime<Utc>,
        source: LogSource,
        device_id: Option<String>,
        raw_data: Option<serde_json::Value>,
        punch: i32,
        status: i32,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            employment_id,
            punch_time,
            source,
            device_id,
            raw_data,
            punch,
            status,
            created_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn employment_id(&self) -> Uuid {
        self.employment_id
    }

    pub fn punch_time(&self) -> DateTime<Utc> {
        self.punch_time
    }

    pub fn source(&self) -> &LogSource {
        &self.source
    }

    pub fn device_id(&self) -> Option<&str> {
        self.device_id.as_deref()
    }

    pub fn raw_data(&self) -> Option<&serde_json::Value> {
        self.raw_data.as_ref()
    }

    pub fn punch(&self) -> i32 {
        self.punch
    }

    pub fn status(&self) -> i32 {
        self.status
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawAttendanceLogWithEmployee {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employment_id: Uuid,
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

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct AttendanceDailySummary {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employment_id: Uuid,
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
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BulkOtEntryError {
    pub line_number: usize,
    pub employment_id: String,
    pub error_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BulkOtUploadResult {
    pub total_rows: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub errors: Vec<BulkOtEntryError>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttendancePolicy {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttendancePolicyVersion {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub rules: AttendanceRules,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct AttendanceRules {
    pub work_unit_minutes: i32,
    pub rounding_policy: String,
    #[serde(default)]
    pub early_arrival_strategy: EarlyArrivalStrategy,
    #[serde(default)]
    pub grace_period_minutes: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
        let old_json = json!({
            "employment_id": Uuid::new_v4(),
            "employee_id": Uuid::new_v4(),
            "employee_name": "Test User",
            "employee_id_number": "EMP001",
            "total_std_units": 160,
            "total_ot_units": 20,
            "total_leave_days": 2
        });

        let summary: AttendanceSummary = serde_json::from_value(old_json).unwrap();

        assert_eq!(summary.total_std_units, 160);
        assert!(matches!(summary.unit_type, TimekeepingUnit::DAY));
        assert!((summary.unit_coefficient_hours - 0.0).abs() < f64::EPSILON);
        assert!((summary.computed_std_hours - 0.0).abs() < f64::EPSILON);
        assert!((summary.computed_ot_hours - 0.0).abs() < f64::EPSILON);
        assert_eq!(summary.employer_id, None);
        assert_eq!(summary.employer_name, None);
    }

    #[test]
    fn test_attendance_summary_multi_unit_serialization() {
        let summary = AttendanceSummary {
            employment_id: Uuid::new_v4(),
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
            employer_id: None,
            employer_name: None,
        };

        let serialized = serde_json::to_string(&summary).unwrap();

        let deserialized: AttendanceSummary = serde_json::from_str(&serialized).unwrap();
        assert!((deserialized.computed_ot_hours - 160.0).abs() < f64::EPSILON);
        assert!((deserialized.unit_coefficient_hours - 8.0).abs() < f64::EPSILON);
    }
}
