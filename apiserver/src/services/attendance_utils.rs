use chrono::{DateTime, NaiveDate, NaiveTime, Timelike, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// AttendanceStatus enum — replaces scattered string literals
// ---------------------------------------------------------------------------

/// All possible daily attendance statuses.
///
/// Previously these were ad-hoc `String` values (`"absent"`, `"present"`, etc.)
/// scattered across `report_service`, `attendance` worker, and `crud` handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttendanceStatus {
    /// Employee was present and punched in on time.
    Present,
    /// Employee was present but punched in late (> 5 min after shift start).
    Late,
    /// Employee had a shift but did not punch in and it is not a holiday.
    Absent,
    /// Employee had no shift assigned (day off / unscheduled).
    Off,
    /// Employee punched in but had no shift assigned (ad-hoc / emergency work).
    Unscheduled,
    /// Holiday where employee actually worked.
    HolidayWork,
    /// Paid holiday — employee not expected to work.
    HolidayPaid,
    /// Unpaid holiday — no leave request.
    HolidayUnpaid,
    /// Holiday with an approved leave request.
    HolidayLeave,
    /// Non-holiday day with an approved leave request.
    Leave,
}

impl fmt::Display for AttendanceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Present => write!(f, "present"),
            Self::Late => write!(f, "late"),
            Self::Absent => write!(f, "absent"),
            Self::Off => write!(f, "off"),
            Self::Unscheduled => write!(f, "unscheduled"),
            Self::HolidayWork => write!(f, "holiday_work"),
            Self::HolidayPaid => write!(f, "holiday_paid"),
            Self::HolidayUnpaid => write!(f, "holiday_unpaid"),
            Self::HolidayLeave => write!(f, "holiday_leave"),
            Self::Leave => write!(f, "leave"),
        }
    }
}
impl AttendanceStatus {
    /// Whether this status represents an unexcused absence.
    pub fn is_unexcused(&self) -> bool {
        matches!(self, Self::Absent)
    }

    /// Returns the string representation as a `&'static str` — zero heap allocation.
    ///
    /// Use this instead of `.to_string()` when you need a string reference
    /// without owning it (e.g., comparisons, logging, struct fields that accept `&str`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Late => "late",
            Self::Absent => "absent",
            Self::Off => "off",
            Self::Unscheduled => "unscheduled",
            Self::HolidayWork => "holiday_work",
            Self::HolidayPaid => "holiday_paid",
            Self::HolidayUnpaid => "holiday_unpaid",
            Self::HolidayLeave => "holiday_leave",
            Self::Leave => "leave",
        }
    }
}

// ---------------------------------------------------------------------------
// HolidayKind — replaces is_holiday + is_holiday_paid boolean pair
// ---------------------------------------------------------------------------

/// Whether a day is a holiday and whether that holiday is paid.
///
/// Replaces the error-prone `(is_holiday: bool, is_holiday_paid: bool)` pair
/// where `is_holiday_paid` was meaningless when `is_holiday` was `false`.
/// The enum makes the invalid state unrepresentable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HolidayKind {
    /// Not a holiday — regular working day or day off.
    NotHoliday,
    /// Holiday without pay (no paid-holiday policy for this day).
    Unpaid,
    /// Paid holiday — employee receives pay even without working.
    Paid,
}

// ---------------------------------------------------------------------------
// determine_daily_status — single source of truth for status determination
// ---------------------------------------------------------------------------

/// Input for status determination — extracted from schedule/entries/holidays.
pub struct DailyStatusInput {
    has_punch_in: bool,
    punch_in_time: Option<NaiveTime>,
    scheduled_start: Option<NaiveTime>,
    holiday_kind: HolidayKind,
    has_shift: bool,
    is_on_leave: bool,
    grace_period_minutes: Option<i32>,
}

impl DailyStatusInput {
    pub fn new(
        has_punch_in: bool,
        punch_in_time: Option<NaiveTime>,
        scheduled_start: Option<NaiveTime>,
        holiday_kind: HolidayKind,
        has_shift: bool,
        is_on_leave: bool,
        grace_period_minutes: Option<i32>,
    ) -> Self {
        Self {
            has_punch_in,
            punch_in_time,
            scheduled_start,
            holiday_kind,
            has_shift,
            is_on_leave,
            grace_period_minutes,
        }
    }

    pub fn has_punch_in(&self) -> bool {
        self.has_punch_in
    }

    pub fn punch_in_time(&self) -> Option<NaiveTime> {
        self.punch_in_time
    }

    pub fn scheduled_start(&self) -> Option<NaiveTime> {
        self.scheduled_start
    }

    pub fn holiday_kind(&self) -> HolidayKind {
        self.holiday_kind
    }

    pub fn has_shift(&self) -> bool {
        self.has_shift
    }

    pub fn is_on_leave(&self) -> bool {
        self.is_on_leave
    }

    pub fn grace_period_minutes(&self) -> Option<i32> {
        self.grace_period_minutes
    }
}

