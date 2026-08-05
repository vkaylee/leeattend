use super::{AttendanceReportArgs, ReportService};
use crate::errors::AppError;
use crate::models::{DailyAttendanceReport, PaginatedResponse};
use crate::utils::i18n::I18nKey;
use crate::utils::tz::parse_workspace_tz;
use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc};
use chrono_tz::Tz;
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

/// Why a scheduled shift is present in the attendance action queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingIssue {
    MissingPunchIn,
    ActiveOvertime,
    PunchOutPending,
    MissingPunchOut,
}

/// No-show response row with recording-specific information.
#[derive(Debug, Clone, Serialize)]
pub struct NoShowReportRow {
    #[serde(flatten)]
    pub report: DailyAttendanceReport,
    pub recording_issue: RecordingIssue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_end_at: Option<DateTime<Utc>>,
}

/// Arguments for the recording-issue report: scheduled shifts that have
/// started and either lack a punch-in or have an open punch after the shift.
///
/// The report covers all shifts that overlap the target calendar day. For today,
/// only shifts whose `start < now` are included (future shifts are skipped).
/// Per-employee timezone determines "today" and shift boundaries.
pub struct NoShowReportArgs {
    pub workspace_id: Uuid,
    /// The target calendar date to query (in the workspace's local frame).
    pub date: NaiveDate,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub employee_ids: Option<Vec<Uuid>>,
    pub department_id: Option<Uuid>,
    /// Comma-separated shift names, mirrored from the daily report query.
    pub shift_type: Option<String>,
}

impl ReportService {
    /// Recording-issue report for scheduled shifts that have started.
    ///
    /// The underlying daily report remains the source of attendance status and
    /// punch timestamps. This method adds a pure, timezone-aware recording
    /// issue classification without changing the general attendance report.
    #[tracing::instrument(skip(self, args), err)]
    #[allow(clippy::too_many_lines)]
    pub async fn get_no_show_report(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        args: NoShowReportArgs,
    ) -> Result<PaginatedResponse<NoShowReportRow>, AppError> {
        let workspace = self
            .workspace_repo
            .get_workspace_by_id(args.workspace_id)
            .await
            .map_err(|e| {
                tracing::error!("Error fetching workspace: {}", e);
                AppError::Internal("Internal error".into())
            })?
            .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?;

        let tz: Tz = parse_workspace_tz(workspace.timezone());
        let date_pre = args
            .date
            .pred_opt()
            .ok_or_else(|| AppError::validation(I18nKey::Validation))?;

        let daily = self
            .get_daily_attendance_report(
                auth_token,
                AttendanceReportArgs {
                    workspace_id: args.workspace_id,
                    start_date: date_pre,
                    end_date: args.date,
                    page: args.page,
                    limit: args.limit,
                    employee_ids: args.employee_ids.clone(),
                    department_id: args.department_id,
                    sort: None,
                    employee_status: None,
                    employer_id: None,
                    employment_id: None,
                    shift_type: args.shift_type.clone(),
                },
            )
            .await?;

        let now = Utc::now();
        let mut tz_cache: HashMap<Uuid, Tz> = HashMap::new();
        let mut filtered = Vec::with_capacity(daily.data.len());

        for mut row in daily.data {
            let emp_tz = if let Some(&cached_tz) = tz_cache.get(&row.employee_id) {
                cached_tz
            } else {
                let emp_tz = self
                    .employee_repo
                    .get_employee_timezone_at_date(auth_token, row.employee_id, args.date)
                    .await
                    .unwrap_or(tz);
                tz_cache.insert(row.employee_id, emp_tz);
                emp_tz
            };

            if let Some((recording_issue, expected_end_at)) =
                classify_recording_issue(&row, args.date, emp_tz, now)
            {
                row.timezone = Some(emp_tz.name().to_string());
                filtered.push(NoShowReportRow {
                    report: row,
                    recording_issue,
                    expected_end_at,
                });
            }
        }

        Ok(PaginatedResponse {
            data: filtered,
            // Pagination is employee-centric in the underlying daily report.
            // Preserve that candidate total so callers continue scanning every
            // employee page even when an intermediate page has no issue rows.
            total: daily.total,
            page: daily.page,
            limit: daily.limit,
            truncated: None,
        })
    }
}

