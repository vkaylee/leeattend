use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a holiday entry in the workspace calendar.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Holiday {
    id: Uuid,
    workspace_id: Uuid,
    name: String,
    date: NaiveDate,
    /// national | company | half_day
    holiday_type: String,
    is_paid: bool,
    /// When is_paid=false, which leave type to auto-deduct from employee balance.
    leave_type_id: Option<Uuid>,
    /// Per-holiday OT multiplier override. None = use workspace default.
    ot_multiplier: Option<Decimal>,
    description: Option<String>,
    /// Threshold % of shift worked to reverse leave deduction. NULL = default 50%.
    work_threshold_percent: Option<i32>,
    /// Whether to credit holiday pay when employee works on this holiday.
    is_credit_on_work: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    location_id: Option<Uuid>,
}

impl Holiday {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        name: String,
        date: NaiveDate,
        holiday_type: String,
        is_paid: bool,
        leave_type_id: Option<Uuid>,
        ot_multiplier: Option<Decimal>,
        description: Option<String>,
        work_threshold_percent: Option<i32>,
        is_credit_on_work: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        location_id: Option<Uuid>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            name,
            date,
            holiday_type,
            is_paid,
            leave_type_id,
            ot_multiplier,
            description,
            work_threshold_percent,
            is_credit_on_work,
            created_at,
            updated_at,
            location_id,
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

    pub fn date(&self) -> NaiveDate {
        self.date
    }

    pub fn holiday_type(&self) -> &str {
        &self.holiday_type
    }

    pub fn is_paid(&self) -> bool {
        self.is_paid
    }

    pub fn leave_type_id(&self) -> Option<Uuid> {
        self.leave_type_id
    }

    pub fn ot_multiplier(&self) -> Option<Decimal> {
        self.ot_multiplier
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn work_threshold_percent(&self) -> Option<i32> {
        self.work_threshold_percent
    }

    pub fn is_credit_on_work(&self) -> bool {
        self.is_credit_on_work
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn location_id(&self) -> Option<Uuid> {
        self.location_id
    }
}

/// Request payload for creating a new holiday.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateHolidayRequest {
    pub name: String,
    pub date: NaiveDate,
    #[serde(default = "default_holiday_type")]
    pub holiday_type: String,
    #[serde(default = "default_true")]
    pub is_paid: bool,
    /// When is_paid=false, which leave type to auto-deduct.
    pub leave_type_id: Option<Uuid>,
    pub ot_multiplier: Option<Decimal>,
    pub description: Option<String>,
    pub work_threshold_percent: Option<i32>,
    #[serde(default = "default_true")]
    pub is_credit_on_work: bool,
    pub location_id: Option<Uuid>,
}

/// Request payload for updating an existing holiday.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateHolidayRequest {
    pub name: Option<String>,
    pub date: Option<NaiveDate>,
    pub holiday_type: Option<String>,
    pub is_paid: Option<bool>,
    pub leave_type_id: Option<Uuid>,
    pub ot_multiplier: Option<Decimal>,
    pub description: Option<String>,
    pub work_threshold_percent: Option<i32>,
    pub is_credit_on_work: Option<bool>,
    pub location_id: Option<Option<Uuid>>,
}

/// Workspace-level strategy for calculating OT when a shift crosses midnight
/// into a holiday (or vice versa).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum HolidayOvernightStrategy {
    /// Split the shift at 00:00 — hours on each side get their respective rate.
    /// Most legally compliant (Vietnam, Korea, Japan).
    #[default]
    SplitAtMidnight,
    /// Entire shift is classified by the punch-in date.
    ByPunchInDate,
    /// Shift is classified by whichever date contains the majority of hours.
    ByMajorityHours,
}

impl std::fmt::Display for HolidayOvernightStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SplitAtMidnight => write!(f, "split_at_midnight"),
            Self::ByPunchInDate => write!(f, "by_punch_in_date"),
            Self::ByMajorityHours => write!(f, "by_majority_hours"),
        }
    }
}

impl std::str::FromStr for HolidayOvernightStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "split_at_midnight" => Ok(Self::SplitAtMidnight),
            "by_punch_in_date" => Ok(Self::ByPunchInDate),
            "by_majority_hours" => Ok(Self::ByMajorityHours),
            _ => Err(format!("Unknown holiday overnight strategy: {s}")),
        }
    }
}

