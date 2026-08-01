use super::{AttendanceReportArgs, CalculateDayUnitsArgs, ReportDataPreamble, ReportService};
use crate::domain::policy::resolver::{PolicyResolver, ShiftPolicyOverrides};
use crate::errors::AppError;
use crate::models::{DailyAttendanceReport, PaginatedResponse};
use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use std::collections::{BTreeMap, HashMap, HashSet};
use uuid::Uuid;

// ─── Pure post-filter ───────────────────────────────────────────────────────

/// Pure post-join shift-type filter. Applied to the already-built report rows
/// (after status computation) so pagination counts stay correct. O(rows).
/// Mirrors the existing in-memory `employer_id` filter — extracted for unit testing.
pub(crate) fn filter_by_shift_type(
    reports: Vec<DailyAttendanceReport>,
    shift_names: Option<Vec<String>>,
) -> Vec<DailyAttendanceReport> {
    if let Some(names) = shift_names {
        reports
            .into_iter()
            .filter(|r| r.shift_name.as_ref().is_some_and(|n| names.contains(n)))
            .collect()
    } else {
        reports
    }
}

// ─── Report extras (holidays + OT + leave) ──────────────────────────────────

/// Extra data shared between daily and segment report builders.
/// Fetched once after the preamble, before per-employee iteration.
pub(crate) struct ReportExtras {
    holidays_map: HashMap<NaiveDate, crate::models::Holiday>,
    ot_status_map: HashMap<(Uuid, NaiveDate), String>,
    leave_date_set: HashSet<(Uuid, NaiveDate)>,
    leave_details_map: HashMap<(Uuid, NaiveDate), bool>,
}

impl ReportExtras {
    fn new(
        holidays_map: HashMap<NaiveDate, crate::models::Holiday>,
        ot_status_map: HashMap<(Uuid, NaiveDate), String>,
        leave_details_map: HashMap<(Uuid, NaiveDate), bool>,
    ) -> Self {
        let leave_date_set = leave_details_map.keys().copied().collect();
        Self {
            holidays_map,
            ot_status_map,
            leave_date_set,
            leave_details_map,
        }
    }
}

// ─── ReportService: calculation + data fetching ─────────────────────────────

impl ReportService {
    /// Shared calculation for a single day's standard/OT units.
    ///
    /// Delegates to [`AttendanceService`] — the same code path used by
    /// payroll, summary recalculation, and worker handlers.
    pub(crate) fn calculate_day_units(args: &CalculateDayUnitsArgs<'_, '_>) -> (i32, i32) {
        if args.daily().shift_id().is_none() {
            return (0, 0);
        }

        let result = crate::services::AttendanceService::compute_daily_attendance(
            crate::services::ComputeDailyInput {
                daily: args.daily(),
                day_entries: args.day_entries(),
                date: args.date(),
                tz: args.tz(),
                rules: args.rules(),
                work_unit_minutes: args.work_unit_minutes(),
                rounding_policy: args.rounding_policy(),
                max_early_ot_minutes: args.daily().max_early_ot_minutes(),
                ot_ceiling: None,
                segment_id: None,
                ot_aggregation_mode:
                    crate::services::attendance_calculator::OtAggregationMode::Aggregate,
            },
        );

        match result {
            Some(r) => r.into_units(args.work_unit_minutes(), args.rounding_policy()),
            None => (0, 0),
        }
    }

    /// Fetch holidays, OT claim statuses, and leave details — the three extra
    /// DB calls that both daily and segment reports need after the preamble.
    async fn fetch_report_extras(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        emp_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<ReportExtras, AppError> {
        let holidays_map = self
            .holiday_repo
            .get_holidays_in_range(auth_token, start_date, end_date)
            .await?;

        let ot_status_map = self
            .attendance_daily_summary_repo
            .get_batch_ot_statuses(auth_token, employment_ids, start_date, end_date)
            .await?;

        let leave_details_map = self
            .leave_repo
            .get_approved_leave_details(auth_token, emp_ids, start_date, end_date)
            .await?;

        Ok(ReportExtras::new(
            holidays_map,
            ot_status_map,
            leave_details_map,
        ))
    }
}

// ─── Utility helpers ────────────────────────────────────────────────────────

impl ReportService {
    /// Parse a comma-separated shift-type filter string into a `Vec<String>`.
    /// Returns `None` when the input is `None`.
    fn parse_shift_names(shift_type: Option<&str>) -> Option<Vec<String>> {
        let s = shift_type?;
        Some(
            s.split(',')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(String::from)
                .collect(),
        )
    }

    /// Build an empty paginated response preserving pagination metadata from
    /// the preamble.
    fn empty_paginated_response(
        preamble: &ReportDataPreamble,
    ) -> PaginatedResponse<DailyAttendanceReport> {
        PaginatedResponse {
            data: Vec::new(),
            total: usize::try_from(preamble.total_employees()).unwrap_or(0),
            page: i64::try_from(preamble.page()).unwrap_or(1),
            limit: i64::try_from(preamble.limit()).unwrap_or(20),
        }
    }

    /// Build the final paginated response from collected report rows.
    fn build_paginated_response(
        preamble: &ReportDataPreamble,
        reports: Vec<DailyAttendanceReport>,
        shift_names: Option<Vec<String>>,
    ) -> PaginatedResponse<DailyAttendanceReport> {
        let reports = filter_by_shift_type(reports, shift_names);
        PaginatedResponse {
            data: reports,
            total: usize::try_from(preamble.total_employees()).unwrap_or(0),
            page: i64::try_from(preamble.page()).unwrap_or(1),
            limit: i64::try_from(preamble.limit()).unwrap_or(20),
        }
    }

