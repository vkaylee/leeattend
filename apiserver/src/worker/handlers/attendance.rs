//! Attendance Report Handler
//!
//! Processes attendance jobs for a batch of employees.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use tracing::info;
use uuid::Uuid;

use crate::cache::{CacheManager, CachedWorkspaceConfig, RosterCache};
use crate::domain::policy::resolver::PolicyResolver;
use crate::models::{AttendanceSummary, DailyAttendanceReport, TimeEntry};
use crate::queue::{Job, JobPayload, ReportType};
use crate::repositories::{PublisherRepository, QueueRepository};
use crate::services::{AttendanceCalculator, PolicyResolverService};

/// Handler for attendance report jobs.
/// Extracts `ReportPayload` from `Job.payload` and processes
/// summary or detail attendance calculations.
pub struct AttendanceHandler;

impl AttendanceHandler {
    /// Process an attendance job for a batch of employees.
    #[allow(clippy::too_many_lines, clippy::too_many_arguments)]
    pub async fn handle(
        queue: &Arc<dyn QueueRepository>,
        publisher: &Arc<dyn PublisherRepository>,
        cache: &Arc<CacheManager>,
        job: &Job,
        roster_cache: Option<&RosterCache>,
        deps: &crate::worker::WorkerDeps,
    ) -> Result<(), String> {
        let JobPayload::AttendanceReport(payload) = &job.payload else {
            return Err(format!(
                "Expected AttendanceReport payload, got {:?}",
                job.task_type
            ));
        };
        let employee_ids = &payload.employee_ids;
        let start_date = payload.date_range.start;
        let end_date = payload.date_range.end;
        let workspace_id = job.workspace_id;

        info!(
            job_id = %job.job_id,
            employees = employee_ids.len(),
            start = %start_date,
            end = %end_date,
            "Processing attendance batch"
        );

        let t0 = std::time::Instant::now();

        // 1. Get workspace config from cache first (needed for timezone)
        let workspace_config = cache
            .workspace
            .get(workspace_id)
            .await
            .map_err(|e| format!("Failed to get workspace config: {e}"))?;

        let tz: Tz = crate::utils::tz::parse_workspace_tz(&workspace_config.timezone);

        // 2. Convert dates to UTC for time entry query
        let (start_dt, end_dt) =
            crate::services::attendance_utils::date_range_to_utc_checked(tz, start_date, end_date)
                .ok_or("Invalid or ambiguous date range")?;

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);

        // Fetch all employers to get names
        let (employers, _) = deps
            .employer_repo
            .get_employers(&auth_token, 1, 1000)
            .await
            .map_err(|e| format!("Failed to fetch employers: {e}"))?;

        let employer_name_map: HashMap<Uuid, String> = employers
            .into_iter()
            .map(|emp| (emp.id, emp.name))
            .collect();

        // Fetch employments to build active employment mappings
        let employments = deps
            .employment_repo
            .get_employments_for_employees(&auth_token, employee_ids)
            .await
            .map_err(|e| format!("Failed to fetch employments: {e}"))?;

        // Map employment_id -> employee_id
        let mut employment_to_employee = HashMap::new();
        // Map employment_id -> (employer_id, employer_name)
        let mut employment_to_employer = HashMap::new();
        let mut employment_ids = Vec::new();

        for emp in &employments {
            employment_to_employee.insert(emp.id(), emp.employee_id());
            let is_overlapping =
                emp.start_date() <= end_date && emp.end_date().is_none_or(|end| end >= start_date);

            if is_overlapping {
                employment_ids.push(emp.id());
                let employer_id = emp.employer_id();
                let employer_name = employer_name_map.get(&employer_id).cloned();
                employment_to_employer.insert(emp.id(), (employer_id, employer_name));
            }
        }

        if employment_ids.is_empty() {
            for emp in &employments {
                employment_ids.push(emp.id());
                let employer_id = emp.employer_id();
                let employer_name = employer_name_map.get(&employer_id).cloned();
                employment_to_employer.insert(emp.id(), (employer_id, employer_name));
            }
        }

        // 3. Fetch schedules AND time entries sequentially (prevent DB pool exhaustion)
        let schedules_result = deps
            .roster_repo
            .get_workspace_schedules(
                roster_cache,
                &auth_token,
                start_date,
                end_date,
                Some(&employment_ids),
            )
            .await;

        let entries_result = deps
            .time_entry_repo
            .get_workspace_time_entries_batch_by_employments(
                &auth_token,
                &employment_ids,
                start_dt,
                end_dt,
            )
            .await;

        let schedules = schedules_result.map_err(|e| format!("Failed to fetch schedules: {e}"))?;

        let t_fetch = t0.elapsed();

        let entries_map =
            entries_result.map_err(|e| format!("Failed to fetch time entries: {e}"))?;

        // 5. Calculate based on report type
        let policy_repo = deps.policy_repo.as_ref();

        let result_json = match payload.config.report_type {
            ReportType::Summary => {
                Self::calculate_summary(CalculateBatchArgs {
                    schedules: &schedules,
                    entries_map: &entries_map,
                    workspace_id,
                    start_date,
                    end_date,
                    _workspace_config: &workspace_config,
                    tz,
                    policy_repo,
                    employee_repo: deps.employee_repo.as_ref(),
                    leave_repo: deps.leave_request_repo.as_ref(),
                    employee_ids,
                    employment_to_employee: &employment_to_employee,
                    employment_to_employer: &employment_to_employer,
                })
                .await?
            }
            ReportType::Detail => {
                Self::calculate_detail(CalculateBatchArgs {
                    schedules: &schedules,
                    entries_map: &entries_map,
                    workspace_id,
                    start_date,
                    end_date,
                    _workspace_config: &workspace_config,
                    tz,
                    policy_repo,
                    employee_repo: deps.employee_repo.as_ref(),
                    leave_repo: deps.leave_request_repo.as_ref(),
                    employee_ids,
                    employment_to_employee: &employment_to_employee,
                    employment_to_employer: &employment_to_employer,
                })
                .await?
            }
        };

        let t_calc = t0.elapsed();

        // 6. Store result in Redis
        queue
            .store_result(job.parent_job_id, job.batch_index, &result_json)
            .await
            .map_err(|e| format!("Failed to store result: {e}"))?;

        let t_store = t0.elapsed();

        // 7. Publish chunk to Redis Pub/Sub for real-time streaming
        let metadata = queue
            .get_job_metadata(job.parent_job_id)
            .await
            .map_err(|e| format!("Failed to get job metadata: {e}"))?
            .ok_or_else(|| "Job metadata not found".to_string())?;

