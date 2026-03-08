use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a holiday entry in the workspace calendar.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Holiday {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub date: NaiveDate,
    /// national | company | half_day
    pub holiday_type: String,
    pub is_paid: bool,
    /// When is_paid=false, which leave type to auto-deduct from employee balance.
    pub leave_type_id: Option<Uuid>,
    /// Per-holiday OT multiplier override. None = use workspace default.
    pub ot_multiplier: Option<Decimal>,
    pub description: Option<String>,
    /// Threshold % of shift worked to reverse leave deduction. NULL = default 50%.
    pub work_threshold_percent: Option<i32>,
    /// Whether to credit holiday pay when employee works on this holiday.
    pub is_credit_on_work: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request payload for creating a new holiday.
#[derive(Debug, Deserialize)]
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
}

/// Request payload for updating an existing holiday.
#[derive(Debug, Deserialize)]
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
}

/// Workspace-level strategy for calculating OT when a shift crosses midnight
/// into a holiday (or vice versa).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
pub enum HolidayOvernightStrategy {
    /// Split the shift at 00:00 — hours on each side get their respective rate.
    /// Most legally compliant (Vietnam, Korea, Japan).
    SplitAtMidnight,
    /// Entire shift is classified by the punch-in date.
    ByPunchInDate,
    /// Shift is classified by whichever date contains the majority of hours.
    ByMajorityHours,
}

impl Default for HolidayOvernightStrategy {
    fn default() -> Self {
        Self::SplitAtMidnight
    }
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
        assert!(result.unwrap_err().contains("Unknown holiday overnight strategy"));
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
        let json = r#"{}"#;
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
        };

        let json = serde_json::to_string(&holiday).unwrap();
        let deserialized: Holiday = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Test Holiday");
        assert_eq!(deserialized.holiday_type, "national");
        assert!(deserialized.is_paid);
        assert_eq!(deserialized.ot_multiplier, Some(Decimal::from(2)));
    }
}
