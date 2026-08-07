use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::{
    AdjustmentStatus, EarlyArrivalStrategy, ImportJobStatus, ImportTargetType, LogSource, OtStatus,
    TimekeepingUnit,
};
use crate::services::attendance_utils::AttendanceStatus;

pub struct NewAttendanceAdjustmentRequestParams {
    pub id: Uuid,
    pub employment_id: Uuid,
    pub time_entry_id: Option<Uuid>,
    pub requested_punch_time: DateTime<Utc>,
    pub reason: Option<String>,
    pub status: AdjustmentStatus,
    pub requested_by_user_id: Option<Uuid>,
    pub approved_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub attachment_file_ids: Option<Vec<Option<Uuid>>>,
}

pub struct NewRawAttendanceLogParams {
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
    pub source_timestamp: Option<String>,
    pub source_timezone: Option<String>,
    pub timestamp_kind: Option<String>,
    pub conversion_status: Option<String>,
    pub conversion_error: Option<String>,
    pub received_at: Option<DateTime<Utc>>,
}

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
    pub fn new(params: NewAttendanceAdjustmentRequestParams) -> Self {
        Self {
            id: params.id,
            employment_id: params.employment_id,
            time_entry_id: params.time_entry_id,
            requested_punch_time: params.requested_punch_time,
            reason: params.reason,
            status: params.status,
            requested_by_user_id: params.requested_by_user_id,
            approved_by_user_id: params.approved_by_user_id,
            created_at: params.created_at,
            updated_at: params.updated_at,
            attachment_file_ids: params.attachment_file_ids,
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
    #[serde(default)]
    pub grace_period_minutes: Option<i32>,
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
    /// IANA timezone for this employee at this date (e.g. "Asia/Ho_Chi_Minh"),
    /// resolved from the employee's work location. Used by the no-show frontend
    /// to compute A/B/C zones in the correct per-employee timezone.
    #[serde(default)]
    pub timezone: Option<String>,
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
    /// Identifies this row as one half of a break-split shift session.
    /// Values: `"first_half"`, `"second_half"`, or `None` for non-split rows.
    #[serde(default)]
    pub break_split: Option<String>,
}

/// ZKTeco `punch` codes stored in [`RawAttendanceLog::punch`].
///
/// These integers are transmitted verbatim by ZKTeco biometric terminals
/// (fingerprint / face-recognition devices) using the ZKTeco Standalone
/// SDK protocol. They flow through the ingestion pipeline unchanged so
/// that the raw payload can be replayed or audited.
pub mod raw_log_punch {
    /// Employee is checking in (start of a work interval).
    pub const CHECK_IN: i32 = 0;
    /// Employee is checking out (end of a work interval).
    pub const CHECK_OUT: i32 = 1;
    /// Employee is starting a break.
    pub const BREAK_OUT: i32 = 2;
    /// Employee is returning from a break.
    pub const BREAK_IN: i32 = 3;
    /// Overtime start (device-flagged).
    pub const OT_IN: i32 = 4;
    /// Overtime end (device-flagged).
    pub const OT_OUT: i32 = 5;
}

/// ZKTeco `status` codes stored in [`RawAttendanceLog::status`].
///
/// Verification method reported by the device — how the employee's
/// identity was confirmed at punch time. Preserved verbatim from the
/// ZKTeco SDK payload for audit and biometric-attack investigation.
pub mod raw_log_status {
    /// Normal / password verification.
    pub const NORMAL: i32 = 0;
    /// Fingerprint verification.
    pub const FINGERPRINT: i32 = 1;
    /// Password verification (explicit).
    pub const PASSWORD: i32 = 2;
    /// RFID / proximity card verification.
    pub const CARD: i32 = 3;
    /// Face-recognition verification.
    pub const FACE: i32 = 15;
}

/// A raw punch event captured directly from an attendance device (or an
/// equivalent source, e.g. the ESS web portal).
///
/// The `punch` and `status` fields carry ZKTeco protocol codes forwarded
/// verbatim from the device. Use the [`raw_log_punch`] and
/// [`raw_log_status`] constants when comparing or writing these values.
/// Unrecognized codes are tolerated so that firmware upgrades that add
/// new codes do not break ingestion.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawAttendanceLog {
    id: Uuid,
    workspace_id: Uuid,
    employment_id: Uuid,
    punch_time: DateTime<Utc>,
    source: LogSource,
    device_id: Option<String>,
    raw_data: Option<serde_json::Value>,
    /// ZKTeco punch code (see [`raw_log_punch`]): 0=IN, 1=OUT, 2/3=break, 4/5=OT.
    punch: i32,
    /// ZKTeco verification method (see [`raw_log_status`]): 0=normal, 1=fingerprint,
    /// 2=password, 3=card, 15=face.
    status: i32,
    created_at: DateTime<Utc>,
    pub source_timestamp: Option<String>,
    pub source_timezone: Option<String>,
    pub timestamp_kind: Option<String>,
    pub conversion_status: Option<String>,
    pub conversion_error: Option<String>,
    pub received_at: Option<DateTime<Utc>>,
}