        publisher
            .publish_chunk(job.parent_job_id, job.batch_index, metadata.total_batches)
            .await
            .map_err(|e| format!("Failed to publish chunk: {e}"))?;

        info!(
            job_id = %job.job_id,
            result_len = result_json.len(),
            "Batch result stored and published"
        );

        // Emit timing breakdown for slow batches
        let t_total = t0.elapsed();
        if t_total.as_secs() > 5 {
            tracing::warn!(
                job_id = %job.job_id,
                employees = employee_ids.len(),
                fetch_parallel_ms = t_fetch.as_millis() as u64,
                calc_ms = t_calc.checked_sub(t_fetch).unwrap_or_default().as_millis() as u64,
                store_ms = t_store.checked_sub(t_calc).unwrap_or_default().as_millis() as u64,
                publish_ms = t_total.checked_sub(t_store).unwrap_or_default().as_millis() as u64,
                result_bytes = result_json.len(),
                total_ms = t_total.as_millis() as u64,
                "Slow attendance batch breakdown"
            );
        }

        Ok(())
    }
}

struct CalculateBatchArgs<'a> {
    schedules: &'a [crate::models::EmployeeSchedule],
    entries_map: &'a HashMap<Uuid, Vec<TimeEntry>>,
    workspace_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    _workspace_config: &'a CachedWorkspaceConfig,
    tz: Tz,
    policy_repo: &'a dyn crate::repositories::AttendancePolicyRepository,
    employee_repo: &'a dyn crate::repositories::EmployeeRepository,
    leave_repo: &'a dyn crate::repositories::leave_request_repository::LeaveRequestRepository,
    employee_ids: &'a [Uuid],
    employment_to_employee: &'a HashMap<Uuid, Uuid>,
    employment_to_employer: &'a HashMap<Uuid, (Uuid, Option<String>)>,
}

impl AttendanceHandler {
    /// Calculate summary attendance for batch of employees.
    #[allow(clippy::too_many_lines)]
    async fn calculate_summary(args: CalculateBatchArgs<'_>) -> Result<String, String> {
        let mut summaries: Vec<AttendanceSummary> = Vec::new();
        let emp_ids = args.employee_ids;

        // Batch fetch leave data
        let leave_data = args
            .leave_repo
            .get_batch_leave_days(
                &crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id),
                emp_ids,
                args.start_date,
                args.end_date,
            )
            .await
            .map_err(|e| format!("Failed to fetch leave data: {e}"))?;

        // Batch fetch working conditions for all employees (eliminates N+1)
        let conditions_map = PolicyResolverService::resolve_attendance_rules_batch(
            args.employee_repo,
            emp_ids,
            args.workspace_id,
            args.start_date,
            args.end_date,
        )
        .await
        .map_err(|e| format!("Failed to fetch working conditions: {e}"))?;

        let attendance_rules_map = PolicyResolverService::resolve_attendance_policy_batch(
            args.policy_repo,
            emp_ids,
            args.workspace_id,
            args.start_date, // Using start_date as proxy for the report period
        )
        .await
        .map_err(|e| format!("Failed to fetch attendance rules: {e}"))?;

        for emp_sched in args.schedules {
            let employee_id = args
                .employment_to_employee
                .get(&emp_sched.employment_id)
                .copied()
                .unwrap_or_else(Uuid::nil);

            let entries = args
                .entries_map
                .get(&emp_sched.employment_id)
                .map_or_else(Default::default, std::vec::Vec::as_slice);
            let mut total_std_units = 0i32;
            let mut total_ot_units = 0i32;

            // Get this employee's conditions once
            let emp_conditions = conditions_map
                .get(&employee_id)
                .map_or_else(Default::default, std::vec::Vec::as_slice);

            let emp_attendance_rules = attendance_rules_map
                .get(&employee_id)
                .cloned()
                .unwrap_or_else(|| crate::models::AttendanceRules {
                    work_unit_minutes: 30,
                    rounding_policy: "floor".to_string(),
                    early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                });

            // Pre-group entries by date once — O(entries) instead of O(entries × days)
            let entries_by_date =
                crate::services::attendance_utils::group_entries_by_date(entries, args.tz);

            for daily in &emp_sched.schedules {
                let date = daily.date;

                let day_entries = entries_by_date
                    .get(&date)
                    .map_or_else(Default::default, std::vec::Vec::as_slice);

                let actual_punch_in = day_entries
                    .first()
                    .map(|e| e.punch_in_time().with_timezone(&args.tz));
                let actual_punch_out = day_entries
                    .last()
                    .and_then(|e| e.punch_out_time().map(|t| t.with_timezone(&args.tz)));

                if let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) {
                    let actual_break_minutes: i32 = day_entries
                        .iter()
                        .filter(|e| e.entry_type() == "break")
                        .map(|e| e.duration_minutes().unwrap_or(0))
                        .sum();

                    // Build rules from pre-fetched conditions (no DB call)
                    let mut rules = PolicyResolver::build_rules_from_conditions(
                        emp_conditions,
                        date,
                        emp_attendance_rules.early_arrival_strategy,
                    );

                    // Override with shift-specific settings
                    if let Some(gp) = daily.early_punch_in_grace_period_minutes {
                        rules.set_grace_period_minutes(gp);
                    }
                    if let Some(bad) = daily.is_break_auto_deduct {
                        rules.set_is_break_auto_deduct(bad);
                    }
                    if let Some(aeo) = daily.allow_early_ot {
                        rules.set_allow_early_ot(aeo);
                    }

                    // Handle shift times
                    let shift_start_dt = daily
                        .start_time
                        .and_then(|t| args.tz.from_local_datetime(&date.and_time(t)).single());
                    let shift_end_dt = daily
                        .end_time
                        .and_then(|t| args.tz.from_local_datetime(&date.and_time(t)).single());

                    let (shift_start, shift_end) =
                        if let (Some(start), Some(mut end)) = (shift_start_dt, shift_end_dt) {
                            if daily.end_time < daily.start_time {
                                end += Duration::days(1);
                            }
                            (Some(start), Some(end))
                        } else {
                            (None, None)
                        };

                    // Build break frame datetimes for Frame Intersection
                    let break_start_dt = daily
                        .break_start
                        .and_then(|t| args.tz.from_local_datetime(&date.and_time(t)).single());
                    let break_end_dt = daily.break_end.and_then(|t| {
                        let mut dt = args.tz.from_local_datetime(&date.and_time(t)).single()?;
                        if daily.break_end < daily.break_start {
                            dt += Duration::days(1);
                        }
                        Some(dt)
                    });

                    let (std_units, ot_units) = AttendanceCalculator::calculate_daily_units(
                        &crate::services::attendance_calculator::DailyCalcInput {
                            punch_in,
                            punch_out,
                            shift_start,
                            shift_end,
                            scheduled_break_minutes: daily.break_duration_minutes.unwrap_or(0),
                            actual_break_minutes,
                            break_start: break_start_dt,
                            break_end: break_end_dt,
                            work_unit: emp_attendance_rules.work_unit_minutes,
                            rounding_policy: emp_attendance_rules.rounding_policy.clone(),
                            rules: rules.clone(),
                            ot_ceiling: None,
                            max_early_ot_minutes: daily.max_early_ot_minutes,
                        },
                    );

                    total_std_units += std_units;
                    total_ot_units += ot_units;
                }
            }

