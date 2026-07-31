use crate::models::ScheduleTrace;
use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A fully-resolved work segment produced by shift resolution.
/// Contains all the information needed to create or match a WorkSegment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct ResolvedSegment {
    pub template_id: Uuid,
    pub interval_id: Option<Uuid>,
    pub sort_order: i32,
    pub scheduled_start: NaiveTime,
    pub scheduled_end: NaiveTime,
    pub source: String, // "roster" | "pattern" | "assignment_rule" | "fixed"
    /// When this segment is resolved for a specific date, it is set here.
    #[serde(default)]
    pub date: Option<NaiveDate>,
    /// Shift metadata from RosterShift.
    #[serde(default)]
    pub shift_name: Option<String>,
    #[serde(default)]
    pub shift_symbol: Option<String>,
    /// Interval name from ShiftDefaultInterval, if this segment came from one.
    #[serde(default)]
    pub interval_name: Option<String>,
    /// Segment type: "work" | "standby" | "on_call" | "training" | "travel".
    #[serde(default = "default_segment_type")]
    pub segment_type: String,
    pub break_duration_minutes: Option<i32>,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_break_auto_deduct: Option<bool>,
    pub break_is_overtime: Option<bool>,
    pub allow_early_ot: bool,
    pub max_early_ot_minutes: Option<i32>,
    pub early_punch_in_grace_period_minutes: i32,
    #[serde(default)]
    pub ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode,
    pub is_night_shift: bool,
    /// Schedule resolution traces.
    #[serde(default)]
    pub traces: Vec<ScheduleTrace>,
    /// Holiday status for the date this segment falls on.
    #[serde(default)]
    pub is_holiday: bool,
    #[serde(default)]
    pub is_holiday_paid: bool,
    #[serde(default)]
    pub holiday_name: Option<String>,
}

fn default_segment_type() -> String {
    "work".to_string()
}
