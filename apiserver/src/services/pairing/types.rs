use crate::errors::AppError;
use crate::models::RawAttendanceLog;
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::TimeEntryRepository;
use crate::services::SummaryRecalculator;

/// Maximum hour considered "overnight" for reprocessing previous day.
pub(crate) const OVERNIGHT_REPROCESS_CUTOFF_HOUR: u32 = 12;

/// Number of days to search adjacent time entries for consumed-log detection.
pub(crate) const ADJACENT_ENTRY_SEARCH_DAYS: i64 = 2;

/// Type-safe entry classification for paired sessions.
///
/// Strict parsing: unknown string values are rejected during deserialization
/// (returns `Err` from `FromStr` and a serde error from `Deserialize`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, utoipa::ToSchema)]
#[schema(as = EntryType, example = "work")]
pub enum EntryType {
    #[default]
    Work,
    Break,
    Overtime,
}

impl std::fmt::Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Work => write!(f, "work"),
            Self::Break => write!(f, "break"),
            Self::Overtime => write!(f, "overtime"),
        }
    }
}

impl std::str::FromStr for EntryType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "work" => Ok(Self::Work),
            "break" => Ok(Self::Break),
            "overtime" => Ok(Self::Overtime),
            _ => Err(()),
        }
    }
}

impl serde::Serialize for EntryType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for EntryType {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(|()| {
            serde::de::Error::invalid_value(serde::de::Unexpected::Str(&s), &"work|break|overtime")
        })
    }
}

/// Type-safe pairing note codes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairingNote {
    MissingPunchOut,
    Custom(String),
}

impl std::fmt::Display for PairingNote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPunchOut => write!(f, "missing_punch_out"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BreakInterval {
    pub(crate) start_minutes: i32,
    pub(crate) end_minutes: i32,
}

impl BreakInterval {
    pub fn new(start_minutes: i32, end_minutes: i32) -> Self {
        Self {
            start_minutes,
            end_minutes,
        }
    }

    pub fn start_minutes(&self) -> i32 {
        self.start_minutes
    }

    pub fn end_minutes(&self) -> i32 {
        self.end_minutes
    }
}

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairingShiftConfig {
    pub(crate) start_minutes: i32,
    pub(crate) end_minutes: i32,
    pub(crate) breaks: Vec<BreakInterval>,
    pub(crate) check_in_buffer_minutes: Option<i32>,
    pub(crate) max_session_gap_minutes: Option<i32>,
    pub(crate) segment_id: Option<Uuid>,
}

impl PairingShiftConfig {
    pub fn new(
        start_minutes: i32,
        end_minutes: i32,
        break_start_minutes: Option<i32>,
        break_end_minutes: Option<i32>,
        check_in_buffer_minutes: Option<i32>,
        max_session_gap_minutes: Option<i32>,
    ) -> Self {
        let mut breaks = Vec::new();
        if let (Some(s), Some(e)) = (break_start_minutes, break_end_minutes) {
            breaks.push(BreakInterval::new(s, e));
        }
        Self {
            start_minutes,
            end_minutes,
            breaks,
            check_in_buffer_minutes,
            max_session_gap_minutes,
            segment_id: None,
        }
    }

    pub fn new_with_breaks(
        start_minutes: i32,
        end_minutes: i32,
        breaks: Vec<BreakInterval>,
        check_in_buffer_minutes: Option<i32>,
        max_session_gap_minutes: Option<i32>,
    ) -> Self {
        Self {
            start_minutes,
            end_minutes,
            breaks,
            check_in_buffer_minutes,
            max_session_gap_minutes,
            segment_id: None,
        }
    }

    pub fn with_segment_id(mut self, segment_id: Uuid) -> Self {
        self.segment_id = Some(segment_id);
        self
    }

    pub fn start_minutes(&self) -> i32 {
        self.start_minutes
    }

    pub fn end_minutes(&self) -> i32 {
        self.end_minutes
    }

    pub fn break_start_minutes(&self) -> Option<i32> {
        self.breaks.first().map(|b| b.start_minutes)
    }

    pub fn break_end_minutes(&self) -> Option<i32> {
        self.breaks.first().map(|b| b.end_minutes)
    }

    pub fn breaks(&self) -> &[BreakInterval] {
        &self.breaks
    }

    pub fn check_in_buffer_minutes(&self) -> Option<i32> {
        self.check_in_buffer_minutes
    }

    pub fn max_session_gap_minutes(&self) -> Option<i32> {
        self.max_session_gap_minutes
    }

    pub fn segment_id(&self) -> Option<Uuid> {
        self.segment_id
    }
}

/// A collection of shift configs for a single day.
/// Used for split-shift / multi-segment pairing where a worker
/// has multiple scheduled work windows in one calendar day.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiSegmentConfig {
    pub segments: Vec<PairingShiftConfig>,
}

