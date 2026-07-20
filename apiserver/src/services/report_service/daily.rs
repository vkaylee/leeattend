use super::{AttendanceReportArgs, CalculateDayUnitsArgs, ReportService};
use crate::domain::policy::resolver::{PolicyResolver, ShiftPolicyOverrides};
use crate::errors::AppError;
use crate::models::{DailyAttendanceReport, PaginatedResponse};
use crate::services::{AttendanceCalculator, PolicyResolverService};
use crate::utils::i18n::I18nKey;
use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use uuid::Uuid;

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

impl ReportService {
    /// Shared calculation for a single day's standard/OT units.
    pub(crate) fn calculate_day_units(args: CalculateDayUnitsArgs<'_, '_>) -> (i32, i32) {
        if args.daily.shift_id().is_none() {
            return (0, 0);
        }

        let actual_punch_in = args
            .day_entries
            .first()
            .map(|e| e.punch_in_time().with_timezone(&args.tz));
        let actual_punch_out = args
            .day_entries
            .last()
            .and_then(|e| e.punch_out_time().map(|t| t.with_timezone(&args.tz)));

        if let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) {
            let actual_break_minutes: i32 = args
                .day_entries
                .iter()
                .filter(|e| e.entry_type() == "break")
                .map(|e| e.duration_minutes().unwrap_or(0))
                .sum();

            let shift_start_dt = args
                .daily
                .start_time()
                .and_then(|t| args.tz.from_local_datetime(&args.date.and_time(t)).single());
            let shift_end_dt = args
                .daily
                .end_time()
                .and_then(|t| args.tz.from_local_datetime(&args.date.and_time(t)).single());

            let break_start_dt = args
                .daily
                .break_start()
                .and_then(|t| args.tz.from_local_datetime(&args.date.and_time(t)).single());
            let break_end_dt = args
                .daily
                .break_end()
                .and_then(|t| args.tz.from_local_datetime(&args.date.and_time(t)).single());

            let (shift_start, shift_end) =
                if let (Some(start), Some(mut end)) = (shift_start_dt, shift_end_dt) {
                    if args.daily.end_time() < args.daily.start_time() {
                        end += chrono::Duration::days(1);
                    }
                    (Some(start), Some(end))
                } else {
                    (None, None)
                };

            let (break_start, break_end) =
                if let (Some(bs), Some(mut be)) = (break_start_dt, break_end_dt) {
                    if args.daily.break_end() < args.daily.break_start() {
                        be += chrono::Duration::days(1);
                    }
                    (Some(bs), Some(be))
                } else {
                    (None, None)
                };

            AttendanceCalculator::calculate_daily_units(
                &crate::services::attendance_calculator::DailyCalcInput {
                    punch_in,
                    punch_out,
                    shift_start,
                    shift_end,
                    scheduled_break_minutes: args.daily.break_duration_minutes().unwrap_or(0),
                    actual_break_minutes,
                    break_start,
                    break_end,
                    work_unit: args.work_unit_minutes,
                    rounding_policy: args.rounding_policy.to_string(),
                    rules: args.rules,
                    ot_ceiling: None,
                    max_early_ot_minutes: args.daily.max_early_ot_minutes(),
                    segment_id: None,
                    ot_aggregation_mode:
                        crate::services::attendance_calculator::OtAggregationMode::Aggregate,
                },
            )
        } else {
            (0, 0)
        }
    }

    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn get_daily_attendance_report(
        &self,
        args: AttendanceReportArgs,
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

        let tz: Tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());

        let page = args.page.unwrap_or(1);
        let limit = args.limit.unwrap_or(20);

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id);

        // Shift-type filter (comma-separated shift names), applied post-join to keep
        // pagination at employee level. Mirrors the existing `employee_ids` split.
        let shift_names: Option<Vec<String>> = args.shift_type.as_ref().map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(String::from)
                .collect()
        });

        let (paginated_employees, total_employees, employments, paginated_employment_ids) =
            if let Some(specific_employment_id) = args.employment_id {
                let emp = self
                    .employment_repo
                    .get_employment_by_id(&auth_token, specific_employment_id)
                    .await?
                    .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?;

                let employee = self
                    .employee_repo
                    .get_employees_by_ids(&[emp.employee_id()], &auth_token)
                    .await?
                    .into_iter()
                    .next()
                    .map(|e| crate::repositories::employee::EmployeeBasicInfo {
                        id: e.id(),
                        full_name: e.full_name().to_string(),
                        employee_id_number: e.employee_id_number().to_string(),
                    })
                    .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?;

                (
                    vec![employee],
                    1i64,
                    vec![emp],
                    vec![specific_employment_id],
                )
            } else {
                let (paginated_employees, total_employees) = self
                    .employee_repo
                    .get_paginated_employee_ids(
                        &auth_token,
                        crate::models::EmployeePaginationArgs::new(
                            args.employee_ids.as_ref(),
                            args.department_id,
                            page,
                            limit,
                            false,
                            Some((args.start_date, args.end_date)),
                            args.employee_status.clone(),
                        ),
                    )
                    .await
                    .map_err(|e| {
                        tracing::error!("Error fetching paginated employees: {}", e);
                        AppError::Internal("Internal error".into())
                    })?;

                if paginated_employees.is_empty() {
                    return Ok(PaginatedResponse {
                        data: Vec::new(),
                        total: usize::try_from(total_employees).unwrap_or(0),
                        page,
                        limit,
                    });
                }

                let emp_ids: Vec<Uuid> = paginated_employees.iter().map(|e| e.id).collect();

                let employments = self
                    .employment_repo
                    .get_employments_for_employees(&auth_token, &emp_ids)
                    .await?;

                let selection = super::employment_selection::select_report_employments(
                    &employments,
                    &emp_ids,
                    args.start_date,
                    args.end_date,
                    args.employer_id,
                )?;
                let paginated_employment_ids = selection.employment_ids;

                (
                    paginated_employees,
                    total_employees,
                    employments,
                    paginated_employment_ids,
                )
            };

        // Map employment_id -> employee_id
        let mut employment_to_employee = std::collections::HashMap::new();
        for emp in &employments {
            employment_to_employee.insert(emp.id(), emp.employee_id());
        }

        let emp_ids: Vec<Uuid> = paginated_employees.iter().map(|e| e.id).collect();

        let (schedules, _) = self
            .roster_repo
            .get_workspace_schedules(
                None,
                &auth_token,
                args.start_date,
                args.end_date,
                Some(&paginated_employment_ids),
                None,
                None,
            )
            .await?;

        let (start_dt, end_dt) = crate::services::attendance_utils::date_range_to_utc_checked(
            tz,
            args.start_date,
            args.end_date,
        )
        .ok_or_else(|| AppError::validation(I18nKey::Validation))?;

        let entries_map = self
            .time_entry_repo
            .get_workspace_time_entries_batch_by_employments(
                &auth_token,
                &paginated_employment_ids,
                start_dt,
                end_dt,
            )
            .await?;

        let holidays_map = self
            .holiday_repo
            .get_holidays_in_range(&auth_token, args.start_date, args.end_date)
            .await?;

        let ot_status_map = self
            .attendance_daily_summary_repo
            .get_batch_ot_statuses(
                &auth_token,
                &paginated_employment_ids,
                args.start_date,
                args.end_date,
            )
            .await?;

        let leave_details_map = self
            .leave_repo
            .get_approved_leave_details(&auth_token, &emp_ids, args.start_date, args.end_date)
            .await?;

        let leave_date_set: std::collections::HashSet<(Uuid, NaiveDate)> =
            leave_details_map.keys().copied().collect();

        let attendance_rules_map = PolicyResolverService::resolve_attendance_policy_batch(
            &*self.policy_repo,
            &emp_ids,
            args.workspace_id,
            args.start_date,
        )
        .await?;

        let working_conditions_map = PolicyResolverService::resolve_attendance_rules_batch(
            &*self.employee_repo,
            &emp_ids,
            args.workspace_id,
            args.start_date,
            args.end_date,
        )
        .await?;
        let condition_type_catalog = PolicyResolverService::load_condition_type_catalog(
            self.condition_type_repo.as_ref(),
            args.workspace_id,
        )
        .await?;

        // Fetch all employers to get names
        let (employers, _) = self
            .employer_repo
            .get_employers(&auth_token, 1, 1000)
            .await?;

        let employer_name_map: std::collections::HashMap<Uuid, String> =
            employers.into_iter().map(|e| (e.id, e.name)).collect();

        let mut reports = Vec::new();

        for emp_sched in schedules {
            let employee_id = employment_to_employee
                .get(&emp_sched.employment_id())
                .copied()
                .unwrap_or_else(Uuid::nil);

            let entries = entries_map
                .get(&emp_sched.employment_id())
                .map_or_else(Default::default, std::vec::Vec::as_slice);
            let entries_by_date =
                crate::services::attendance_utils::group_entries_by_date(entries, tz);

            for daily in emp_sched.schedules() {
                let date = daily.date();

                let day_entries = entries_by_date
                    .get(&date)
                    .map_or_else(Default::default, std::vec::Vec::as_slice);

                let actual_punch_in = day_entries
                    .first()
                    .map(|e| e.punch_in_time().with_timezone(&tz));
                let actual_punch_out = day_entries
                    .last()
                    .and_then(|e| e.punch_out_time().map(|t| t.with_timezone(&tz)));

                let emp_attendance_rules = attendance_rules_map
                    .get(&employee_id)
                    .cloned()
                    .unwrap_or_else(|| crate::models::AttendanceRules {
                        work_unit_minutes: 30,
                        rounding_policy: "floor".to_string(),
                        early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                        grace_period_minutes: None,
                    });

                let emp_working_conditions = working_conditions_map
                    .get(&employee_id)
                    .map_or_else(Default::default, std::vec::Vec::as_slice);

                let base_rules = PolicyResolver::build_rules_from_conditions(
                    emp_working_conditions,
                    date,
                    emp_attendance_rules.early_arrival_strategy,
                    emp_attendance_rules
                        .grace_period_minutes
                        .unwrap_or(crate::domain::policy::defaults::DEFAULT_GRACE_PERIOD_MINUTES),
                    Some(&condition_type_catalog),
                );

                let overrides = ShiftPolicyOverrides::new(
                    daily.early_punch_in_grace_period_minutes(),
                    daily.is_break_auto_deduct(),
                    daily.allow_early_ot(),
                    emp_attendance_rules.early_arrival_strategy,
                    daily.break_is_overtime().unwrap_or(false),
                );
                let rules = PolicyResolver::apply_shift_overrides(base_rules, &overrides);

                // Resolved late/absence grace for this employee-day (default 15 when the
                // attendance policy leaves it unset). Captured once and reused both for
                // status determination and as the row-level `grace_period_minutes` field
                // so the frontend computes A/B/C no-show zones with the *same* value.
                let grace_minutes = rules.grace_period_minutes();

                let (std_work_units, ot_work_units) =
                    Self::calculate_day_units(CalculateDayUnitsArgs {
                        day_entries,
                        daily,
                        date,
                        tz,
                        employee_id,
                        workspace_id: workspace.id(),
                        rules: rules.clone(),
                        work_unit_minutes: emp_attendance_rules.work_unit_minutes,
                        rounding_policy: &emp_attendance_rules.rounding_policy,
                    });

                let holiday_entry = holidays_map.get(&date);
                let is_hol = holiday_entry.is_some();
                let is_on_leave = leave_date_set.contains(&(employee_id, date));
                let is_leave_paid = if is_on_leave {
                    leave_details_map.get(&(employee_id, date)).copied()
                } else {
                    None
                };

                let status = crate::services::attendance_utils::determine_daily_status(
                    &crate::services::attendance_utils::DailyStatusInput::new(
                        actual_punch_in.is_some(),
                        actual_punch_in.map(|p| p.time()),
                        daily.start_time(),
                        is_hol,
                        holiday_entry.is_none_or(crate::models::Holiday::is_paid),
                        daily.shift_id().is_some(),
                        is_on_leave,
                        Some(grace_minutes),
                    ),
                );

                let mut metadata_map = serde_json::Map::new();

                if !day_entries.is_empty() {
                    metadata_map.insert("time_entries".to_string(), serde_json::json!(day_entries));
                }

                if !daily.traces().is_empty() {
                    metadata_map.insert(
                        "schedule_traces".to_string(),
                        serde_json::json!(daily.traces()),
                    );
                }

                let combined_metadata = if metadata_map.is_empty() {
                    None
                } else {
                    Some(serde_json::Value::Object(metadata_map))
                };

                let employer_id = employments
                    .iter()
                    .find(|e| e.id() == emp_sched.employment_id())
                    .map(crate::models::Employment::employer_id);
                let employer_name = employer_id.and_then(|id| employer_name_map.get(&id).cloned());

                reports.push(DailyAttendanceReport {
                    date,
                    employment_id: emp_sched.employment_id(),
                    employee_id,
                    employee_name: emp_sched.full_name().to_string(),
                    employee_id_number: emp_sched.employee_id_number().to_string(),
                    shift_symbol: daily.shift_symbol().map(String::from),
                    shift_name: daily.shift_name().map(String::from),
                    scheduled_start: daily.start_time(),
                    scheduled_end: daily.end_time(),
                    grace_period_minutes: Some(grace_minutes),
                    break_duration_minutes: daily.break_duration_minutes(),
                    break_start: daily.break_start(),
                    break_end: daily.break_end(),
                    is_break_auto_deduct: daily.is_break_auto_deduct(),
                    actual_punch_in: actual_punch_in.map(|t| t.with_timezone(&Utc)),
                    actual_punch_out: actual_punch_out.map(|t| t.with_timezone(&Utc)),
                    std_work_units,
                    // Segment fields (legacy path — none)
                    segment_id: None,
                    shift_id: None,
                    interval_id: None,
                    interval_name: None,
                    segment_type: None,
                    sort_order: None,
                    segment_count: None,
                    ot_work_units,
                    ot_claim_status: ot_status_map
                        .get(&(emp_sched.employment_id(), date))
                        .cloned(),
                    unit_type: crate::models::enums::TimekeepingUnit::DAY,
                    unit_coefficient_hours: f64::from(emp_attendance_rules.work_unit_minutes)
                        / 60.0,
                    computed_std_hours: f64::from(std_work_units)
                        * (f64::from(emp_attendance_rules.work_unit_minutes) / 60.0),
                    computed_ot_hours: f64::from(ot_work_units)
                        * (f64::from(emp_attendance_rules.work_unit_minutes) / 60.0),
                    status,
                    is_holiday: is_hol,
                    is_unscheduled: status
                        == crate::services::attendance_utils::AttendanceStatus::Unscheduled,
                    holiday_name: holiday_entry.map(|h| h.name().to_string()),
                    generation_metadata: combined_metadata,
                    is_leave_paid,
                    employer_name,
                    timezone: None,
                });
            }
        }

        // Post-join shift-type filter (O(rows), mirrors the existing employer_id filter).
        let reports = filter_by_shift_type(reports, shift_names);

        Ok(PaginatedResponse {
            data: reports,
            total: usize::try_from(total_employees).unwrap_or(0),
            page,
            limit,
        })
    }

    /// Segment-centric daily attendance report.
    ///
    /// Mirrors [`get_daily_attendance_report`] but uses
    /// [`RosterRepository::get_workspace_segments`] and produces one row per
    /// resolved segment.  Time entries are matched to segments by time-range
    /// overlap, and [`AttendanceCalculator::calculate_segment_units`] is used
    /// for multi-segment days.
    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn get_segment_attendance_report(
        &self,
        args: AttendanceReportArgs,
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

        let tz: Tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());

        let page = args.page.unwrap_or(1);
        let limit = args.limit.unwrap_or(20);

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id);

        // Shift-type filter (comma-separated shift names), applied post-join to keep
        // pagination at employee level. Mirrors the existing `employee_ids` split.
        let shift_names: Option<Vec<String>> = args.shift_type.as_ref().map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(String::from)
                .collect()
        });

        // Fetch segments (segment-centric data path)
        let (paginated_employees, total_employees, employments, paginated_employment_ids) =
            if let Some(specific_employment_id) = args.employment_id {
                let emp = self
                    .employment_repo
                    .get_employment_by_id(&auth_token, specific_employment_id)
                    .await?
                    .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?;
                let employee = self
                    .employee_repo
                    .get_employees_by_ids(&[emp.employee_id()], &auth_token)
                    .await?
                    .into_iter()
                    .next()
                    .map(|e| crate::repositories::employee::EmployeeBasicInfo {
                        id: e.id(),
                        full_name: e.full_name().to_string(),
                        employee_id_number: e.employee_id_number().to_string(),
                    })
                    .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?;
                (
                    vec![employee],
                    1i64,
                    vec![emp],
                    vec![specific_employment_id],
                )
            } else {
                let (paginated_employees, total_employees) = self
                    .employee_repo
                    .get_paginated_employee_ids(
                        &auth_token,
                        crate::models::EmployeePaginationArgs::new(
                            args.employee_ids.as_ref(),
                            args.department_id,
                            page,
                            limit,
                            false,
                            Some((args.start_date, args.end_date)),
                            args.employee_status.clone(),
                        ),
                    )
                    .await
                    .map_err(|e| {
                        tracing::error!("Error fetching paginated employees: {}", e);
                        AppError::Internal("Internal error".into())
                    })?;
                if paginated_employees.is_empty() {
                    return Ok(PaginatedResponse {
                        data: Vec::new(),
                        total: usize::try_from(total_employees).unwrap_or(0),
                        page,
                        limit,
                    });
                }
                let emp_ids: Vec<Uuid> = paginated_employees.iter().map(|e| e.id).collect();
                let employments = self
                    .employment_repo
                    .get_employments_for_employees(&auth_token, &emp_ids)
                    .await?;
                let selection = super::employment_selection::select_report_employments(
                    &employments,
                    &emp_ids,
                    args.start_date,
                    args.end_date,
                    args.employer_id,
                )?;
                let paginated_employment_ids = selection.employment_ids;
                (
                    paginated_employees,
                    total_employees,
                    employments,
                    paginated_employment_ids,
                )
            };

        let mut employment_to_employee = std::collections::HashMap::new();
        for emp in &employments {
            employment_to_employee.insert(emp.id(), emp.employee_id());
        }
        let emp_ids: Vec<Uuid> = paginated_employees.iter().map(|e| e.id).collect();

        // ── Segment-centric schedule resolution ──
        let (employee_segments, _) = self
            .roster_repo
            .get_workspace_segments(
                None,
                &auth_token,
                args.start_date,
                args.end_date,
                Some(&paginated_employment_ids),
                None,
                None,
            )
            .await?;

        let (start_dt, end_dt) = crate::services::attendance_utils::date_range_to_utc_checked(
            tz,
            args.start_date,
            args.end_date,
        )
        .ok_or_else(|| AppError::validation(I18nKey::Validation))?;

        let entries_map = self
            .time_entry_repo
            .get_workspace_time_entries_batch_by_employments(
                &auth_token,
                &paginated_employment_ids,
                start_dt,
                end_dt,
            )
            .await?;

        let holidays_map = self
            .holiday_repo
            .get_holidays_in_range(&auth_token, args.start_date, args.end_date)
            .await?;

        let ot_status_map = self
            .attendance_daily_summary_repo
            .get_batch_ot_statuses(
                &auth_token,
                &paginated_employment_ids,
                args.start_date,
                args.end_date,
            )
            .await?;

        let leave_details_map = self
            .leave_repo
            .get_approved_leave_details(&auth_token, &emp_ids, args.start_date, args.end_date)
            .await?;

        let leave_date_set: std::collections::HashSet<(Uuid, NaiveDate)> =
            leave_details_map.keys().copied().collect();

        let attendance_rules_map = PolicyResolverService::resolve_attendance_policy_batch(
            &*self.policy_repo,
            &emp_ids,
            args.workspace_id,
            args.start_date,
        )
        .await?;

        let working_conditions_map = PolicyResolverService::resolve_attendance_rules_batch(
            &*self.employee_repo,
            &emp_ids,
            args.workspace_id,
            args.start_date,
            args.end_date,
        )
        .await?;
        let condition_type_catalog = PolicyResolverService::load_condition_type_catalog(
            self.condition_type_repo.as_ref(),
            args.workspace_id,
        )
        .await?;

        let (employers, _) = self
            .employer_repo
            .get_employers(&auth_token, 1, 1000)
            .await?;
        let employer_name_map: std::collections::HashMap<Uuid, String> =
            employers.into_iter().map(|e| (e.id, e.name)).collect();

        // ── Build segment-centric report rows ──
        let mut reports = Vec::new();

        for emp_segs in &employee_segments {
            let employee_id = employment_to_employee
                .get(&emp_segs.employment_id())
                .copied()
                .unwrap_or_else(Uuid::nil);

            let entries = entries_map
                .get(&emp_segs.employment_id())
                .map_or_else(Default::default, std::vec::Vec::as_slice);
            let entries_by_date =
                crate::services::attendance_utils::group_entries_by_date(entries, tz);

            // Group segments by date for per-day context
            let mut segments_by_date: std::collections::BTreeMap<NaiveDate, Vec<usize>> =
                std::collections::BTreeMap::new();
            let all_segs = emp_segs.segments();
            for (i, seg) in all_segs.iter().enumerate() {
                if let Some(date) = seg.date {
                    segments_by_date.entry(date).or_default().push(i);
                }
            }

            let emp_attendance_rules = attendance_rules_map
                .get(&employee_id)
                .cloned()
                .unwrap_or_else(|| crate::models::AttendanceRules {
                    work_unit_minutes: 30,
                    rounding_policy: "floor".to_string(),
                    early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                    grace_period_minutes: None,
                });

            let emp_working_conditions = working_conditions_map
                .get(&employee_id)
                .map_or_else(Default::default, std::vec::Vec::as_slice);

            for (date, seg_indices) in &segments_by_date {
                let day_entries = entries_by_date
                    .get(date)
                    .map_or_else(Default::default, std::vec::Vec::as_slice);

                let holiday_entry = holidays_map.get(date);
                let is_hol = holiday_entry.is_some();
                let is_on_leave = leave_date_set.contains(&(employee_id, *date));
                let is_leave_paid = if is_on_leave {
                    leave_details_map.get(&(employee_id, *date)).copied()
                } else {
                    None
                };

                let day_segs: Vec<&crate::models::ResolvedSegment> =
                    seg_indices.iter().map(|&i| &all_segs[i]).collect();

                let segment_count = i32::try_from(day_segs.len()).unwrap_or(1);

                // Build per-segment calc inputs
                let mut segment_calc_inputs: Vec<
                    crate::services::attendance_calculator::DailyCalcInput<chrono_tz::Tz>,
                > = Vec::new();

                for seg in &day_segs {
                    let matched = Self::match_entries_to_segment(day_entries, seg, *date, tz);

                    let actual_punch_in = matched
                        .first()
                        .map(|e| e.punch_in_time().with_timezone(&tz));
                    let actual_punch_out = matched
                        .last()
                        .and_then(|e| e.punch_out_time().map(|t| t.with_timezone(&tz)));

                    let shift_start_dt = tz
                        .from_local_datetime(&date.and_time(seg.scheduled_start))
                        .single();
                    let shift_end_dt = {
                        if seg.scheduled_end < seg.scheduled_start {
                            // overnight segment
                            let raw = tz
                                .from_local_datetime(&date.and_time(seg.scheduled_end))
                                .single();
                            raw.map(|dt| dt + chrono::Duration::days(1))
                        } else {
                            tz.from_local_datetime(&date.and_time(seg.scheduled_end))
                                .single()
                        }
                    };

                    let base_rules = PolicyResolver::build_rules_from_conditions(
                        emp_working_conditions,
                        *date,
                        emp_attendance_rules.early_arrival_strategy,
                        emp_attendance_rules.grace_period_minutes.unwrap_or(
                            crate::domain::policy::defaults::DEFAULT_GRACE_PERIOD_MINUTES,
                        ),
                        Some(&condition_type_catalog),
                    );

                    let overrides = ShiftPolicyOverrides::new(
                        Some(seg.early_punch_in_grace_period_minutes),
                        seg.is_break_auto_deduct,
                        Some(seg.allow_early_ot),
                        emp_attendance_rules.early_arrival_strategy,
                        seg.break_is_overtime.unwrap_or(false),
                    );
                    let rules = PolicyResolver::apply_shift_overrides(base_rules, &overrides);

                    // Compute OT ceiling from next segment
                    let ot_ceiling = day_segs
                        .iter()
                        .filter(|s| s.scheduled_start > seg.scheduled_start)
                        .min_by_key(|s| s.scheduled_start)
                        .and_then(|next| {
                            tz.from_local_datetime(&date.and_time(next.scheduled_start))
                                .single()
                        });

                    let actual_break_minutes: i32 = matched
                        .iter()
                        .filter(|e| e.entry_type() == "break")
                        .map(|e| e.duration_minutes().unwrap_or(0))
                        .sum();

                    segment_calc_inputs.push(
                        crate::services::attendance_calculator::DailyCalcInput {
                            punch_in: actual_punch_in.unwrap_or_else(|| tz.from_utc_datetime(&Utc::now().naive_utc())),
                        punch_out: actual_punch_out.unwrap_or_else(|| tz.from_utc_datetime(&Utc::now().naive_utc())),
                            shift_start: shift_start_dt,
                            shift_end: shift_end_dt,
                            scheduled_break_minutes: seg.break_duration_minutes.unwrap_or(0),
                            actual_break_minutes,
                            break_start: seg.break_start.and_then(|t| {
                                tz.from_local_datetime(&date.and_time(t)).single()
                            }),
                            break_end: seg.break_end.and_then(|t| {
                                tz.from_local_datetime(&date.and_time(t)).single()
                            }),
                            work_unit: emp_attendance_rules.work_unit_minutes,
                            rounding_policy: emp_attendance_rules.rounding_policy.clone(),
                            rules: rules.clone(),
                            ot_ceiling,
                            max_early_ot_minutes: seg.max_early_ot_minutes,
                            segment_id: seg.interval_id,
                            ot_aggregation_mode: match seg.ot_aggregation_mode.as_str() {
                                "per_segment" => crate::services::attendance_calculator::OtAggregationMode::PerSegment,
                                _ => crate::services::attendance_calculator::OtAggregationMode::Aggregate,
                            },
                        },
                    );
                }

                // Calculate units (per-segment or aggregated depending on mode)
                let calc_result = if segment_calc_inputs.len() > 1 {
                    crate::services::AttendanceCalculator::calculate_segment_units(
                        &segment_calc_inputs,
                    )
                } else {
                    let (std_units, ot_units) =
                        crate::services::AttendanceCalculator::calculate_daily_units(
                            &segment_calc_inputs[0],
                        );
                    crate::services::attendance_calculator::SegmentCalcResult {
                        total_standard_units: std_units,
                        total_ot_units: ot_units,
                        per_segment_buckets: Vec::new(),
                    }
                };

                // Build report rows for each segment
                for (seg_idx, seg) in day_segs.iter().enumerate() {
                    let matched = Self::match_entries_to_segment(day_entries, seg, *date, tz);
                    let actual_punch_in = matched
                        .first()
                        .map(|e| e.punch_in_time().with_timezone(&Utc));
                    let actual_punch_out = matched
                        .last()
                        .and_then(|e| e.punch_out_time().map(|t| t.with_timezone(&Utc)));

                    let (seg_std, seg_ot) = calc_result
                        .per_segment_buckets
                        .get(seg_idx)
                        .map(|(_, buckets)| {
                            let std_mins = i64::from(buckets.standard_minutes);
                            let ot_mins = i64::from(
                                buckets.early_ot_minutes
                                    + buckets.late_ot_minutes
                                    + buckets.break_ot_minutes,
                            );
                            let work_unit = i64::from(emp_attendance_rules.work_unit_minutes);
                            let std_units =
                                i32::try_from((std_mins + work_unit - 1) / work_unit).unwrap_or(0);
                            let ot_units =
                                i32::try_from((ot_mins + work_unit - 1) / work_unit).unwrap_or(0);
                            (std_units, ot_units)
                        })
                        .unwrap_or((calc_result.total_standard_units, calc_result.total_ot_units));

                    // Resolved late/absence grace for this segment row (default 15).
                    let segment_grace = emp_attendance_rules
                        .grace_period_minutes
                        .unwrap_or(crate::domain::policy::defaults::DEFAULT_GRACE_PERIOD_MINUTES);

                    let status = crate::services::attendance_utils::determine_daily_status(
                        &crate::services::attendance_utils::DailyStatusInput::new(
                            actual_punch_in.is_some(),
                            actual_punch_in.map(|p| p.with_timezone(&tz).time()),
                            Some(seg.scheduled_start),
                            is_hol,
                            holiday_entry.is_none_or(crate::models::Holiday::is_paid),
                            true,
                            is_on_leave,
                            Some(segment_grace),
                        ),
                    );

                    let employer_id = employments
                        .iter()
                        .find(|e| e.id() == emp_segs.employment_id())
                        .map(crate::models::Employment::employer_id);
                    let employer_name =
                        employer_id.and_then(|id| employer_name_map.get(&id).cloned());

                    let unit_coeff = f64::from(emp_attendance_rules.work_unit_minutes) / 60.0;

                    reports.push(DailyAttendanceReport {
                        date: *date,
                        employment_id: emp_segs.employment_id(),
                        employee_id,
                        employee_name: emp_segs.full_name().to_string(),
                        employee_id_number: emp_segs.employee_id_number().to_string(),
                        shift_symbol: seg.shift_symbol.clone(),
                        shift_name: seg.shift_name.clone(),
                        scheduled_start: Some(seg.scheduled_start),
                        scheduled_end: Some(seg.scheduled_end),
                        grace_period_minutes: Some(segment_grace),
                        break_duration_minutes: seg.break_duration_minutes,
                        break_start: seg.break_start,
                        break_end: seg.break_end,
                        is_break_auto_deduct: seg.is_break_auto_deduct,
                        actual_punch_in,
                        actual_punch_out,
                        std_work_units: seg_std,
                        ot_work_units: seg_ot,
                        ot_claim_status: ot_status_map
                            .get(&(emp_segs.employment_id(), *date))
                            .cloned(),
                        unit_type: crate::models::enums::TimekeepingUnit::DAY,
                        unit_coefficient_hours: unit_coeff,
                        computed_std_hours: f64::from(seg_std) * unit_coeff,
                        computed_ot_hours: f64::from(seg_ot) * unit_coeff,
                        status,
                        is_holiday: is_hol,
                        is_unscheduled: status
                            == crate::services::attendance_utils::AttendanceStatus::Unscheduled,
                        holiday_name: holiday_entry.map(|h| h.name().to_string()),
                        generation_metadata: None,
                        is_leave_paid,
                        employer_name,
                        timezone: None,
                        // Segment fields
                        segment_id: seg.interval_id,
                        shift_id: Some(seg.shift_id),
                        interval_id: seg.interval_id,
                        interval_name: seg.interval_name.clone(),
                        segment_type: Some(seg.segment_type.clone()),
                        sort_order: Some(i32::try_from(seg_idx).unwrap_or(0)),
                        segment_count: Some(segment_count),
                    });
                }
            }
        }

        // Post-join shift-type filter (O(rows), mirrors the existing employer_id filter).
        let reports = filter_by_shift_type(reports, shift_names);

        Ok(PaginatedResponse {
            data: reports,
            total: usize::try_from(total_employees).unwrap_or(0),
            page,
            limit,
        })
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
        let seg_start_dt = tz
            .from_local_datetime(&date.and_time(segment.scheduled_start))
            .single();
        let seg_end_dt = {
            let raw = tz
                .from_local_datetime(&date.and_time(segment.scheduled_end))
                .single();
            if segment.scheduled_end < segment.scheduled_start {
                raw.map(|dt| dt + chrono::Duration::days(1))
            } else {
                raw
            }
        };

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
        entry_type: &str,
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
            entry_type.to_string(),
            None,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    fn make_test_segment(start_h: u32, start_m: u32, end_h: u32, end_m: u32) -> ResolvedSegment {
        ResolvedSegment {
            shift_id: Uuid::new_v4(),
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
            ot_aggregation_mode: "aggregate".to_string(),
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

        let entry = make_test_entry(punch_in, Some(punch_out), "work");
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

        let entry = make_test_entry(punch_in, Some(punch_out), "work");
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

        let entry = make_test_entry(punch_in, None, "work");
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

        let entry = make_test_entry(punch_in, None, "work");
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

        let entry1 = make_test_entry(e1_in, Some(e1_out), "work");
        let entry2 = make_test_entry(e2_in, Some(e2_out), "work");
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

        let break_entry = make_test_entry(break_in, Some(break_out), "break");
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

        let entry = make_test_entry(punch_in, Some(punch_out), "work");
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
        let entry = make_test_entry(punch_in, None, "work");
        let entries = vec![&entry];

        let matched = ReportService::match_entries_to_segment(&entries, &seg, date, tz);
        assert_eq!(matched.len(), 1, "entry at 23:00 within overnight segment");

        // Entry at 03:00 next day → should also match
        let punch_in = tz
            .with_ymd_and_hms(2026, 6, 16, 3, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let entry = make_test_entry(punch_in, None, "work");
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
        let entry = make_test_entry(punch_in, None, "work");
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
        let entry = make_test_entry(punch_in, None, "work");
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

        let entry1 = make_test_entry(e1_in, Some(e1_out), "work");
        let entry2 = make_test_entry(e2_in, Some(e2_out), "work");
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
}
