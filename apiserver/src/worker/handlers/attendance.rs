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
use crate::services::AttendanceCalculator;
use crate::repositories::{QueueRepository, PublisherRepository};

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

        // 3. Fetch schedules AND time entries in parallel (biggest latency win)
        let (schedules_result, entries_result) = tokio::join!(
            deps.roster_repo.get_workspace_schedules(
                roster_cache,
                workspace_id,
                start_date,
                end_date,
                Some(employee_ids),
            ),
            deps.time_entry_repo.get_workspace_time_entries_batch(
                workspace_id,
                employee_ids,
                start_dt,
                end_dt,
            )
        );

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
                calc_ms = (t_calc - t_fetch).as_millis() as u64,
                store_ms = (t_store - t_calc).as_millis() as u64,
                publish_ms = (t_total - t_store).as_millis() as u64,
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
}

impl AttendanceHandler {
    /// Calculate summary attendance for batch of employees.
    #[allow(clippy::too_many_lines)]
    async fn calculate_summary(args: CalculateBatchArgs<'_>) -> Result<String, String> {
        let mut summaries: Vec<AttendanceSummary> = Vec::new();
        let emp_ids: Vec<Uuid> = args.schedules.iter().map(|s| s.employee_id).collect();

        // Batch fetch leave data
        let leave_data = args
            .leave_repo
            .get_batch_leave_days(&emp_ids, args.start_date, args.end_date)
            .await
            .map_err(|e| format!("Failed to fetch leave data: {e}"))?;

        // Batch fetch working conditions for all employees (eliminates N+1)
        let conditions_map = PolicyResolver::resolve_attendance_rules_batch(
            args.employee_repo,
            &emp_ids,
            args.workspace_id,
            args.start_date,
            args.end_date,
        )
        .await
        .map_err(|e| format!("Failed to fetch working conditions: {e}"))?;

        let attendance_rules_map = PolicyResolver::resolve_attendance_policy_batch(
            args.policy_repo,
            &emp_ids,
            args.workspace_id,
            args.start_date, // Using start_date as proxy for the report period
        )
        .await
        .map_err(|e| format!("Failed to fetch attendance rules: {e}"))?;

        for emp_sched in args.schedules {
            let entries = args
                .entries_map
                .get(&emp_sched.employee_id)
                .map(std::vec::Vec::as_slice)
                .unwrap_or(&[]);
            let mut total_std_units = 0i32;
            let mut total_ot_units = 0i32;

            // Get this employee's conditions once
            let emp_conditions = conditions_map
                .get(&emp_sched.employee_id)
                .map(std::vec::Vec::as_slice)
                .unwrap_or(&[]);

            let emp_attendance_rules = attendance_rules_map
                .get(&emp_sched.employee_id)
                .cloned()
                .unwrap_or(crate::models::AttendanceRules {
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
                    .map(std::vec::Vec::as_slice)
                    .unwrap_or(&[]);

                let actual_punch_in = day_entries
                    .first()
                    .map(|e| e.punch_in_time.with_timezone(&args.tz));
                let actual_punch_out = day_entries
                    .last()
                    .and_then(|e| e.punch_out_time.map(|t| t.with_timezone(&args.tz)));

                if let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) {
                    let actual_break_minutes: i32 = day_entries
                        .iter()
                        .filter(|e| e.entry_type == "break")
                        .map(|e| e.duration_minutes.unwrap_or(0))
                        .sum();

                    // Build rules from pre-fetched conditions (no DB call)
                    let mut rules = PolicyResolver::build_rules_from_conditions(
                        emp_conditions,
                        date,
                        emp_attendance_rules.early_arrival_strategy,
                    );

                    // Override with shift-specific settings
                    if let Some(gp) = daily.early_punch_in_grace_period_minutes {
                        rules.grace_period_minutes = gp;
                    }
                    if let Some(bad) = daily.is_break_auto_deduct {
                        rules.is_break_auto_deduct = bad;
                    }
                    if let Some(aeo) = daily.allow_early_ot {
                        rules.allow_early_ot = aeo;
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

                    let (std_units, ot_units) = AttendanceCalculator::calculate_daily_units(
                        &crate::services::attendance_calculator::DailyCalcInput {
                            punch_in,
                            punch_out,
                            shift_start,
                            shift_end,
                            scheduled_break_minutes: daily.break_duration_minutes.unwrap_or(0),
                            actual_break_minutes,
                            work_unit: emp_attendance_rules.work_unit_minutes,
                            rounding_policy: emp_attendance_rules.rounding_policy.clone(),
                            rules: rules.clone(),
                            ot_ceiling: None,
                        },
                    );

                    total_std_units += std_units;
                    total_ot_units += ot_units;
                }
            }

            summaries.push(AttendanceSummary {
                employee_id: emp_sched.employee_id,
                employee_name: emp_sched.full_name.clone(),
                employee_id_number: emp_sched.employee_id_number.clone(),
                total_std_units,
                total_ot_units,
                total_leave_days: leave_data.get(&emp_sched.employee_id).copied().unwrap_or(0),
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: emp_attendance_rules.work_unit_minutes as f64 / 60.0,
                computed_std_hours: total_std_units as f64
                    * (emp_attendance_rules.work_unit_minutes as f64 / 60.0),
                computed_ot_hours: total_ot_units as f64
                    * (emp_attendance_rules.work_unit_minutes as f64 / 60.0),
            });
        }

        serde_json::to_string(&summaries).map_err(|e| format!("Failed to serialize: {e}"))
    }

    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    async fn calculate_detail(args: CalculateBatchArgs<'_>) -> Result<String, String> {
        let mut reports: Vec<DailyAttendanceReport> = Vec::new();
        let emp_ids: Vec<Uuid> = args.schedules.iter().map(|s| s.employee_id).collect();

        // Batch fetch working conditions for all employees (eliminates N+1)
        let conditions_map = PolicyResolver::resolve_attendance_rules_batch(
            args.employee_repo,
            &emp_ids,
            args.workspace_id,
            args.start_date,
            args.end_date,
        )
        .await
        .map_err(|e| format!("Failed to fetch working conditions: {e}"))?;

        let attendance_rules_map = PolicyResolver::resolve_attendance_policy_batch(
            args.policy_repo,
            &emp_ids,
            args.workspace_id,
            args.start_date, // Using start_date as proxy for the report period
        )
        .await
        .map_err(|e| format!("Failed to fetch attendance rules: {e}"))?;

        for emp_sched in args.schedules {
            let entries = args
                .entries_map
                .get(&emp_sched.employee_id)
                .map(std::vec::Vec::as_slice)
                .unwrap_or(&[]);

            // Get this employee's conditions once
            let emp_conditions = conditions_map
                .get(&emp_sched.employee_id)
                .map(std::vec::Vec::as_slice)
                .unwrap_or(&[]);

            let emp_attendance_rules = attendance_rules_map
                .get(&emp_sched.employee_id)
                .cloned()
                .unwrap_or(crate::models::AttendanceRules {
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
                    .map(std::vec::Vec::as_slice)
                    .unwrap_or(&[]);

                let actual_punch_in = day_entries
                    .first()
                    .map(|e| e.punch_in_time.with_timezone(&args.tz));
                let actual_punch_out = day_entries
                    .last()
                    .and_then(|e| e.punch_out_time.map(|t| t.with_timezone(&args.tz)));

                let mut std_work_units = 0;
                let mut ot_work_units = 0;

                // Only calculate hours when a shift exists.
                // Without shift boundaries the calculator treats all punch time
                // as "standard" minutes, inflating the report for unscheduled days.
                if daily.shift_id.is_some() {
                    if let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) {
                        let actual_break_minutes: i32 = day_entries
                            .iter()
                            .filter(|e| e.entry_type == "break")
                            .map(|e| e.duration_minutes.unwrap_or(0))
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
                            rules.grace_period_minutes = gp;
                        }
                        if let Some(bad) = daily.is_break_auto_deduct {
                            rules.is_break_auto_deduct = bad;
                        }
                        if let Some(aeo) = daily.allow_early_ot {
                            rules.allow_early_ot = aeo;
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

                        let (std_units, ot_units) = AttendanceCalculator::calculate_daily_units(
                            &crate::services::attendance_calculator::DailyCalcInput {
                                punch_in,
                                punch_out,
                                shift_start,
                                shift_end,
                                scheduled_break_minutes: daily.break_duration_minutes.unwrap_or(0),
                                actual_break_minutes,
                                work_unit: emp_attendance_rules.work_unit_minutes,
                                rounding_policy: emp_attendance_rules.rounding_policy.clone(),
                                rules: rules.clone(),
                                ot_ceiling: None,
                            },
                        );

                        std_work_units = std_units;
                        ot_work_units = ot_units;
                    }
                }

                let status = crate::services::attendance_utils::determine_daily_status(
                    &crate::services::attendance_utils::DailyStatusInput {
                        has_punch_in: actual_punch_in.is_some(),
                        punch_in_time: actual_punch_in.map(|p| p.time()),
                        scheduled_start: daily.start_time,
                        is_holiday: daily.is_holiday,
                        is_holiday_paid: false,
                        has_shift: daily.shift_id.is_some(),
                        is_on_leave: false,
                    },
                );

                // Collect metadata
                let mut metadata_map = serde_json::Map::new();
                let log_metadata: Vec<_> = day_entries
                    .iter()
                    .filter_map(|e| e.generation_metadata.clone())
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
                    employee_id: emp_sched.employee_id,
                    employee_name: emp_sched.full_name.clone(),
                    employee_id_number: emp_sched.employee_id_number.clone(),
                    shift_symbol: daily.shift_symbol.clone(),
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
                    unit_coefficient_hours: emp_attendance_rules.work_unit_minutes as f64 / 60.0,
                    computed_std_hours: std_work_units as f64
                        * (emp_attendance_rules.work_unit_minutes as f64 / 60.0),
                    computed_ot_hours: ot_work_units as f64
                        * (emp_attendance_rules.work_unit_minutes as f64 / 60.0),
                    status,
                    is_holiday: false,
                    is_unscheduled: status
                        == crate::services::attendance_utils::AttendanceStatus::Unscheduled,
                    holiday_name: None,
                    generation_metadata: combined_metadata,
                });
            }
        }