    /// Look up attendance rules for an employee, falling back to defaults.
    fn resolve_attendance_rules(
        preamble: &ReportDataPreamble,
        employee_id: Uuid,
    ) -> crate::models::AttendanceRules {
        preamble
            .attendance_rules_map()
            .get(&employee_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Resolve policy rules from working conditions + shift-level overrides.
    fn resolve_policy_rules(
        working_conditions: &[crate::models::WorkingCondition],
        date: NaiveDate,
        attendance_rules: &crate::models::AttendanceRules,
        grace_period_minutes: Option<i32>,
        is_break_auto_deduct: Option<bool>,
        allow_early_ot: Option<bool>,
        break_is_overtime: bool,
        condition_type_catalog: &HashMap<String, crate::models::ConditionTypeDefinition>,
    ) -> crate::models::PolicyRules {
        let base_rules = PolicyResolver::build_rules_from_conditions(
            working_conditions,
            date,
            attendance_rules.early_arrival_strategy,
            grace_period_minutes
                .unwrap_or(crate::domain::policy::defaults::DEFAULT_GRACE_PERIOD_MINUTES),
            Some(condition_type_catalog),
        );
        let overrides = ShiftPolicyOverrides::new(
            grace_period_minutes,
            is_break_auto_deduct,
            allow_early_ot,
            attendance_rules.early_arrival_strategy,
            break_is_overtime,
        );
        PolicyResolver::apply_shift_overrides(base_rules, &overrides)
    }

    /// Determine attendance status from punch data and schedule context.
    fn determine_status(
        has_punch_in: bool,
        punch_in_time: Option<chrono::NaiveTime>,
        scheduled_start: Option<chrono::NaiveTime>,
        holiday_entry: Option<&crate::models::Holiday>,
        has_shift: bool,
        is_on_leave: bool,
        grace_minutes: i32,
    ) -> crate::services::attendance_utils::AttendanceStatus {
        crate::services::attendance_utils::determine_daily_status(
            &crate::services::attendance_utils::DailyStatusInput::new(
                has_punch_in,
                punch_in_time,
                scheduled_start,
                match holiday_entry {
                    Some(h) if h.is_paid() => crate::services::attendance_utils::HolidayKind::Paid,
                    Some(_) => crate::services::attendance_utils::HolidayKind::Unpaid,
                    None => crate::services::attendance_utils::HolidayKind::NotHoliday,
                },
                has_shift,
                is_on_leave,
                Some(grace_minutes),
            ),
        )
    }
}

// ─── Report row builders (free functions — pure, no &self) ──────────────────

/// Build the shared base of a `DailyAttendanceReport` row.
///
/// Fills all fields common to both the daily (legacy) and segment paths.
/// Callers add the path-specific fields (segment identifiers, per-path units).
fn build_base_report_row(
    date: NaiveDate,
    employment_id: Uuid,
    employee_id: Uuid,
    employee_name: &str,
    employee_id_number: &str,
    shift_symbol: Option<String>,
    shift_name: Option<String>,
    scheduled_start: Option<chrono::NaiveTime>,
    scheduled_end: Option<chrono::NaiveTime>,
    grace_period_minutes: i32,
    break_duration_minutes: Option<i32>,
    break_start: Option<chrono::NaiveTime>,
    break_end: Option<chrono::NaiveTime>,
    is_break_auto_deduct: Option<bool>,
    actual_punch_in: Option<chrono::DateTime<Utc>>,
    actual_punch_out: Option<chrono::DateTime<Utc>>,
    std_work_units: i32,
    ot_work_units: i32,
    ot_claim_status: Option<String>,
    work_unit_minutes: i32,
    status: crate::services::attendance_utils::AttendanceStatus,
    holiday_entry: Option<&crate::models::Holiday>,
    generation_metadata: Option<serde_json::Value>,
    is_leave_paid: Option<bool>,
    employer_name: Option<String>,
) -> DailyAttendanceReport {
    let unit_coeff = f64::from(work_unit_minutes) / 60.0;
    DailyAttendanceReport {
        date,
        employment_id,
        employee_id,
        employee_name: employee_name.to_string(),
        employee_id_number: employee_id_number.to_string(),
        shift_symbol,
        shift_name,
        scheduled_start,
        scheduled_end,
        grace_period_minutes: Some(grace_period_minutes),
        break_duration_minutes,
        break_start,
        break_end,
        is_break_auto_deduct,
        actual_punch_in,
        actual_punch_out,
        std_work_units,
        ot_work_units,
        ot_claim_status,
        unit_type: crate::models::enums::TimekeepingUnit::DAY,
        unit_coefficient_hours: unit_coeff,
        computed_std_hours: f64::from(std_work_units) * unit_coeff,
        computed_ot_hours: f64::from(ot_work_units) * unit_coeff,
        status,
        is_holiday: holiday_entry.is_some(),
        is_unscheduled: status == crate::services::attendance_utils::AttendanceStatus::Unscheduled,
        holiday_name: holiday_entry.map(|h| h.name().to_string()),
        generation_metadata,
        is_leave_paid,
        employer_name,
        timezone: None,
        // Segment fields — overridden by the segment path
        segment_id: None,
        shift_id: None,
        interval_id: None,
        interval_name: None,
        segment_type: None,
        sort_order: None,
        segment_count: None,
    }
}

/// Build optional generation metadata from day entries and schedule traces.
fn build_daily_metadata(
    day_entries: &[&crate::models::TimeEntry],
    daily: &crate::models::EffectiveSchedule,
) -> Option<serde_json::Value> {
    let mut m = serde_json::Map::new();
    if !day_entries.is_empty() {
        m.insert("time_entries".to_string(), serde_json::json!(day_entries));
    }
    if !daily.traces().is_empty() {
        m.insert(
            "schedule_traces".to_string(),
            serde_json::json!(daily.traces()),
        );
    }
    if m.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(m))
    }
}

/// Build a single daily-attendance report row for one employee-day.
///
/// Encapsulates the full pipeline: policy resolution → unit calculation →
/// status determination → row construction. Pure function — no I/O.
fn build_daily_report_row(
    employee_id: Uuid,
    employment_id: Uuid,
    daily: &crate::models::EffectiveSchedule,
    day_entries: &[&crate::models::TimeEntry],
    tz: Tz,
    preamble: &ReportDataPreamble,
    extras: &ReportExtras,
    employee_name: &str,
    employee_id_number: &str,
) -> DailyAttendanceReport {
    let date = daily.date();

    let actual_punch_in = day_entries
        .first()
        .map(|e| e.punch_in_time().with_timezone(&tz));
    let actual_punch_out = day_entries
        .last()
        .and_then(|e| e.punch_out_time().map(|t| t.with_timezone(&tz)));

    let att_rules = ReportService::resolve_attendance_rules(preamble, employee_id);

    let emp_working_conditions = preamble
        .working_conditions_map()
        .get(&employee_id)
        .map_or_else(Default::default, std::vec::Vec::as_slice);

    let rules = ReportService::resolve_policy_rules(
        emp_working_conditions,
        date,
        &att_rules,
        daily.early_punch_in_grace_period_minutes(),
        daily.is_break_auto_deduct(),
        daily.allow_early_ot(),
        daily.break_is_overtime().unwrap_or(false),
        preamble.condition_type_catalog(),
    );

    let grace_minutes = rules.grace_period_minutes();

    let (std_work_units, ot_work_units) =
        ReportService::calculate_day_units(&CalculateDayUnitsArgs::new(
            day_entries,
            daily,
            date,
            tz,
            employee_id,
            preamble.workspace_id(),
            att_rules.work_unit_minutes,
            rules,
            att_rules.rounding_policy,
        ));

    let holiday_entry = extras.holidays_map.get(&date);
    let is_on_leave = extras.leave_date_set.contains(&(employee_id, date));
    let is_leave_paid = if is_on_leave {
        extras.leave_details_map.get(&(employee_id, date)).copied()
    } else {
        None
    };

    let status = ReportService::determine_status(
        actual_punch_in.is_some(),
        actual_punch_in.map(|p| p.time()),
        daily.start_time(),
        holiday_entry,
        daily.shift_id().is_some(),
        is_on_leave,
        grace_minutes,
    );

    let combined_metadata = build_daily_metadata(day_entries, daily);

    // Employer lookup
    let employer_name = preamble
        .employments()
        .iter()
        .find(|e| e.id() == employment_id)
        .map(crate::models::Employment::employer_id)
        .and_then(|id| preamble.employer_name_map().get(&id).cloned());

    build_base_report_row(
        date,
        employment_id,
        employee_id,
        employee_name,
        employee_id_number,
        daily.shift_symbol().map(String::from),
        daily.shift_name().map(String::from),
        daily.start_time(),
        daily.end_time(),
        grace_minutes,
        daily.break_duration_minutes(),
        daily.break_start(),
        daily.break_end(),
        daily.is_break_auto_deduct(),
        actual_punch_in.map(|t| t.with_timezone(&Utc)),
        actual_punch_out.map(|t| t.with_timezone(&Utc)),
        std_work_units,
        ot_work_units,
        extras.ot_status_map.get(&(employment_id, date)).cloned(),
        att_rules.work_unit_minutes,
        status,
        holiday_entry,
        combined_metadata,
        is_leave_paid,
        employer_name,
    )
}

/// Build report rows for all segments of one employee-day.
///
/// Handles status determination and row construction for each segment,
/// reusing pre-computed matched entries and resolved rules from
/// [`build_segment_calc_inputs`].
fn build_segment_report_rows(
    employee_id: Uuid,
    employment_id: Uuid,
    date: NaiveDate,
    day_segs: &[&crate::models::ResolvedSegment],
    cached_matched: &[Vec<&crate::models::TimeEntry>],
    cached_rules: &[crate::models::PolicyRules],
    segment_result: &crate::services::SegmentAttendanceResult,
    att_rules: &crate::models::AttendanceRules,
    preamble: &ReportDataPreamble,
    extras: &ReportExtras,
    employee_name: &str,
    employee_id_number: &str,
    tz: Tz,
) -> Vec<DailyAttendanceReport> {
    let holiday_entry = extras.holidays_map.get(&date);
    let is_on_leave = extras.leave_date_set.contains(&(employee_id, date));
    let is_leave_paid = if is_on_leave {
        extras.leave_details_map.get(&(employee_id, date)).copied()
    } else {
        None
    };
    let segment_count = i32::try_from(day_segs.len()).unwrap_or(1);
    let employer_name = preamble
        .employments()
        .iter()
        .find(|e| e.id() == employment_id)
        .map(crate::models::Employment::employer_id)
        .and_then(|id| preamble.employer_name_map().get(&id).cloned());

    let mut rows = Vec::new();

    for (seg_idx, seg) in day_segs.iter().enumerate() {
        let matched = &cached_matched[seg_idx];
        let seg_rules = &cached_rules[seg_idx];

        let actual_punch_in = matched
            .first()
            .map(|e| e.punch_in_time().with_timezone(&Utc));
        let actual_punch_out = matched
            .last()
            .and_then(|e| e.punch_out_time().map(|t| t.with_timezone(&Utc)));

        let (seg_std, seg_ot) = segment_result.segment_units(
            seg_idx,
            att_rules.work_unit_minutes,
            att_rules.rounding_policy,
        );

        let segment_grace = seg_rules.grace_period_minutes();

        let status = ReportService::determine_status(
            actual_punch_in.is_some(),
            actual_punch_in.map(|p| p.with_timezone(&tz).time()),
            Some(seg.scheduled_start),
            holiday_entry,
            true,
            is_on_leave,
            segment_grace,
        );

        let mut row = build_base_report_row(
            date,
            employment_id,
            employee_id,
            employee_name,
            employee_id_number,
            seg.shift_symbol.clone(),
            seg.shift_name.clone(),
            Some(seg.scheduled_start),
            Some(seg.scheduled_end),
            segment_grace,
            seg.break_duration_minutes,
            seg.break_start,
            seg.break_end,
            seg.is_break_auto_deduct,
            actual_punch_in,
            actual_punch_out,
            seg_std,
            seg_ot,
            extras.ot_status_map.get(&(employment_id, date)).cloned(),
            att_rules.work_unit_minutes,
            status,
            holiday_entry,
            None,
            is_leave_paid,
            employer_name.clone(),
        );

        // Segment-specific fields
        row.segment_id = seg.interval_id;
        row.shift_id = Some(seg.template_id);
        row.interval_id = seg.interval_id;
        row.interval_name.clone_from(&seg.interval_name);
        row.segment_type = Some(seg.segment_type.clone());
        row.sort_order = Some(i32::try_from(seg_idx).unwrap_or(0));
        row.segment_count = Some(segment_count);

        rows.push(row);
    }

    rows
}

/// Group segment indices by their date field for per-day processing.
fn group_segments_by_date(
    segments: &[crate::models::ResolvedSegment],
) -> BTreeMap<NaiveDate, Vec<usize>> {
    let mut map: BTreeMap<NaiveDate, Vec<usize>> = BTreeMap::new();
    for (i, seg) in segments.iter().enumerate() {
        if let Some(date) = seg.date {
            map.entry(date).or_default().push(i);
        }
    }
    map
}

/// Midnight fallback datetime for segments with no punch data.
fn midnight_datetime(date: NaiveDate, tz: Tz) -> chrono::DateTime<Tz> {
    tz.from_utc_datetime(
        &date
            .and_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap_or_default())
            .and_utc()
            .naive_utc(),
    )
}

