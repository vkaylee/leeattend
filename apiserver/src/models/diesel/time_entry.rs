use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::models::TimeEntry;
use crate::schema::time_entries;
use crate::services::pairing::types::EntryType;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = time_entries)]
pub struct TimeEntryEntity {
    pub id: Uuid,
    pub punch_in_time: DateTime<Utc>,
    pub punch_out_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub entry_type: String,
    pub note: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_break_overtime: bool,
    pub template_id: Option<Uuid>,
    pub generation_metadata: Option<Value>,
    pub workspace_id: Uuid,
    pub employment_id: Uuid,
    pub segment_id: Option<Uuid>,
}

impl From<TimeEntryEntity> for TimeEntry {
    fn from(val: TimeEntryEntity) -> Self {
        let entry_type = val.entry_type.parse::<EntryType>().unwrap_or_else(|()| {
            tracing::warn!(
                entry_id = %val.id,
                raw_value = %val.entry_type,
                "Unrecognized entry_type value from database, falling back to Work"
            );
            EntryType::default()
        });
        TimeEntry::new(
            val.id,
            val.employment_id,
            val.workspace_id,
            val.template_id,
            val.segment_id,
            val.punch_in_time,
            val.punch_out_time,
            val.duration_minutes,
            val.is_break_overtime,
            entry_type,
            val.note,
            val.generation_metadata,
            val.created_at.unwrap_or_else(Utc::now),
            val.updated_at.unwrap_or_else(Utc::now),
        )
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = time_entries)]
pub struct NewTimeEntryEntity {
    pub id: Uuid,
    pub employment_id: Uuid,
    pub workspace_id: Uuid,
    pub punch_in_time: DateTime<Utc>,
    pub punch_out_time: Option<DateTime<Utc>>,
    pub entry_type: String,
    pub is_break_overtime: bool,
    pub note: Option<String>,
    pub template_id: Option<Uuid>,
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
