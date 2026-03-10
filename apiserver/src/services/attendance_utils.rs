use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::fmt;
use uuid::Uuid;

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
    /// Holiday where employee actually worked.
    HolidayWork,
    /// Paid holiday — employee not expected to work.
    HolidayPaid,
    /// Unpaid holiday — no leave request.
    HolidayUnpaid,
    /// Holiday with an approved leave request.
    HolidayLeave,
}

impl fmt::Display for AttendanceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Present => write!(f, "present"),
            Self::Late => write!(f, "late"),
            Self::Absent => write!(f, "absent"),
            Self::Off => write!(f, "off"),
            Self::HolidayWork => write!(f, "holiday_work"),
            Self::HolidayPaid => write!(f, "holiday_paid"),
            Self::HolidayUnpaid => write!(f, "holiday_unpaid"),
            Self::HolidayLeave => write!(f, "holiday_leave"),
        }
    }
}

impl AttendanceStatus {
    /// Parse from the legacy string representation.
    pub fn from_str_legacy(s: &str) -> Option<Self> {
        match s {
            "present" => Some(Self::Present),
            "late" => Some(Self::Late),
            "absent" => Some(Self::Absent),
            "off" => Some(Self::Off),
            "holiday_work" => Some(Self::HolidayWork),
            "holiday_paid" => Some(Self::HolidayPaid),
            "holiday_unpaid" => Some(Self::HolidayUnpaid),
            "holiday_leave" => Some(Self::HolidayLeave),
            _ => None,
        }
    }

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
            Self::HolidayWork => "holiday_work",
            Self::HolidayPaid => "holiday_paid",
            Self::HolidayUnpaid => "holiday_unpaid",
            Self::HolidayLeave => "holiday_leave",
        }
    }
}

// ---------------------------------------------------------------------------
// determine_daily_status — single source of truth for status determination
// ---------------------------------------------------------------------------

/// Input for status determination — extracted from schedule/entries/holidays.
pub struct DailyStatusInput {
    /// Whether the employee punched in on this day.
    pub has_punch_in: bool,
    /// The time the employee punched in (if any), for late detection.
    pub punch_in_time: Option<NaiveTime>,
    /// The scheduled shift start time (if any).
    pub scheduled_start: Option<NaiveTime>,
    /// Whether this day is a holiday.
    pub is_holiday: bool,
    /// Whether the holiday is paid (only relevant when `is_holiday` is true).
    pub is_holiday_paid: bool,
    /// Whether the employee has a shift assigned.
    pub has_shift: bool,
    /// Whether the employee has an approved leave request for this day.
    pub is_on_leave: bool,
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
    if input.has_punch_in {
        if input.is_holiday {
            return AttendanceStatus::HolidayWork;
        }
        if let (Some(punch_in), Some(sched_start)) = (input.punch_in_time, input.scheduled_start) {
            if punch_in > sched_start + chrono::Duration::minutes(5) {
                return AttendanceStatus::Late;
            }
        }
        return AttendanceStatus::Present;
    }

    if input.is_holiday {
        if input.is_holiday_paid {
            return AttendanceStatus::HolidayPaid;
        }
        if input.is_on_leave {
            return AttendanceStatus::HolidayLeave;
        }
        return AttendanceStatus::HolidayUnpaid;
    }

    if !input.has_shift {
        return AttendanceStatus::Off;
    }