/// Build per-segment calculation inputs for one employee-day.
///
/// Matches time entries to each segment, resolves policy rules, and constructs
/// `DailyCalcInput` values for [`AttendanceService::compute_segment_attendance`].
/// Returns `(matched_entries, resolved_rules, calc_inputs)` — the first two are
/// cached for reuse in the report-row builder.
fn build_segment_calc_inputs<'a>(
    day_segs: &[&crate::models::ResolvedSegment],
    day_entries: &[&'a crate::models::TimeEntry],
    date: NaiveDate,
    tz: Tz,
    working_conditions: &[crate::models::WorkingCondition],
    att_rules: &crate::models::AttendanceRules,
    condition_type_catalog: &HashMap<String, crate::models::ConditionTypeDefinition>,
) -> (
    Vec<Vec<&'a crate::models::TimeEntry>>,
    Vec<crate::models::PolicyRules>,
    Vec<crate::services::attendance_calculator::DailyCalcInput<Tz>>,
) {
    let mut cached_matched: Vec<Vec<&crate::models::TimeEntry>> = Vec::new();
    let mut cached_rules: Vec<crate::models::PolicyRules> = Vec::new();
    let mut segment_calc_inputs: Vec<crate::services::attendance_calculator::DailyCalcInput<Tz>> =
        Vec::new();

    for seg in day_segs {
        let matched = ReportService::match_entries_to_segment(day_entries, seg, date, tz);

        let actual_punch_in = matched
            .first()
            .map(|e| e.punch_in_time().with_timezone(&tz));
        let actual_punch_out = matched
            .last()
            .and_then(|e| e.punch_out_time().map(|t| t.with_timezone(&tz)));

        let shift_start_dt = crate::utils::tz::naive_time_to_tz(seg.scheduled_start, date, tz);
        let shift_end_dt = crate::utils::tz::naive_time_to_tz_overnight(
            seg.scheduled_end,
            seg.scheduled_start,
            date,
            tz,
        );

        let rules = ReportService::resolve_policy_rules(
            working_conditions,
            date,
            att_rules,
            Some(seg.early_punch_in_grace_period_minutes),
            seg.is_break_auto_deduct,
            Some(seg.allow_early_ot),
            seg.break_is_overtime.unwrap_or(false),
            condition_type_catalog,
        );

        // OT ceiling from next segment
        let ot_ceiling = day_segs
            .iter()
            .filter(|s| s.scheduled_start > seg.scheduled_start)
            .min_by_key(|s| s.scheduled_start)
            .and_then(|next| crate::utils::tz::naive_time_to_tz(next.scheduled_start, date, tz));

        let actual_break_minutes: i32 = matched
            .iter()
            .filter(|e| e.entry_type() == crate::services::pairing::types::EntryType::Break)
            .map(|e| e.duration_minutes().unwrap_or(0))
            .sum();

        cached_matched.push(matched.clone());
        cached_rules.push(rules.clone());

        let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) else {
            // No punch data — use shift start as fallback
            let no_punch = shift_start_dt.unwrap_or_else(|| midnight_datetime(date, tz));
            segment_calc_inputs.push(crate::services::attendance_calculator::DailyCalcInput {
                punch_in: no_punch,
                punch_out: no_punch,
                shift_start: shift_start_dt,
                shift_end: shift_end_dt,
                scheduled_break_minutes: seg.break_duration_minutes.unwrap_or(0),
                actual_break_minutes: 0,
                break_start: None,
                break_end: None,
                work_unit: att_rules.work_unit_minutes,
                rounding_policy: att_rules.rounding_policy,
                rules: rules.clone(),
                ot_ceiling,
                max_early_ot_minutes: seg.max_early_ot_minutes,
                segment_id: seg.interval_id,
                ot_aggregation_mode: seg.ot_aggregation_mode,
            });
            continue;
        };

        segment_calc_inputs.push(crate::services::attendance_calculator::DailyCalcInput {
            punch_in,
            punch_out,
            shift_start: shift_start_dt,
            shift_end: shift_end_dt,
            scheduled_break_minutes: seg.break_duration_minutes.unwrap_or(0),
            actual_break_minutes,
            break_start: seg
                .break_start
                .and_then(|t| crate::utils::tz::naive_time_to_tz(t, date, tz)),
            break_end: seg
                .break_end
                .and_then(|t| crate::utils::tz::naive_time_to_tz(t, date, tz)),
            work_unit: att_rules.work_unit_minutes,
            rounding_policy: att_rules.rounding_policy,
            rules: rules.clone(),
            ot_ceiling,
            max_early_ot_minutes: seg.max_early_ot_minutes,
            segment_id: seg.interval_id,
            ot_aggregation_mode: seg.ot_aggregation_mode,
        });
    }

    (cached_matched, cached_rules, segment_calc_inputs)
}