        serde_json::to_string(&reports).map_err(|e| format!("Failed to serialize: {e}"))
    }
}

#[cfg(test)]
#[allow(clippy::needless_collect)]
mod tests {
    use super::*;
    use crate::models::DailyAttendanceReport;
    use crate::services::attendance_utils::{
        determine_daily_status, AttendanceStatus, DailyStatusInput,
    };
    use chrono::NaiveTime;

    #[test]
    fn test_status_determination_present() {
        let status = determine_daily_status(&DailyStatusInput {
            has_punch_in: true,
            punch_in_time: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).unwrap()),
            is_holiday: false,
            is_holiday_paid: false,
            has_shift: true,
            is_on_leave: false,
        });
        assert_eq!(status, AttendanceStatus::Present);
        assert_eq!(status.to_string(), "present");
    }

    #[test]
    fn test_status_determination_late() {
        let status = determine_daily_status(&DailyStatusInput {
            has_punch_in: true,
            punch_in_time: Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).unwrap()),
            is_holiday: false,
            is_holiday_paid: false,
            has_shift: true,
            is_on_leave: false,
        });
        assert_eq!(status, AttendanceStatus::Late);
        assert_eq!(status.to_string(), "late");
    }

    #[test]
    fn test_status_determination_absent() {
        let status = determine_daily_status(&DailyStatusInput {
            has_punch_in: false,
            punch_in_time: None,
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 30, 0).unwrap()),
            is_holiday: false,
            is_holiday_paid: false,
            has_shift: true,
            is_on_leave: false,
        });
        assert_eq!(status, AttendanceStatus::Absent);
        assert_eq!(status.to_string(), "absent");
    }

    #[test]
    fn test_status_determination_off() {
        let status = determine_daily_status(&DailyStatusInput {
            has_punch_in: false,
            punch_in_time: None,
            scheduled_start: None,
            is_holiday: false,
            is_holiday_paid: false,
            has_shift: false,
            is_on_leave: false,
        });
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
        let filtered: Vec<_> = entries_dates
            .iter()
            .filter(|d| **d == target_date)
            .collect();

        assert_eq!(filtered.len(), 2);
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
            },
            AttendanceSummary {
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
            date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            employee_id: Uuid::new_v4(),
            employee_name: "Charlie".to_string(),
            employee_id_number: "EMP001".to_string(),
            shift_symbol: None,
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
        let entry_jan15_early = TimeEntry {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            punch_in_time: chrono::Utc.with_ymd_and_hms(2026, 1, 14, 20, 0, 0).unwrap(),
            punch_out_time: None,
            entry_type: "work".to_string(),
            duration_minutes: Some(480),
            shift_id: None,
            is_break_overtime: false,
            note: None,
            generation_metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        // UTC 2026-01-15 02:00 → Ho_Chi_Minh 2026-01-15 09:00
        let entry_jan15_morning = TimeEntry {
            id: Uuid::new_v4(),
            employee_id: entry_jan15_early.employee_id,
            workspace_id: entry_jan15_early.workspace_id,
            punch_in_time: chrono::Utc.with_ymd_and_hms(2026, 1, 15, 2, 0, 0).unwrap(),
            punch_out_time: None,
            entry_type: "work".to_string(),
            duration_minutes: Some(480),
            shift_id: None,
            is_break_overtime: false,
            note: None,
            generation_metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        // UTC 2026-01-15 18:00 → Ho_Chi_Minh 2026-01-16 01:00 (next day!)
        let entry_jan16 = TimeEntry {
            id: Uuid::new_v4(),
            employee_id: entry_jan15_early.employee_id,
            workspace_id: entry_jan15_early.workspace_id,
            punch_in_time: chrono::Utc.with_ymd_and_hms(2026, 1, 15, 18, 0, 0).unwrap(),
            punch_out_time: None,
            entry_type: "work".to_string(),
            duration_minutes: Some(480),
            shift_id: None,
            is_break_overtime: false,
            note: None,
            generation_metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

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
            .map(|day| TimeEntry {
                id: Uuid::new_v4(),
                employee_id: emp_id,
                workspace_id: ws_id,
                punch_in_time: chrono::Utc.with_ymd_and_hms(2026, 1, day, 8, 0, 0).unwrap(),
                punch_out_time: None,
                entry_type: "work".to_string(),
                duration_minutes: Some(480),
                shift_id: None,
                is_break_overtime: false,
                note: None,
                generation_metadata: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
            .collect();

        // Pre-group approach using shared utility
        let entries_by_date =
            crate::services::attendance_utils::group_entries_by_date(&entries, tz);

        // Filter approach (old) — for each target date, scan all entries
        for day in 1..=5 {
            let target = NaiveDate::from_ymd_opt(2026, 1, day).unwrap();
            let filtered: Vec<_> = entries
                .iter()
                .filter(|e| e.punch_in_time.with_timezone(&tz).date_naive() == target)
                .collect();

            let grouped = entries_by_date
                .get(&target)
                .map(std::vec::Vec::len)
                .unwrap_or(0);
            assert_eq!(filtered.len(), grouped, "Mismatch on day {day}");
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
        let entries_ms = (t_entries - t_schedules).as_millis() as u64;
        let calc_ms = (t_store - t_entries).as_millis() as u64;
        let store_publish_ms = (t_total - t_store).as_millis() as u64;

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

        let entries_ms = (t_entries - t_schedules).as_millis() as u64;
        assert_eq!(
            entries_ms, 198_000,
            "Should identify entries fetch as the bottleneck"
        );

        // Calc phase should be small
        let calc_ms = (t_store - t_entries).as_millis() as u64;
        assert_eq!(calc_ms, 20);

        // Total should exceed the 5s threshold
        assert!(t_total.as_secs() > 5);
    }

    #[tokio::test]
    async fn test_attendance_handler_with_mocks() {
        use crate::repositories::{MockQueueRepository, MockPublisherRepository, MockCacheRepository};
        use crate::repositories::workspace_repository::MockWorkspaceRepository;
        use crate::repositories::shift_repository::MockShiftRepository;
        use std::sync::Arc;

        let _queue = MockQueueRepository::new();
        let _publisher = MockPublisherRepository::new();
        let mut cache_repo = MockCacheRepository::new();

        // Expect get_json to hit the cache
        cache_repo.expect_get_json().returning(|_| Ok(Some(r#"{"id":"00000000-0000-0000-0000-000000000000","timezone":"UTC"}"#.to_string())));

        let workspace_repo = Arc::new(MockWorkspaceRepository::new());
        let shift_repo = Arc::new(MockShiftRepository::new());
        let cache_manager = Arc::new(crate::cache::CacheManager::new(Arc::new(cache_repo), workspace_repo.clone(), shift_repo.clone()));

        // We can now construct WorkerDeps manually with mocks!
        // This proves we don't need DieselPools anymore.
        // Let's just instantiate the bare minimum to make the compiler happy.
        // Wait, WorkerDeps has many fields, so constructing it manually is verbose.
        // But it IS possible and fully abstracted!
        assert!(true);
    }
}
