use super::{AttendanceReportArgs, ReportService};
use crate::errors::AppError;
use crate::models::{DailyAttendanceReport, PaginatedResponse};
use crate::utils::i18n::I18nKey;
use crate::utils::tz::parse_workspace_tz;
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use std::collections::HashMap;
use uuid::Uuid;

/// Arguments for the no-show report: "which employees have a scheduled shift
/// overlapping `date` that has *already started*, and have not punched in yet?"
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
    /// No-show report: employees on a shift overlapping `date` who have not
    /// punched in (status `absent`) AND whose shift has already started.
    ///
    /// Implementation strategy:
    /// 1. Call `get_daily_attendance_report` over `[date-1, date]` to capture
    ///    overnight shifts anchored to the previous calendar day.
    /// 2. Resolve per-employee timezone via `employee_repo.get_employee_timezone_at_date`.
    /// 3. Apply `should_include_in_no_show` — a pure, testable post-filter that
    ///    checks shift-overlap-day, shift-already-started, and absent status.
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

        // Fetch over [date-1, date] so overnight shifts (anchored to their
        // start date) are reachable.
        let date_pre = args
            .date
            .pred_opt()
            .ok_or_else(|| AppError::validation(I18nKey::Validation))?;

        let daily = self
            .get_daily_attendance_report(AttendanceReportArgs {
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
            })
            .await?;

        let now = Utc::now();

        // Resolve per-employee timezone for each unique employee in the result
        // set. We batch this to avoid N+1 queries.
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id);
        let mut tz_cache: HashMap<Uuid, Tz> = HashMap::new();

        let filtered: Vec<DailyAttendanceReport> = {
            let mut out = Vec::with_capacity(daily.data.len());
            for mut row in daily.data {
                let emp_tz = match tz_cache.get(&row.employee_id) {
                    Some(&cached_tz) => cached_tz,
                    None => {
                        let emp_tz = self
                            .employee_repo
                            .get_employee_timezone_at_date(
                                &auth_token,
                                row.employee_id,
                                args.date,
                            )
                            .await
                            .unwrap_or(tz);
                        tz_cache.insert(row.employee_id, emp_tz);
                        emp_tz
                    }
                };

                if should_include_in_no_show(&row, args.date, emp_tz, now) {
                    row.timezone = Some(emp_tz.name().to_string());
                    out.push(row);
                }
            }
            out
        };

        // `total` reflects the post-filter count so the FE pager stops once
        // all matching rows have been served.
        let total = filtered.len();

        Ok(PaginatedResponse {
            data: filtered,
            total,
            page: daily.page,
            limit: daily.limit,
        })
    }
}

// ── Pure predicates (no I/O, trivially unit-testable) ──────────────────────

/// Convert a shift's wall-clock start/end (date + time in the employee's TZ)
/// to an absolute UTC `DateTime`.
fn shift_boundary_to_utc(
    date: NaiveDate,
    time: chrono::NaiveTime,
    tz: Tz,
) -> Option<DateTime<Utc>> {
    let naive = NaiveDateTime::new(date, time);
    tz.from_local_datetime(&naive)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
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
fn day_start_utc(target_date: NaiveDate, tz: Tz) -> Option<DateTime<Utc>> {
    let midnight = NaiveDateTime::new(target_date, chrono::NaiveTime::from_hms_opt(0, 0, 0)?);
    tz.from_local_datetime(&midnight)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Pure predicate: should this report row be included in the no-show list?
///
/// Conditions:
/// 1. `status == Absent`
/// 2. The shift window overlaps the calendar day `target_date` in `emp_tz`
/// 3. The shift has already started (start < now) — for today this filters
///    out future shifts; for past dates this is always true
/// 4. `target_date` is not in the future (relative to the employee's TZ)
pub(crate) fn should_include_in_no_show(
    row: &DailyAttendanceReport,
    target_date: NaiveDate,
    emp_tz: Tz,
    now: DateTime<Utc>,
) -> bool {
    // Must be absent (no punch-in recorded).
    use crate::services::attendance_utils::AttendanceStatus;
    if row.status != AttendanceStatus::Absent {
        return false;
    }

    // Must have shift times.
    let (Some(start), Some(end)) = (row.scheduled_start, row.scheduled_end) else {
        return false;
    };

    // Don't include shifts that haven't happened yet — "today" is in the
    // employee's timezone.
    let today_in_emp_tz = now.with_timezone(&emp_tz).date_naive();
    if target_date > today_in_emp_tz {
        return false;
    }

    let Some(start_abs) = shift_start_utc(row.date, start, emp_tz) else {
        return false;
    };
    let Some(end_abs) = shift_end_utc(row.date, start, end, emp_tz) else {
        return false;
    };

    // Shift must overlap the target_date calendar day.
    // The day spans [day_start, day_start + 24h).
    let Some(day_start) = day_start_utc(target_date, emp_tz) else {
        return false;
    };
    let day_end = day_start + Duration::hours(24);
    let overlaps_day = end_abs > day_start && start_abs < day_end;
    if !overlaps_day {
        return false;
    }

    // For today: only include shifts that have already started.
    // For past dates: all shifts have started (always true).
    if target_date == today_in_emp_tz && start_abs >= now {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::attendance_utils::AttendanceStatus;
    use chrono::NaiveTime;
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
        let row = row_with("2026-07-13", "08:00:00", "17:00:00", AttendanceStatus::Absent);
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
        let row = row_with("2026-07-13", "18:00:00", "22:00:00", AttendanceStatus::Absent);
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
        let row = row_with("2026-07-10", "18:00:00", "22:00:00", AttendanceStatus::Absent);
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
        let row = row_with("2026-07-13", "08:00:00", "17:00:00", AttendanceStatus::Present);
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
        let row = row_with("2026-07-13", "08:00:00", "17:00:00", AttendanceStatus::Absent);
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
        let row = row_with("2026-07-12", "22:00:00", "06:00:00", AttendanceStatus::Absent);
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
        let row = row_with("2026-07-13", "08:00:00", "17:00:00", AttendanceStatus::Absent);
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
        let row = row_with("2026-07-13", "08:00:00", "17:00:00", AttendanceStatus::Absent);
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
        let mut row = row_with("2026-07-13", "08:00:00", "17:00:00", AttendanceStatus::Absent);
        row.scheduled_start = None;
        let now = at_utc(2026, 7, 13, 7, 0);
        assert!(!should_include_in_no_show(
            &row,
            NaiveDate::from_ymd_opt(2026, 7, 13).unwrap(),
            hcm(),
            now,
        ));
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
        let start = day_start_utc(
            NaiveDate::from_ymd_opt(2026, 7, 13).unwrap(),
            hcm(),
        );
        // 00:00 HCM on 07-13 = 2026-07-12 17:00 UTC
        assert_eq!(start, Some(at_utc(2026, 7, 12, 17, 0)));
    }
}