// ─── Public report methods ──────────────────────────────────────────────────

impl ReportService {
    #[tracing::instrument(skip_all, err)]
    pub async fn get_daily_attendance_report(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        args: AttendanceReportArgs,
    ) -> Result<PaginatedResponse<DailyAttendanceReport>, AppError> {
        let preamble = self.fetch_preamble(auth_token, &args, None).await?;
        if preamble.paginated_employees().is_empty() {
            return Ok(Self::empty_paginated_response(&preamble));
        }

        let shift_names = Self::parse_shift_names(args.shift_type.as_deref());
        let tz = preamble.workspace_tz();
        let extras = self
            .fetch_report_extras(
                auth_token,
                preamble.paginated_employment_ids(),
                preamble.emp_ids(),
                args.start_date,
                args.end_date,
            )
            .await?;

        let (schedules, _) = self
            .roster_repo
            .get_workspace_schedules(
                None,
                auth_token,
                args.start_date,
                args.end_date,
                Some(preamble.paginated_employment_ids()),
                None,
                None,
            )
            .await?;

        let mut reports = Vec::new();

        for emp_sched in schedules {
            let Some(employee_id) = preamble
                .employment_to_employee()
                .get(&emp_sched.employment_id())
                .copied()
            else {
                tracing::warn!(
                    employment_id = %emp_sched.employment_id(),
                    "employment→employee mapping miss — skipping row"
                );
                continue;
            };

            let entries = preamble
                .entries_map()
                .get(&emp_sched.employment_id())
                .map_or_else(Default::default, std::vec::Vec::as_slice);
            let entries_by_date =
                crate::services::attendance_utils::group_entries_by_date(entries, tz);

            for daily in emp_sched.schedules() {
                let day_entries = entries_by_date
                    .get(&daily.date())
                    .map_or_else(Default::default, std::vec::Vec::as_slice);

                reports.push(build_daily_report_row(
                    employee_id,
                    emp_sched.employment_id(),
                    daily,
                    day_entries,
                    tz,
                    &preamble,
                    &extras,
                    emp_sched.full_name(),
                    emp_sched.employee_id_number(),
                ));
            }
        }

        Ok(Self::build_paginated_response(
            &preamble,
            reports,
            shift_names,
        ))
    }

