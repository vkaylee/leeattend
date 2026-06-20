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
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
#[allow(clippy::struct_excessive_bools)]
pub struct Shift {
    id: Uuid,
    workspace_id: Uuid,
    name: String,
    report_symbol: Option<String>,
    start_time: NaiveTime,
    end_time: NaiveTime,
    break_duration_minutes: i32,
    break_start: Option<NaiveTime>,
    break_end: Option<NaiveTime>,
    is_break_auto_deduct: bool,
    break_is_overtime: bool,
    early_punch_in_grace_period_minutes: i32,
    allow_early_ot: bool,
    max_early_ot_minutes: Option<i32>,
    is_active: bool,
    parent_shift_id: Option<Uuid>,
    location_id: Option<Uuid>,
    employer_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Shift {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        name: String,
        report_symbol: Option<String>,
        start_time: NaiveTime,
        end_time: NaiveTime,
        break_duration_minutes: i32,
        break_start: Option<NaiveTime>,
        break_end: Option<NaiveTime>,
        is_break_auto_deduct: bool,
        break_is_overtime: bool,
        early_punch_in_grace_period_minutes: i32,
        allow_early_ot: bool,
        max_early_ot_minutes: Option<i32>,
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
            start_time,
            end_time,
            break_duration_minutes,
            break_start,
            break_end,
            is_break_auto_deduct,
            break_is_overtime,
            early_punch_in_grace_period_minutes,
            allow_early_ot,
            max_early_ot_minutes,
            is_active,
            parent_shift_id,
            location_id,
            employer_id,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid { self.id }
    pub fn workspace_id(&self) -> Uuid { self.workspace_id }
    pub fn name(&self) -> &str { &self.name }
    pub fn report_symbol(&self) -> Option<&str> { self.report_symbol.as_deref() }
    pub fn start_time(&self) -> NaiveTime { self.start_time }
    pub fn end_time(&self) -> NaiveTime { self.end_time }
    pub fn break_duration_minutes(&self) -> i32 { self.break_duration_minutes }
    pub fn break_start(&self) -> Option<NaiveTime> { self.break_start }
    pub fn break_end(&self) -> Option<NaiveTime> { self.break_end }
    pub fn is_break_auto_deduct(&self) -> bool { self.is_break_auto_deduct }
    pub fn break_is_overtime(&self) -> bool { self.break_is_overtime }
    pub fn early_punch_in_grace_period_minutes(&self) -> i32 { self.early_punch_in_grace_period_minutes }
    pub fn allow_early_ot(&self) -> bool { self.allow_early_ot }
    pub fn max_early_ot_minutes(&self) -> Option<i32> { self.max_early_ot_minutes }
    pub fn is_active(&self) -> bool { self.is_active }
    pub fn parent_shift_id(&self) -> Option<Uuid> { self.parent_shift_id }
    pub fn location_id(&self) -> Option<Uuid> { self.location_id }
    pub fn employer_id(&self) -> Option<Uuid> { self.employer_id }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }

    /// Mutable setter for name (used by update_shift_direct in mock).
    pub fn set_name(&mut self, name: String) { self.name = name; }
}

pub(super) fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateShiftRequest {
    pub name: String,
    #[serde(default)]
    pub report_symbol: Option<String>,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
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
    pub location_id: Option<Uuid>,
    #[serde(default)]
    pub employer_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[allow(clippy::struct_excessive_bools)]
pub struct UpdateShiftRequest {
    pub name: String,
    #[serde(default)]
    pub report_symbol: Option<String>,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
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
    pub location_id: Option<Uuid>,
    #[serde(default)]
    pub employer_id: Option<Uuid>,
    /// If true, update the current record (Correction).
    /// If false (default), create a new version if time-related fields changed.
    #[serde(default)]
    pub is_correction: bool,
    /// Effective date for the new version. If not provided, defaults to today.
    pub effective_date: Option<NaiveDate>,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_shift_request_defaults() {
        let json = r#"{
            "name": "Morning",
            "start_time": "08:00:00",
            "end_time": "17:00:00",
            "break_duration_minutes": 60
        }"#;
        let req: CreateShiftRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Morning");
        assert_eq!(req.start_time, NaiveTime::from_hms_opt(8, 0, 0).unwrap());
        assert_eq!(req.end_time, NaiveTime::from_hms_opt(17, 0, 0).unwrap());
        assert_eq!(req.break_duration_minutes, 60);
        assert!(req.is_break_auto_deduct); // default_true()
        assert!(!req.break_is_overtime); // default false
        assert_eq!(req.early_punch_in_grace_period_minutes, 0);
        assert!(!req.allow_early_ot);
        assert!(req.max_early_ot_minutes.is_none());
        assert!(req.report_symbol.is_none());
        assert!(req.break_start.is_none());
        assert!(req.break_end.is_none());
    }

    #[test]
    fn test_create_shift_request_with_all_fields() {
        let json = r#"{
            "name": "Night",
            "report_symbol": "N",
            "start_time": "22:00:00",
            "end_time": "06:00:00",
            "break_duration_minutes": 30,
            "break_start": "01:00:00",
            "break_end": "01:30:00",
            "is_break_auto_deduct": false,
            "break_is_overtime": true,
            "early_punch_in_grace_period_minutes": 15,
            "allow_early_ot": true,
            "max_early_ot_minutes": 120
        }"#;
        let req: CreateShiftRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Night");
        assert_eq!(req.report_symbol.as_deref(), Some("N"));
        assert!(!req.is_break_auto_deduct);
        assert!(req.break_is_overtime);
        assert_eq!(req.early_punch_in_grace_period_minutes, 15);
        assert!(req.allow_early_ot);
        assert_eq!(req.max_early_ot_minutes, Some(120));
    }

    #[test]
    fn test_update_shift_request_with_correction() {
        let json = r#"{
            "name": "Morning Updated",
            "start_time": "07:30:00",
            "end_time": "16:30:00",
            "break_duration_minutes": 45,
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
            "start_time": "08:00:00",
            "end_time": "17:00:00",
            "break_duration_minutes": 60
        }"#;
        let req: UpdateShiftRequest = serde_json::from_str(json).unwrap();
        assert!(!req.is_correction);
        assert!(req.effective_date.is_none());
        assert!(req.is_break_auto_deduct); // default_true
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