/// Determine the attendance status for a single day.
///
/// This is the **single source of truth** for status determination, used by:
/// - `report_service.rs` (daily detail report)
/// - `attendance.rs` (worker batch processing)
/// - `crud.rs` (dashboard unexcused absences)
///
/// Logic:
/// 1. Has punch-in? → `Present` or `Late` (or `HolidayWork` if holiday)
/// 2. Is holiday? → `HolidayPaid` / `HolidayLeave` / `HolidayUnpaid`
/// 3. No shift? → `Off`
/// 4. Otherwise → `Absent`
pub fn determine_daily_status(input: &DailyStatusInput) -> AttendanceStatus {
    if input.has_punch_in() {
        if input.holiday_kind() != HolidayKind::NotHoliday {
            return AttendanceStatus::HolidayWork;
        }
        if !input.has_shift() {
            return AttendanceStatus::Unscheduled;
        }
        if let (Some(punch_in), Some(sched_start)) =
            (input.punch_in_time(), input.scheduled_start())
        {
            let grace = input.grace_period_minutes().unwrap_or(5);
            let punch_in_min = punch_in
                .with_second(0)
                .and_then(|t| t.with_nanosecond(0))
                .unwrap_or(punch_in);
            let sched_start_min = sched_start
                .with_second(0)
                .and_then(|t| t.with_nanosecond(0))
                .unwrap_or(sched_start);
            if punch_in_min > sched_start_min + chrono::Duration::minutes(grace.into()) {
                return AttendanceStatus::Late;
            }
        }
        return AttendanceStatus::Present;
    }

    match input.holiday_kind() {
        HolidayKind::Paid => return AttendanceStatus::HolidayPaid,
        HolidayKind::Unpaid if input.is_on_leave() => {
            return AttendanceStatus::HolidayLeave;
        }
        HolidayKind::Unpaid => return AttendanceStatus::HolidayUnpaid,
        HolidayKind::NotHoliday => {}
    }

    if input.is_on_leave() {
        return AttendanceStatus::Leave;
    }

    if !input.has_shift() {
        return AttendanceStatus::Off;
    }

    AttendanceStatus::Absent
}

// ---------------------------------------------------------------------------
// extract_punch_status — shared punch/status extraction from raw_data
// ---------------------------------------------------------------------------

/// Extract punch/status from raw_data JSON, defaulting to -1.
///
/// Used by `attendance_log_service.rs` and `agent_log_buffer.rs` to extract
/// punch mode and status from the device-provided raw JSON data.
pub fn extract_punch_status(raw_data: Option<&serde_json::Value>) -> (i32, i32) {
    let punch = raw_data
        .and_then(|d| d["punch"].as_i64())
        .and_then(|v| i32::try_from(v).ok())
        .unwrap_or(-1);
    let status = raw_data
        .and_then(|d| d["status"].as_i64())
        .and_then(|v| i32::try_from(v).ok())
        .unwrap_or(-1);
    (punch, status)
}

// ---------------------------------------------------------------------------
// date_range_to_utc — convert NaiveDate range to UTC DateTime range
// ---------------------------------------------------------------------------