    /// Segment-centric daily attendance report.
    ///
    /// Mirrors [`get_daily_attendance_report`] but uses
    /// [`RosterRepository::get_workspace_segments`] and produces one row per
    /// resolved segment.  Time entries are matched to segments by time-range
    /// overlap, and [`AttendanceCalculator::calculate_segment_units`] is used
    /// for multi-segment days.
    #[tracing::instrument(skip_all, err)]
    pub async fn get_segment_attendance_report(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        args: AttendanceReportArgs,
    ) -> Result<PaginatedResponse<DailyAttendanceReport>, AppError> {
        let preamble = self.fetch_preamble(auth_token, &args, None).await?;
        if preamble.paginated_employees().is_empty() {
            return Ok(Self::empty_paginated_response(&preamble));
        }

        let shift_names = Self::parse_shift_names(args.shift_type.as_deref());
        let tz = preamble.workspace_tz();
        let extras = self
            .fetch_report_extras(
                auth_token,
                preamble.paginated_employment_ids(),
                preamble.emp_ids(),
                args.start_date,
                args.end_date,
            )
            .await?;

        // Resolve segments: DB-first with compute fallback.
        let employee_segments = self
            .resolve_employee_segments(auth_token, &preamble, args.start_date, args.end_date)
            .await?;

        let mut reports = Vec::new();

        for emp_segs in &employee_segments {
            let Some(employee_id) = preamble
                .employment_to_employee()
                .get(&emp_segs.employment_id())
                .copied()
            else {
                tracing::warn!(
                    employment_id = %emp_segs.employment_id(),
                    "employment→employee mapping miss — skipping row"
                );
                continue;
            };

            let entries = preamble
                .entries_map()
                .get(&emp_segs.employment_id())
                .map_or_else(Default::default, std::vec::Vec::as_slice);
            let entries_by_date =
                crate::services::attendance_utils::group_entries_by_date(entries, tz);

            // Group segments by date
            let segments_by_date = group_segments_by_date(emp_segs.segments());
            let all_segs = emp_segs.segments();

            let att_rules = Self::resolve_attendance_rules(&preamble, employee_id);
            let emp_working_conditions = preamble
                .working_conditions_map()
                .get(&employee_id)
                .map_or_else(Default::default, std::vec::Vec::as_slice);

            for (date, seg_indices) in &segments_by_date {
                let day_segs: Vec<&crate::models::ResolvedSegment> =
                    seg_indices.iter().map(|&i| &all_segs[i]).collect();

                let (cached_matched, cached_rules, segment_calc_inputs) = build_segment_calc_inputs(
                    &day_segs,
                    entries_by_date
                        .get(date)
                        .map_or_else(Default::default, std::vec::Vec::as_slice),
                    *date,
                    tz,
                    emp_working_conditions,
                    &att_rules,
                    preamble.condition_type_catalog(),
                );

                let segment_result = crate::services::AttendanceService::compute_segment_attendance(
                    &segment_calc_inputs,
                );

                let seg_rows = build_segment_report_rows(
                    employee_id,
                    emp_segs.employment_id(),
                    *date,
                    &day_segs,
                    &cached_matched,
                    &cached_rules,
                    &segment_result,
                    &att_rules,
                    &preamble,
                    &extras,
                    emp_segs.full_name(),
                    emp_segs.employee_id_number(),
                    tz,
                );
                reports.extend(seg_rows);
            }
        }

        // Generate default no-shift rows for employments without any segments.
        // This mirrors the legacy schedule path which creates a row per day even
        // when no shift is assigned (status = Off).
        let segmented_employment_ids: std::collections::HashSet<Uuid> = employee_segments
            .iter()
            .filter(|e| !e.segments().is_empty())
            .map(|e| e.employment_id())
            .collect();

        let employee_map: std::collections::HashMap<Uuid, &crate::repositories::employee::EmployeeBasicInfo> =
            preamble.paginated_employees().iter().map(|e| (e.id, e)).collect();

        for &employment_id in preamble.paginated_employment_ids() {
            if segmented_employment_ids.contains(&employment_id) {
                continue;
            }
            let Some(employee_id) = preamble.employment_to_employee().get(&employment_id).copied()
            else {
                continue;
            };
            let Some(emp_info) = employee_map.get(&employee_id) else {
                continue;
            };

            let att_rules = Self::resolve_attendance_rules(&preamble, employee_id);
            let grace_minutes = att_rules.grace_period_minutes.unwrap_or(0);
            let employer_name = preamble
                .employments()
                .iter()
                .find(|e| e.id() == employment_id)
                .and_then(|e| preamble.employer_name_map().get(&e.employer_id()).cloned());

            let mut date = args.start_date;
            while date <= args.end_date {
                let row = build_base_report_row(
                    date,
                    employment_id,
                    employee_id,
                    &emp_info.full_name,
                    &emp_info.employee_id_number,
                    None, // shift_symbol
                    None, // shift_name
                    None, // scheduled_start
                    None, // scheduled_end
                    grace_minutes,
                    None, // break_duration_minutes
                    None, // break_start
                    None, // break_end
                    None, // is_break_auto_deduct
                    None, // actual_punch_in
                    None, // actual_punch_out
                    0,    // std_work_units
                    0,    // ot_work_units
                    None, // ot_claim_status
                    att_rules.work_unit_minutes,
                    crate::services::attendance_utils::AttendanceStatus::Off,
                    None, // holiday_entry
                    None, // generation_metadata
                    None, // is_leave_paid
                    employer_name.clone(),
                );
                reports.push(row);
                date = date.succ_opt().unwrap_or(date);
            }
        }

        Ok(Self::build_paginated_response(
            &preamble,
            reports,
            shift_names,
        ))
    }

