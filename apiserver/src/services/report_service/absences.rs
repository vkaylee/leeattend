use super::ReportService;
use crate::errors::AppError;
use crate::models::UnexcusedAbsencesPoint;
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

impl ReportService {
    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn get_unexcused_absences_stats(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        group_by: &str,
    ) -> Result<Vec<UnexcusedAbsencesPoint>, AppError> {
        use crate::services::attendance_utils::{
            self, determine_daily_status, AttendanceStatus, DailyStatusInput,
        };
        use chrono_tz::Tz;
        use std::collections::HashMap;

        let today = Utc::now().date_naive();

        // 1. Get workspace timezone
        let workspace = self
            .workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("Error fetching workspace: {e}")))?
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::NotFound))?;

        let tz: Tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());

        // 2. Get schedules for ALL active employees (reuses 6-priority chain logic)
        let schedules = self
            .roster_repo
            .get_workspace_schedules(
                None, // Cache is option, query DB directly
                auth_token, start_date, end_date, None, // all employees
            )
            .await
            .map_err(|e| AppError::Internal(format!("Error fetching schedules: {e}")))?;

        // Fetch active employments to map employment_id -> employee_id
        let active_employments = self
            .employment_repo
            .get_active_employments_for_workspace(auth_token)
            .await
            .map_err(|e| AppError::Internal(format!("Error fetching employments: {e}")))?;

        let mut emp_id_to_employee_id = HashMap::new();
        for emp in &active_employments {
            emp_id_to_employee_id.insert(emp.id(), emp.employee_id());
        }

        // 3. Batch-fetch time entries (using shared date_range_to_utc)
        let employment_ids: Vec<Uuid> = schedules.iter().map(|s| s.employment_id).collect();
        let employee_ids: Vec<Uuid> = employment_ids
            .iter()
            .filter_map(|emp_id| emp_id_to_employee_id.get(emp_id).copied())
            .collect();
        let (start_dt, end_dt) = attendance_utils::date_range_to_utc(tz, start_date, end_date)
            .map_err(|_e| AppError::validation(crate::utils::i18n::I18nKey::Validation))?;

        let entries_map = self
            .time_entry_repo
            .get_workspace_time_entries_batch_by_employments(
                auth_token,
                &employment_ids,
                start_dt,
                end_dt,
            )
            .await
            .map_err(|e| AppError::Internal(format!("Error fetching time entries: {e}")))?;

        // 4. Batch-fetch approved leave requests using leave_repo
        let leave_details_map = self
            .leave_repo
            .get_approved_leave_details(auth_token, &employee_ids, start_date, end_date)
            .await?;

        let leave_date_set: std::collections::HashSet<(Uuid, NaiveDate)> =
            leave_details_map.keys().copied().collect();

        // 5. Batch-fetch department info for each employee
        let dept_map = self
            .employee_repo
            .get_employee_departments_at(auth_token, end_date)
            .await?;

        // 6. Determine status per day using attendance_utils::determine_daily_status
        struct EmployeeStats {
            employee_name: String,
            department_id: Option<Uuid>,
            department_name: String,
            unexcused_count: i64,
            total_absent_count: i64,
            consecutive_count: i64,
        }

        let mut emp_stats: HashMap<Uuid, EmployeeStats> = HashMap::new();
        let unassigned_id = Uuid::nil();

        for emp_sched in &schedules {
            let employee_id = emp_id_to_employee_id
                .get(&emp_sched.employment_id)
                .copied()
                .unwrap_or_else(Uuid::nil);

            let entries = entries_map
                .get(&emp_sched.employment_id)
                .cloned()
                .unwrap_or_default();

            // Pre-group entries by date using shared utility
            let entries_by_date = attendance_utils::group_entries_by_date(&entries, tz);

            let (dept_id, dept_name) = dept_map
                .get(&employee_id)
                .cloned()
                .unwrap_or_else(|| (None, "Unassigned".to_string()));

            let mut unexcused = 0i64;
            let mut total_absent = 0i64;
            let mut consecutive = 0i64;
            let mut last_present_date: Option<NaiveDate> = None;

            // Find last present date for consecutive calculation
            for entry in &entries {
                let d = entry.punch_in_time().with_timezone(&tz).date_naive();
                if last_present_date.is_none_or(|lpd| d > lpd) {
                    last_present_date = Some(d);
                }
            }

            for daily in &emp_sched.schedules {
                let date = daily.date;
                if date > today {
                    continue;
                }

                let is_on_leave = leave_date_set.contains(&(employee_id, date));

                let status = determine_daily_status(&DailyStatusInput::new(
                    entries_by_date.contains_key(&date),
                    None, // Not needed for unexcused calculation
                    daily.start_time,
                    daily.is_holiday,
                    false, // Not needed for unexcused calculation
                    daily.shift_id.is_some(),
                    is_on_leave,
                ));

                if status == AttendanceStatus::Absent {
                    total_absent += 1;

                    if !is_on_leave {
                        unexcused += 1;

                        // Consecutive: days after last punch-in without leave
                        let is_after_last_punch = last_present_date.is_none_or(|lpd| date > lpd);
                        if is_after_last_punch {
                            consecutive += 1;
                        }
                    }
                }
            }

            emp_stats.insert(
                emp_sched.employment_id,
                EmployeeStats {
                    employee_name: emp_sched.full_name.clone(),
                    department_id: dept_id,
                    department_name: dept_name,
                    unexcused_count: unexcused,
                    total_absent_count: total_absent,
                    consecutive_count: consecutive,
                },
            );
        }

        // 7. Build result based on group_by
        if group_by == "employee" {
            let mut result: Vec<UnexcusedAbsencesPoint> = emp_stats
                .into_iter()
                .map(|(emp_id, stats)| UnexcusedAbsencesPoint {
                    department_id: stats.department_id.unwrap_or(unassigned_id),
                    department_name: stats.department_name,
                    employee_id: Some(emp_id),
                    employee_name: Some(stats.employee_name),
                    unexcused_count: stats.unexcused_count,
                    consecutive_count: stats.consecutive_count,
                    total_absent_count: stats.total_absent_count,
                    total_leave_count: 0,
                })
                .collect();
            result.sort_by(|a, b| b.unexcused_count.cmp(&a.unexcused_count));
            Ok(result)
        } else {
            // Aggregate by department
            let mut dept_agg: HashMap<(Uuid, String), (i64, i64, i64)> = HashMap::new();
            for stats in emp_stats.values() {
                let key = (
                    stats.department_id.unwrap_or(unassigned_id),
                    stats.department_name.clone(),
                );
                let entry = dept_agg.entry(key).or_insert((0, 0, 0));
                entry.0 += stats.unexcused_count;
                entry.1 += stats.total_absent_count;
                entry.2 += stats.consecutive_count;
            }

            let mut result: Vec<UnexcusedAbsencesPoint> = dept_agg
                .into_iter()
                .map(
                    |((dept_id, dept_name), (unexcused, total_absent, consecutive))| {
                        UnexcusedAbsencesPoint {
                            department_id: dept_id,
                            department_name: dept_name,
                            employee_id: None,
                            employee_name: None,
                            unexcused_count: unexcused,
                            consecutive_count: consecutive,
                            total_absent_count: total_absent,
                            total_leave_count: 0,
                        }
                    },
                )
                .collect();
            result.sort_by(|a, b| b.unexcused_count.cmp(&a.unexcused_count));
            Ok(result)
        }
    }
}