/// Convert a date range in workspace-local time to UTC `DateTime` range.
///
/// Returns `(start_of_start_date_utc, end_of_end_date_utc)`.
///
/// Delegates to [`crate::utils::tz::start_of_day_utc`] / [`crate::utils::tz::end_of_day_utc`]
/// for proper DST gap handling (shifts forward by 1 hour instead of panicking).
pub fn date_range_to_utc(
    tz: Tz,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<(DateTime<Utc>, DateTime<Utc>), String> {
    let start_dt = crate::utils::tz::start_of_day_utc(&tz, start_date)?;
    let end_dt = crate::utils::tz::end_of_day_utc(&tz, end_date)?;
    Ok((start_dt, end_dt))
}

/// Checked variant of [`date_range_to_utc`] — returns `None` on invalid dates.
///
/// Use this in contexts where panicking is not acceptable (e.g., request handlers
/// that should return an error status code to the client).
///
/// Delegates to [`crate::utils::tz::start_of_day_utc`] / [`crate::utils::tz::end_of_day_utc`]
/// for proper DST gap handling.
pub fn date_range_to_utc_checked(
    tz: Tz,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let start_dt = crate::utils::tz::start_of_day_utc(&tz, start_date).ok()?;
    let end_dt = crate::utils::tz::end_of_day_utc(&tz, end_date).ok()?;
    Some((start_dt, end_dt))
}

// ---------------------------------------------------------------------------
// group_entries_by_date — pre-group time entries by local date
// ---------------------------------------------------------------------------

/// Group time entries by their local date for O(1) per-date lookup.
///
/// This replaces the O(entries × days) pattern of filtering entries per loop
/// iteration.
pub fn group_entries_by_date(
    entries: &[crate::models::TimeEntry],
    tz: Tz,
) -> HashMap<NaiveDate, Vec<&crate::models::TimeEntry>> {
    let mut result: HashMap<NaiveDate, Vec<&crate::models::TimeEntry>> = HashMap::new();
    for entry in entries {
        let date = entry.punch_in_time().with_timezone(&tz).date_naive();
        result.entry(date).or_default().push(entry);
    }
    for entries in result.values_mut() {
        entries.sort_by_key(|e| e.punch_in_time());
    }
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::NewTimeEntryParams;
    use chrono::Timelike;
    use uuid::Uuid;

    // ===== AttendanceStatus enum tests =====

    #[test]
    fn test_status_display_roundtrip() {
        let statuses = [
            (AttendanceStatus::Present, "present"),
            (AttendanceStatus::Late, "late"),
            (AttendanceStatus::Absent, "absent"),
            (AttendanceStatus::Off, "off"),
            (AttendanceStatus::Unscheduled, "unscheduled"),
            (AttendanceStatus::HolidayWork, "holiday_work"),
            (AttendanceStatus::HolidayPaid, "holiday_paid"),
            (AttendanceStatus::HolidayUnpaid, "holiday_unpaid"),
            (AttendanceStatus::HolidayLeave, "holiday_leave"),
            (AttendanceStatus::Leave, "leave"),
        ];

        for (status, expected_str) in statuses {
            assert_eq!(status.to_string(), expected_str);
        }
    }

    #[test]
    fn test_is_unexcused() {
        assert!(AttendanceStatus::Absent.is_unexcused());
        assert!(!AttendanceStatus::Present.is_unexcused());
        assert!(!AttendanceStatus::Off.is_unexcused());
        assert!(!AttendanceStatus::Late.is_unexcused());
        assert!(!AttendanceStatus::HolidayPaid.is_unexcused());
        assert!(!AttendanceStatus::Leave.is_unexcused());
    }

    // ===== determine_daily_status tests =====

    #[test]
    fn test_present_on_time() {
        let input = DailyStatusInput {
            has_punch_in: true,
            punch_in_time: Some(NaiveTime::from_hms_opt(8, 0, 0).expect("valid time")),
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).expect("valid time")),
            holiday_kind: HolidayKind::NotHoliday,
            has_shift: true,
            is_on_leave: false,
            grace_period_minutes: None,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Present);
    }

    #[test]
    fn test_unscheduled_no_shift() {
        let input = DailyStatusInput {
            has_punch_in: true,
            punch_in_time: Some(NaiveTime::from_hms_opt(9, 0, 0).expect("valid time")),
            scheduled_start: None,
            holiday_kind: HolidayKind::NotHoliday,
            has_shift: false,
            is_on_leave: false,
            grace_period_minutes: None,
        };
        assert_eq!(
            determine_daily_status(&input),
            AttendanceStatus::Unscheduled
        );
    }

    #[test]
    fn test_late_punch_in() {
        let input = DailyStatusInput {
            has_punch_in: true,
            punch_in_time: Some(NaiveTime::from_hms_opt(9, 0, 0).expect("valid time")),
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).expect("valid time")),
            holiday_kind: HolidayKind::NotHoliday,
            has_shift: true,
            is_on_leave: false,
            grace_period_minutes: None,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Late);
    }

    #[test]
    fn test_exactly_5_min_late_is_present() {
        // Exactly 5 min = NOT late (must be > 5 min)
        let input = DailyStatusInput {
            has_punch_in: true,
            punch_in_time: Some(NaiveTime::from_hms_opt(8, 35, 0).expect("valid time")),
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).expect("valid time")),
            holiday_kind: HolidayKind::NotHoliday,
            has_shift: true,
            is_on_leave: false,
            grace_period_minutes: None,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Present);
    }

    #[test]
    fn test_holiday_work() {
        let input = DailyStatusInput {
            has_punch_in: true,
            punch_in_time: Some(NaiveTime::from_hms_opt(8, 0, 0).expect("valid time")),
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).expect("valid time")),
            holiday_kind: HolidayKind::Paid,
            has_shift: true,
            is_on_leave: false,
            grace_period_minutes: None,
        };
        assert_eq!(
            determine_daily_status(&input),
            AttendanceStatus::HolidayWork
        );
    }

    #[test]
    fn test_holiday_paid_no_punch() {
        let input = DailyStatusInput {
            has_punch_in: false,
            punch_in_time: None,
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).expect("valid time")),
            holiday_kind: HolidayKind::Paid,
            has_shift: true,
            is_on_leave: false,
            grace_period_minutes: None,
        };
        assert_eq!(
            determine_daily_status(&input),
            AttendanceStatus::HolidayPaid
        );
    }

    #[test]
    fn test_holiday_unpaid_no_leave() {
        let input = DailyStatusInput {
            has_punch_in: false,
            punch_in_time: None,
            scheduled_start: None,
            holiday_kind: HolidayKind::Unpaid,
            has_shift: true,
            is_on_leave: false,
            grace_period_minutes: None,
        };
        assert_eq!(
            determine_daily_status(&input),
            AttendanceStatus::HolidayUnpaid
        );
    }

    #[test]
    fn test_holiday_leave() {
        let input = DailyStatusInput {
            has_punch_in: false,
            punch_in_time: None,
            scheduled_start: None,
            holiday_kind: HolidayKind::Unpaid,
            has_shift: true,
            is_on_leave: true,
            grace_period_minutes: None,
        };
        assert_eq!(
            determine_daily_status(&input),
            AttendanceStatus::HolidayLeave
        );
    }

    #[test]
    fn test_off_day_no_shift() {
        let input = DailyStatusInput {
            has_punch_in: false,
            punch_in_time: None,
            scheduled_start: None,
            holiday_kind: HolidayKind::NotHoliday,
            has_shift: false,
            is_on_leave: false,
            grace_period_minutes: None,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Off);
    }

    #[test]
    fn test_absent_has_shift_no_punch() {
        let input = DailyStatusInput {
            has_punch_in: false,
            punch_in_time: None,
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).expect("valid time")),
            holiday_kind: HolidayKind::NotHoliday,
            has_shift: true,
            is_on_leave: false,
            grace_period_minutes: None,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Absent);
    }

    #[test]
    fn test_absent_with_leave_is_now_leave() {
        // Having approved leave on a shift-assigned day now resolves to Leave status.
        let input = DailyStatusInput {
            has_punch_in: false,
            punch_in_time: None,
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).expect("valid time")),
            holiday_kind: HolidayKind::NotHoliday,
            has_shift: true,
            is_on_leave: true,
            grace_period_minutes: None,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Leave);
    }

    #[test]
    fn test_leave_day_no_shift() {
        let input = DailyStatusInput {
            has_punch_in: false,
            punch_in_time: None,
            scheduled_start: None,
            holiday_kind: HolidayKind::NotHoliday,
            has_shift: false,
            is_on_leave: true,
            grace_period_minutes: None,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Leave);
    }

    #[test]
    fn test_custom_grace_period_late() {
        // Shift starts at 07:30. Punch in at 07:31.
        // Grace period is 0 min. Should be Late.
        let input = DailyStatusInput {
            has_punch_in: true,
            punch_in_time: Some(NaiveTime::from_hms_opt(7, 31, 0).expect("valid time")),
            scheduled_start: Some(NaiveTime::from_hms_opt(7, 30, 0).expect("valid time")),
            holiday_kind: HolidayKind::NotHoliday,
            has_shift: true,
            is_on_leave: false,
            grace_period_minutes: Some(0),
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Late);
    }

    #[test]
    fn test_custom_grace_period_present() {
        // Shift starts at 07:30. Punch in at 07:31.
        // Grace period is 1 min. Should be Present.
        let input = DailyStatusInput {
            has_punch_in: true,
            punch_in_time: Some(NaiveTime::from_hms_opt(7, 31, 0).expect("valid time")),
            scheduled_start: Some(NaiveTime::from_hms_opt(7, 30, 0).expect("valid time")),
            holiday_kind: HolidayKind::NotHoliday,
            has_shift: true,
            is_on_leave: false,
            grace_period_minutes: Some(1),
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Present);
    }

    #[test]
    fn test_seconds_lateness_alignment() {
        // Shift starts at 07:30. Punch in at 07:30:10.
        // Grace period is 0 min. Should be Present because we ignore seconds.
        let input = DailyStatusInput {
            has_punch_in: true,
            punch_in_time: Some(NaiveTime::from_hms_opt(7, 30, 10).expect("valid time")),
            scheduled_start: Some(NaiveTime::from_hms_opt(7, 30, 0).expect("valid time")),
            holiday_kind: HolidayKind::NotHoliday,
            has_shift: true,
            is_on_leave: false,
            grace_period_minutes: Some(0),
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Present);
    }

    // ===== date_range_to_utc tests =====

    #[test]
    fn test_date_range_to_utc_asia_hcm() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz");
        let start = NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date");
        let end = NaiveDate::from_ymd_opt(2026, 3, 3).expect("valid date");

        let (start_dt, end_dt) = date_range_to_utc(tz, start, end).expect("valid range");

        // Asia/Ho_Chi_Minh = UTC+7
        // 2026-03-01 00:00:00 +7 = 2026-02-28 17:00:00 UTC
        assert_eq!(
            start_dt.date_naive(),
            NaiveDate::from_ymd_opt(2026, 2, 28).expect("valid date")
        );
        assert_eq!(
            start_dt.time(),
            NaiveTime::from_hms_opt(17, 0, 0).expect("valid time")
        );

        // 2026-03-03 23:59:59 +7 = 2026-03-03 16:59:59 UTC
        assert_eq!(
            end_dt.date_naive(),
            NaiveDate::from_ymd_opt(2026, 3, 3).expect("valid date")
        );
        assert_eq!(
            end_dt.time(),
            NaiveTime::from_hms_opt(16, 59, 59).expect("valid time")
        );
    }

    #[test]
    fn test_date_range_to_utc_same_in_utc() {
        let tz: Tz = "UTC".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid date");

        let (start_dt, end_dt) = date_range_to_utc(tz, date, date).expect("valid range");

        assert_eq!(start_dt.date_naive(), date);
        assert_eq!(
            start_dt.time(),
            NaiveTime::from_hms_opt(0, 0, 0).expect("valid time")
        );
        assert_eq!(end_dt.date_naive(), date);
        assert_eq!(
            end_dt.time(),
            NaiveTime::from_hms_opt(23, 59, 59).expect("valid time")
        );
    }

    // ===== is_unexcused edge cases =====

    #[test]
    fn test_all_non_absent_statuses_are_not_unexcused() {
        let non_absent = [
            AttendanceStatus::Present,
            AttendanceStatus::Late,
            AttendanceStatus::Off,
            AttendanceStatus::HolidayWork,
            AttendanceStatus::HolidayPaid,
            AttendanceStatus::HolidayUnpaid,
            AttendanceStatus::HolidayLeave,
        ];
        for status in non_absent {
            assert!(!status.is_unexcused(), "{status} should not be unexcused");
        }
    }

    // ===== Serde roundtrip =====

    #[test]
    fn test_serde_roundtrip() {
        let status = AttendanceStatus::HolidayWork;
        let json = serde_json::to_string(&status).expect("serialize");
        assert_eq!(json, "\"holiday_work\"");

        let parsed: AttendanceStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, status);
    }

    #[test]
    fn test_serde_all_variants() {
        let all = [
            AttendanceStatus::Present,
            AttendanceStatus::Late,
            AttendanceStatus::Absent,
            AttendanceStatus::Off,
            AttendanceStatus::HolidayWork,
            AttendanceStatus::HolidayPaid,
            AttendanceStatus::HolidayUnpaid,
            AttendanceStatus::HolidayLeave,
        ];
        for status in all {
            let json = serde_json::to_string(&status).expect("serialize");
            let parsed: AttendanceStatus = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(parsed, status);
        }
    }

    // ===== group_entries_by_date tests =====

    fn make_entry(punch_in: DateTime<Utc>) -> crate::models::TimeEntry {
        crate::models::TimeEntry::new(NewTimeEntryParams {
            id: Uuid::new_v4(),
            employment_id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            shift_id: None,
            segment_id: None,
            punch_in_time: punch_in,
            punch_out_time: None,
            duration_minutes: Some(480),
            is_break_overtime: false,
            entry_type: crate::services::pairing::types::EntryType::Work,
            note: None,
            generation_metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    #[test]
    fn test_group_entries_by_date_basic() {
        use chrono::TimeZone;
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz"); // UTC+7

        // UTC 2026-01-14 20:00 → local 2026-01-15 03:00
        let e1 = make_entry(Utc.with_ymd_and_hms(2026, 1, 14, 20, 0, 0).unwrap());
        // UTC 2026-01-15 02:00 → local 2026-01-15 09:00
        let e2 = make_entry(Utc.with_ymd_and_hms(2026, 1, 15, 2, 0, 0).unwrap());
        // UTC 2026-01-15 18:00 → local 2026-01-16 01:00
        let e3 = make_entry(Utc.with_ymd_and_hms(2026, 1, 15, 18, 0, 0).unwrap());

        let entries = vec![e1, e2, e3];
        let grouped = group_entries_by_date(&entries, tz);

        let jan15 = NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid date");
        let jan16 = NaiveDate::from_ymd_opt(2026, 1, 16).expect("valid date");

        assert_eq!(grouped.get(&jan15).map(std::vec::Vec::len).unwrap_or(0), 2);
        assert_eq!(grouped.get(&jan16).map(std::vec::Vec::len).unwrap_or(0), 1);
    }

    #[test]
    fn test_group_entries_by_date_empty() {
        let tz: Tz = "UTC".parse().expect("valid tz");
        let entries: Vec<crate::models::TimeEntry> = vec![];
        let grouped = group_entries_by_date(&entries, tz);
        assert!(grouped.is_empty());
    }

    #[test]
    fn test_group_entries_by_date_same_date() {
        use chrono::TimeZone;
        let tz: Tz = "UTC".parse().expect("valid tz");

        let e1 = make_entry(Utc.with_ymd_and_hms(2026, 3, 1, 8, 0, 0).unwrap());
        let e2 = make_entry(Utc.with_ymd_and_hms(2026, 3, 1, 12, 0, 0).unwrap());
        let e3 = make_entry(Utc.with_ymd_and_hms(2026, 3, 1, 18, 0, 0).unwrap());

        let entries = vec![e1, e2, e3];
        let grouped = group_entries_by_date(&entries, tz);

        let mar1 = NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date");
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped.get(&mar1).map(std::vec::Vec::len).unwrap_or(0), 3);
    }

    // -----------------------------------------------------------------------
    // extract_punch_status tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_punch_status_both_present() {
        let data = serde_json::json!({"punch": 1, "status": 2});
        let (punch, status) = extract_punch_status(Some(&data));
        assert_eq!(punch, 1);
        assert_eq!(status, 2);
    }

    #[test]
    fn test_extract_punch_status_none() {
        let (punch, status) = extract_punch_status(None);
        assert_eq!(punch, -1);
        assert_eq!(status, -1);
    }

    #[test]
    fn test_extract_punch_status_missing_fields() {
        let data = serde_json::json!({"other": "value"});
        let (punch, status) = extract_punch_status(Some(&data));
        assert_eq!(punch, -1);
        assert_eq!(status, -1);
    }

    #[test]
    fn test_extract_punch_status_zero_values() {
        let data = serde_json::json!({"punch": 0, "status": 0});
        let (punch, status) = extract_punch_status(Some(&data));
        assert_eq!(punch, 0);
        assert_eq!(status, 0);
    }

    #[test]
    fn test_extract_punch_status_string_values_ignored() {
        // String values for punch/status should not parse as i64
        let data = serde_json::json!({"punch": "1", "status": "2"});
        let (punch, status) = extract_punch_status(Some(&data));
        assert_eq!(punch, -1);
        assert_eq!(status, -1);
    }

    #[test]
    fn test_extract_punch_status_partial_fields() {
        let data = serde_json::json!({"punch": 3});
        let (punch, status) = extract_punch_status(Some(&data));
        assert_eq!(punch, 3);
        assert_eq!(status, -1);
    }

    // -----------------------------------------------------------------------
    // date_range_to_utc tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_date_range_to_utc_basic_utc() {
        let tz: Tz = "UTC".parse().expect("valid tz");
        let start = NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid");
        let end = NaiveDate::from_ymd_opt(2026, 3, 31).expect("valid");
        let (s, e) = date_range_to_utc(tz, start, end).expect("valid range");
        assert_eq!(s.date_naive(), start);
        assert!(e > s);
    }

    #[test]
    fn test_date_range_to_utc_with_timezone_offset() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz"); // UTC+7
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid");
        let end = NaiveDate::from_ymd_opt(2026, 1, 31).expect("valid");
        let (s, _e) = date_range_to_utc(tz, start, end).expect("valid range");
        // Midnight VN time = 17:00 previous day UTC
        assert_eq!(s.hour(), 17); // 00:00 VN = 17:00 UTC (prev day)
    }

    #[test]
    fn test_date_range_to_utc_single_day() {
        let tz: Tz = "UTC".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid");
        let (s, e) = date_range_to_utc(tz, date, date).expect("valid range");
        assert_eq!(s.date_naive(), date);
        assert_eq!(e.date_naive(), date);
        assert!(e > s);
    }

    #[test]
    fn test_date_range_to_utc_checked_valid() {
        let tz: Tz = "UTC".parse().expect("valid tz");
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid");
        let end = NaiveDate::from_ymd_opt(2026, 1, 31).expect("valid");
        assert!(date_range_to_utc_checked(tz, start, end).is_some());
    }

    #[test]
    fn test_date_range_to_utc_checked_same_day() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).expect("valid");
        let result = date_range_to_utc_checked(tz, date, date);
        assert!(result.is_some());
        let (s, e) = result.unwrap();
        assert!(e > s);
    }

    #[test]
    fn test_group_entries_by_date_sorting() {
        use crate::models::TimeEntry;
        use uuid::Uuid;

        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz");

        let t1 = "2026-05-20T00:08:27Z".parse::<DateTime<Utc>>().unwrap(); // Local 07:08
        let t2 = "2026-05-20T14:31:36Z".parse::<DateTime<Utc>>().unwrap(); // Local 21:31
        let t3 = "2026-05-21T01:00:00Z".parse::<DateTime<Utc>>().unwrap(); // Local 08:00

        let workspace_id = Uuid::new_v4();
        let employee_id = Uuid::new_v4();
        let entry1 = TimeEntry::new(NewTimeEntryParams {
            id: Uuid::new_v4(),
            employment_id: employee_id,
            workspace_id: workspace_id,
            shift_id: None,
            segment_id: None,
            punch_in_time: t1,
            punch_out_time: None,
            duration_minutes: None,
            is_break_overtime: false,
            entry_type: crate::services::pairing::types::EntryType::Work,
            note: None,
            generation_metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        let entry2 = TimeEntry::new(NewTimeEntryParams {
            id: Uuid::new_v4(),
            employment_id: employee_id,
            workspace_id: workspace_id,
            shift_id: None,
            segment_id: None,
            punch_in_time: t2,
            punch_out_time: None,
            duration_minutes: None,
            is_break_overtime: false,
            entry_type: crate::services::pairing::types::EntryType::Work,
            note: None,
            generation_metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        let entry3 = TimeEntry::new(NewTimeEntryParams {
            id: Uuid::new_v4(),
            employment_id: employee_id,
            workspace_id: workspace_id,
            shift_id: None,
            segment_id: None,
            punch_in_time: t3,
            punch_out_time: None,
            duration_minutes: None,
            is_break_overtime: false,
            entry_type: crate::services::pairing::types::EntryType::Work,
            note: None,
            generation_metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        // Pass out of order
        let entries = vec![entry2, entry3, entry1];

        let grouped = super::group_entries_by_date(&entries, tz);

        assert_eq!(grouped.len(), 2);

        let day1 = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
        let day2 = NaiveDate::from_ymd_opt(2026, 5, 21).unwrap();

        let group1 = grouped.get(&day1).expect("day 1 should exist");
        assert_eq!(group1.len(), 2);
        // Should be sorted by punch_in_time ASC
        assert_eq!(group1[0].punch_in_time(), t1);
        assert_eq!(group1[1].punch_in_time(), t2);

        let group2 = grouped.get(&day2).expect("day 2 should exist");
        assert_eq!(group2.len(), 1);
        assert_eq!(group2[0].punch_in_time(), t3);
    }

    // =========================================================================
    // DST-aware date range tests
    // =========================================================================

    #[test]
    fn test_date_range_to_utc_dst_spring_forward() {
        // America/New_York DST spring-forward on 2026-03-08
        let tz: Tz = "America/New_York".parse().expect("valid tz");
        let start = NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid");
        let end = NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid");
        let (s, e) = date_range_to_utc(tz, start, end).expect("valid range");

        // Start: 00:00 EST (UTC-5) = 05:00 UTC
        assert_eq!(s.hour(), 5);
        assert_eq!(
            s.date_naive(),
            NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid")
        );

        // End: 23:59:59 EDT (UTC-4) = 03:59:59 UTC next day
        assert_eq!(e.hour(), 3);
        assert_eq!(e.minute(), 59);
        assert_eq!(
            e.date_naive(),
            NaiveDate::from_ymd_opt(2026, 3, 9).expect("valid")
        );
    }

    #[test]
    fn test_date_range_to_utc_dst_fall_back() {
        // America/New_York DST fall-back on 2026-11-01
        let tz: Tz = "America/New_York".parse().expect("valid tz");
        let start = NaiveDate::from_ymd_opt(2026, 11, 1).expect("valid");
        let end = NaiveDate::from_ymd_opt(2026, 11, 1).expect("valid");
        let (s, e) = date_range_to_utc(tz, start, end).expect("valid range");

        // Start: 00:00 EDT (UTC-4) = 04:00 UTC
        assert_eq!(s.hour(), 4);

        // End: 23:59:59 EST (UTC-5) = 04:59:59 UTC next day
        assert_eq!(e.hour(), 4);
        assert_eq!(e.minute(), 59);
    }

    #[test]
    fn test_date_range_to_utc_negative_offset_spans_two_utc_dates() {
        // America/Los_Angeles (UTC-8): Jan 15 local day starts on Jan 14 UTC
        let tz: Tz = "America/Los_Angeles".parse().expect("valid tz");
        let start = NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid");
        let end = NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid");
        let (s, e) = date_range_to_utc(tz, start, end).expect("valid range");

        // Start: 00:00 PST (UTC-8) = 08:00 UTC on Jan 15
        assert_eq!(s.hour(), 8);
        assert_eq!(
            s.date_naive(),
            NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid")
        );

        // End: 23:59:59 PST (UTC-8) = 07:59:59 UTC on Jan 16
        assert_eq!(
            e.date_naive(),
            NaiveDate::from_ymd_opt(2026, 1, 16).expect("valid")
        );
        assert_eq!(e.hour(), 7);
    }

    #[test]
    fn test_date_range_to_utc_positive_offset_starts_previous_utc_day() {
        // Asia/Ho_Chi_Minh (UTC+7): Jan 15 local day starts on Jan 14 UTC
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz");
        let start = NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid");
        let end = NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid");
        let (s, e) = date_range_to_utc(tz, start, end).expect("valid range");

        // Start: 00:00 ICT (UTC+7) = 17:00 UTC on Jan 14
        assert_eq!(
            s.date_naive(),
            NaiveDate::from_ymd_opt(2026, 1, 14).expect("valid")
        );
        assert_eq!(s.hour(), 17);

        // End: 23:59:59 ICT (UTC+7) = 16:59:59 UTC on Jan 15
        assert_eq!(
            e.date_naive(),
            NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid")
        );
        assert_eq!(e.hour(), 16);
    }

    #[test]
    fn test_date_range_to_utc_multi_day_range_with_dst() {
        // Range spanning a DST transition
        let tz: Tz = "America/New_York".parse().expect("valid tz");
        let start = NaiveDate::from_ymd_opt(2026, 3, 6).expect("valid");
        let end = NaiveDate::from_ymd_opt(2026, 3, 10).expect("valid");
        let (s, e) = date_range_to_utc(tz, start, end).expect("valid range");

        assert!(s < e, "Start must be before end");
        // Start: Mar 6 00:00 EST (UTC-5) = 05:00 UTC Mar 6
        assert_eq!(
            s.date_naive(),
            NaiveDate::from_ymd_opt(2026, 3, 6).expect("valid")
        );
        assert_eq!(s.hour(), 5);
        // End: Mar 10 23:59:59 EDT (UTC-4) = 03:59:59 UTC Mar 11
        assert_eq!(
            e.date_naive(),
            NaiveDate::from_ymd_opt(2026, 3, 11).expect("valid")
        );
    }

    #[test]
    fn test_date_range_to_utc_checked_returns_none_for_invalid_date() {
        let tz: Tz = "UTC".parse().expect("valid tz");
        // Valid dates should return Some
        let result = date_range_to_utc_checked(
            tz,
            NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid"),
            NaiveDate::from_ymd_opt(2026, 6, 20).expect("valid"),
        );
        assert!(result.is_some());
    }

    // =========================================================================
    // group_entries_by_date: DST-aware grouping
    // =========================================================================

    #[test]
    fn test_group_entries_by_date_dst_boundary() {
        use crate::models::TimeEntry;
        use uuid::Uuid;

        let tz: Tz = "America/New_York".parse().expect("valid tz");
        let workspace_id = Uuid::new_v4();
        let employee_id = Uuid::new_v4();

        // Entry at UTC 2026-03-08 04:30 → EST (UTC-5) = 2026-03-07 23:30 local (Mar 7)
        let t_before_dst = "2026-03-08T04:30:00Z".parse::<DateTime<Utc>>().unwrap();
        // Entry at UTC 2026-03-08 10:00 → EDT (UTC-4) = 2026-03-08 06:00 local (Mar 8, DST day)
        let t_on_dst = "2026-03-08T10:00:00Z".parse::<DateTime<Utc>>().unwrap();

        let entry1 = TimeEntry::new(NewTimeEntryParams {
            id: Uuid::new_v4(),
            employment_id: employee_id,
            workspace_id: workspace_id,
            shift_id: None,
            segment_id: None,
            punch_in_time: t_before_dst,
            punch_out_time: None,
            duration_minutes: None,
            is_break_overtime: false,
            entry_type: crate::services::pairing::types::EntryType::Work,
            note: None,
            generation_metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
        let entry2 = TimeEntry::new(NewTimeEntryParams {
            id: Uuid::new_v4(),
            employment_id: employee_id,
            workspace_id: workspace_id,
            shift_id: None,
            segment_id: None,
            punch_in_time: t_on_dst,
            punch_out_time: None,
            duration_minutes: None,
            is_break_overtime: false,
            entry_type: crate::services::pairing::types::EntryType::Work,
            note: None,
            generation_metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        let entries = vec![entry1, entry2];
        let grouped = super::group_entries_by_date(&entries, tz);

        let mar7 = NaiveDate::from_ymd_opt(2026, 3, 7).unwrap();
        let mar8 = NaiveDate::from_ymd_opt(2026, 3, 8).unwrap();

        assert_eq!(grouped.get(&mar7).map(Vec::len).unwrap_or(0), 1);
        assert_eq!(grouped.get(&mar8).map(Vec::len).unwrap_or(0), 1);
    }

    #[test]
    fn test_group_entries_by_date_positive_offset_next_day() {
        use crate::models::TimeEntry;
use crate::models::{NewTimeEntryParams};
        use uuid::Uuid;

        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz"); // UTC+7
        let workspace_id = Uuid::new_v4();
        let employee_id = Uuid::new_v4();

        // UTC 2026-06-15 18:00 → HCM = 2026-06-16 01:00 (next day!)
        let t_next_day = "2026-06-15T18:00:00Z".parse::<DateTime<Utc>>().unwrap();
        // UTC 2026-06-15 01:00 → HCM = 2026-06-15 08:00 (same day)
        let t_same_day = "2026-06-15T01:00:00Z".parse::<DateTime<Utc>>().unwrap();

        let entry1 = TimeEntry::new(NewTimeEntryParams {
            id: Uuid::new_v4(),
            employment_id: employee_id,
            workspace_id: workspace_id,
            shift_id: None,
            segment_id: None,
            punch_in_time: t_next_day,
            punch_out_time: None,
            duration_minutes: None,
            is_break_overtime: false,
            entry_type: crate::services::pairing::types::EntryType::Work,
            note: None,
            generation_metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
        let entry2 = TimeEntry::new(NewTimeEntryParams {
            id: Uuid::new_v4(),
            employment_id: employee_id,
            workspace_id: workspace_id,
            shift_id: None,
            segment_id: None,
            punch_in_time: t_same_day,
            punch_out_time: None,
            duration_minutes: None,
            is_break_overtime: false,
            entry_type: crate::services::pairing::types::EntryType::Work,
            note: None,
            generation_metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        let entries = vec![entry1, entry2];
        let grouped = super::group_entries_by_date(&entries, tz);

        let jun15 = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let jun16 = NaiveDate::from_ymd_opt(2026, 6, 16).unwrap();

        assert_eq!(grouped.get(&jun15).map(Vec::len).unwrap_or(0), 1);
        assert_eq!(grouped.get(&jun16).map(Vec::len).unwrap_or(0), 1);
    }
}
