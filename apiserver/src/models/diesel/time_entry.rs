use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::models::TimeEntry;
use crate::schema::time_entries;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = time_entries)]
pub struct TimeEntryEntity {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub punch_in_time: DateTime<Utc>,
    pub punch_out_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub entry_type: String,
    pub note: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_break_overtime: bool,
    pub shift_id: Option<Uuid>,
    pub generation_metadata: Option<Value>,
    pub workspace_id: Uuid,
}

impl From<TimeEntryEntity> for TimeEntry {
    fn from(val: TimeEntryEntity) -> Self {
        TimeEntry {
            id: val.id,
            employee_id: val.employee_id,
            workspace_id: val.workspace_id,
            shift_id: val.shift_id,
            punch_in_time: val.punch_in_time,
            punch_out_time: val.punch_out_time,
            duration_minutes: val.duration_minutes,
            is_break_overtime: val.is_break_overtime,
            entry_type: val.entry_type,
            note: val.note,
            generation_metadata: val.generation_metadata,
            created_at: val.created_at.unwrap_or_else(Utc::now),
            updated_at: val.updated_at.unwrap_or_else(Utc::now),
        }
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = time_entries)]
pub struct NewTimeEntryEntity {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub workspace_id: Uuid,
    pub punch_in_time: DateTime<Utc>,
    pub punch_out_time: Option<DateTime<Utc>>,
    pub entry_type: String,
    pub is_break_overtime: bool,
    pub note: Option<String>,
    pub shift_id: Option<Uuid>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = time_entries)]
pub struct UpdateTimeEntryEntity {
    pub punch_in_time: Option<DateTime<Utc>>,
    pub punch_out_time: Option<Option<DateTime<Utc>>>,
    pub entry_type: Option<String>,
    pub note: Option<Option<String>>,
    pub updated_at: Option<DateTime<Utc>>,
}