    /// Match time entries to a segment by time-range overlap.
    ///
    /// An entry is assigned to the segment if its `punch_in_time` (in local
    /// timezone) falls within `[segment.scheduled_start, segment.scheduled_end]`.
    /// Overnight segments are handled (end < start ⇒ end += 1 day).
    #[allow(clippy::cast_possible_truncation)]
    fn match_entries_to_segment<'a>(
        day_entries: &[&'a crate::models::TimeEntry],
        segment: &crate::models::ResolvedSegment,
        date: NaiveDate,
        tz: Tz,
    ) -> Vec<&'a crate::models::TimeEntry> {
        let seg_start_dt = crate::utils::tz::naive_time_to_tz(segment.scheduled_start, date, tz);
        let seg_end_dt = crate::utils::tz::naive_time_to_tz_overnight(
            segment.scheduled_end,
            segment.scheduled_start,
            date,
            tz,
        );

        let (Some(start), Some(end)) = (seg_start_dt, seg_end_dt) else {
            return Vec::new();
        };

        day_entries
            .iter()
            .filter(|e| {
                let punch_in = e.punch_in_time().with_timezone(&tz);
                punch_in >= start && punch_in < end
            })
            .copied()
            .collect()
    }

    /// Read pre-computed WorkSegments from DB and map them into EmployeeSegments
    /// for the segment-centric report path.
    async fn build_employee_segments_from_db(
        &self,
        ws_repo: &dyn crate::repositories::work_segment_repository::WorkSegmentRepository,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        preamble: &super::ReportDataPreamble,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<crate::models::EmployeeSegments>, AppError> {
        let employment_ids = preamble.paginated_employment_ids();
        if employment_ids.is_empty() {
            return Ok(Vec::new());
        }

        let work_segments = ws_repo
            .get_segments_by_date_range(auth_token, employment_ids, start_date, end_date)
            .await?;

        if work_segments.is_empty() {
            return Ok(Vec::new());
        }

        let employment_to_employee = preamble.employment_to_employee();

        // Build employee info lookup from preamble's paginated_employees
        let employee_info_lookup: HashMap<Uuid, (String, String)> = preamble
            .paginated_employees()
            .iter()
            .map(|emp| {
                (
                    emp.id,
                    (emp.employee_id_number.clone(), emp.full_name.clone()),
                )
            })
            .collect();

        // Group work segments by employment_id
        let mut by_employment: HashMap<Uuid, Vec<crate::models::WorkSegment>> = HashMap::new();
        for ws in work_segments {
            by_employment.entry(ws.employment_id).or_default().push(ws);
        }

        let mut result = Vec::new();
        for (employment_id, segments) in by_employment {
            let Some(&employee_id) = employment_to_employee.get(&employment_id) else {
                continue;
            };
            let Some((id_number, full_name)) = employee_info_lookup.get(&employee_id) else {
                continue;
            };

            let resolved_segments: Vec<crate::models::ResolvedSegment> =
                segments.iter().map(work_segment_to_resolved).collect();

            result.push(crate::models::EmployeeSegments::new(
                employment_id,
                employee_id,
                id_number.clone(),
                full_name.clone(),
                resolved_segments,
            ));
        }

        Ok(result)
    }

    /// Resolve employee segments via DB-first with compute fallback.
    ///
    /// When `work_segment_repo` is available and returns non-empty data, uses
    /// pre-resolved segments from the database. Otherwise falls back to
    /// on-the-fly segment resolution from roster data.
    ///
    /// Logs the resolution source (`database` or `compute`) for observability.
    async fn resolve_employee_segments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        preamble: &super::ReportDataPreamble,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<crate::models::EmployeeSegments>, AppError> {
        // Try DB path first
        if let Some(ws_repo) = self.work_segment_repo() {
            match self
                .build_employee_segments_from_db(
                    ws_repo, auth_token, preamble, start_date, end_date,
                )
                .await
            {
                Ok(emps) if !emps.is_empty() => {
                    tracing::info!(
                        workspace_id = %auth_token.workspace_id(),
                        segment_source = "database",
                        segment_count = emps.len(),
                        date_range = %format!("{start_date}..{end_date}"),
                        "segment_report.source"
                    );
                    return Ok(emps);
                }
                Ok(_) => {
                    tracing::info!(
                        "DB segment read returned empty for date range {start_date}..{end_date}, falling back to compute",
                    );
                }
                Err(e) => {
                    tracing::warn!("DB segment read failed, falling back to compute: {e}");
                }
            }
        }

        // Compute fallback
        let (emps, _) = self
            .roster_repo
            .get_workspace_segments(
                None,
                auth_token,
                start_date,
                end_date,
                Some(preamble.paginated_employment_ids()),
                None,
                None,
            )
            .await?;
        Ok(emps)
    }
}