impl MultiSegmentConfig {
    pub fn new(segments: Vec<PairingShiftConfig>) -> Self {
        Self { segments }
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn single(config: PairingShiftConfig) -> Self {
        Self {
            segments: vec![config],
        }
    }

    pub fn first(&self) -> Option<&PairingShiftConfig> {
        self.segments.first()
    }

    pub fn iter(&self) -> impl Iterator<Item = &PairingShiftConfig> {
        self.segments.iter()
    }
}

impl From<PairingShiftConfig> for MultiSegmentConfig {
    fn from(config: PairingShiftConfig) -> Self {
        Self::single(config)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairedSession {
    pub(crate) start_minutes: i32,
    pub(crate) end_minutes: Option<i32>,
    pub(crate) start_time: DateTime<Utc>,
    pub(crate) end_time: Option<DateTime<Utc>>,
    pub(crate) entry_type: EntryType,
    pub(crate) note: Option<PairingNote>,
    pub(crate) is_break_overtime: bool,
    pub(crate) metadata: Option<serde_json::Value>,
}

impl PairedSession {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        start_minutes: i32,
        end_minutes: Option<i32>,
        start_time: DateTime<Utc>,
        end_time: Option<DateTime<Utc>>,
        entry_type: EntryType,
        note: Option<PairingNote>,
        is_break_overtime: bool,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            start_minutes,
            end_minutes,
            start_time,
            end_time,
            entry_type,
            note,
            is_break_overtime,
            metadata,
        }
    }

    pub fn start_minutes(&self) -> i32 {
        self.start_minutes
    }

    pub fn end_minutes(&self) -> Option<i32> {
        self.end_minutes
    }

    pub fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }

    pub fn end_time(&self) -> Option<DateTime<Utc>> {
        self.end_time
    }

    pub fn entry_type(&self) -> &EntryType {
        &self.entry_type
    }

    pub fn note(&self) -> Option<&PairingNote> {
        self.note.as_ref()
    }

    pub fn is_break_overtime(&self) -> bool {
        self.is_break_overtime
    }

    pub fn metadata(&self) -> Option<&serde_json::Value> {
        self.metadata.as_ref()
    }

    pub fn into_metadata(self) -> Option<serde_json::Value> {
        self.metadata
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEvent {
    pub(crate) id: Uuid,
    pub(crate) minutes: i32,
    pub(crate) punch_time: DateTime<Utc>,
    pub(crate) raw_data: Option<serde_json::Value>,
    pub(crate) all_ids: Vec<Uuid>,
}

impl LogEvent {
    pub fn new(
        id: Uuid,
        minutes: i32,
        punch_time: DateTime<Utc>,
        raw_data: Option<serde_json::Value>,
        all_ids: Vec<Uuid>,
    ) -> Self {
        Self {
            id,
            minutes,
            punch_time,
            raw_data,
            all_ids,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn minutes(&self) -> i32 {
        self.minutes
    }

    pub fn punch_time(&self) -> DateTime<Utc> {
        self.punch_time
    }

    pub fn raw_data(&self) -> Option<&serde_json::Value> {
        self.raw_data.as_ref()
    }

    pub fn all_ids(&self) -> &[Uuid] {
        &self.all_ids
    }

    pub fn push_id(&mut self, id: Uuid) {
        self.all_ids.push(id);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingMetadata {
    pub(crate) log_ids: Vec<Uuid>,
}

impl PairingMetadata {
    pub fn new(log_ids: Vec<Uuid>) -> Self {
        Self { log_ids }
    }

    pub fn log_ids(&self) -> &[Uuid] {
        &self.log_ids
    }
}

pub struct PairingServiceArgs {
    pub(crate) time_entry_repo: Arc<dyn TimeEntryRepository>,
    pub(crate) roster_repo: Arc<dyn RosterRepository>,
    pub(crate) employee_repo: Arc<dyn crate::repositories::employee::EmployeeRepository>,
    pub(crate) raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
    pub(crate) summary_service: Arc<dyn SummaryRecalculator>,
}

impl PairingServiceArgs {
    pub fn new(
        time_entry_repo: Arc<dyn TimeEntryRepository>,
        roster_repo: Arc<dyn RosterRepository>,
        employee_repo: Arc<dyn crate::repositories::employee::EmployeeRepository>,
        raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
        summary_service: Arc<dyn SummaryRecalculator>,
    ) -> Self {
        Self {
            time_entry_repo,
            roster_repo,
            employee_repo,
            raw_log_repo,
            summary_service,
        }
    }
}

pub struct PairingContext {
    pub(crate) date: NaiveDate,
    pub(crate) tz: Tz,
}

#[async_trait::async_trait]
pub trait PairingProcessor: Send + Sync {
    async fn process_new_raw_log(&self, log: &RawAttendanceLog) -> Result<(), AppError>;

    async fn reprocess_daily_attendance(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
        tz: Tz,
    ) -> Result<(), AppError>;

    async fn reprocess_attendance_range(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        tz: Tz,
    ) -> Result<(), AppError>;
}
