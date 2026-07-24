use super::{AttendanceReportArgs, CalculateDayUnitsArgs, ReportService};
use crate::domain::policy::resolver::{PolicyResolver, ShiftPolicyOverrides};
use crate::errors::AppError;
use crate::models::{AttendanceReportSort, AttendanceSummary, PaginatedResponse};
use crate::services::PolicyResolverService;
use crate::utils::i18n::I18nKey;
use chrono_tz::Tz;
use std::collections::HashMap;
use uuid::Uuid;

impl ReportService {
    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn get_workspace_attendance_summary(
        &self,
        args: AttendanceReportArgs,
    ) -> Result<PaginatedResponse<AttendanceSummary>, AppError> {
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

        let sort_desc = matches!(args.sort, Some(AttendanceReportSort::NameDesc));

        let is_computed_sort = matches!(
            args.sort,
            Some(
                AttendanceReportSort::WorkHoursAsc
                    | AttendanceReportSort::WorkHoursDesc
                    | AttendanceReportSort::LeaveDaysAsc
                    | AttendanceReportSort::LeaveDaysDesc
                    | AttendanceReportSort::OtHoursAsc
                    | AttendanceReportSort::OtHoursDesc
                    | AttendanceReportSort::TotalHoursAsc
                    | AttendanceReportSort::TotalHoursDesc
            )
        );

        let (paginated_employees, total_employees) = if is_computed_sort {
            let (_, total) = self
                .employee_repo
                .get_paginated_employee_ids(
                    &crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id),
                    crate::models::EmployeePaginationArgs::new(
                        args.employee_ids.as_ref(),
                        args.department_id,
                        1,
                        1,
                        sort_desc,
                        Some((args.start_date, args.end_date)),
                        args.employee_status.clone(),
                    ),
                )
                .await
                .map_err(|e| {
                    tracing::error!("Error fetching employee count: {}", e);
                    AppError::Internal("Internal error".into())
                })?;

            if total == 0 {
                (Vec::new(), 0)
            } else {
                let (emps, _) = self
                    .employee_repo
                    .get_paginated_employee_ids(
                        &crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id),
                        crate::models::EmployeePaginationArgs::new(
                            args.employee_ids.as_ref(),
                            args.department_id,
                            1,
                            total,
                            sort_desc,
                            Some((args.start_date, args.end_date)),
                            args.employee_status.clone(),
                        ),
                    )
                    .await
                    .map_err(|e| {
                        tracing::error!("Error fetching all employees: {}", e);
                        AppError::Internal("Internal error".into())
                    })?;
                (emps, total)
            }
        } else {
            self.employee_repo
                .get_paginated_employee_ids(
                    &crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id),
                    crate::models::EmployeePaginationArgs::new(
                        args.employee_ids.as_ref(),
                        args.department_id,
                        page,
                        limit,
                        sort_desc,
                        Some((args.start_date, args.end_date)),
                        args.employee_status.clone(),
                    ),
                )
                .await
                .map_err(|e| {
                    tracing::error!("Error fetching paginated employees: {}", e);
                    AppError::Internal("Internal error".into())
                })?
        };

        if paginated_employees.is_empty() {
            return Ok(PaginatedResponse {
                data: Vec::new(),
                total: usize::try_from(total_employees).unwrap_or(0),
                page,
                limit,
            });
        }

        let emp_ids: Vec<Uuid> = paginated_employees.iter().map(|e| e.id).collect();

        // Fetch employments to build maps
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id);
        let employments = self
            .employment_repo
            .get_employments_for_employees(&auth_token, &emp_ids)
            .await?;

        let employments: Vec<_> = employments
            .into_iter()
            .filter(|emp| args.employment_id.is_none_or(|id| emp.id() == id))
            .collect();
        let selection = super::employment_selection::select_report_employments(
            &employments,
            &emp_ids,
            args.start_date,
            args.end_date,
            args.employer_id,
        )?;
        let paginated_employment_ids = selection.employment_ids.clone();

        // Map employment_id -> employee_id
        let mut employment_to_employee = std::collections::HashMap::new();
        for emp in &employments {
            employment_to_employee.insert(emp.id(), emp.employee_id());
        }

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

        let leave_data = self
            .leave_repo
            .get_batch_leave_days(&auth_token, &emp_ids, args.start_date, args.end_date)
            .await?;

        let attendance_rules_map = PolicyResolverService::resolve_attendance_policy_batch(
            &*self.policy_repo,
            &emp_ids,
            &auth_token,
            args.start_date,
        )
        .await?;

        let working_conditions_map = PolicyResolverService::resolve_attendance_rules_batch(
            &*self.employee_repo,
            &emp_ids,
            &auth_token,
            args.start_date,
            args.end_date,
        )
        .await?;
        let condition_type_catalog = PolicyResolverService::load_condition_type_catalog(
            self.condition_type_repo.as_ref(),
            &auth_token,
        )
        .await?;

        let mut employee_totals: HashMap<Uuid, (i32, i32, i32)> = HashMap::new();

        for emp_sched in schedules {
            let employee_id = employment_to_employee
                .get(&emp_sched.employment_id())
                .copied()
                .unwrap_or_else(Uuid::nil);

            let entries = entries_map
                .get(&emp_sched.employment_id())
                .map_or_else(Default::default, std::vec::Vec::as_slice);

            let emp_attendance_rules = attendance_rules_map
                .get(&employee_id)
                .cloned()
                .unwrap_or_else(|| crate::models::AttendanceRules {
                    work_unit_minutes: 30,
                    rounding_policy: "floor".to_string(),
                    early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                    grace_period_minutes: None,
                });
            let mut employee_total_std = 0i32;
            let mut employee_total_ot = 0i32;

            let entries_by_date =
                crate::services::attendance_utils::group_entries_by_date(entries, tz);

            for daily in emp_sched.schedules() {
                let date = daily.date();

                let day_entries = entries_by_date
                    .get(&date)
                    .map_or_else(Default::default, std::vec::Vec::as_slice);

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
                let (std_units, ot_units) = Self::calculate_day_units(CalculateDayUnitsArgs {
                    day_entries,
                    daily,
                    date,
                    tz,
                    employee_id,
                    workspace_id: workspace.id(),
                    rules,
                    work_unit_minutes: emp_attendance_rules.work_unit_minutes,
                    rounding_policy: &emp_attendance_rules.rounding_policy,
                });

                employee_total_std += std_units;
                employee_total_ot += ot_units;
            }

            employee_totals.insert(
                emp_sched.employment_id(),
                (
                    employee_total_std,
                    employee_total_ot,
                    emp_attendance_rules.work_unit_minutes,
                ),
            );
        }

        // Fetch all employers to get names
        let (employers, _) = self
            .employer_repo
            .get_employers(&auth_token, 1, 1000)
            .await?;

        let employer_name_map: HashMap<Uuid, String> =
            employers.into_iter().map(|e| (e.id, e.name)).collect();

        // Create a map from employee_id to EmployeeBasicInfo info so we can lookup full name & employee_id_number
        let employee_map: HashMap<Uuid, &crate::repositories::employee::EmployeeBasicInfo> =
            paginated_employees
                .iter()
                .map(|emp| (emp.id, emp))
                .collect();

        let mut results = Vec::new();

        for group in &selection.groups {
            let Some(emp_info) = employee_map.get(&group.employee_id) else {
                continue;
            };

            let mut total_std_units = 0;
            let mut total_ot_units = 0;
            let mut computed_std_hours = 0.0;
            let mut computed_ot_hours = 0.0;
            let mut representative_work_unit_minutes = 30;

            for employment_id in &group.employment_ids {
                let (std_units, ot_units, work_unit_minutes) = employee_totals
                    .get(employment_id)
                    .copied()
                    .unwrap_or((0, 0, 30));
                let coefficient = f64::from(work_unit_minutes) / 60.0;
                total_std_units += std_units;
                total_ot_units += ot_units;
                computed_std_hours = f64::from(std_units).mul_add(coefficient, computed_std_hours);
                computed_ot_hours = f64::from(ot_units).mul_add(coefficient, computed_ot_hours);
                if *employment_id == group.representative_employment_id {
                    representative_work_unit_minutes = work_unit_minutes;
                }
            }

            results.push(AttendanceSummary {
                employment_id: group.representative_employment_id,
                employee_id: group.employee_id,
                employee_name: emp_info.full_name.clone(),
                employee_id_number: emp_info.employee_id_number.clone(),
                total_std_units,
                total_ot_units,
                total_leave_days: leave_data.get(&group.employee_id).copied().unwrap_or(0),
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: f64::from(representative_work_unit_minutes) / 60.0,
                computed_std_hours,
                computed_ot_hours,
                employer_id: Some(group.employer_id),
                employer_name: employer_name_map.get(&group.employer_id).cloned(),
            });
        }

        let logical_total = results.len();

        if let Some(s) = &args.sort {
            match s {
                AttendanceReportSort::NameAsc | AttendanceReportSort::NameDesc => {}
                AttendanceReportSort::WorkHoursAsc => results.sort_by(|a, b| {
                    (a.total_std_units + a.total_ot_units)
                        .cmp(&(b.total_std_units + b.total_ot_units))
                }),
                AttendanceReportSort::WorkHoursDesc => results.sort_by(|a, b| {
                    (b.total_std_units + b.total_ot_units)
                        .cmp(&(a.total_std_units + a.total_ot_units))
                }),
                AttendanceReportSort::LeaveDaysAsc => {
                    results.sort_by(|a, b| a.total_leave_days.cmp(&b.total_leave_days));
                }
                AttendanceReportSort::LeaveDaysDesc => {
                    results.sort_by(|a, b| b.total_leave_days.cmp(&a.total_leave_days));
                }
                AttendanceReportSort::OtHoursAsc => {
                    results.sort_by(|a, b| a.total_ot_units.cmp(&b.total_ot_units));
                }
                AttendanceReportSort::OtHoursDesc => {
                    results.sort_by(|a, b| b.total_ot_units.cmp(&a.total_ot_units));
                }
                AttendanceReportSort::TotalHoursAsc => results.sort_by(|a, b| {
                    (a.total_std_units + a.total_ot_units)
                        .cmp(&(b.total_std_units + b.total_ot_units))
                }),
                AttendanceReportSort::TotalHoursDesc => results.sort_by(|a, b| {
                    (b.total_std_units + b.total_ot_units)
                        .cmp(&(a.total_std_units + a.total_ot_units))
                }),
            }
        }

        if is_computed_sort {
            let offset = usize::try_from((page - 1) * limit).unwrap_or(0);
            let total = results.len();
            if offset >= total {
                results = Vec::new();
            } else {
                results = results
                    .into_iter()
                    .skip(offset)
                    .take(usize::try_from(limit).unwrap_or(0))
                    .collect();
            }
        }

        Ok(PaginatedResponse {
            data: results,
            total: if is_computed_sort {
                logical_total
            } else {
                usize::try_from(total_employees).unwrap_or(0)
            },
            page,
            limit,
        })
    }
}