/// Convert a persisted WorkSegment into a ResolvedSegment for the report pipeline.
///
/// Many ResolvedSegment fields (break config, OT config, holiday flags, traces) are
/// not stored on WorkSegment. These are left as None/false/defaults — the report
/// row builder enriches them from `ReportExtras` and `ReportDataPreamble` anyway.
fn work_segment_to_resolved(ws: &crate::models::WorkSegment) -> crate::models::ResolvedSegment {
    crate::models::ResolvedSegment {
        template_id: ws.template_id,
        interval_id: ws.interval_id,
        sort_order: ws.sort_order,
        scheduled_start: ws.scheduled_start,
        scheduled_end: ws.scheduled_end,
        source: ws.source.clone(),
        date: Some(ws.date),
        shift_name: None,
        shift_symbol: None,
        interval_name: None,
        segment_type: ws.segment_type.clone(),
        break_duration_minutes: None,
        break_start: None,
        break_end: None,
        is_break_auto_deduct: None,
        break_is_overtime: None,
        allow_early_ot: false,
        max_early_ot_minutes: None,
        early_punch_in_grace_period_minutes: 0,
        ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode::default(),
        is_night_shift: false,
        traces: Vec::new(),
        is_holiday: false,
        is_holiday_paid: false,
        holiday_name: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ResolvedSegment, TimeEntry};
    use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
    use chrono_tz::Tz;

    fn make_test_entry(
        punch_in_utc: chrono::DateTime<Utc>,
        punch_out_utc: Option<chrono::DateTime<Utc>>,
        entry_type: crate::services::pairing::types::EntryType,
    ) -> TimeEntry {
        TimeEntry::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            None,
            punch_in_utc,
            punch_out_utc,
            Some(60),
            false,
            entry_type,
            None,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    fn make_test_segment(start_h: u32, start_m: u32, end_h: u32, end_m: u32) -> ResolvedSegment {
        ResolvedSegment {
            template_id: Uuid::new_v4(),
            interval_id: None,
            sort_order: 0,
            scheduled_start: NaiveTime::from_hms_opt(start_h, start_m, 0).unwrap(),
            scheduled_end: NaiveTime::from_hms_opt(end_h, end_m, 0).unwrap(),
            source: "fixed".to_string(),
            date: None,
            shift_name: None,
            shift_symbol: None,
            interval_name: None,
            segment_type: "work".to_string(),
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            break_is_overtime: None,
            allow_early_ot: false,
            max_early_ot_minutes: None,
            early_punch_in_grace_period_minutes: 0,
            ot_aggregation_mode:
                crate::services::attendance_calculator::OtAggregationMode::Aggregate,
            is_night_shift: false,
            traces: vec![],
            is_holiday: false,
            is_holiday_paid: false,
            holiday_name: None,
        }
    }

    fn hcm_tz() -> Tz {
        "Asia/Ho_Chi_Minh".parse().unwrap()
    }

    // ── match_entries_to_segment tests ──────────────────────────────────

    #[test]
    fn test_entry_fully_within_segment() {
        let tz = hcm_tz();
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let seg = make_test_segment(8, 0, 12, 0);

        // Punch in at 08:05 ICT (01:05 UTC), out at 11:55 ICT (04:55 UTC)
        let punch_in = tz
            .with_ymd_and_hms(2026, 6, 15, 8, 5, 0)
            .unwrap()
            .with_timezone(&Utc);
        let punch_out = tz
            .with_ymd_and_hms(2026, 6, 15, 11, 55, 0)
            .unwrap()
            .with_timezone(&Utc);

        let entry = make_test_entry(
            punch_in,
            Some(punch_out),
            crate::services::pairing::types::EntryType::Work,
        );
        let entries = vec![&entry];

        let matched = ReportService::match_entries_to_segment(&entries, &seg, date, tz);
        assert_eq!(matched.len(), 1, "entry fully within segment should match");
    }

    #[test]
    fn test_entry_starts_before_segment() {
        let tz = hcm_tz();
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let seg = make_test_segment(8, 0, 12, 0);

        // Punch in at 07:50 ICT (before segment start)
        let punch_in = tz
            .with_ymd_and_hms(2026, 6, 15, 7, 50, 0)
            .unwrap()
            .with_timezone(&Utc);
        let punch_out = tz
            .with_ymd_and_hms(2026, 6, 15, 11, 0, 0)
            .unwrap()
            .with_timezone(&Utc);

        let entry = make_test_entry(
            punch_in,
            Some(punch_out),
            crate::services::pairing::types::EntryType::Work,
        );
        let entries = vec![&entry];

        let matched = ReportService::match_entries_to_segment(&entries, &seg, date, tz);
        assert_eq!(
            matched.len(),
            0,
            "entry starting before segment should NOT match"
        );
    }

    #[test]
    fn test_entry_exactly_at_segment_boundary() {
        let tz = hcm_tz();
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let seg = make_test_segment(8, 0, 12, 0);

        // Punch in exactly at segment end (12:00)
        let punch_in = tz
            .with_ymd_and_hms(2026, 6, 15, 12, 0, 0)
            .unwrap()
            .with_timezone(&Utc);

        let entry = make_test_entry(
            punch_in,
            None,
            crate::services::pairing::types::EntryType::Work,
        );
        let entries = vec![&entry];

        let matched = ReportService::match_entries_to_segment(&entries, &seg, date, tz);
        assert_eq!(
            matched.len(),
            0,
            "entry at segment end (exclusive) should NOT match"
        );

        // Punch in exactly at segment start (08:00)
        let punch_in = tz
            .with_ymd_and_hms(2026, 6, 15, 8, 0, 0)
            .unwrap()
            .with_timezone(&Utc);

        let entry = make_test_entry(
            punch_in,
            None,
            crate::services::pairing::types::EntryType::Work,
        );
        let entries = vec![&entry];

        let matched = ReportService::match_entries_to_segment(&entries, &seg, date, tz);
        assert_eq!(
            matched.len(),
            1,
            "entry at segment start (inclusive) should match"
        );
    }

    #[test]
    fn test_multiple_entries_matched_to_same_segment() {
        let tz = hcm_tz();
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let seg = make_test_segment(8, 0, 17, 0);

        let e1_in = tz
            .with_ymd_and_hms(2026, 6, 15, 8, 5, 0)
            .unwrap()
            .with_timezone(&Utc);
        let e1_out = tz
            .with_ymd_and_hms(2026, 6, 15, 12, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let e2_in = tz
            .with_ymd_and_hms(2026, 6, 15, 13, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let e2_out = tz
            .with_ymd_and_hms(2026, 6, 15, 17, 0, 0)
            .unwrap()
            .with_timezone(&Utc);

        let entry1 = make_test_entry(
            e1_in,
            Some(e1_out),
            crate::services::pairing::types::EntryType::Work,
        );
        let entry2 = make_test_entry(
            e2_in,
            Some(e2_out),
            crate::services::pairing::types::EntryType::Work,
        );
        let entries = vec![&entry1, &entry2];

        let matched = ReportService::match_entries_to_segment(&entries, &seg, date, tz);
        assert_eq!(matched.len(), 2, "both entries within segment");
        assert_eq!(matched[0].punch_in_time(), e1_in);
        assert_eq!(matched[1].punch_in_time(), e2_in);
    }

    #[test]
    fn test_break_entries_also_matched() {
        let tz = hcm_tz();
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let seg = make_test_segment(8, 0, 17, 0);

        let break_in = tz
            .with_ymd_and_hms(2026, 6, 15, 12, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let break_out = tz
            .with_ymd_and_hms(2026, 6, 15, 13, 0, 0)
            .unwrap()
            .with_timezone(&Utc);

        let break_entry = make_test_entry(
            break_in,
            Some(break_out),
            crate::services::pairing::types::EntryType::Break,
        );
        let entries = vec![&break_entry];

        let matched = ReportService::match_entries_to_segment(&entries, &seg, date, tz);
        assert_eq!(matched.len(), 1, "break entries are also matched by time");
    }

    #[test]
    fn test_no_entries_for_segment() {
        let tz = hcm_tz();
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let seg = make_test_segment(8, 0, 12, 0);

        // Entry belongs to a different time range
        let punch_in = tz
            .with_ymd_and_hms(2026, 6, 15, 14, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let punch_out = tz
            .with_ymd_and_hms(2026, 6, 15, 18, 0, 0)
            .unwrap()
            .with_timezone(&Utc);

        let entry = make_test_entry(
            punch_in,
            Some(punch_out),
            crate::services::pairing::types::EntryType::Work,
        );
        let entries = vec![&entry];

        let matched = ReportService::match_entries_to_segment(&entries, &seg, date, tz);
        assert_eq!(matched.len(), 0, "entry outside segment time → no match");
    }

    #[test]
    fn test_overnight_segment() {
        let tz = hcm_tz();
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        // Night shift: 22:00 → 06:00 next day
        let seg = make_test_segment(22, 0, 6, 0);

        // Entry at 23:00 same day → should match (within [22:00, 06:00+1day])
        let punch_in = tz
            .with_ymd_and_hms(2026, 6, 15, 23, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let entry = make_test_entry(
            punch_in,
            None,
            crate::services::pairing::types::EntryType::Work,
        );
        let entries = vec![&entry];

        let matched = ReportService::match_entries_to_segment(&entries, &seg, date, tz);
        assert_eq!(matched.len(), 1, "entry at 23:00 within overnight segment");

        // Entry at 03:00 next day → should also match
        let punch_in = tz
            .with_ymd_and_hms(2026, 6, 16, 3, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let entry = make_test_entry(
            punch_in,
            None,
            crate::services::pairing::types::EntryType::Work,
        );
        let entries = vec![&entry];

        let matched = ReportService::match_entries_to_segment(&entries, &seg, date, tz);
        assert_eq!(
            matched.len(),
            1,
            "entry at 03:00 next day within overnight segment"
        );

        // Entry at 08:00 same day → outside overnight segment
        let punch_in = tz
            .with_ymd_and_hms(2026, 6, 15, 8, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let entry = make_test_entry(
            punch_in,
            None,
            crate::services::pairing::types::EntryType::Work,
        );
        let entries = vec![&entry];

        let matched = ReportService::match_entries_to_segment(&entries, &seg, date, tz);
        assert_eq!(matched.len(), 0, "entry at 08:00 outside overnight segment");
    }

    #[test]
    fn test_segment_between_two_segments() {
        // Two segments: morning 08-12, afternoon 13-17
        // Entry at 12:30 should match NEITHER (gap between segments)
        let tz = hcm_tz();
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();

        let morning = make_test_segment(8, 0, 12, 0);
        let afternoon = make_test_segment(13, 0, 17, 0);

        let punch_in = tz
            .with_ymd_and_hms(2026, 6, 15, 12, 30, 0)
            .unwrap()
            .with_timezone(&Utc);
        let entry = make_test_entry(
            punch_in,
            None,
            crate::services::pairing::types::EntryType::Work,
        );
        let entries = vec![&entry];

        let matched_morning = ReportService::match_entries_to_segment(&entries, &morning, date, tz);
        let matched_afternoon =
            ReportService::match_entries_to_segment(&entries, &afternoon, date, tz);

        assert_eq!(matched_morning.len(), 0, "12:30 outside morning");
        assert_eq!(matched_afternoon.len(), 0, "12:30 outside afternoon");
    }

    #[test]
    fn test_empty_entries_list() {
        let tz = hcm_tz();
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let seg = make_test_segment(8, 0, 17, 0);
        let entries: Vec<&TimeEntry> = vec![];

        let matched = ReportService::match_entries_to_segment(&entries, &seg, date, tz);
        assert_eq!(matched.len(), 0, "no entries → empty result");
    }

    #[test]
    fn test_two_segments_with_entries_in_each() {
        // Morning 08-12 → entry1, Afternoon 13-17 → entry2
        let tz = hcm_tz();
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();

        let morning = make_test_segment(8, 0, 12, 0);
        let afternoon = make_test_segment(13, 0, 17, 0);

        let e1_in = tz
            .with_ymd_and_hms(2026, 6, 15, 8, 5, 0)
            .unwrap()
            .with_timezone(&Utc);
        let e1_out = tz
            .with_ymd_and_hms(2026, 6, 15, 12, 3, 0)
            .unwrap()
            .with_timezone(&Utc);
        let e2_in = tz
            .with_ymd_and_hms(2026, 6, 15, 13, 1, 0)
            .unwrap()
            .with_timezone(&Utc);
        let e2_out = tz
            .with_ymd_and_hms(2026, 6, 15, 16, 59, 0)
            .unwrap()
            .with_timezone(&Utc);

        let entry1 = make_test_entry(
            e1_in,
            Some(e1_out),
            crate::services::pairing::types::EntryType::Work,
        );
        let entry2 = make_test_entry(
            e2_in,
            Some(e2_out),
            crate::services::pairing::types::EntryType::Work,
        );
        let entries = vec![&entry1, &entry2];

        let matched_morning = ReportService::match_entries_to_segment(&entries, &morning, date, tz);
        let matched_afternoon =
            ReportService::match_entries_to_segment(&entries, &afternoon, date, tz);

        assert_eq!(matched_morning.len(), 1, "morning gets its entry");
        assert_eq!(matched_afternoon.len(), 1, "afternoon gets its entry");
        // Verify each segment gets the correct entry
        assert_eq!(matched_morning[0].punch_in_time(), e1_in);
        assert_eq!(matched_afternoon[0].punch_in_time(), e2_in);
    }

    // ── parse_shift_names tests ──────────────────────────────────────────

    #[test]
    fn test_parse_shift_names_none() {
        assert!(ReportService::parse_shift_names(None).is_none());
    }

    #[test]
    fn test_parse_shift_names_single() {
        let result = ReportService::parse_shift_names(Some("Morning Shift"));
        assert_eq!(result, Some(vec!["Morning Shift".to_string()]));
    }

    #[test]
    fn test_parse_shift_names_multiple() {
        let result = ReportService::parse_shift_names(Some("Morning, Night"));
        assert_eq!(
            result,
            Some(vec!["Morning".to_string(), "Night".to_string()])
        );
    }

    #[test]
    fn test_parse_shift_names_trims_whitespace() {
        let result = ReportService::parse_shift_names(Some("  A , B , C  "));
        assert_eq!(
            result,
            Some(vec!["A".to_string(), "B".to_string(), "C".to_string()])
        );
    }

    #[test]
    fn test_parse_shift_names_filters_empty() {
        let result = ReportService::parse_shift_names(Some("A,,B, ,C"));
        assert_eq!(
            result,
            Some(vec!["A".to_string(), "B".to_string(), "C".to_string()])
        );
    }

    #[test]
    fn test_parse_shift_names_empty_string() {
        let result = ReportService::parse_shift_names(Some(""));
        assert_eq!(result, Some(vec![]));
    }
}