    AttendanceStatus::Absent
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
) -> (DateTime<Utc>, DateTime<Utc>) {
    let start_dt = crate::utils::tz::start_of_day_utc(&tz, start_date)
        .expect("valid start-of-day conversion");
    let end_dt = crate::utils::tz::end_of_day_utc(&tz, end_date)
        .expect("valid end-of-day conversion");
    (start_dt, end_dt)
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
// build_leave_date_set — batch-fetch approved leaves into a HashSet
// ---------------------------------------------------------------------------

/// Batch-fetch approved leave requests and build a `HashSet<(employee_id, date)>`.
#[tracing::instrument(skip_all, err)]
pub async fn build_leave_date_set(
    pool: &PgPool,
    workspace_id: Uuid,
    emp_ids: &[Uuid],
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<HashSet<(Uuid, NaiveDate)>, sqlx::Error> {
    let leave_rows = sqlx::query!(
        r#"SELECT lr.employee_id, lr.start_date, lr.end_date
           FROM leave_requests lr
           JOIN employees e ON lr.employee_id = e.id
           WHERE e.workspace_id = $1
             AND lr.employee_id = ANY($2)
             AND lr.status = 'approved'
             AND lr.start_date <= $4
             AND lr.end_date >= $3"#,
        workspace_id,
        emp_ids,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    let mut set = HashSet::new();
    for lr in &leave_rows {
        let mut d = lr.start_date;
        while d <= lr.end_date {
            if d >= start_date && d <= end_date {
                set.insert((lr.employee_id, d));
            }
            d = match d.succ_opt() {
                Some(next) => next,
                None => break,
            };
        }
    }

    Ok(set)
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
        let date = entry.punch_in_time.with_timezone(&tz).date_naive();
        result.entry(date).or_default().push(entry);
    }
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ===== AttendanceStatus enum tests =====

    #[test]
    fn test_status_display_roundtrip() {
        let statuses = [
            (AttendanceStatus::Present, "present"),
            (AttendanceStatus::Late, "late"),
            (AttendanceStatus::Absent, "absent"),
            (AttendanceStatus::Off, "off"),
            (AttendanceStatus::HolidayWork, "holiday_work"),
            (AttendanceStatus::HolidayPaid, "holiday_paid"),
            (AttendanceStatus::HolidayUnpaid, "holiday_unpaid"),
            (AttendanceStatus::HolidayLeave, "holiday_leave"),
        ];

        for (status, expected_str) in statuses {
            assert_eq!(status.to_string(), expected_str);
            assert_eq!(
                AttendanceStatus::from_str_legacy(expected_str),
                Some(status)
            );
        }
    }

    #[test]
    fn test_from_str_legacy_unknown_returns_none() {
        assert_eq!(AttendanceStatus::from_str_legacy("unknown"), None);
        assert_eq!(AttendanceStatus::from_str_legacy(""), None);
    }

    #[test]
    fn test_is_unexcused() {
        assert!(AttendanceStatus::Absent.is_unexcused());
        assert!(!AttendanceStatus::Present.is_unexcused());
        assert!(!AttendanceStatus::Off.is_unexcused());
        assert!(!AttendanceStatus::Late.is_unexcused());
        assert!(!AttendanceStatus::HolidayPaid.is_unexcused());
    }

    // ===== determine_daily_status tests =====

    #[test]
    fn test_present_on_time() {
        let input = DailyStatusInput {
            has_punch_in: true,
            punch_in_time: Some(NaiveTime::from_hms_opt(8, 0, 0).expect("valid time")),
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).expect("valid time")),
            is_holiday: false,
            is_holiday_paid: false,
            has_shift: true,
            is_on_leave: false,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Present);
    }

    #[test]
    fn test_present_no_schedule() {
        let input = DailyStatusInput {
            has_punch_in: true,
            punch_in_time: Some(NaiveTime::from_hms_opt(9, 0, 0).expect("valid time")),
            scheduled_start: None,
            is_holiday: false,
            is_holiday_paid: false,
            has_shift: false,
            is_on_leave: false,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Present);
    }

    #[test]
    fn test_late_punch_in() {
        let input = DailyStatusInput {
            has_punch_in: true,
            punch_in_time: Some(NaiveTime::from_hms_opt(9, 0, 0).expect("valid time")),
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).expect("valid time")),
            is_holiday: false,
            is_holiday_paid: false,
            has_shift: true,
            is_on_leave: false,
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
            is_holiday: false,
            is_holiday_paid: false,
            has_shift: true,
            is_on_leave: false,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Present);
    }

    #[test]
    fn test_holiday_work() {
        let input = DailyStatusInput {
            has_punch_in: true,
            punch_in_time: Some(NaiveTime::from_hms_opt(8, 0, 0).expect("valid time")),
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).expect("valid time")),
            is_holiday: true,
            is_holiday_paid: true,
            has_shift: true,
            is_on_leave: false,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::HolidayWork);
    }

    #[test]
    fn test_holiday_paid_no_punch() {
        let input = DailyStatusInput {
            has_punch_in: false,
            punch_in_time: None,
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).expect("valid time")),
            is_holiday: true,
            is_holiday_paid: true,
            has_shift: true,
            is_on_leave: false,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::HolidayPaid);
    }

    #[test]
    fn test_holiday_unpaid_no_leave() {
        let input = DailyStatusInput {
            has_punch_in: false,
            punch_in_time: None,
            scheduled_start: None,
            is_holiday: true,
            is_holiday_paid: false,
            has_shift: true,
            is_on_leave: false,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::HolidayUnpaid);
    }

    #[test]
    fn test_holiday_leave() {
        let input = DailyStatusInput {
            has_punch_in: false,
            punch_in_time: None,
            scheduled_start: None,
            is_holiday: true,
            is_holiday_paid: false,
            has_shift: true,
            is_on_leave: true,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::HolidayLeave);
    }

    #[test]
    fn test_off_day_no_shift() {
        let input = DailyStatusInput {
            has_punch_in: false,
            punch_in_time: None,
            scheduled_start: None,
            is_holiday: false,
            is_holiday_paid: false,
            has_shift: false,
            is_on_leave: false,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Off);
    }

    #[test]
    fn test_absent_has_shift_no_punch() {
        let input = DailyStatusInput {
            has_punch_in: false,
            punch_in_time: None,
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).expect("valid time")),
            is_holiday: false,
            is_holiday_paid: false,
            has_shift: true,
            is_on_leave: false,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Absent);
    }

    #[test]
    fn test_absent_with_leave_is_still_absent() {
        // Having approved leave does NOT change status from Absent.
        // Leave status is tracked separately (leave_date_set).
        // This function only determines the base attendance status.
        let input = DailyStatusInput {
            has_punch_in: false,
            punch_in_time: None,
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).expect("valid time")),
            is_holiday: false,
            is_holiday_paid: false,
            has_shift: true,
            is_on_leave: true,
        };
        assert_eq!(determine_daily_status(&input), AttendanceStatus::Absent);
    }

    // ===== date_range_to_utc tests =====

    #[test]
    fn test_date_range_to_utc_asia_hcm() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz");
        let start = NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date");
        let end = NaiveDate::from_ymd_opt(2026, 3, 3).expect("valid date");

        let (start_dt, end_dt) = date_range_to_utc(tz, start, end);

        // Asia/Ho_Chi_Minh = UTC+7
        // 2026-03-01 00:00:00 +7 = 2026-02-28 17:00:00 UTC
        assert_eq!(start_dt.date_naive(), NaiveDate::from_ymd_opt(2026, 2, 28).expect("valid date"));
        assert_eq!(start_dt.time(), NaiveTime::from_hms_opt(17, 0, 0).expect("valid time"));

        // 2026-03-03 23:59:59 +7 = 2026-03-03 16:59:59 UTC
        assert_eq!(end_dt.date_naive(), NaiveDate::from_ymd_opt(2026, 3, 3).expect("valid date"));
        assert_eq!(end_dt.time(), NaiveTime::from_hms_opt(16, 59, 59).expect("valid time"));
    }

    #[test]
    fn test_date_range_to_utc_same_in_utc() {
        let tz: Tz = "UTC".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid date");

        let (start_dt, end_dt) = date_range_to_utc(tz, date, date);

        assert_eq!(start_dt.date_naive(), date);
        assert_eq!(start_dt.time(), NaiveTime::from_hms_opt(0, 0, 0).expect("valid time"));
        assert_eq!(end_dt.date_naive(), date);
        assert_eq!(end_dt.time(), NaiveTime::from_hms_opt(23, 59, 59).expect("valid time"));
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
        crate::models::TimeEntry {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            punch_in_time: punch_in,
            punch_out_time: None,
            entry_type: "work".to_string(),
            duration_minutes: Some(480),
            shift_id: None,
            is_break_overtime: false,
            note: None,
            generation_metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
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

        assert_eq!(grouped.get(&jan15).map(|v| v.len()).unwrap_or(0), 2);
        assert_eq!(grouped.get(&jan16).map(|v| v.len()).unwrap_or(0), 1);
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
        assert_eq!(grouped.get(&mar1).map(|v| v.len()).unwrap_or(0), 3);
    }
}
