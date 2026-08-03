use super::{AttendanceReportArgs, ReportService};
use crate::errors::AppError;
use crate::models::{AttendanceReportSort, AttendanceSummary, PaginatedResponse};
use chrono_tz::Tz;
use std::collections::HashMap;
use uuid::Uuid;

/// Safety ceiling for computed-sort summary queries.
///
/// When sorting by computed fields (work hours, OT, leave days) the system must
/// load every employee into memory, compute their aggregates, sort in-memory,
/// and then paginate. This constant caps the maximum number of employees that
/// will be fetched to prevent OOM on workspaces with 10 000+ employees.
///
/// When the workspace has more employees than this ceiling, the response is
/// flagged with `truncated: true` so the frontend can warn the user.
const MAX_COMPUTED_SORT_EMPLOYEES: i64 = 1_000;

impl ReportService {
    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn get_workspace_attendance_summary(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        args: AttendanceReportArgs,
    ) -> Result<PaginatedResponse<AttendanceSummary>, AppError> {
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

        // ── Employee resolution ──
        // Computed-sort needs ALL employees for in-memory sorting, so it cannot use
        // fetch_preamble (which paginates at the DB level). Non-computed-sort reuses
        // fetch_preamble to eliminate the duplicated setup code.
        let (paginated_employees, total_employees, preamble) = if is_computed_sort {
            let page = args.page.unwrap_or(1);
            let limit = args.limit.unwrap_or(20);

            // Fetch ALL employees for in-memory sorting — override pagination.
            let all_args = AttendanceReportArgs {
                page: Some(1),
                limit: Some(MAX_COMPUTED_SORT_EMPLOYEES),
                ..args.clone()
            };
            let preamble = self
                .fetch_preamble(auth_token, &all_args, args.employment_id)
                .await?;
            if preamble.paginated_employees().is_empty() {
                return Ok(PaginatedResponse {
                    data: Vec::new(),
                    total: 0,
                    page,
                    limit,
                    truncated: None,
                });
            }

            let total = preamble.total_employees();
            let truncated = total > MAX_COMPUTED_SORT_EMPLOYEES;
            if truncated {
                tracing::warn!(
                    total_employees = %total,
                    ceiling = %MAX_COMPUTED_SORT_EMPLOYEES,
                    workspace_id = %args.workspace_id,
                    "Report summary computed sort: employee count exceeds safety ceiling — results truncated"
                );
            }

            let emp_ids: Vec<Uuid> = preamble
                .paginated_employees()
                .iter()
                .map(|e| e.id)
                .collect();

            let leave_data = self
                .leave_repo
                .get_batch_leave_days(auth_token, &emp_ids, args.start_date, args.end_date)
                .await?;

            let employee_totals = self
                .accumulate_segment_totals(auth_token, &preamble, args.start_date, args.end_date)
                .await?;

            let selection = super::employment_selection::select_report_employments(
                preamble.employments(),
                &emp_ids,
                args.start_date,
                args.end_date,
                args.employer_id,
            )?;

            let results = Self::build_summary_results(
                &selection,
                preamble.paginated_employees(),
                &employee_totals,
                &leave_data,
                preamble.employer_name_map(),
            );

            return Ok(Self::finalize_summary(
                results,
                args.sort,
                is_computed_sort,
                total,
                page,
                limit,
                truncated,
            ));
        } else {
            let preamble = self
                .fetch_preamble(auth_token, &args, args.employment_id)
                .await?;
            if preamble.paginated_employees().is_empty() {
                return Ok(PaginatedResponse {
                    data: Vec::new(),
                    total: usize::try_from(preamble.total_employees()).unwrap_or(0),
                    page: i64::try_from(preamble.page()).unwrap_or(1),
                    limit: i64::try_from(preamble.limit()).unwrap_or(20),
                    truncated: None,
                });
            }
            (
                preamble.paginated_employees().to_vec(),
                preamble.total_employees(),
                Some(preamble),
            )
        };

        // Non-computed-sort path: use preamble data.
        // Safety: the else-branch always sets preamble to Some; the empty-employee
        // check above returns early before reaching this point.
        let Some(preamble) = preamble else {
            return Ok(PaginatedResponse {
                data: Vec::new(),
                total: 0,
                page: 1,
                limit: 20,
                truncated: None,
            });
        };
        let page = i64::try_from(preamble.page()).unwrap_or(1);
        let limit = i64::try_from(preamble.limit()).unwrap_or(20);

        let emp_ids: Vec<Uuid> = paginated_employees.iter().map(|e| e.id).collect();

        let selection = super::employment_selection::select_report_employments(
            preamble.employments(),
            &emp_ids,
            args.start_date,
            args.end_date,
            args.employer_id,
        )?;

        let leave_data = self
            .leave_repo
            .get_batch_leave_days(auth_token, &emp_ids, args.start_date, args.end_date)
            .await?;

        let employee_totals = self
            .accumulate_segment_totals(auth_token, &preamble, args.start_date, args.end_date)
            .await?;

        let results = Self::build_summary_results(
            &selection,
            &paginated_employees,
            &employee_totals,
            &leave_data,
            preamble.employer_name_map(),
        );

        Ok(Self::finalize_summary(
            results,
            args.sort,
            is_computed_sort,
            total_employees,
            page,
            limit,
            false,
        ))
    }