impl RawAttendanceLog {
    pub fn new(params: NewRawAttendanceLogParams) -> Self {
        Self {
            id: params.id,
            workspace_id: params.workspace_id,
            employment_id: params.employment_id,
            punch_time: params.punch_time,
            source: params.source,
            device_id: params.device_id,
            raw_data: params.raw_data,
            punch: params.punch,
            status: params.status,
            created_at: params.created_at,
            source_timestamp: params.source_timestamp,
            source_timezone: params.source_timezone,
            timestamp_kind: params.timestamp_kind,
            conversion_status: params.conversion_status,
            conversion_error: params.conversion_error,
            received_at: params.received_at,
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
    pub standard_minutes: i32,
    pub early_ot_minutes: i32,
    pub late_ot_minutes: i32,
    pub break_ot_minutes: i32,
    pub is_weekend: bool,
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

/// Rounding strategy for converting work minutes into work units.
///
/// Serialized as lowercase snake_case (`"floor"`, `"round"`, `"ceil"`, `"none"`).
/// Deserialization is tolerant: unknown legacy values (e.g. `"round_down"`) fall
/// back to `None`, which truncates like the previous string-matching code did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoundingPolicy {
    #[default]
    Floor,
    Round,
    Ceil,
    None,
}

impl std::fmt::Display for RoundingPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Floor => write!(f, "floor"),
            Self::Round => write!(f, "round"),
            Self::Ceil => write!(f, "ceil"),
            Self::None => write!(f, "none"),
        }
    }
}

impl std::str::FromStr for RoundingPolicy {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "floor" => Ok(Self::Floor),
            "round" => Ok(Self::Round),
            "ceil" => Ok(Self::Ceil),
            "none" => Ok(Self::None),
            _ => Err(()),
        }
    }
}

impl<'de> Deserialize<'de> for RoundingPolicy {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        // Tolerant: preserves prior "unknown → truncate" (== Floor for positives)
        // semantics found in `AttendanceCalculator::calculate_units`.
        Ok(match s.as_str() {
            "floor" => Self::Floor,
            "round" => Self::Round,
            "ceil" => Self::Ceil,
            _ => Self::None,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct AttendanceRules {
    pub work_unit_minutes: i32,
    pub rounding_policy: RoundingPolicy,
    #[serde(default)]
    pub early_arrival_strategy: EarlyArrivalStrategy,
    #[serde(default)]
    pub grace_period_minutes: Option<i32>,
}

/// Default attendance rules used across LeeAttend.
///
/// - `work_unit_minutes: 30` — standard 30-minute work units
/// - `rounding_policy: Floor` — conservative rounding (employee-friendly)
/// - `early_arrival_strategy: Block` — no early OT by default
impl Default for AttendanceRules {
    fn default() -> Self {
        Self {
            work_unit_minutes: 30,
            rounding_policy: RoundingPolicy::Floor,
            early_arrival_strategy: EarlyArrivalStrategy::Block,
            grace_period_minutes: None,
        }
    }
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

    #[test]
    fn attendance_rules_default_values() {
        let rules = AttendanceRules::default();
        assert_eq!(rules.work_unit_minutes, 30);
        assert!(matches!(rules.rounding_policy, RoundingPolicy::Floor));
        assert!(matches!(
            rules.early_arrival_strategy,
            EarlyArrivalStrategy::Block
        ));
        assert_eq!(rules.grace_period_minutes, None);
    }

    #[test]
    fn attendance_rules_default_is_consistent() {
        let manual = AttendanceRules {
            work_unit_minutes: 30,
            rounding_policy: RoundingPolicy::Floor,
            early_arrival_strategy: EarlyArrivalStrategy::Block,
            grace_period_minutes: None,
        };
        let defaulted = AttendanceRules::default();
        assert_eq!(manual.work_unit_minutes, defaulted.work_unit_minutes);
        assert_eq!(manual.rounding_policy, defaulted.rounding_policy);
        assert_eq!(
            manual.early_arrival_strategy,
            defaulted.early_arrival_strategy
        );
        assert_eq!(manual.grace_period_minutes, defaulted.grace_period_minutes);
    }

    #[test]
    fn corrupted_json_falls_back_to_default_not_zero() {
        let bad_json = serde_json::json!({"unknown_field": "garbage"});
        let result: AttendanceRules = serde_json::from_value(bad_json).unwrap_or_default();
        assert_eq!(result.work_unit_minutes, 30);
        assert!(matches!(result.rounding_policy, RoundingPolicy::Floor));
    }
}
