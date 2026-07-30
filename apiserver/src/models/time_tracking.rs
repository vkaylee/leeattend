use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::pairing::types::EntryType;

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
    segment_id: Option<Uuid>,
    punch_in_time: DateTime<Utc>,
    punch_out_time: Option<DateTime<Utc>>,
    duration_minutes: Option<i32>,
    is_break_overtime: bool,
    entry_type: EntryType,
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
        segment_id: Option<Uuid>,
        punch_in_time: DateTime<Utc>,
        punch_out_time: Option<DateTime<Utc>>,
        duration_minutes: Option<i32>,
        is_break_overtime: bool,
        entry_type: EntryType,
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
            segment_id,
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

    pub fn segment_id(&self) -> Option<Uuid> {
        self.segment_id
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

    pub fn entry_type(&self) -> EntryType {
        self.entry_type
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

    pub fn set_segment_id(&mut self, sid: Option<Uuid>) {
        self.segment_id = sid;
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

    pub fn set_entry_type(&mut self, et: EntryType) {
        self.entry_type = et;
    }

    pub fn set_note(&mut self, note: Option<String>) {
        self.note = note;
    }

    fn recalculate_duration(&mut self) {
        if let Some(po) = self.punch_out_time {
            let duration = po.signed_duration_since(self.punch_in_time);
            self.duration_minutes = Some(i32::try_from(duration.num_minutes()).unwrap_or(0));
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
    segment_id: Option<Uuid>,
    punch_in_time: DateTime<Utc>,
    punch_out_time: Option<DateTime<Utc>>,
    duration_minutes: Option<i32>,
    is_break_overtime: bool,
    entry_type: EntryType,
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
        segment_id: Option<Uuid>,
        punch_in_time: DateTime<Utc>,
        punch_out_time: Option<DateTime<Utc>>,
        duration_minutes: Option<i32>,
        is_break_overtime: bool,
        entry_type: EntryType,
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
            segment_id,
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

    pub fn segment_id(&self) -> Option<Uuid> {
        self.segment_id
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

    pub fn entry_type(&self) -> EntryType {
        self.entry_type
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
    segment_id: Option<Uuid>,
    entry_type: Option<EntryType>,
    is_break_overtime: Option<bool>,
    note: Option<String>,
}

impl PunchInRequest {
    pub fn new(
        employment_id: Uuid,
        segment_id: Option<Uuid>,
        entry_type: Option<EntryType>,
        is_break_overtime: Option<bool>,
        note: Option<String>,
    ) -> Self {
        Self {
            employment_id,
            segment_id,
            entry_type,
            is_break_overtime,
            note,
        }
    }

    pub fn employment_id(&self) -> Uuid {
        self.employment_id
    }

    pub fn segment_id(&self) -> Option<Uuid> {
        self.segment_id
    }

    pub fn entry_type(&self) -> Option<EntryType> {
        self.entry_type
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
    segment_id: Option<Uuid>,
    entry_type: Option<EntryType>,
    note: Option<String>,
}

impl UpdateTimeEntryRequest {
    pub fn new(
        punch_in_time: Option<DateTime<Utc>>,
        punch_out_time: Option<DateTime<Utc>>,
        segment_id: Option<Uuid>,
        entry_type: Option<EntryType>,
        note: Option<String>,
    ) -> Self {
        Self {
            punch_in_time,
            punch_out_time,
            segment_id,
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

    pub fn segment_id(&self) -> Option<Uuid> {
        self.segment_id
    }

    pub fn entry_type(&self) -> Option<EntryType> {
        self.entry_type
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    #[test]
    fn test_time_entry_log_filter() {
        let emp_id = Uuid::new_v4();
        let from = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let to = Utc.with_ymd_and_hms(2026, 1, 31, 23, 59, 59).unwrap();
        let filter = TimeEntryLogFilter::new(Some(emp_id), Some(from), Some(to), 10, 20);

        assert_eq!(filter.employment_id(), Some(emp_id));
        assert_eq!(filter.from(), Some(from));
        assert_eq!(filter.to(), Some(to));
        assert_eq!(filter.limit(), 10);
        assert_eq!(filter.offset(), 20);
    }

    #[test]
    fn test_time_entry() {
        let id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let now = Utc::now();
        let mut entry = TimeEntry::new(
            id,
            emp_id,
            ws_id,
            Some(shift_id),
            None,
            now,
            None,
            None,
            false,
            EntryType::Work,
            Some("Note".to_string()),
            Some(serde_json::json!({"test": true})),
            now,
            now,
        );

        assert_eq!(entry.id(), id);
        assert_eq!(entry.employment_id(), emp_id);
        assert_eq!(entry.workspace_id(), ws_id);
        assert_eq!(entry.shift_id(), Some(shift_id));
        assert_eq!(entry.punch_in_time(), now);
        assert_eq!(entry.punch_out_time(), None);
        assert_eq!(entry.duration_minutes(), None);
        assert!(!entry.is_break_overtime());
        assert_eq!(entry.entry_type(), EntryType::Work);
        assert_eq!(entry.note(), Some("Note"));
        assert_eq!(
            entry.generation_metadata(),
            Some(&serde_json::json!({"test": true}))
        );
        assert_eq!(entry.created_at(), now);
        assert_eq!(entry.updated_at(), now);

        let later = now + chrono::Duration::hours(4);
        entry.set_punch_in_time(now);
        entry.set_punch_out_time(Some(later));
        assert_eq!(entry.duration_minutes(), Some(240));

        entry.set_is_break_overtime(true);
        assert!(entry.is_break_overtime());

        entry.set_entry_type(EntryType::Break);
        assert_eq!(entry.entry_type(), EntryType::Break);

        entry.set_note(None);
        assert_eq!(entry.note(), None);

        entry.set_punch_out_time(None);
        assert_eq!(entry.duration_minutes(), None);
    }

    #[test]
    fn test_time_entry_with_employee() {
        let id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let now = Utc::now();
        let entry = TimeEntryWithEmployee::new(
            id,
            emp_id,
            ws_id,
            Some(shift_id),
            None,
            now,
            Some(now),
            Some(0),
            false,
            EntryType::Work,
            Some("Note".to_string()),
            None,
            now,
            now,
            "John Doe".to_string(),
            "EMP001".to_string(),
        );

        assert_eq!(entry.id(), id);
        assert_eq!(entry.employment_id(), emp_id);
        assert_eq!(entry.workspace_id(), ws_id);
        assert_eq!(entry.shift_id(), Some(shift_id));
        assert_eq!(entry.punch_in_time(), now);
        assert_eq!(entry.punch_out_time(), Some(now));
        assert_eq!(entry.duration_minutes(), Some(0));
        assert!(!entry.is_break_overtime());
        assert_eq!(entry.entry_type(), EntryType::Work);
        assert_eq!(entry.note(), Some("Note"));
        assert_eq!(entry.generation_metadata(), None);
        assert_eq!(entry.created_at(), now);
        assert_eq!(entry.updated_at(), now);
        assert_eq!(entry.employee_full_name(), "John Doe");
        assert_eq!(entry.employee_id_number(), "EMP001");
    }

    #[test]
    fn test_punch_in_request() {
        let emp_id = Uuid::new_v4();
        let req = PunchInRequest::new(
            emp_id,
            None,
            Some(EntryType::Work),
            Some(true),
            Some("Note".to_string()),
        );

        assert_eq!(req.employment_id(), emp_id);
        assert_eq!(req.segment_id(), None);
        assert_eq!(req.entry_type(), Some(EntryType::Work));
        assert_eq!(req.is_break_overtime(), Some(true));
        assert_eq!(req.note(), Some("Note"));
    }

    #[test]
    fn test_punch_out_request() {
        let req = PunchOutRequest::new(Some("Note".to_string()));
        assert_eq!(req.note(), Some("Note"));
    }

    #[test]
    fn test_update_time_entry_request() {
        let now = Utc::now();
        let req = UpdateTimeEntryRequest::new(
            Some(now),
            Some(now),
            None,
            Some(EntryType::Work),
            Some("Note".to_string()),
        );

        assert_eq!(req.punch_in_time(), Some(now));
        assert_eq!(req.punch_out_time(), Some(now));
        assert_eq!(req.entry_type(), Some(EntryType::Work));
        assert_eq!(req.note(), Some("Note"));
    }
}
