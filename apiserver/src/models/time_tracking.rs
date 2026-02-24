use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct TimeEntry {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub workspace_id: Uuid,
    pub shift_id: Option<Uuid>,
    pub punch_in_time: DateTime<Utc>,
    pub punch_out_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub is_break_overtime: bool,
    pub entry_type: String,
    pub note: Option<String>,
    pub generation_metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct TimeEntryWithEmployee {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub workspace_id: Uuid,
    pub shift_id: Option<Uuid>,
    pub punch_in_time: DateTime<Utc>,
    pub punch_out_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub is_break_overtime: bool,
    pub entry_type: String,
    pub note: Option<String>,
    pub generation_metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub employee_full_name: String,
    pub employee_id_number: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PunchInRequest {
    pub employee_id: Uuid,
    pub entry_type: Option<String>,
    pub is_break_overtime: Option<bool>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PunchOutRequest {
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateTimeEntryRequest {
    pub punch_in_time: Option<DateTime<Utc>>,
    pub punch_out_time: Option<DateTime<Utc>>,
    pub entry_type: Option<String>,
    pub note: Option<String>,
}
