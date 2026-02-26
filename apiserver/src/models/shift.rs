use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct EmployeeShift {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub shift_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AssignShiftRequest {
    pub employee_id: Uuid,
    pub shift_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Shift {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub report_symbol: Option<String>,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub break_duration_minutes: i32,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_break_auto_deduct: bool,
    pub break_is_overtime: bool,
    pub early_punch_in_grace_period_minutes: i32,
    pub allow_early_ot: bool,
    pub is_active: bool,
    pub parent_shift_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub(super) fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
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
}

#[derive(Debug, Deserialize)]
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
    /// If true, update the current record (Correction).
    /// If false (default), create a new version if time-related fields changed.
    #[serde(default)]
    pub is_correction: bool,
    /// Effective date for the new version. If not provided, defaults to today.
    pub effective_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ShiftRoster {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub shift_id: Option<Uuid>,
    pub date: NaiveDate,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRosterRequest {
    pub employee_id: Uuid,
    pub shift_id: Option<Uuid>,
    pub date: NaiveDate,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RosterItem {
    pub date: NaiveDate,
    pub shift_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BulkRosterRequest {
    pub items: Vec<RosterItem>,
}
