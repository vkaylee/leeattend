use super::{AttendanceReportArgs, ReportService};
use crate::errors::AppError;
use crate::models::{DailyAttendanceReport, PaginatedResponse};
use crate::utils::i18n::I18nKey;
use crate::utils::tz::parse_workspace_tz;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use uuid::Uuid;

/// Arguments for the no-show report: "which employees are on a shift that is
/// *active* at the reference instant `A`, but have not punched in yet?"
///
/// Unlike `AttendanceReportArgs` (a date *range*), this is a single point in
/// time — the timeline scrubber's pinned center. The service widens it to the
/// `[date(A)-1, date(A)]` window so overnight shifts anchored to their start
/// date are still reachable.
pub struct NoShowReportArgs {
    pub workspace_id: Uuid,
    /// The scrubber's pinned reference instant `A`, in UTC.
    pub reference_time: DateTime<Utc>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub employee_ids: Option<Vec<Uuid>>,
    pub department_id: Option<Uuid>,
    /// Comma-separated shift names, mirrored from the daily report query.
    pub shift_type: Option<String>,
}

impl ReportService {
    /// No-show report: employees on a shift active at `reference_time` that have
    /// not punched in (status `absent`).
    ///
    /// Implementation strategy: reuse the daily-report pipeline over the 2-day
    /// window `[date(A)-1, date(A)]` (which already resolves grace, status, and
    /// employee/department/shift data) and then apply a pure, in-memory filter
    /// that keeps only rows whose *shift window* contains `A`. This avoids
    /// duplicating the heavyweight policy/time-entry resolution and keeps the
    /// time-domain logic in one testable place (`shift_contains_time`).
    #[tracing::instrument(skip(self, args), err)]
    #[allow(clippy::too_many_lines)]
    pub async fn get_no_show_report(
        &self,
        args: NoShowReportArgs,
    ) -> Result<PaginatedResponse<DailyAttendanceReport>, AppError> {
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

        // `date(A)` and the day before, in the workspace timezone — overnight
        // shifts are anchored to their start date, so a shift active at A may be
        // recorded against either calendar day.
        let ref_local = args.reference_time.with_timezone(&tz);
        let date_a = ref_local.date_naive();
        let date_pre = date_a
            .pred_opt()
            .ok_or_else(|| AppError::validation(I18nKey::Validation))?;

        let daily = self
            .get_daily_attendance_report(AttendanceReportArgs {
                workspace_id: args.workspace_id,
                start_date: date_pre,
                end_date: date_a,
                page: args.page,
                limit: args.limit,
                employee_ids: args.employee_ids,
                department_id: args.department_id,
                sort: None,
                employee_status: None,
                employer_id: None,
                employment_id: None,
                shift_type: args.shift_type.clone(),
            })
            .await?;

        // Pure time-domain filter — see `shift_contains_time` / `is_no_show_row`.
        let filtered: Vec<DailyAttendanceReport> = daily
            .data
            .into_iter()
            .filter(|row| is_no_show_row(row, args.reference_time, tz))
            .collect();

        // `total` reflects the post-filter count so the FE pager stops once
        // all matching rows have been served; the pre-filter daily total is
        // only an upper bound and would over-report.
        let total = filtered.len();

        Ok(PaginatedResponse {
            data: filtered,
            total,
            page: daily.page,
            limit: daily.limit,
        })
    }
}

/// Pure predicate: does this report row describe a shift that is *active* at
/// `reference_time` (its window contains A) AND represent an employee who has
/// not punched in yet? No DB, no I/O — trivially unit-testable.
fn is_no_show_row(row: &DailyAttendanceReport, reference_time: DateTime<Utc>, tz: Tz) -> bool {
    // Must be absent (no punch-in recorded). Anything else (present, on leave,
    // holiday, unscheduled, day-off) is not a no-show.
    if row.status != crate::services::attendance_utils::AttendanceStatus::Absent {
        return false;
    }
    // A shift with no resolved times has no window to be active in.
    let (Some(start), Some(end)) = (row.scheduled_start, row.scheduled_end) else {
        return false;
    };
    shift_contains_time(row.date, start, end, reference_time, tz)
}

