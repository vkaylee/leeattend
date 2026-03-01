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
