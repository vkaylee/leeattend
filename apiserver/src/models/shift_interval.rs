use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct ShiftDefaultInterval {
    pub id: Uuid,
    pub shift_id: Uuid,
    pub sort_order: i32,
    pub interval_name: Option<String>,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub break_duration_minutes: Option<i32>,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_break_auto_deduct: Option<bool>,
    pub break_is_overtime: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateShiftIntervalRequest {
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default)]
    pub interval_name: Option<String>,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    #[serde(default)]
    pub break_duration_minutes: Option<i32>,
    #[serde(default)]
    pub break_start: Option<NaiveTime>,
    #[serde(default)]
    pub break_end: Option<NaiveTime>,
    #[serde(default)]
    pub is_break_auto_deduct: Option<bool>,
    #[serde(default)]
    pub break_is_overtime: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateShiftIntervalRequest {
    #[serde(default)]
    pub sort_order: Option<i32>,
    #[serde(default)]
    pub interval_name: Option<String>,
    #[serde(default)]
    pub start_time: Option<NaiveTime>,
    #[serde(default)]
    pub end_time: Option<NaiveTime>,
    #[serde(default)]
    pub break_duration_minutes: Option<i32>,
    #[serde(default)]
    pub break_start: Option<NaiveTime>,
    #[serde(default)]
    pub break_end: Option<NaiveTime>,
    #[serde(default)]
    pub is_break_auto_deduct: Option<bool>,
    #[serde(default)]
    pub break_is_overtime: Option<bool>,
}