            let (employer_id, employer_name) = args
                .employment_to_employer
                .get(&emp_sched.employment_id)
                .cloned()
                .map(|(id, name)| (Some(id), name))
                .unwrap_or((None, None));

            summaries.push(AttendanceSummary {
                employment_id: emp_sched.employment_id,
                employee_id,
                employee_name: emp_sched.full_name.clone(),
                employee_id_number: emp_sched.employee_id_number.clone(),
                total_std_units,
                total_ot_units,
                total_leave_days: leave_data.get(&employee_id).copied().unwrap_or(0),
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: f64::from(emp_attendance_rules.work_unit_minutes) / 60.0,
                computed_std_hours: f64::from(total_std_units)
                    * (f64::from(emp_attendance_rules.work_unit_minutes) / 60.0),
                computed_ot_hours: f64::from(total_ot_units)
                    * (f64::from(emp_attendance_rules.work_unit_minutes) / 60.0),
                employer_id,
                employer_name,
            });
        }

        serde_json::to_string(&summaries).map_err(|e| format!("Failed to serialize: {e}"))
    }

    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    async fn calculate_detail(args: CalculateBatchArgs<'_>) -> Result<String, String> {
        let mut reports: Vec<DailyAttendanceReport> = Vec::new();
        let emp_ids = args.employee_ids;

        // Batch fetch working conditions for all employees (eliminates N+1)
        let conditions_map = PolicyResolverService::resolve_attendance_rules_batch(
            args.employee_repo,
            emp_ids,
            args.workspace_id,
            args.start_date,
            args.end_date,
        )
        .await
        .map_err(|e| format!("Failed to fetch working conditions: {e}"))?;

        let attendance_rules_map = PolicyResolverService::resolve_attendance_policy_batch(
            args.policy_repo,
            emp_ids,
            args.workspace_id,
            args.start_date, // Using start_date as proxy for the report period
        )
        .await
        .map_err(|e| format!("Failed to fetch attendance rules: {e}"))?;

        let leave_details_map = args
            .leave_repo
            .get_approved_leave_details(
                &crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id),
                emp_ids,
                args.start_date,
                args.end_date,
            )
            .await
            .map_err(|e| format!("Failed to fetch leave details: {e}"))?;

        let leave_date_set: std::collections::HashSet<(Uuid, NaiveDate)> =
            leave_details_map.keys().copied().collect();

        for emp_sched in args.schedules {
            let employee_id = args
                .employment_to_employee
                .get(&emp_sched.employment_id)
                .copied()
                .unwrap_or_else(Uuid::nil);

            let employer_name = args
                .employment_to_employer
                .get(&emp_sched.employment_id)
                .and_then(|(_, name)| name.clone());

            let entries = args
                .entries_map
                .get(&emp_sched.employment_id)
                .map_or_else(Default::default, std::vec::Vec::as_slice);

            // Get this employee's conditions once
            let emp_conditions = conditions_map
                .get(&employee_id)
                .map_or_else(Default::default, std::vec::Vec::as_slice);

            let emp_attendance_rules = attendance_rules_map
                .get(&employee_id)
                .cloned()
                .unwrap_or_else(|| crate::models::AttendanceRules {
                    work_unit_minutes: 30,
                    rounding_policy: "floor".to_string(),
                    early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                });

            // Pre-group entries by date once — O(entries) instead of O(entries × days)
            let entries_by_date =
                crate::services::attendance_utils::group_entries_by_date(entries, args.tz);

            for daily in &emp_sched.schedules {
                let date = daily.date;
                let is_on_leave = leave_date_set.contains(&(employee_id, date));
                let is_leave_paid = if is_on_leave {
                    leave_details_map.get(&(employee_id, date)).copied()
                } else {
                    None
                };

                let day_entries = entries_by_date
                    .get(&date)
                    .map_or_else(Default::default, std::vec::Vec::as_slice);

                let actual_punch_in = day_entries
                    .first()
                    .map(|e| e.punch_in_time().with_timezone(&args.tz));
                let actual_punch_out = day_entries
                    .last()
                    .and_then(|e| e.punch_out_time().map(|t| t.with_timezone(&args.tz)));

                let mut std_work_units = 0;
                let mut ot_work_units = 0;

                // Only calculate hours when a shift exists.
                // Without shift boundaries the calculator treats all punch time
                // as "standard" minutes, inflating the report for unscheduled days.
                if daily.shift_id.is_some() {
                    if let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) {
                        let actual_break_minutes: i32 = day_entries
                            .iter()
                            .filter(|e| e.entry_type() == "break")
                            .map(|e| e.duration_minutes().unwrap_or(0))
                            .sum();

                        let shift_start_dt = daily
                            .start_time
                            .and_then(|t| args.tz.from_local_datetime(&date.and_time(t)).single());
                        let shift_end_dt = daily
                            .end_time
                            .and_then(|t| args.tz.from_local_datetime(&date.and_time(t)).single());

                        // Build rules from pre-fetched conditions (no DB call)
                        let mut rules = PolicyResolver::build_rules_from_conditions(
                            emp_conditions,
                            date,
                            emp_attendance_rules.early_arrival_strategy,
                        );

                        if let Some(gp) = daily.early_punch_in_grace_period_minutes {
                            rules.set_grace_period_minutes(gp);
                        }
                        if let Some(bad) = daily.is_break_auto_deduct {
                            rules.set_is_break_auto_deduct(bad);
                        }
                        if let Some(aeo) = daily.allow_early_ot {
                            rules.set_allow_early_ot(aeo);
                        }
                        if let Some(bio) = daily.break_is_overtime {
                            rules.set_is_break_overtime(bio);
                        }

                        let (shift_start, shift_end) =
                            if let (Some(start), Some(mut end)) = (shift_start_dt, shift_end_dt) {
                                if daily.end_time < daily.start_time {
                                    end += Duration::days(1);
                                }
                                (Some(start), Some(end))
                            } else {
                                (None, None)
                            };

                        // Build break frame datetimes for Frame Intersection
                        let break_start_dt = daily
                            .break_start
                            .and_then(|t| args.tz.from_local_datetime(&date.and_time(t)).single());
                        let break_end_dt = daily.break_end.and_then(|t| {
                            let mut dt = args.tz.from_local_datetime(&date.and_time(t)).single()?;
                            if daily.break_end < daily.break_start {
                                dt += Duration::days(1);
                            }
                            Some(dt)
                        });

                        let (std_units, ot_units) = AttendanceCalculator::calculate_daily_units(
                            &crate::services::attendance_calculator::DailyCalcInput {
                                punch_in,
                                punch_out,
                                shift_start,
                                shift_end,
                                scheduled_break_minutes: daily.break_duration_minutes.unwrap_or(0),
                                actual_break_minutes,
                                break_start: break_start_dt,
                                break_end: break_end_dt,
                                work_unit: emp_attendance_rules.work_unit_minutes,
                                rounding_policy: emp_attendance_rules.rounding_policy.clone(),
                                rules: rules.clone(),
                                ot_ceiling: None,
                                max_early_ot_minutes: daily.max_early_ot_minutes,
                            },
                        );

                        std_work_units = std_units;
                        ot_work_units = ot_units;
                    }
                }

                let status = crate::services::attendance_utils::determine_daily_status(
                    &crate::services::attendance_utils::DailyStatusInput::new(
                        actual_punch_in.is_some(),
                        actual_punch_in.map(|p| p.time()),
                        daily.start_time,
                        daily.is_holiday,
                        daily.is_holiday_paid,
                        daily.shift_id.is_some(),
                        is_on_leave,
                    ),
                );

                // Collect metadata
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
                    ot_claim_status: None,
                    unit_type: crate::models::enums::TimekeepingUnit::DAY,
                    unit_coefficient_hours: f64::from(emp_attendance_rules.work_unit_minutes)
                        / 60.0,
                    computed_std_hours: f64::from(std_work_units)
                        * (f64::from(emp_attendance_rules.work_unit_minutes) / 60.0),
                    computed_ot_hours: f64::from(ot_work_units)
                        * (f64::from(emp_attendance_rules.work_unit_minutes) / 60.0),
                    status,
                    is_holiday: daily.is_holiday,
                    is_unscheduled: status
                        == crate::services::attendance_utils::AttendanceStatus::Unscheduled,
                    holiday_name: daily.holiday_name.clone(),
                    generation_metadata: combined_metadata,
                    is_leave_paid,
                    employer_name: employer_name.clone(),
                });
            }
        }

        serde_json::to_string(&reports).map_err(|e| format!("Failed to serialize: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DailyAttendanceReport;
    use crate::services::attendance_utils::{
        determine_daily_status, AttendanceStatus, DailyStatusInput,
    };
    use chrono::NaiveTime;

    #[test]
    fn test_status_determination_present() {
        let status = determine_daily_status(&DailyStatusInput::new(
            true,
            Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(8, 30, 0).unwrap()),
            false,
            false,
            true,
            false,
        ));
        assert_eq!(status, AttendanceStatus::Present);
        assert_eq!(status.to_string(), "present");
    }

    #[test]
    fn test_status_determination_late() {
        let status = determine_daily_status(&DailyStatusInput::new(
            true,
            Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(8, 30, 0).unwrap()),
            false,
            false,
            true,
            false,
        ));
        assert_eq!(status, AttendanceStatus::Late);
        assert_eq!(status.to_string(), "late");
    }

    #[test]
    fn test_status_determination_absent() {
        let status = determine_daily_status(&DailyStatusInput::new(
            false,
            None,
            Some(NaiveTime::from_hms_opt(8, 30, 0).unwrap()),
            false,
            false,
            true,
            false,
        ));
        assert_eq!(status, AttendanceStatus::Absent);
        assert_eq!(status.to_string(), "absent");
    }

    #[test]
    fn test_status_determination_off() {
        let status = determine_daily_status(&DailyStatusInput::new(
            false, None, None, false, false, false, false,
        ));
        assert_eq!(status, AttendanceStatus::Off);
        assert_eq!(status.to_string(), "off");
    }

    #[test]
    fn test_entries_filtering_by_date() {
        // Simulate filtering entries by date (as done in the handler)
        let entries_dates = [
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        ];

        let target_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let filtered_count = entries_dates.iter().filter(|d| **d == target_date).count();

        assert_eq!(filtered_count, 2);
    }

    #[test]
    fn test_break_minutes_aggregation() {
        let break_entries = [15, 30, 15]; // Minutes
        let total: i32 = break_entries.iter().sum();
        assert_eq!(total, 60);
    }

    #[test]
    fn test_overnight_shift_detection() {
        let start = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(6, 0, 0).unwrap();

        // Overnight: end < start
        assert!(end < start);
    }

    #[test]
    fn test_summary_serialization() {
        let summaries = vec![
            AttendanceSummary {
                employment_id: Uuid::new_v4(),
                employee_id: Uuid::new_v4(),
                employee_name: "Alice".to_string(),
                employee_id_number: String::new(),
                total_std_units: 140,
                total_ot_units: 20,
                total_leave_days: 2,
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: 8.0,
                computed_std_hours: 140.0,
                computed_ot_hours: 20.0,
                employer_id: None,
                employer_name: None,
            },
            AttendanceSummary {
                employment_id: Uuid::new_v4(),
                employee_id: Uuid::new_v4(),
                employee_name: "Bob".to_string(),
                employee_id_number: String::new(),
                total_std_units: 152,
                total_ot_units: 0,
                total_leave_days: 4,
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: 8.0,
                computed_std_hours: 152.0,
                computed_ot_hours: 0.0,
                employer_id: None,
                employer_name: None,
            },
        ];

        let json = serde_json::to_string(&summaries).unwrap();
        let decoded: Vec<AttendanceSummary> = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].employee_name, "Alice");
        assert_eq!(decoded[0].total_std_units, 140);
        assert_eq!(decoded[0].total_ot_units, 20);
    }

    #[test]
    fn test_detail_report_serialization() {
        let reports = vec![DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "Alice".to_string(),
            employee_id_number: "EMP001".to_string(),
            shift_symbol: None,
            shift_name: None,
            scheduled_start: Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
            scheduled_end: Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
            break_duration_minutes: Some(60),
            break_start: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            break_end: Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
            is_break_auto_deduct: Some(true),
            actual_punch_in: None,
            actual_punch_out: None,
            std_work_units: 0,
            ot_work_units: 0,
            ot_claim_status: None,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 0.0,
            computed_ot_hours: 0.0,
            status: crate::services::attendance_utils::AttendanceStatus::Absent,
            is_holiday: false,
            is_unscheduled: false,
            holiday_name: None,
            generation_metadata: None,
            is_leave_paid: None,
            employer_name: None,
        }];

        let json = serde_json::to_string(&reports).unwrap();
        let decoded: Vec<DailyAttendanceReport> = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.len(), 1);
        assert_eq!(
            decoded[0].status,
            crate::services::attendance_utils::AttendanceStatus::Absent
        );
        assert_eq!(decoded[0].std_work_units, 0);
        assert_eq!(decoded[0].ot_work_units, 0);
    }

    #[test]
    fn test_timezone_parsing() {
        let valid_tz: Result<Tz, _> = "Asia/Ho_Chi_Minh".parse();
        assert!(valid_tz.is_ok());

        let invalid_tz: Result<Tz, _> = "Invalid/Timezone".parse();
        assert!(invalid_tz.is_err());

        // Default fallback
        let tz: Tz = "invalid".parse().unwrap_or(chrono_tz::UTC);
        assert_eq!(tz, chrono_tz::UTC);
    }

    // Status determination now uses attendance_utils::determine_daily_status
    // (old local determine_status helper removed in the refactoring)

    /// Test that entries are correctly pre-grouped by date using timezone-aware conversion.
    /// This validates optimization #3: O(entries) pre-grouping instead of O(entries × days) scanning.
    #[test]
    fn test_entries_pregrouping_by_date_with_timezone() {
        use chrono::TimeZone;
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap(); // UTC+7

        // Create entries at specific UTC times that map to different local dates
        // UTC 2026-01-14 20:00 → Ho_Chi_Minh 2026-01-15 03:00
        let entry_jan15_early = TimeEntry::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            chrono::Utc.with_ymd_and_hms(2026, 1, 14, 20, 0, 0).unwrap(),
            None,
            Some(480),
            false,
            "work".to_string(),
            None,
            None,
            chrono::Utc::now(),
            chrono::Utc::now(),
        );
        // UTC 2026-01-15 02:00 → Ho_Chi_Minh 2026-01-15 09:00
        let entry_jan15_morning = TimeEntry::new(
            Uuid::new_v4(),
            entry_jan15_early.employment_id(),
            entry_jan15_early.workspace_id(),
            None,
            chrono::Utc.with_ymd_and_hms(2026, 1, 15, 2, 0, 0).unwrap(),
            None,
            Some(480),
            false,
            "work".to_string(),
            None,
            None,
            chrono::Utc::now(),
            chrono::Utc::now(),
        );
        // UTC 2026-01-15 18:00 → Ho_Chi_Minh 2026-01-16 01:00 (next day!)
        let entry_jan16 = TimeEntry::new(
            Uuid::new_v4(),
            entry_jan15_early.employment_id(),
            entry_jan15_early.workspace_id(),
            None,
            chrono::Utc.with_ymd_and_hms(2026, 1, 15, 18, 0, 0).unwrap(),
            None,
            Some(480),
            false,
            "work".to_string(),
            None,
            None,
            chrono::Utc::now(),
            chrono::Utc::now(),
        );

        let entries = vec![entry_jan15_early, entry_jan15_morning, entry_jan16];

        // Pre-group using shared utility (same as calculate_summary/calculate_detail)
        let entries_by_date =
            crate::services::attendance_utils::group_entries_by_date(&entries, tz);

        let jan15 = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let jan16 = NaiveDate::from_ymd_opt(2026, 1, 16).unwrap();

        // 2 entries on Jan 15 (in Ho_Chi_Minh timezone)
        assert_eq!(
            entries_by_date
                .get(&jan15)
                .map(std::vec::Vec::len)
                .unwrap_or(0),
            2
        );
        // 1 entry on Jan 16 (UTC 18:00 → local 01:00 next day)
        assert_eq!(
            entries_by_date
                .get(&jan16)
                .map(std::vec::Vec::len)
                .unwrap_or(0),
            1
        );
    }

    /// Test that pre-grouping produces the same results as the old filter-per-day approach.
    #[test]
    fn test_pregrouping_matches_filter_approach() {
        use chrono::TimeZone;
        let tz: Tz = "UTC".parse().unwrap();

        let emp_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let entries: Vec<TimeEntry> = (1..=5)
            .map(|day| {
                TimeEntry::new(
                    Uuid::new_v4(),
                    emp_id,
                    ws_id,
                    None,
                    chrono::Utc.with_ymd_and_hms(2026, 1, day, 8, 0, 0).unwrap(),
                    None,
                    Some(480),
                    false,
                    "work".to_string(),
                    None,
                    None,
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                )
            })
            .collect();

        // Pre-group approach using shared utility
        let entries_by_date =
            crate::services::attendance_utils::group_entries_by_date(&entries, tz);

        // Filter approach (old) — for each target date, scan all entries
        for day in 1..=5 {
            let target = NaiveDate::from_ymd_opt(2026, 1, day).unwrap();
            let filtered_count = entries
                .iter()
                .filter(|e| e.punch_in_time().with_timezone(&tz).date_naive() == target)
                .count();

            let grouped = entries_by_date
                .get(&target)
                .map(std::vec::Vec::len)
                .unwrap_or(0);
            assert_eq!(filtered_count, grouped, "Mismatch on day {day}");
        }
    }

    /// Test that empty entries produce an empty grouping.
    #[test]
    fn test_pregrouping_empty_entries() {
        let tz: Tz = "UTC".parse().unwrap();
        let entries: Vec<TimeEntry> = vec![];

        let entries_by_date =
            crate::services::attendance_utils::group_entries_by_date(&entries, tz);

        assert!(entries_by_date.is_empty());

        // Looking up any date returns empty
        let jan1 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let day_entries = entries_by_date
            .get(&jan1)
            .map(std::vec::Vec::as_slice)
            .unwrap_or(&[]);
        assert!(day_entries.is_empty());
    }

    #[test]
    fn test_slow_attendance_batch_threshold_is_5_seconds() {
        // The per-phase timing breakdown threshold is 5 seconds.
        let fast = std::time::Duration::from_secs(4);
        assert!(
            fast.as_secs() <= 5,
            "4s batch should not trigger breakdown log"
        );

        let slow = std::time::Duration::from_secs(6);
        assert!(slow.as_secs() > 5, "6s batch should trigger breakdown log");

        // Exactly 5s should NOT trigger (> 5).
        let edge = std::time::Duration::from_secs(5);
        assert!((edge.as_secs() <= 5), "Exactly 5s should not trigger");
    }

    #[test]
    fn test_phase_timing_subtraction_produces_correct_deltas() {
        // Simulate the phase timing pattern used in attendance.rs:
        // t_schedules, t_entries, t_store, t_total are all elapsed from t0
        let t_schedules = std::time::Duration::from_millis(50);
        let t_entries = std::time::Duration::from_millis(200);
        let t_store = std::time::Duration::from_millis(6_500);
        let t_total = std::time::Duration::from_millis(6_550);

        let schedules_ms = t_schedules.as_millis() as u64;
        let entries_ms = t_entries.checked_sub(t_schedules).unwrap().as_millis() as u64;
        let calc_ms = t_store.checked_sub(t_entries).unwrap().as_millis() as u64;
        let store_publish_ms = t_total.checked_sub(t_store).unwrap().as_millis() as u64;

        assert_eq!(schedules_ms, 50);
        assert_eq!(entries_ms, 150);
        assert_eq!(calc_ms, 6_300);
        assert_eq!(store_publish_ms, 50);

        // Verify phases sum to total
        let sum = schedules_ms + entries_ms + calc_ms + store_publish_ms;
        assert_eq!(sum, t_total.as_millis() as u64);
    }

    #[test]
    fn test_phase_timing_with_dominant_single_phase() {
        // Simulate the scenario from logs: one phase takes most of the time
        // (e.g., time entry query was slow)
        let t_schedules = std::time::Duration::from_millis(30);
        let t_entries = std::time::Duration::from_millis(198_030); // 198s for entries!
        let t_store = std::time::Duration::from_millis(198_050);
        let t_total = std::time::Duration::from_millis(198_100);

        let entries_ms = t_entries.checked_sub(t_schedules).unwrap().as_millis() as u64;
        assert_eq!(
            entries_ms, 198_000,
            "Should identify entries fetch as the bottleneck"
        );

        // Calc phase should be small
        let calc_ms = t_store.checked_sub(t_entries).unwrap().as_millis() as u64;
        assert_eq!(calc_ms, 20);

        // Total should exceed the 5s threshold
        assert!(t_total.as_secs() > 5);
    }

    #[tokio::test]
    async fn test_attendance_handler_with_mocks() {
        use crate::repositories::cache_repository::MockCacheRepository;
        use crate::repositories::publisher_repository::MockPublisherRepository;
        use crate::repositories::queue_repository::MockQueueRepository;
        use crate::repositories::shift_repository::MockShiftRepository;
        use crate::repositories::workspace_repository::MockWorkspaceRepository;
        use crate::worker::test_utils::MockWorkerDepsBuilder;
        use std::collections::HashMap;
        use std::sync::Arc;

        let mut queue = MockQueueRepository::new();
        let mut publisher = MockPublisherRepository::new();
        let mut cache_repo = MockCacheRepository::new();

        // Setup expectations
        queue.expect_store_result().returning(|_, _, _| Ok(()));
        queue.expect_get_job_metadata().returning(|_| {
            Ok(Some(crate::queue::JobMetadata {
                job_id: Uuid::new_v4(),
                requester_id: Uuid::new_v4(),
                total_batches: 1,
                completed_batches: 0,
                failed_batches: 0,
                status: crate::queue::JobStatus::Pending,
                workspace_id: Uuid::new_v4(),
                report_type: Some(crate::queue::ReportType::Summary),
                date_range: None,
                created_at: chrono::Utc::now(),
                completed_at: None,
                error_message: None,
                task_type: None,
                payload: None,
            }))
        });

        publisher.expect_publish_chunk().returning(|_, _, _| Ok(()));

        cache_repo.expect_get_json().returning(|_| {
            Ok(Some(
                r#"{"id":"00000000-0000-0000-0000-000000000000","timezone":"UTC"}"#.to_string(),
            ))
        });

        let workspace_repo = Arc::new(MockWorkspaceRepository::new());
        let shift_repo = Arc::new(MockShiftRepository::new());
        let cache_manager = Arc::new(crate::cache::CacheManager::new(
            Arc::new(cache_repo),
            workspace_repo.clone(),
            shift_repo.clone(),
        ));

        // Use Builder to auto-fill 20 mock repositories
        let mut builder = MockWorkerDepsBuilder::new();

        builder
            .employer_repo
            .expect_get_employers()
            .returning(|_, _, _| Ok((vec![], 0)));

        builder
            .roster_repo
            .expect_get_workspace_schedules()
            .returning(|_, _, _, _, _| Box::pin(async move { Ok(vec![]) }));
        builder
            .time_entry_repo
            .expect_get_workspace_time_entries_batch_by_employments()
            .returning(|_, _, _, _| Ok(HashMap::new()));
        builder
            .leave_request_repo
            .expect_get_batch_leave_days()
            .returning(|_, _, _, _| Box::pin(async move { Ok(HashMap::new()) }));
        builder
            .employee_repo
            .expect_get_working_conditions_batch_by_employments()
            .returning(|_, _, _, _| Ok(Default::default()));
        builder
            .policy_repo
            .expect_get_employee_effective_policies_batch()
            .returning(|_, _, _| Ok(HashMap::new()));

        let deps = builder.build();

        let job = crate::queue::Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            task_type: crate::queue::TaskType::AttendanceReport,
            workspace_id: Uuid::new_v4(),
            payload: crate::queue::JobPayload::AttendanceReport(crate::queue::ReportPayload {
                employee_ids: vec![],
                date_range: crate::queue::DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: crate::queue::ReportConfig {
                    report_type: crate::queue::ReportType::Summary,
                    timezone: "UTC".to_string(),
                    work_unit_minutes: 30,
                    rounding_policy: "floor".to_string(),
                },
            }),
            priority: crate::queue::JobPriority::Normal,
            created_at: chrono::Utc::now(),
            retry_count: 0,
            max_retries: 3,
        };

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let pub_arc: Arc<dyn crate::repositories::PublisherRepository> = Arc::new(publisher);

        let result =
            AttendanceHandler::handle(&queue_arc, &pub_arc, &cache_manager, &job, None, &deps)
                .await;

        assert!(result.is_ok(), "Handler should succeed with mocks");
    }

    #[tokio::test]
    async fn test_attendance_handler_calculate_detail_late_early() {
        use crate::repositories::cache_repository::MockCacheRepository;
        use crate::repositories::publisher_repository::MockPublisherRepository;
        use crate::repositories::queue_repository::MockQueueRepository;
        use crate::repositories::shift_repository::MockShiftRepository;
        use crate::repositories::workspace_repository::MockWorkspaceRepository;
        use crate::worker::test_utils::MockWorkerDepsBuilder;
        use chrono::{NaiveDate, NaiveTime, TimeZone};
        use std::collections::HashMap;
        use std::sync::Arc;

        let mut queue = MockQueueRepository::new();
        let mut publisher = MockPublisherRepository::new();
        let mut cache_repo = MockCacheRepository::new();

        let emp_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();

        queue
            .expect_store_result()
            .returning(move |_, _, result_json| {
                let reports: Vec<DailyAttendanceReport> =
                    serde_json::from_str(result_json).unwrap();
                assert_eq!(reports.len(), 1);
                let report = &reports[0];
                assert_eq!(report.status, AttendanceStatus::Late);
                // 8h shift (480 mins) - 15m late - 15m early = 450 mins. 450 / 30 = 15 units.
                assert_eq!(report.std_work_units, 15);
                assert_eq!(report.ot_work_units, 0);
                Ok(())
            });

        queue.expect_get_job_metadata().returning(|_| {
            Ok(Some(crate::queue::JobMetadata {
                job_id: Uuid::new_v4(),
                requester_id: Uuid::new_v4(),
                total_batches: 1,
                completed_batches: 0,
                failed_batches: 0,
                status: crate::queue::JobStatus::Pending,
                workspace_id: Uuid::new_v4(),
                report_type: Some(crate::queue::ReportType::Detail),
                date_range: None,
                created_at: chrono::Utc::now(),
                completed_at: None,
                error_message: None,
                task_type: None,
                payload: None,
            }))
        });

        publisher.expect_publish_chunk().returning(|_, _, _| Ok(()));
        cache_repo.expect_get_json().returning(|_| {
            Ok(Some(
                r#"{"id":"00000000-0000-0000-0000-000000000000","timezone":"UTC"}"#.to_string(),
            ))
        });

        let workspace_repo = Arc::new(MockWorkspaceRepository::new());
        let shift_repo = Arc::new(MockShiftRepository::new());
        let cache_manager = Arc::new(crate::cache::CacheManager::new(
            Arc::new(cache_repo),
            workspace_repo.clone(),
            shift_repo.clone(),
        ));

        let mut builder = MockWorkerDepsBuilder::new();

        builder
            .employer_repo
            .expect_get_employers()
            .returning(|_, _, _| Ok((vec![], 0)));

        let sched_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let schedule = crate::models::EmployeeSchedule {
            employment_id: emp_id,
            employee_id: Uuid::new_v4(),
            full_name: "Test User".to_string(),
            employee_id_number: "EMP01".to_string(),
            schedules: vec![crate::models::EffectiveSchedule {
                date: sched_date,
                is_holiday: false,
                is_holiday_paid: false,
                holiday_name: None,
                source: "fixed".to_string(),
                shift_name: Some("Day Shift".to_string()),
                shift_id: Some(Uuid::new_v4()),
                shift_symbol: Some("HC".to_string()),
                start_time: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
                end_time: Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
                break_duration_minutes: Some(60),
                break_start: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                break_end: Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
                is_break_auto_deduct: Some(true),
                early_punch_in_grace_period_minutes: Some(0),
                allow_early_ot: Some(false),
                break_is_overtime: Some(false),
                max_early_ot_minutes: None,
                traces: vec![],
            }],
        };

        builder
            .roster_repo
            .expect_get_workspace_schedules()
            .returning(move |_, _, _, _, _| {
                let sched = schedule.clone();
                Box::pin(async move { Ok(vec![sched]) })
            });

        let punch_in = chrono::Utc.with_ymd_and_hms(2026, 1, 15, 8, 15, 0).unwrap();
        let punch_out = chrono::Utc
            .with_ymd_and_hms(2026, 1, 15, 16, 45, 0)
            .unwrap();
        let entry = TimeEntry::new(
            Uuid::new_v4(),
            emp_id,
            ws_id,
            None,
            punch_in,
            Some(punch_out),
            Some(450),
            false,
            "work".to_string(),
            None,
            None,
            chrono::Utc::now(),
            chrono::Utc::now(),
        );

        builder
            .time_entry_repo
            .expect_get_workspace_time_entries_batch_by_employments()
            .returning(move |_, _, _, _| {
                let mut map = HashMap::new();
                map.insert(emp_id, vec![entry.clone()]);
                Ok(map)
            });

        builder
            .leave_request_repo
            .expect_get_batch_leave_days()
            .returning(|_, _, _, _| Box::pin(async move { Ok(HashMap::new()) }));
        builder
            .employee_repo
            .expect_get_working_conditions_batch_by_employments()
            .returning(|_, _, _, _| Ok(Default::default()));
        builder
            .policy_repo
            .expect_get_employee_effective_policies_batch()
            .returning(|_, _, _| Ok(HashMap::new()));

        let deps = builder.build();

        let job = crate::queue::Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            task_type: crate::queue::TaskType::AttendanceReport,
            workspace_id: ws_id,
            payload: crate::queue::JobPayload::AttendanceReport(crate::queue::ReportPayload {
                employee_ids: vec![emp_id],
                date_range: crate::queue::DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                },
                config: crate::queue::ReportConfig {
                    report_type: crate::queue::ReportType::Detail,
                    timezone: "UTC".to_string(),
                    work_unit_minutes: 30,
                    rounding_policy: "floor".to_string(),
                },
            }),
            priority: crate::queue::JobPriority::Normal,
            created_at: chrono::Utc::now(),
            retry_count: 0,
            max_retries: 3,
        };

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let pub_arc: Arc<dyn crate::repositories::PublisherRepository> = Arc::new(publisher);

        let result =
            AttendanceHandler::handle(&queue_arc, &pub_arc, &cache_manager, &job, None, &deps)
                .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_attendance_handler_calculate_summary_with_data() {
        use crate::repositories::cache_repository::MockCacheRepository;
        use crate::repositories::publisher_repository::MockPublisherRepository;
        use crate::repositories::queue_repository::MockQueueRepository;
        use crate::repositories::shift_repository::MockShiftRepository;
        use crate::repositories::workspace_repository::MockWorkspaceRepository;
        use crate::worker::test_utils::MockWorkerDepsBuilder;
        use chrono::{NaiveDate, NaiveTime, TimeZone};
        use std::collections::HashMap;
        use std::sync::Arc;

        let mut queue = MockQueueRepository::new();
        let mut publisher = MockPublisherRepository::new();
        let mut cache_repo = MockCacheRepository::new();

        let emp_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();

        queue
            .expect_store_result()
            .returning(move |_, _, result_json| {
                let summaries: Vec<AttendanceSummary> = serde_json::from_str(result_json).unwrap();
                assert_eq!(summaries.len(), 1);
                let summary = &summaries[0];

                // 8:00 - 17:00 shift. 60m break. 540m total -> std=540-60 = 480m => 16 units (30m each)
                // Punch in 8:00, punch out 18:30 -> OT 90m => 3 units
                assert_eq!(summary.total_std_units, 16);
                assert_eq!(summary.total_ot_units, 3);
                assert_eq!(summary.total_leave_days, 2);
                Ok(())
            });

        queue.expect_get_job_metadata().returning(|_| {
            Ok(Some(crate::queue::JobMetadata {
                job_id: Uuid::new_v4(),
                requester_id: Uuid::new_v4(),
                total_batches: 1,
                completed_batches: 0,
                failed_batches: 0,
                status: crate::queue::JobStatus::Pending,
                workspace_id: Uuid::new_v4(),
                report_type: Some(crate::queue::ReportType::Summary),
                date_range: None,
                created_at: chrono::Utc::now(),
                completed_at: None,
                error_message: None,
                task_type: None,
                payload: None,
            }))
        });

        publisher.expect_publish_chunk().returning(|_, _, _| Ok(()));
        cache_repo.expect_get_json().returning(|_| {
            Ok(Some(
                r#"{"id":"00000000-0000-0000-0000-000000000000","timezone":"UTC"}"#.to_string(),
            ))
        });

        let workspace_repo = Arc::new(MockWorkspaceRepository::new());
        let shift_repo = Arc::new(MockShiftRepository::new());
        let cache_manager = Arc::new(crate::cache::CacheManager::new(
            Arc::new(cache_repo),
            workspace_repo.clone(),
            shift_repo.clone(),
        ));

        let mut builder = MockWorkerDepsBuilder::new();

        builder
            .employer_repo
            .expect_get_employers()
            .returning(|_, _, _| Ok((vec![], 0)));

        let sched_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let schedule = crate::models::EmployeeSchedule {
            employment_id: emp_id,
            employee_id: Uuid::new_v4(),
            full_name: "Test User".to_string(),
            employee_id_number: "EMP01".to_string(),
            schedules: vec![crate::models::EffectiveSchedule {
                date: sched_date,
                is_holiday: false,
                is_holiday_paid: false,
                holiday_name: None,
                source: "fixed".to_string(),
                shift_name: Some("Day Shift".to_string()),
                shift_id: Some(Uuid::new_v4()),
                shift_symbol: Some("HC".to_string()),
                start_time: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
                end_time: Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
                break_duration_minutes: Some(60),
                break_start: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                break_end: Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
                is_break_auto_deduct: Some(true),
                early_punch_in_grace_period_minutes: Some(0),
                allow_early_ot: Some(false),
                break_is_overtime: Some(false),
                max_early_ot_minutes: None,
                traces: vec![],
            }],
        };

        builder
            .roster_repo
            .expect_get_workspace_schedules()
            .returning(move |_, _, _, _, _| {
                let sched = schedule.clone();
                Box::pin(async move { Ok(vec![sched]) })
            });

        let punch_in = chrono::Utc.with_ymd_and_hms(2026, 1, 15, 8, 0, 0).unwrap();
        let punch_out = chrono::Utc
            .with_ymd_and_hms(2026, 1, 15, 18, 30, 0)
            .unwrap();
        let entry = TimeEntry::new(
            Uuid::new_v4(),
            emp_id,
            ws_id,
            None,
            punch_in,
            Some(punch_out),
            Some(570),
            false,
            "work".to_string(),
            None,
            None,
            chrono::Utc::now(),
            chrono::Utc::now(),
        );

        builder
            .time_entry_repo
            .expect_get_workspace_time_entries_batch_by_employments()
            .returning(move |_, _, _, _| {
                let mut map = HashMap::new();
                map.insert(emp_id, vec![entry.clone()]);
                Ok(map)
            });

        builder
            .leave_request_repo
            .expect_get_batch_leave_days()
            .returning(move |_, _, _, _| {
                let mut map = HashMap::new();
                map.insert(emp_id, 2);
                Box::pin(async move { Ok(map) })
            });

        builder
            .employee_repo
            .expect_get_working_conditions_batch_by_employments()
            .returning(|_, _, _, _| Ok(Default::default()));
        builder
            .policy_repo
            .expect_get_employee_effective_policies_batch()
            .returning(|_, _, _| Ok(HashMap::new()));

        let deps = builder.build();

        let job = crate::queue::Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            task_type: crate::queue::TaskType::AttendanceReport,
            workspace_id: ws_id,
            payload: crate::queue::JobPayload::AttendanceReport(crate::queue::ReportPayload {
                employee_ids: vec![emp_id],
                date_range: crate::queue::DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                },
                config: crate::queue::ReportConfig {
                    report_type: crate::queue::ReportType::Summary,
                    timezone: "UTC".to_string(),
                    work_unit_minutes: 30,
                    rounding_policy: "floor".to_string(),
                },
            }),
            priority: crate::queue::JobPriority::Normal,
            created_at: chrono::Utc::now(),
            retry_count: 0,
            max_retries: 3,
        };

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let pub_arc: Arc<dyn crate::repositories::PublisherRepository> = Arc::new(publisher);

        let result =
            AttendanceHandler::handle(&queue_arc, &pub_arc, &cache_manager, &job, None, &deps)
                .await;

        assert!(result.is_ok());
    }
}