// ── Pure predicates (no I/O, trivially unit-testable) ──────────────────────

/// Convert a shift's wall-clock start/end (date + time in the employee's TZ)
/// to an absolute UTC `DateTime`.
///
/// Delegates to `tz::local_to_utc` for unified DST handling (ambiguous → first,
/// gap → shift +1h).
fn shift_boundary_to_utc(
    date: NaiveDate,
    time: chrono::NaiveTime,
    tz: Tz,
) -> Option<DateTime<Utc>> {
    crate::utils::tz::local_to_utc(&tz, date, time.hour(), time.minute(), time.second()).ok()
}

/// Compute the absolute UTC start of the shift, anchored to the shift's
/// calendar date in the employee's timezone.
fn shift_start_utc(date: NaiveDate, start: chrono::NaiveTime, tz: Tz) -> Option<DateTime<Utc>> {
    shift_boundary_to_utc(date, start, tz)
}

/// Compute the absolute UTC end of the shift, handling overnight shifts
/// (end ≤ start → end +24h).
fn shift_end_utc(
    date: NaiveDate,
    start: chrono::NaiveTime,
    end: chrono::NaiveTime,
    tz: Tz,
) -> Option<DateTime<Utc>> {
    let mut end_date = date;
    if end <= start {
        end_date = end_date
            .succ_opt()
            .unwrap_or_else(|| date + Duration::days(1));
    }
    shift_boundary_to_utc(end_date, end, tz)
}

/// The UTC instant corresponding to 00:00:00 on `target_date` in the given
/// timezone. Used as the left boundary of the calendar day.
///
/// Delegates to `tz::start_of_day_utc` for unified DST handling.
fn day_start_utc(target_date: NaiveDate, tz: Tz) -> Option<DateTime<Utc>> {
    crate::utils::tz::start_of_day_utc(&tz, target_date).ok()
}

/// Classify a report row into a recording issue, if it belongs in the
/// unrecorded action feed.
///
/// An open punch after today's scheduled end is deliberately marked pending
/// unless the entry itself is explicitly an overtime entry. This avoids
/// calling legitimate, unregistered overtime a forgotten punch-out while the
/// employee may still be working. Past-date open punches are actionable.
pub(crate) fn classify_recording_issue(
    row: &DailyAttendanceReport,
    target_date: NaiveDate,
    emp_tz: Tz,
    now: DateTime<Utc>,
) -> Option<(RecordingIssue, Option<DateTime<Utc>>)> {
    use crate::services::attendance_utils::AttendanceStatus;

    let (Some(start), Some(end)) = (row.scheduled_start, row.scheduled_end) else {
        return None;
    };

    let today_in_emp_tz = now.with_timezone(&emp_tz).date_naive();
    if target_date > today_in_emp_tz {
        return None;
    }

    let start_abs = shift_start_utc(row.date, start, emp_tz)?;
    let end_abs = shift_end_utc(row.date, start, end, emp_tz)?;

    let day_start = day_start_utc(target_date, emp_tz)?;
    let day_end = crate::utils::tz::start_of_next_day_utc(&emp_tz, target_date)
        .unwrap_or_else(|_| day_start + Duration::hours(24));
    if end_abs <= day_start || start_abs >= day_end {
        return None;
    }

    if target_date == today_in_emp_tz && start_abs >= now {
        return None;
    }

    if row.status == AttendanceStatus::Absent && row.actual_punch_in.is_none() {
        return Some((RecordingIssue::MissingPunchIn, None));
    }

    // A completed punch is not a recording issue. Inspect the entry metadata
    // as well because a day may contain a completed session followed by an
    // open session; `actual_punch_out` alone cannot represent that state.
    if row.actual_punch_in.is_none() || !has_open_work_entry(row) {
        return None;
    }

    if target_date < today_in_emp_tz {
        return Some((RecordingIssue::MissingPunchOut, Some(end_abs)));
    }

    if has_open_overtime_entry(row) {
        return Some((RecordingIssue::ActiveOvertime, None));
    }

    if now > end_abs {
        return Some((RecordingIssue::PunchOutPending, Some(end_abs)));
    }

    None
}

