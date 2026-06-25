use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A fully-resolved work segment produced by shift resolution.
/// Contains all the information needed to create or match a WorkSegment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct ResolvedSegment {
    pub shift_id: Uuid,
    pub interval_id: Option<Uuid>,
    pub sort_order: i32,
    pub scheduled_start: NaiveTime,
    pub scheduled_end: NaiveTime,
    pub source: String, // "roster" | "pattern" | "rule" | "fixed"
    pub break_duration_minutes: Option<i32>,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_break_auto_deduct: Option<bool>,
    pub break_is_overtime: Option<bool>,
    pub allow_early_ot: bool,
    pub max_early_ot_minutes: Option<i32>,
    pub early_punch_in_grace_period_minutes: i32,
    pub ot_aggregation_mode: String,
    pub is_night_shift: bool,
}