    /// Accumulate per-employment (std_units, ot_units, work_unit_minutes) using
    /// the segment-centric computation path. This ensures the summary totals
    /// match the detail drawer, which also uses the segment path.
    async fn accumulate_segment_totals(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        preamble: &super::ReportDataPreamble,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<HashMap<Uuid, (i32, i32, i32)>, AppError> {
        let tz = preamble.workspace_tz();

        let employee_segments = self
            .resolve_employee_segments(auth_token, preamble, start_date, end_date)
            .await?;

        let mut totals: HashMap<Uuid, (i32, i32, i32)> = HashMap::new();

        for emp_segs in &employee_segments {
            let Some(employee_id) = preamble
                .employment_to_employee()
                .get(&emp_segs.employment_id())
                .copied()
            else {
                continue;
            };

            let entries = preamble
                .entries_map()
                .get(&emp_segs.employment_id())
                .map_or_else(Default::default, std::vec::Vec::as_slice);
            let entries_by_date =
                crate::services::attendance_utils::group_entries_by_date(entries, tz);

            let segments_by_date = super::daily::group_segments_by_date(emp_segs.segments());
            let all_segs = emp_segs.segments();

            let att_rules = Self::resolve_attendance_rules(preamble, employee_id);
            let emp_working_conditions = preamble
                .working_conditions_map()
                .get(&employee_id)
                .map_or_else(Default::default, std::vec::Vec::as_slice);

            let mut emp_std = 0i32;
            let mut emp_ot = 0i32;

            for (date, seg_indices) in &segments_by_date {
                let day_segs: Vec<&crate::models::ResolvedSegment> =
                    seg_indices.iter().map(|&i| &all_segs[i]).collect();

                let (_matched, _rules, segment_calc_inputs) =
                    super::daily::build_segment_calc_inputs(
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

                let input_groups: Vec<
                    &[crate::services::attendance_calculator::DailyCalcInput<Tz>],
                > = segment_calc_inputs.iter().map(Vec::as_slice).collect();
                let segment_result =
                    crate::services::AttendanceService::compute_segment_attendance_sessions(
                        &input_groups,
                    );

                // Use the segment calculator's day-level totals. In aggregate
                // mode this preserves the intended rounding after combining
                // multiple sessions; summing `segment_units` would round each
                // session independently and can over/under-count the summary.
                emp_std += segment_result.total_standard_units();
                emp_ot += segment_result.total_ot_units();
            }

            totals.insert(
                emp_segs.employment_id(),
                (emp_std, emp_ot, att_rules.work_unit_minutes),
            );
        }

        Ok(totals)
    }

    /// Build `AttendanceSummary` rows from employment groups and accumulated totals.
    fn build_summary_results(
        selection: &super::employment_selection::EmploymentSelection,
        paginated_employees: &[crate::repositories::employee::EmployeeBasicInfo],
        employee_totals: &HashMap<Uuid, (i32, i32, i32)>,
        leave_data: &HashMap<Uuid, i64>,
        employer_name_map: &HashMap<Uuid, String>,
    ) -> Vec<AttendanceSummary> {
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
        results
    }

    /// Apply sorting and pagination to the summary results.
    fn finalize_summary(
        mut results: Vec<AttendanceSummary>,
        sort: Option<AttendanceReportSort>,
        is_computed_sort: bool,
        total_employees: i64,
        page: i64,
        limit: i64,
        truncated: bool,
    ) -> PaginatedResponse<AttendanceSummary> {
        let logical_total = results.len();

        if let Some(s) = sort {
            match s {
                AttendanceReportSort::NameAsc | AttendanceReportSort::NameDesc => {}
                AttendanceReportSort::WorkHoursAsc | AttendanceReportSort::TotalHoursAsc => {
                    results.sort_by(|a, b| {
                        (a.total_std_units + a.total_ot_units)
                            .cmp(&(b.total_std_units + b.total_ot_units))
                    });
                }
                AttendanceReportSort::WorkHoursDesc | AttendanceReportSort::TotalHoursDesc => {
                    results.sort_by(|a, b| {
                        (b.total_std_units + b.total_ot_units)
                            .cmp(&(a.total_std_units + a.total_ot_units))
                    });
                }
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

        PaginatedResponse {
            data: results,
            total: if is_computed_sort {
                logical_total
            } else {
                usize::try_from(total_employees).unwrap_or(0)
            },
            page,
            limit,
            truncated: if truncated { Some(true) } else { None },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::enums::TimekeepingUnit;

    fn make_summary(employee_id: Uuid, std_units: i32, ot_units: i32, leave_days: i64) -> AttendanceSummary {
        AttendanceSummary {
            employment_id: Uuid::new_v4(),
            employee_id,
            employee_name: format!("Employee {employee_id}"),
            employee_id_number: format!("EMP{:03}", std_units),
            total_std_units: std_units,
            total_ot_units: ot_units,
            total_leave_days: leave_days,
            unit_type: TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: f64::from(std_units) * 8.0,
            computed_ot_hours: f64::from(ot_units) * 8.0,
            employer_id: None,
            employer_name: None,
        }
    }

    #[test]
    fn test_finalize_summary_truncated_true() {
        let eid = Uuid::new_v4();
        let results = vec![make_summary(eid, 10, 2, 1)];
        let resp = ReportService::finalize_summary(
            results,
            Some(AttendanceReportSort::WorkHoursDesc),
            true,  // is_computed_sort
            1500,  // total_employees
            1,     // page
            20,    // limit
            true,  // truncated
        );
        assert_eq!(resp.truncated, Some(true));
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.total, 1);
    }

    #[test]
    fn test_finalize_summary_truncated_false() {
        let eid = Uuid::new_v4();
        let results = vec![make_summary(eid, 10, 2, 1)];
        let resp = ReportService::finalize_summary(
            results,
            None,
            false, // is_computed_sort
            42,    // total_employees
            1,     // page
            20,    // limit
            false, // truncated
        );
        assert_eq!(resp.truncated, None);
        // Non-computed sort: total comes from total_employees param, not results.len()
        assert_eq!(resp.total, 42);
    }

    #[test]
    fn test_finalize_summary_computed_sort_pagination() {
        let eid = Uuid::new_v4();
        // 5 results, page 2 with limit 2 → expect 2 items (3rd and 4th)
        let results: Vec<_> = (0..5)
            .map(|i| make_summary(eid, i * 10, i, i as i64))
            .collect();

        let resp = ReportService::finalize_summary(
            results,
            Some(AttendanceReportSort::WorkHoursAsc),
            true,  // is_computed_sort
            500,   // total_employees
            2,     // page
            2,     // limit
            false, // truncated (under ceiling)
        );

        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.total, 5); // logical total = results.len() pre-pagination
        assert_eq!(resp.page, 2);
        assert_eq!(resp.limit, 2);
        assert_eq!(resp.truncated, None);

        // First result on page 2 should be the 3rd element (index 2), sorted ascending by std+ot
        assert_eq!(resp.data[0].total_std_units, 20);
        assert_eq!(resp.data[1].total_std_units, 30);
    }

    #[test]
    fn test_finalize_summary_offset_out_of_range_returns_empty() {
        let eid = Uuid::new_v4();
        let results = vec![make_summary(eid, 10, 2, 1)];
        let resp = ReportService::finalize_summary(
            results,
            None,
            true, // is_computed_sort
            500,  // total_employees
            10,   // page — way beyond range
            20,   // limit
            false,
        );
        assert!(resp.data.is_empty());
        assert_eq!(resp.total, 1); // logical total still reflects original count
    }

    #[test]
    fn test_finalize_summary_non_computed_sort_no_pagination() {
        let eid = Uuid::new_v4();
        let results = (0..3)
            .map(|i| make_summary(eid, i * 5, 0, 0))
            .collect::<Vec<_>>();

        let resp = ReportService::finalize_summary(
            results,
            Some(AttendanceReportSort::NameAsc),
            false, // non-computed sort
            100,   // total_employees
            1,     // page
            20,    // limit
            false,
        );
        // Non-computed sort: no manual pagination, all 3 results returned
        assert_eq!(resp.data.len(), 3);
        assert_eq!(resp.total, 100); // from total_employees
        assert_eq!(resp.truncated, None);
    }

    #[test]
    fn test_finalize_summary_leave_days_sort() {
        let eid = Uuid::new_v4();
        let results = vec![
            make_summary(eid, 0, 0, 5),  // leave_days = 5
            make_summary(eid, 0, 0, 1),  // leave_days = 1
            make_summary(eid, 0, 0, 3),  // leave_days = 3
        ];

        let resp = ReportService::finalize_summary(
            results,
            Some(AttendanceReportSort::LeaveDaysAsc),
            true,  // is_computed_sort
            500,
            1,
            20,
            false,
        );

        assert_eq!(resp.data[0].total_leave_days, 1);
        assert_eq!(resp.data[1].total_leave_days, 3);
        assert_eq!(resp.data[2].total_leave_days, 5);
    }

    #[test]
    fn test_finalize_summary_ot_hours_sort_desc() {
        let eid = Uuid::new_v4();
        let results = vec![
            make_summary(eid, 0, 2, 0),  // ot = 2
            make_summary(eid, 0, 8, 0),  // ot = 8
            make_summary(eid, 0, 1, 0),  // ot = 1
        ];

        let resp = ReportService::finalize_summary(
            results,
            Some(AttendanceReportSort::OtHoursDesc),
            true,
            500,
            1,
            20,
            false,
        );

        assert_eq!(resp.data[0].total_ot_units, 8);
        assert_eq!(resp.data[1].total_ot_units, 2);
        assert_eq!(resp.data[2].total_ot_units, 1);
    }
}