/// Compatibility predicate for existing no-show tests.
#[cfg(test)]
pub(crate) fn should_include_in_no_show(
    row: &DailyAttendanceReport,
    target_date: NaiveDate,
    emp_tz: Tz,
    now: DateTime<Utc>,
) -> bool {
    matches!(
        classify_recording_issue(row, target_date, emp_tz, now),
        Some((RecordingIssue::MissingPunchIn, _))
    )
}

/// Detect an open non-break work entry from the existing report metadata.
fn has_open_work_entry(row: &DailyAttendanceReport) -> bool {
    let Some(entries) = row
        .generation_metadata
        .as_ref()
        .and_then(|metadata| metadata.get("time_entries"))
        .and_then(serde_json::Value::as_array)
    else {
        return row.actual_punch_in.is_some() && row.actual_punch_out.is_none();
    };

    entries.iter().any(|entry| {
        entry.get("entry_type").and_then(serde_json::Value::as_str) != Some("break")
            && entry
                .get("punch_out_time")
                .is_some_and(serde_json::Value::is_null)
    })
}

/// Detect an explicitly open overtime entry from the existing report metadata.
/// The general daily report intentionally keeps time-entry details in metadata;
/// using it here avoids a per-row database lookup and keeps the attendance
/// report service independent of the payroll request repository.
fn has_open_overtime_entry(row: &DailyAttendanceReport) -> bool {
    let Some(entries) = row
        .generation_metadata
        .as_ref()
        .and_then(|metadata| metadata.get("time_entries"))
        .and_then(serde_json::Value::as_array)
    else {
        return false;
    };

    entries.iter().any(|entry| {
        entry.get("entry_type").and_then(serde_json::Value::as_str) == Some("overtime")
            && entry
                .get("punch_out_time")
                .is_some_and(serde_json::Value::is_null)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::attendance_utils::AttendanceStatus;
    use chrono::{NaiveTime, TimeZone};
    use uuid::Uuid;

    fn hcm() -> Tz {
        "Asia/Ho_Chi_Minh".parse().unwrap()
    }

    fn row_with(
        date: &str,
        start: &str,
        end: &str,
        status: AttendanceStatus,
    ) -> DailyAttendanceReport {
        DailyAttendanceReport {
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "Test Employee".into(),
            employee_id_number: "EMP001".into(),
            shift_symbol: None,
            shift_name: Some("Day".into()),
            scheduled_start: Some(NaiveTime::parse_from_str(start, "%H:%M:%S").unwrap()),
            scheduled_end: Some(NaiveTime::parse_from_str(end, "%H:%M:%S").unwrap()),
            grace_period_minutes: Some(15),
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: None,
            actual_punch_out: None,
            std_work_units: 0,
            ot_work_units: 0,
            ot_claim_status: None,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 0.0,
            computed_std_hours: 0.0,
            computed_ot_hours: 0.0,
            status,
            is_holiday: false,
            is_unscheduled: false,
            holiday_name: None,
            generation_metadata: None,
            is_leave_paid: None,
            employer_name: None,
            timezone: None,
            segment_id: None,
            shift_id: None,
            interval_id: None,
            interval_name: None,
            segment_type: None,
            sort_order: None,
            segment_count: None,
            break_split: None,
        }
    }

    fn at_utc(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
    }

    // ── should_include_in_no_show ──────────────────────────────────────────

    #[test]
    fn includes_absent_shift_overlapping_today_and_started() {
        // Shift 08:00-17:00 HCM on 2026-07-13, now = 14:00 HCM (07:00 UTC).
        // target_date = 2026-07-13 = today in HCM → shift has started → include.
        let row = row_with(
            "2026-07-13",
            "08:00:00",
            "17:00:00",
            AttendanceStatus::Absent,
        );
        let now = at_utc(2026, 7, 13, 7, 0); // 14:00 HCM
        assert!(should_include_in_no_show(
            &row,
            NaiveDate::from_ymd_opt(2026, 7, 13).unwrap(),
            hcm(),
            now,
        ));
    }

    #[test]
    fn excludes_future_shift_on_today() {
        // Shift 18:00-22:00 HCM, now = 14:00 HCM → shift hasn't started yet.
        let row = row_with(
            "2026-07-13",
            "18:00:00",
            "22:00:00",
            AttendanceStatus::Absent,
        );
        let now = at_utc(2026, 7, 13, 7, 0); // 14:00 HCM
        assert!(!should_include_in_no_show(
            &row,
            NaiveDate::from_ymd_opt(2026, 7, 13).unwrap(),
            hcm(),
            now,
        ));
    }

    #[test]
    fn includes_all_shifts_for_past_date() {
        // Past date: 2026-07-10. Even a "future" (relative to now's time-of-day)
        // shift is included because on a past date ALL shifts have started.
        let row = row_with(
            "2026-07-10",
            "18:00:00",
            "22:00:00",
            AttendanceStatus::Absent,
        );
        let now = at_utc(2026, 7, 13, 5, 0); // Now = 12:00 HCM on 07-13
        assert!(should_include_in_no_show(
            &row,
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            hcm(),
            now,
        ));
    }

    #[test]
    fn excludes_present_employee() {
        let row = row_with(
            "2026-07-13",
            "08:00:00",
            "17:00:00",
            AttendanceStatus::Present,
        );
        let now = at_utc(2026, 7, 13, 7, 0);
        assert!(!should_include_in_no_show(
            &row,
            NaiveDate::from_ymd_opt(2026, 7, 13).unwrap(),
            hcm(),
            now,
        ));
    }

    #[test]
    fn excludes_shift_not_overlapping_target_day() {
        // Shift on 2026-07-13 08:00-17:00 HCM. target_date = 2026-07-12.
        // The shift is entirely outside the target day → exclude.
        let row = row_with(
            "2026-07-13",
            "08:00:00",
            "17:00:00",
            AttendanceStatus::Absent,
        );
        let now = at_utc(2026, 7, 12, 7, 0);
        assert!(!should_include_in_no_show(
            &row,
            NaiveDate::from_ymd_opt(2026, 7, 12).unwrap(),
            hcm(),
            now,
        ));
    }

    #[test]
    fn includes_overnight_shift_overlapping_target_day() {
        // Night shift 22:00-06:00 on 2026-07-12 (anchored to 07-12).
        // target_date = 2026-07-13 (next day). The shift overlaps 07-13
        // (00:00-06:00) → include.
        let row = row_with(
            "2026-07-12",
            "22:00:00",
            "06:00:00",
            AttendanceStatus::Absent,
        );
        let now = at_utc(2026, 7, 13, 7, 0); // 14:00 HCM on 07-13
        assert!(should_include_in_no_show(
            &row,
            NaiveDate::from_ymd_opt(2026, 7, 13).unwrap(),
            hcm(),
            now,
        ));
    }

    #[test]
    fn excludes_future_date() {
        // target_date = 2026-07-15, but today = 2026-07-13 → future → exclude.
        let row = row_with(
            "2026-07-13",
            "08:00:00",
            "17:00:00",
            AttendanceStatus::Absent,
        );
        let now = at_utc(2026, 7, 13, 7, 0);
        assert!(!should_include_in_no_show(
            &row,
            NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
            hcm(),
            now,
        ));
    }

    #[test]
    fn per_employee_timezone_past_date() {
        // Employee in Tokyo (+09), target_date = 2026-07-13 at now = 23:00 UTC.
        // In Tokyo: today is already 2026-07-14 08:00, so target_date 07-13 is a
        // past date for this employee → all shifts included (they all happened).
        let row = row_with(
            "2026-07-13",
            "08:00:00",
            "17:00:00",
            AttendanceStatus::Absent,
        );
        let now = at_utc(2026, 7, 13, 23, 0); // 2026-07-14 08:00 Tokyo
        assert!(should_include_in_no_show(
            &row,
            NaiveDate::from_ymd_opt(2026, 7, 13).unwrap(),
            "Asia/Tokyo".parse().unwrap(),
            now,
        ));
    }

    #[test]
    fn excludes_rows_without_shift_times() {
        let mut row = row_with(
            "2026-07-13",
            "08:00:00",
            "17:00:00",
            AttendanceStatus::Absent,
        );
        row.scheduled_start = None;
        let now = at_utc(2026, 7, 13, 7, 0);
        assert!(!should_include_in_no_show(
            &row,
            NaiveDate::from_ymd_opt(2026, 7, 13).unwrap(),
            hcm(),
            now,
        ));
    }

    #[test]
    fn open_punch_after_today_shift_is_pending() {
        let mut row = row_with(
            "2026-07-13",
            "08:00:00",
            "17:00:00",
            AttendanceStatus::Present,
        );
        row.actual_punch_in = Some(at_utc(2026, 7, 13, 1, 0));
        row.generation_metadata = Some(serde_json::json!({
            "time_entries": [{
                "entry_type": "work",
                "punch_out_time": null
            }]
        }));

        let issue = classify_recording_issue(
            &row,
            NaiveDate::from_ymd_opt(2026, 7, 13).unwrap(),
            hcm(),
            at_utc(2026, 7, 13, 11, 0),
        );

        assert_eq!(
            issue,
            Some((
                RecordingIssue::PunchOutPending,
                Some(at_utc(2026, 7, 13, 10, 0)),
            ))
        );
    }

    #[test]
    fn open_overtime_punch_is_not_missing_punch_out() {
        let mut row = row_with(
            "2026-07-13",
            "08:00:00",
            "17:00:00",
            AttendanceStatus::Present,
        );
        row.actual_punch_in = Some(at_utc(2026, 7, 13, 1, 0));
        row.generation_metadata = Some(serde_json::json!({
            "time_entries": [{
                "entry_type": "overtime",
                "punch_out_time": null
            }]
        }));

        let issue = classify_recording_issue(
            &row,
            NaiveDate::from_ymd_opt(2026, 7, 13).unwrap(),
            hcm(),
            at_utc(2026, 7, 13, 11, 0),
        );

        assert_eq!(issue, Some((RecordingIssue::ActiveOvertime, None)));
    }

    #[test]
    fn open_punch_on_past_date_is_missing_punch_out() {
        let mut row = row_with(
            "2026-07-10",
            "08:00:00",
            "17:00:00",
            AttendanceStatus::Present,
        );
        row.actual_punch_in = Some(at_utc(2026, 7, 10, 1, 0));
        row.generation_metadata = Some(serde_json::json!({
            "time_entries": [{
                "entry_type": "work",
                "punch_out_time": null
            }]
        }));

        let issue = classify_recording_issue(
            &row,
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            hcm(),
            at_utc(2026, 7, 13, 5, 0),
        );

        assert_eq!(
            issue,
            Some((
                RecordingIssue::MissingPunchOut,
                Some(at_utc(2026, 7, 10, 10, 0)),
            ))
        );
    }

    // ── Shift boundary helpers ─────────────────────────────────────────────

    #[test]
    fn shift_end_utc_overnight() {
        // 22:00 → 06:00 next day, anchored to 2026-07-10 HCM.
        let end = shift_end_utc(
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            hcm(),
        );
        // 06:00 HCM on 07-11 = 2026-07-10 23:00 UTC
        assert_eq!(end, Some(at_utc(2026, 7, 10, 23, 0)));
    }

    #[test]
    fn shift_end_utc_ordinary() {
        let end = shift_end_utc(
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            hcm(),
        );
        // 17:00 HCM = 10:00 UTC
        assert_eq!(end, Some(at_utc(2026, 7, 10, 10, 0)));
    }

    #[test]
    fn day_start_utc_returns_midnight() {
        let start = day_start_utc(NaiveDate::from_ymd_opt(2026, 7, 13).unwrap(), hcm());
        // 00:00 HCM on 07-13 = 2026-07-12 17:00 UTC
        assert_eq!(start, Some(at_utc(2026, 7, 12, 17, 0)));
    }

    // ── DST boundary tests ──────────────────────────────────────────────────

    fn ny() -> Tz {
        "America/New_York".parse().unwrap()
    }

    #[test]
    fn day_boundary_spring_forward_23h_day() {
        // Mar 8 2026 in NY: clocks skip 02:00→03:00 (23h day).
        // day_end should be midnight of Mar 9, not day_start + 24h.
        let date = NaiveDate::from_ymd_opt(2026, 3, 8).unwrap();
        let day_start = day_start_utc(date, ny()).unwrap();
        let day_end = crate::utils::tz::start_of_next_day_utc(&ny(), date)
            .unwrap_or_else(|_| day_start + Duration::hours(24));

        // day_start: 00:00 EST = 05:00 UTC
        assert_eq!(day_start, at_utc(2026, 3, 8, 5, 0));
        // day_end: 00:00 EDT (Mar 9) = 04:00 UTC (Mar 9)
        assert_eq!(day_end, at_utc(2026, 3, 9, 4, 0));

        // Duration should be 23h (not 24h)
        let hours = day_end.signed_duration_since(day_start).num_hours();
        assert_eq!(hours, 23, "Spring-forward day should be 23h, got {hours}h");
    }

    #[test]
    fn day_boundary_fall_back_25h_day() {
        // Nov 1 2026 in NY: clocks fall back 02:00→01:00 (25h day).
        let date = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
        let day_start = day_start_utc(date, ny()).unwrap();
        let day_end = crate::utils::tz::start_of_next_day_utc(&ny(), date)
            .unwrap_or_else(|_| day_start + Duration::hours(24));

        // day_start: 00:00 EDT = 04:00 UTC
        assert_eq!(day_start, at_utc(2026, 11, 1, 4, 0));
        // day_end: 00:00 EST (Nov 2) = 05:00 UTC (Nov 2)
        assert_eq!(day_end, at_utc(2026, 11, 2, 5, 0));

        // Duration should be 25h (not 24h)
        let hours = day_end.signed_duration_since(day_start).num_hours();
        assert_eq!(hours, 25, "Fall-back day should be 25h, got {hours}h");
    }

    #[test]
    fn no_show_shift_overlaps_spring_forward_day() {
        // Shift 22:00-06:00 on Mar 7 (day before spring-forward).
        // The shift ends at 06:00 EDT on Mar 8. Must overlap Mar 8.
        let row = row_with(
            "2026-03-07",
            "22:00:00",
            "06:00:00",
            AttendanceStatus::Absent,
        );
        let target = NaiveDate::from_ymd_opt(2026, 3, 8).unwrap();
        let now = at_utc(2026, 3, 8, 12, 0); // well past shift end
        assert!(should_include_in_no_show(&row, target, ny(), now));
    }
}