/// Pure predicate: does the shift window `[start, end)` of the shift scheduled
/// on `date` (in `tz`) contain `reference_time`? Overnight shifts (end ≤ start)
/// push the end +24h. A shift is "active at A" iff `start_abs ≤ A < end_abs`.
///
/// Unit-tested against ordinary, overnight, and out-of-window cases.
pub(crate) fn shift_contains_time(
    date: chrono::NaiveDate,
    start: chrono::NaiveTime,
    end: chrono::NaiveTime,
    reference_time: DateTime<Utc>,
    tz: Tz,
) -> bool {
    let start_abs = NaiveDateTime::new(date, start);
    let start_abs = match tz.from_local_datetime(&start_abs).single() {
        Some(dt) => dt.with_timezone(&Utc),
        None => return false,
    };
    let mut end_abs = NaiveDateTime::new(date, end);
    if end <= start {
        // Overnight: the end is on the next calendar day.
        end_abs += chrono::Duration::days(1);
    }
    let end_abs = match tz.from_local_datetime(&end_abs).single() {
        Some(dt) => dt.with_timezone(&Utc),
        None => return false,
    };
    reference_time >= start_abs && reference_time < end_abs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::attendance_utils::AttendanceStatus;
    use chrono::{NaiveDate, NaiveTime};
    use uuid::Uuid;

    fn hcm() -> Tz {
        "Asia/Ho_Chi_Minh".parse().unwrap()
    }

    fn row_with(date: &str, start: &str, end: &str, status: AttendanceStatus) -> DailyAttendanceReport {
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
            segment_id: None,
            shift_id: None,
            interval_id: None,
            interval_name: None,
            segment_type: None,
            sort_order: None,
            segment_count: None,
        }
    }

    fn at_utc(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
    }

    #[test]
    fn shift_contains_time_ordinary_inside() {
        // Shift 08:00-17:00 HCM on 2026-07-10; A = 12:00 HCM (= 05:00 UTC).
        let tz = hcm();
        assert!(shift_contains_time(
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            at_utc(2026, 7, 10, 5, 0),
            tz,
        ));
    }

    #[test]
    fn shift_contains_time_ordinary_before() {
        let tz = hcm();
        assert!(!shift_contains_time(
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            at_utc(2026, 7, 9, 23, 30), // 06:30 HCM, before shift
            tz,
        ));
    }

    #[test]
    fn shift_contains_time_ordinary_after_end() {
        let tz = hcm();
        // A = 17:00 HCM exactly → window is [start, end), so NOT contained.
        assert!(!shift_contains_time(
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            at_utc(2026, 7, 10, 10, 0),
            tz,
        ));
    }

    #[test]
    fn shift_contains_time_overnight() {
        let tz = hcm();
        // Night shift 22:00 → 06:00 next day. A = 23:00 same day → inside.
        assert!(shift_contains_time(
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            at_utc(2026, 7, 10, 16, 0), // 23:00 HCM
            tz,
        ));
        // A = 03:00 next day (still before 06:00) → inside.
        assert!(shift_contains_time(
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            at_utc(2026, 7, 10, 20, 0), // 03:00 HCM next day
            tz,
        ));
    }

    #[test]
    fn is_no_show_row_only_matches_absent_active_shift() {
        let tz = hcm();
        let a = at_utc(2026, 7, 10, 5, 0); // 12:00 HCM

        let absent = row_with("2026-07-10", "08:00:00", "17:00:00", AttendanceStatus::Absent);
        assert!(is_no_show_row(&absent, a, tz));

        // Present employee is not a no-show even though the shift is active.
        let present = row_with("2026-07-10", "08:00:00", "17:00:00", AttendanceStatus::Present);
        assert!(!is_no_show_row(&present, a, tz));

        // Absent but the shift is not active at A (A is before start).
        let a_night = at_utc(2026, 7, 10, 20, 0); // 03:00 HCM next day
        assert!(!is_no_show_row(&absent, a_night, tz));
    }

    #[test]
    fn is_no_show_row_ignores_rows_without_times() {
        let mut row = row_with("2026-07-10", "08:00:00", "17:00:00", AttendanceStatus::Absent);
        row.scheduled_start = None;
        assert!(!is_no_show_row(&row, at_utc(2026, 7, 10, 5, 0), hcm()));
    }

    // ── Boundary edge cases ─────────────────────────────────────────────────

    #[test]
    fn shift_contains_time_inclusive_at_start() {
        let tz = hcm();
        // A = 08:00 HCM exactly (= start). Window is [start, end) → contained.
        assert!(shift_contains_time(
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            at_utc(2026, 7, 10, 1, 0),
            tz,
        ));
    }

    #[test]
    fn shift_contains_time_exclusive_at_end() {
        let tz = hcm();
        // A = 17:00 HCM exactly (= end). [start, end) → NOT contained.
        assert!(!shift_contains_time(
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            at_utc(2026, 7, 10, 10, 0),
            tz,
        ));
    }

    #[test]
    fn shift_contains_time_overnight_next_morning() {
        // Night shift 22:00 → 06:00 next day, anchored to 2026-07-10. A = 02:00
        // HCM on 2026-07-11 (the post-midnight tail) — still inside.
        let tz = hcm();
        assert!(shift_contains_time(
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            at_utc(2026, 7, 10, 19, 0), // 02:00 HCM on 07-11
            tz,
        ));
    }

    #[test]
    fn shift_contains_time_overnight_before_start_next_day() {
        // Same night shift; A = 21:00 HCM on 07-10 (before 22:00 start) → outside.
        let tz = hcm();
        assert!(!shift_contains_time(
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            at_utc(2026, 7, 10, 14, 0), // 21:00 HCM on 07-10
            tz,
        ));
    }

    #[test]
    fn shift_contains_time_overnight_after_end_next_day() {
        // Night shift 22:00→06:00; A = 07:00 HCM on 07-11 (after 06:00 end) → out.
        let tz = hcm();
        assert!(!shift_contains_time(
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            at_utc(2026, 7, 11, 0, 0), // 07:00 HCM on 07-11
            tz,
        ));
    }

    #[test]
    fn shift_contains_time_exact_length_overnight() {
        // A shift where end == start + exactly pushes equality: 08:00→08:00 would
        // be a degenerate 24h shift (end <= start → +24h). A = 08:00 → contained.
        let tz = hcm();
        assert!(shift_contains_time(
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            at_utc(2026, 7, 10, 1, 0), // 08:00 HCM
            tz,
        ));
    }
}
