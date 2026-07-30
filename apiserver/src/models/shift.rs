use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct EmployeeShift {
    pub id: Uuid,
    pub employment_id: Uuid,
    pub shift_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AssignShiftRequest {
    pub employment_id: Uuid,
    pub shift_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    /// If true, soft compliance warnings are accepted without override reason
    #[serde(default)]
    pub allow_warnings: bool,
    /// Manager-provided reason to override soft compliance violations
    #[serde(default)]
    pub override_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct Shift {
    id: Uuid,
    workspace_id: Uuid,
    name: String,
    report_symbol: Option<String>,
    is_active: bool,
    parent_shift_id: Option<Uuid>,
    location_id: Option<Uuid>,
    employer_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    /// Intervals are populated by the handler when listing shifts (not on create/update).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intervals: Option<Vec<ShiftDefaultIntervalEntity>>,
}

/// Re-export the diesel entity as the API interval response type
pub use crate::models::diesel::roster::ShiftDefaultIntervalEntity;

impl Shift {
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        name: String,
        report_symbol: Option<String>,
        is_active: bool,
        parent_shift_id: Option<Uuid>,
        location_id: Option<Uuid>,
        employer_id: Option<Uuid>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            name,
            report_symbol,
            is_active,
            parent_shift_id,
            location_id,
            employer_id,
            created_at,
            updated_at,
            intervals: None,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn report_symbol(&self) -> Option<&str> {
        self.report_symbol.as_deref()
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn parent_shift_id(&self) -> Option<Uuid> {
        self.parent_shift_id
    }
    pub fn location_id(&self) -> Option<Uuid> {
        self.location_id
    }
    pub fn employer_id(&self) -> Option<Uuid> {
        self.employer_id
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Mutable setter for name (used by update_shift_direct in mock).
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

pub(super) fn default_true() -> bool {
    true
}

/// Interval config within a shift create/update request.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
#[allow(clippy::struct_excessive_bools)]
pub struct CreateIntervalRequest {
    #[serde(default)]
    pub interval_name: Option<String>,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    #[serde(default)]
    pub break_duration_minutes: i32,
    #[serde(default)]
    pub break_start: Option<NaiveTime>,
    #[serde(default)]
    pub break_end: Option<NaiveTime>,
    #[serde(default = "default_true")]
    pub is_break_auto_deduct: bool,
    #[serde(default)]
    pub break_is_overtime: bool,
    #[serde(default)]
    pub early_punch_in_grace_period_minutes: i32,
    #[serde(default)]
    pub allow_early_ot: bool,
    #[serde(default)]
    pub max_early_ot_minutes: Option<i32>,
    #[serde(default)]
    pub ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode,
    #[serde(default)]
    pub is_night_shift: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateShiftRequest {
    pub name: String,
    #[serde(default)]
    pub report_symbol: Option<String>,
    #[serde(default)]
    pub location_id: Option<Uuid>,
    #[serde(default)]
    pub employer_id: Option<Uuid>,
    /// Intervals (time blocks) within this shift. At least 1 required.
    /// If empty, a default interval spanning the full shift time is auto-created
    /// (but callers SHOULD provide at least 1).
    #[serde(default)]
    pub intervals: Vec<CreateIntervalRequest>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateShiftRequest {
    pub name: String,
    #[serde(default)]
    pub report_symbol: Option<String>,
    #[serde(default)]
    pub location_id: Option<Uuid>,
    #[serde(default)]
    pub employer_id: Option<Uuid>,
    /// Intervals (time blocks) to replace the existing intervals for this shift.
    /// At least 1 required.
    #[serde(default)]
    pub intervals: Vec<CreateIntervalRequest>,
    /// If true, update the current record (Correction).
    /// If false (default), create a new version if interval config changed.
    #[serde(default)]
    pub is_correction: bool,
    /// Effective date for the new version. If not provided, defaults to today.
    pub effective_date: Option<NaiveDate>,
    /// Reason for this version change (audit trail).
    #[serde(default)]
    pub change_reason: Option<String>,
}

/// Version history item returned by GET /shifts/:id/versions.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct ShiftVersionHistoryItem {
    pub id: Uuid,
    pub template_id: Uuid,
    pub version_number: i32,
    pub name: String,
    pub report_symbol: Option<String>,
    pub location_id: Option<Uuid>,
    pub employer_id: Option<Uuid>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub change_type: String,
    pub change_reason: Option<String>,
    pub superseded_by: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub intervals: Vec<ShiftDefaultIntervalEntity>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct ShiftRoster {
    pub id: Uuid,
    pub employment_id: Uuid,
    pub shift_id: Option<Uuid>,
    pub date: NaiveDate,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateRosterRequest {
    pub employment_id: Uuid,
    pub shift_id: Option<Uuid>,
    pub date: NaiveDate,
    pub notes: Option<String>,
    /// If true, soft compliance warnings are accepted
    #[serde(default)]
    pub allow_warnings: bool,
    /// Manager-provided override reason
    #[serde(default)]
    pub override_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct RosterItem {
    pub date: NaiveDate,
    pub shift_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct BulkRosterRequest {
    pub items: Vec<RosterItem>,
    /// Employment ID for bulk roster operations
    #[serde(default)]
    pub employment_id: Option<Uuid>,
    /// If true, soft compliance warnings are accepted
    #[serde(default)]
    pub allow_warnings: bool,
    /// Manager-provided override reason
    #[serde(default)]
    pub override_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_shift_request_defaults() {
        let json = r#"{
            "name": "Morning",
            "intervals": [{
                "start_time": "08:00:00",
                "end_time": "17:00:00",
                "break_duration_minutes": 60
            }]
        }"#;
        let req: CreateShiftRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Morning");
        assert!(req.report_symbol.is_none());
        assert_eq!(req.intervals.len(), 1);
        let iv = &req.intervals[0];
        assert_eq!(iv.start_time, NaiveTime::from_hms_opt(8, 0, 0).unwrap());
        assert_eq!(iv.end_time, NaiveTime::from_hms_opt(17, 0, 0).unwrap());
        assert_eq!(iv.break_duration_minutes, 60);
        assert!(iv.is_break_auto_deduct); // default_true()
        assert!(!iv.break_is_overtime); // default false
        assert_eq!(iv.early_punch_in_grace_period_minutes, 0);
        assert!(!iv.allow_early_ot);
        assert!(iv.max_early_ot_minutes.is_none());
        assert!(iv.break_start.is_none());
        assert!(iv.break_end.is_none());
    }

    #[test]
    fn test_create_shift_request_with_all_fields() {
        let json = r#"{
            "name": "Night",
            "report_symbol": "N",
            "intervals": [{
                "interval_name": "Night Block",
                "start_time": "22:00:00",
                "end_time": "06:00:00",
                "break_duration_minutes": 30,
                "break_start": "01:00:00",
                "break_end": "01:30:00",
                "is_break_auto_deduct": false,
                "break_is_overtime": true,
                "early_punch_in_grace_period_minutes": 15,
                "allow_early_ot": true,
                "max_early_ot_minutes": 120,
                "is_night_shift": true
            }]
        }"#;
        let req: CreateShiftRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Night");
        assert_eq!(req.report_symbol.as_deref(), Some("N"));
        assert_eq!(req.intervals.len(), 1);
        let iv = &req.intervals[0];
        assert_eq!(iv.interval_name.as_deref(), Some("Night Block"));
        assert!(!iv.is_break_auto_deduct);
        assert!(iv.break_is_overtime);
        assert_eq!(iv.early_punch_in_grace_period_minutes, 15);
        assert!(iv.allow_early_ot);
        assert_eq!(iv.max_early_ot_minutes, Some(120));
        assert!(iv.is_night_shift);
    }

    #[test]
    fn test_update_shift_request_with_correction() {
        let json = r#"{
            "name": "Morning Updated",
            "intervals": [{
                "start_time": "07:30:00",
                "end_time": "16:30:00",
                "break_duration_minutes": 45
            }],
            "is_correction": true,
            "effective_date": "2026-04-01"
        }"#;
        let req: UpdateShiftRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Morning Updated");
        assert!(req.is_correction);
        assert_eq!(
            req.effective_date,
            Some(NaiveDate::from_ymd_opt(2026, 4, 1).unwrap())
        );
    }

    #[test]
    fn test_update_shift_request_defaults() {
        let json = r#"{
            "name": "Morning",
            "intervals": [{
                "start_time": "08:00:00",
                "end_time": "17:00:00",
                "break_duration_minutes": 60
            }]
        }"#;
        let req: UpdateShiftRequest = serde_json::from_str(json).unwrap();
        assert!(!req.is_correction);
        assert!(req.effective_date.is_none());
        assert_eq!(req.intervals.len(), 1);
        assert!(req.intervals[0].is_break_auto_deduct); // default_true
    }

    #[test]
    fn test_roster_item_serde() {
        let item = RosterItem {
            date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            shift_id: Some(Uuid::new_v4()),
            notes: Some("Off day".to_string()),
        };
        let json = serde_json::to_string(&item).unwrap();
        let back: RosterItem = serde_json::from_str(&json).unwrap();
        assert_eq!(back.date, item.date);
        assert!(back.shift_id.is_some());
        assert_eq!(back.notes.as_deref(), Some("Off day"));
    }

    #[test]
    fn test_bulk_roster_request_serde() {
        let req = BulkRosterRequest {
            items: vec![
                RosterItem {
                    date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                    shift_id: Some(Uuid::new_v4()),
                    notes: None,
                },
                RosterItem {
                    date: NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(),
                    shift_id: None,
                    notes: Some("Holiday".to_string()),
                },
            ],
            employment_id: None,
            allow_warnings: false,
            override_reason: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: BulkRosterRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.items.len(), 2);
    }

    #[test]
    fn test_create_roster_request_serde() {
        let json = r#"{
            "employment_id": "00000000-0000-0000-0000-000000000001",
            "shift_id": "00000000-0000-0000-0000-000000000002",
            "date": "2026-03-01",
            "notes": "First day"
        }"#;
        let req: CreateRosterRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.date, NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
        assert_eq!(req.notes.as_deref(), Some("First day"));
    }

    #[test]
    fn test_assign_shift_request_serde() {
        let json = r#"{
            "employment_id": "00000000-0000-0000-0000-000000000001",
            "shift_id": "00000000-0000-0000-0000-000000000002",
            "start_date": "2026-01-01",
            "end_date": "2026-12-31"
        }"#;
        let req: AssignShiftRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            req.employment_id,
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
        );
        assert_eq!(req.start_date, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(
            req.end_date,
            Some(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap())
        );
    }

    #[test]
    fn test_assign_shift_request_no_end_date() {
        let json = r#"{
            "employment_id": "00000000-0000-0000-0000-000000000001",
            "shift_id": "00000000-0000-0000-0000-000000000002",
            "start_date": "2026-01-01"
        }"#;
        let req: AssignShiftRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            req.employment_id,
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
        );
        assert!(req.end_date.is_none());
    }

    #[test]
    fn test_default_true_helper() {
        assert!(default_true());
    }
}
