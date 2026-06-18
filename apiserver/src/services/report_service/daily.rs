use super::{AttendanceReportArgs, CalculateDayUnitsArgs, ReportService};
use crate::domain::policy::resolver::{PolicyResolver, ShiftPolicyOverrides};
use crate::errors::AppError;
use crate::models::{DailyAttendanceReport, PaginatedResponse};
use crate::services::{AttendanceCalculator, PolicyResolverService};
use crate::utils::i18n::I18nKey;
use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use uuid::Uuid;

impl ReportService {
    /// Shared calculation for a single day's standard/OT units.
    pub(crate) fn calculate_day_units(args: CalculateDayUnitsArgs<'_, '_>) -> (i32, i32) {
        if args.daily.shift_id.is_none() {
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
                .start_time
                .and_then(|t| args.tz.from_local_datetime(&args.date.and_time(t)).single());
            let shift_end_dt = args
                .daily
                .end_time
                .and_then(|t| args.tz.from_local_datetime(&args.date.and_time(t)).single());

            let break_start_dt = args
                .daily
                .break_start
                .and_then(|t| args.tz.from_local_datetime(&args.date.and_time(t)).single());
            let break_end_dt = args
                .daily
                .break_end
                .and_then(|t| args.tz.from_local_datetime(&args.date.and_time(t)).single());

            let (shift_start, shift_end) =
                if let (Some(start), Some(mut end)) = (shift_start_dt, shift_end_dt) {
                    if args.daily.end_time < args.daily.start_time {
                        end += chrono::Duration::days(1);
                    }
                    (Some(start), Some(end))
                } else {
                    (None, None)
                };

            let (break_start, break_end) =
                if let (Some(bs), Some(mut be)) = (break_start_dt, break_end_dt) {
                    if args.daily.break_end < args.daily.break_start {
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
                    scheduled_break_minutes: args.daily.break_duration_minutes.unwrap_or(0),
                    actual_break_minutes,
                    break_start,
                    break_end,
                    work_unit: args.work_unit_minutes,
                    rounding_policy: args.rounding_policy.to_string(),
                    rules: args.rules,
                    ot_ceiling: None,
                    max_early_ot_minutes: args.daily.max_early_ot_minutes,
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

                // Filter by employer_id if provided
                let employments: Vec<_> = if let Some(employer_id) = args.employer_id {
                    employments
                        .into_iter()
                        .filter(|emp| emp.employer_id() == employer_id)
                        .collect()
                } else {
                    employments
                };

                let mut paginated_employment_ids = Vec::new();
                for &employee_id in &emp_ids {
                    let mut emp_employments: Vec<_> = employments
                        .iter()
                        .filter(|e| e.employee_id() == employee_id)
                        .collect();

                    if emp_employments.is_empty() {
                        continue;
                    }

                    let overlapping: Vec<_> = emp_employments
                        .iter()
                        .filter(|emp| {
                            emp.start_date() <= args.end_date
                                && emp.end_date().is_none_or(|end| end >= args.start_date)
                        })
                        .collect();

                    if overlapping.is_empty() {
                        // Fallback to the latest employment
                        emp_employments.sort_by(|a, b| {
                            b.start_date()
                                .cmp(&a.start_date())
                                .then_with(|| b.id().cmp(&a.id()))
                        });
                        if let Some(latest) = emp_employments.first() {
                            paginated_employment_ids.push(latest.id());
                        }
                    } else {
                        for emp in overlapping {
                            paginated_employment_ids.push(emp.id());
                        }
                    }
                }

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

        let schedules = self
            .roster_repo
            .get_workspace_schedules(
                None,
                &auth_token,
                args.start_date,
                args.end_date,
                Some(&paginated_employment_ids),
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
                .get(&emp_sched.employment_id)
                .copied()
                .unwrap_or_else(Uuid::nil);

            let entries = entries_map
                .get(&emp_sched.employment_id)
                .map_or_else(Default::default, std::vec::Vec::as_slice);
            let entries_by_date =
                crate::services::attendance_utils::group_entries_by_date(entries, tz);

            for daily in emp_sched.schedules {
                let date = daily.date;

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
                );

                let overrides = ShiftPolicyOverrides::new(
                    daily.early_punch_in_grace_period_minutes,
                    daily.is_break_auto_deduct,
                    daily.allow_early_ot,
                    emp_attendance_rules.early_arrival_strategy,
                    daily.break_is_overtime.unwrap_or(false),
                );
                let rules = PolicyResolver::apply_shift_overrides(base_rules, &overrides);

                let (std_work_units, ot_work_units) =
                    Self::calculate_day_units(CalculateDayUnitsArgs {
                        day_entries,
                        daily: &daily,
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
                        daily.start_time,
                        is_hol,
                        holiday_entry.is_none_or(crate::models::Holiday::is_paid),
                        daily.shift_id.is_some(),
                        is_on_leave,
                        Some(rules.grace_period_minutes()),
                    ),
                );

                let mut metadata_map = serde_json::Map::new();

                let log_metadata: Vec<_> = day_entries
                    .iter()
                    .filter_map(|e| e.generation_metadata().cloned())
                    .collect();

                if !log_metadata.is_empty() {
                    metadata_map
                        .insert("time_entries".to_string(), serde_json::json!(log_metadata));
                }

                if !daily.traces.is_empty() {
                    metadata_map.insert(
                        "schedule_traces".to_string(),
                        serde_json::json!(daily.traces),
                    );
                }

                let combined_metadata = if metadata_map.is_empty() {
                    None
                } else {
                    Some(serde_json::Value::Object(metadata_map))
                };

                let employer_id = employments
                    .iter()
                    .find(|e| e.id() == emp_sched.employment_id)
                    .map(crate::models::Employment::employer_id);
                let employer_name = employer_id.and_then(|id| employer_name_map.get(&id).cloned());

                reports.push(DailyAttendanceReport {
                    date,
                    employment_id: emp_sched.employment_id,
                    employee_id,
                    employee_name: emp_sched.full_name.clone(),
                    employee_id_number: emp_sched.employee_id_number.clone(),
                    shift_symbol: daily.shift_symbol.clone(),
                    shift_name: daily.shift_name.clone(),
                    scheduled_start: daily.start_time,
                    scheduled_end: daily.end_time,
                    break_duration_minutes: daily.break_duration_minutes,
                    break_start: daily.break_start,
                    break_end: daily.break_end,
                    is_break_auto_deduct: daily.is_break_auto_deduct,
                    actual_punch_in: actual_punch_in.map(|t| t.with_timezone(&Utc)),
                    actual_punch_out: actual_punch_out.map(|t| t.with_timezone(&Utc)),
                    std_work_units,
                    ot_work_units,
                    ot_claim_status: ot_status_map.get(&(emp_sched.employment_id, date)).cloned(),
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
                });
            }
        }

        Ok(PaginatedResponse {
            data: reports,
            total: usize::try_from(total_employees).unwrap_or(0),
            page,
            limit,
        })
    }
}
