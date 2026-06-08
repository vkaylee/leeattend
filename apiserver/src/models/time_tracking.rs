use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct TimeEntryLogFilter {
    employment_id: Option<Uuid>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    limit: i64,
    offset: i64,
}

impl TimeEntryLogFilter {
    pub fn new(
        employment_id: Option<Uuid>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Self {
        Self {
            employment_id,
            from,
            to,
            limit,
            offset,
        }
    }

    pub fn employment_id(&self) -> Option<Uuid> {
        self.employment_id
    }

    pub fn from(&self) -> Option<DateTime<Utc>> {
        self.from
    }

    pub fn to(&self) -> Option<DateTime<Utc>> {
        self.to
    }

    pub fn limit(&self) -> i64 {
        self.limit
    }

    pub fn offset(&self) -> i64 {
        self.offset
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TimeEntry {
    id: Uuid,
    employment_id: Uuid,
    workspace_id: Uuid,
    shift_id: Option<Uuid>,
    punch_in_time: DateTime<Utc>,
    punch_out_time: Option<DateTime<Utc>>,
    duration_minutes: Option<i32>,
    is_break_overtime: bool,
    entry_type: String,
    note: Option<String>,
    generation_metadata: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TimeEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        employment_id: Uuid,
        workspace_id: Uuid,
        shift_id: Option<Uuid>,
        punch_in_time: DateTime<Utc>,
        punch_out_time: Option<DateTime<Utc>>,
        duration_minutes: Option<i32>,
        is_break_overtime: bool,
        entry_type: String,
        note: Option<String>,
        generation_metadata: Option<serde_json::Value>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            employment_id,
            workspace_id,
            shift_id,
            punch_in_time,
            punch_out_time,
            duration_minutes,
            is_break_overtime,
            entry_type,
            note,
            generation_metadata,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn employment_id(&self) -> Uuid {
        self.employment_id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn shift_id(&self) -> Option<Uuid> {
        self.shift_id
    }

    pub fn punch_in_time(&self) -> DateTime<Utc> {
        self.punch_in_time
    }

    pub fn punch_out_time(&self) -> Option<DateTime<Utc>> {
        self.punch_out_time
    }

    pub fn duration_minutes(&self) -> Option<i32> {
        self.duration_minutes
    }

    pub fn is_break_overtime(&self) -> bool {
        self.is_break_overtime
    }

    pub fn entry_type(&self) -> &str {
        &self.entry_type
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn generation_metadata(&self) -> Option<&serde_json::Value> {
        self.generation_metadata.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn set_punch_in_time(&mut self, t: DateTime<Utc>) {
        self.punch_in_time = t;
        self.recalculate_duration();
    }

    pub fn set_punch_out_time(&mut self, t: Option<DateTime<Utc>>) {
        self.punch_out_time = t;
        self.recalculate_duration();
    }

    pub fn set_is_break_overtime(&mut self, b: bool) {
        self.is_break_overtime = b;
    }

    pub fn set_entry_type(&mut self, et: String) {
        self.entry_type = et;
    }

    pub fn set_note(&mut self, note: Option<String>) {
        self.note = note;
    }

    fn recalculate_duration(&mut self) {
        if let Some(po) = self.punch_out_time {
            let duration = po.signed_duration_since(self.punch_in_time);
            self.duration_minutes = Some(duration.num_minutes() as i32);
        } else {
            self.duration_minutes = None;
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeEntryWithEmployee {
    id: Uuid,
    employment_id: Uuid,
    workspace_id: Uuid,
    shift_id: Option<Uuid>,
    punch_in_time: DateTime<Utc>,
    punch_out_time: Option<DateTime<Utc>>,
    duration_minutes: Option<i32>,
    is_break_overtime: bool,
    entry_type: String,
    note: Option<String>,
    generation_metadata: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    employee_full_name: String,
    employee_id_number: String,
}

impl TimeEntryWithEmployee {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        employment_id: Uuid,
        workspace_id: Uuid,
        shift_id: Option<Uuid>,
        punch_in_time: DateTime<Utc>,
        punch_out_time: Option<DateTime<Utc>>,
        duration_minutes: Option<i32>,
        is_break_overtime: bool,
        entry_type: String,
        note: Option<String>,
        generation_metadata: Option<serde_json::Value>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        employee_full_name: String,
        employee_id_number: String,
    ) -> Self {
        Self {
            id,
            employment_id,
            workspace_id,
            shift_id,
            punch_in_time,
            punch_out_time,
            duration_minutes,
            is_break_overtime,
            entry_type,
            note,
            generation_metadata,
            created_at,
            updated_at,
            employee_full_name,
            employee_id_number,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn employment_id(&self) -> Uuid {
        self.employment_id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn shift_id(&self) -> Option<Uuid> {
        self.shift_id
    }

    pub fn punch_in_time(&self) -> DateTime<Utc> {
        self.punch_in_time
    }

    pub fn punch_out_time(&self) -> Option<DateTime<Utc>> {
        self.punch_out_time
    }

    pub fn duration_minutes(&self) -> Option<i32> {
        self.duration_minutes
    }

    pub fn is_break_overtime(&self) -> bool {
        self.is_break_overtime
    }

    pub fn entry_type(&self) -> &str {
        &self.entry_type
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn generation_metadata(&self) -> Option<&serde_json::Value> {
        self.generation_metadata.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn employee_full_name(&self) -> &str {
        &self.employee_full_name
    }

    pub fn employee_id_number(&self) -> &str {
        &self.employee_id_number
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct PunchInRequest {
    employment_id: Uuid,
    entry_type: Option<String>,
    is_break_overtime: Option<bool>,
    note: Option<String>,
}

impl PunchInRequest {
    pub fn new(
        employment_id: Uuid,
        entry_type: Option<String>,
        is_break_overtime: Option<bool>,
        note: Option<String>,
    ) -> Self {
        Self {
            employment_id,
            entry_type,
            is_break_overtime,
            note,
        }
    }

    pub fn employment_id(&self) -> Uuid {
        self.employment_id
    }

    pub fn entry_type(&self) -> Option<&str> {
        self.entry_type.as_deref()
    }

    pub fn is_break_overtime(&self) -> Option<bool> {
        self.is_break_overtime
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct PunchOutRequest {
    note: Option<String>,
}

impl PunchOutRequest {
    pub fn new(note: Option<String>) -> Self {
        Self { note }
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct UpdateTimeEntryRequest {
    punch_in_time: Option<DateTime<Utc>>,
    punch_out_time: Option<DateTime<Utc>>,
    entry_type: Option<String>,
    note: Option<String>,
}

impl UpdateTimeEntryRequest {
    pub fn new(
        punch_in_time: Option<DateTime<Utc>>,
        punch_out_time: Option<DateTime<Utc>>,
        entry_type: Option<String>,
        note: Option<String>,
    ) -> Self {
        Self {
            punch_in_time,
            punch_out_time,
            entry_type,
            note,
        }
    }

    pub fn punch_in_time(&self) -> Option<DateTime<Utc>> {
        self.punch_in_time
    }

    pub fn punch_out_time(&self) -> Option<DateTime<Utc>> {
        self.punch_out_time
    }

    pub fn entry_type(&self) -> Option<&str> {
        self.entry_type.as_deref()
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}