fn default_holiday_type() -> String {
    "national".to_string()
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_holiday_overnight_strategy_default() {
        let strategy = HolidayOvernightStrategy::default();
        assert_eq!(strategy, HolidayOvernightStrategy::SplitAtMidnight);
    }

    #[test]
    fn test_holiday_overnight_strategy_display() {
        assert_eq!(
            HolidayOvernightStrategy::SplitAtMidnight.to_string(),
            "split_at_midnight"
        );
        assert_eq!(
            HolidayOvernightStrategy::ByPunchInDate.to_string(),
            "by_punch_in_date"
        );
        assert_eq!(
            HolidayOvernightStrategy::ByMajorityHours.to_string(),
            "by_majority_hours"
        );
    }

    #[test]
    fn test_holiday_overnight_strategy_from_str() {
        assert_eq!(
            HolidayOvernightStrategy::from_str("split_at_midnight").unwrap(),
            HolidayOvernightStrategy::SplitAtMidnight
        );
        assert_eq!(
            HolidayOvernightStrategy::from_str("by_punch_in_date").unwrap(),
            HolidayOvernightStrategy::ByPunchInDate
        );
        assert_eq!(
            HolidayOvernightStrategy::from_str("by_majority_hours").unwrap(),
            HolidayOvernightStrategy::ByMajorityHours
        );
    }

    #[test]
    fn test_holiday_overnight_strategy_from_str_invalid() {
        let result = HolidayOvernightStrategy::from_str("invalid_strategy");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unknown holiday overnight strategy"));
    }

    #[test]
    fn test_holiday_overnight_strategy_serde_roundtrip() {
        let strategies = vec![
            HolidayOvernightStrategy::SplitAtMidnight,
            HolidayOvernightStrategy::ByPunchInDate,
            HolidayOvernightStrategy::ByMajorityHours,
        ];
        for strategy in strategies {
            let json = serde_json::to_string(&strategy).unwrap();
            let deserialized: HolidayOvernightStrategy = serde_json::from_str(&json).unwrap();
            assert_eq!(strategy, deserialized);
        }
    }

    #[test]
    fn test_default_holiday_type() {
        assert_eq!(default_holiday_type(), "national");
    }

    #[test]
    fn test_default_true() {
        assert!(default_true());
    }

    #[test]
    fn test_create_holiday_request_deserialization() {
        let json = r#"{
            "name": "New Year",
            "date": "2026-01-01"
        }"#;
        let req: CreateHolidayRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "New Year");
        assert_eq!(req.date, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(req.holiday_type, "national"); // default
        assert!(req.is_paid); // default true
        assert!(req.is_credit_on_work); // default true
        assert!(req.leave_type_id.is_none());
        assert!(req.ot_multiplier.is_none());
    }

    #[test]
    fn test_create_holiday_request_with_all_fields() {
        let json = r#"{
            "name": "Company Day",
            "date": "2026-06-15",
            "holiday_type": "company",
            "is_paid": false,
            "ot_multiplier": "2.5",
            "description": "Annual company day",
            "work_threshold_percent": 75,
            "is_credit_on_work": false
        }"#;
        let req: CreateHolidayRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Company Day");
        assert_eq!(req.holiday_type, "company");
        assert!(!req.is_paid);
        assert!(!req.is_credit_on_work);
        assert_eq!(req.work_threshold_percent, Some(75));
        assert!(req.description.is_some());
    }

    #[test]
    fn test_update_holiday_request_all_none() {
        let json = r"{}";
        let req: UpdateHolidayRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_none());
        assert!(req.date.is_none());
        assert!(req.holiday_type.is_none());
        assert!(req.is_paid.is_none());
    }

    #[test]
    fn test_holiday_serialization_roundtrip() {
        let holiday = Holiday {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            name: "Test Holiday".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            holiday_type: "national".to_string(),
            is_paid: true,
            leave_type_id: None,
            ot_multiplier: Some(Decimal::from(2)),
            description: Some("A test holiday".to_string()),
            work_threshold_percent: Some(50),
            is_credit_on_work: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            location_id: None,
        };

        let json = serde_json::to_string(&holiday).unwrap();
        let deserialized: Holiday = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Test Holiday");
        assert_eq!(deserialized.holiday_type, "national");
        assert!(deserialized.is_paid);
        assert_eq!(deserialized.ot_multiplier, Some(Decimal::from(2)));
    }
}
