//! Attendance Report Handler
//!
//! Processes attendance report jobs for a batch of employees.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

use crate::cache::{CacheManager, CachedWorkspaceConfig};
use crate::domain::policy::resolver::PolicyResolver;
use crate::models::{AttendanceSummary, DailyAttendanceReport, PolicyRules, TimeEntry};
use crate::queue::{ReportJob, ReportQueue, ReportType};
use crate::repositories::{roster_repository, time_entry_repository};
use crate::services::AttendanceCalculator;

/// Handler for attendance report jobs.
pub struct AttendanceHandler;

impl AttendanceHandler {
    /// Process an attendance report job for a batch of employees.
    pub async fn handle(
        pool: &PgPool,
        queue: &ReportQueue,
        cache: &Arc<CacheManager>,
        job: &ReportJob,
    ) -> Result<(), String> {
        let employee_ids = &job.employee_ids;
        let start_date = job.date_range.start;
        let end_date = job.date_range.end;
        let workspace_id = job.workspace_id;

        info!(
            job_id = %job.job_id,
            employees = employee_ids.len(),
            start = %start_date,
            end = %end_date,
            "Processing attendance batch"
        );

        // 1. Fetch schedules for this batch's employees
        let schedules = roster_repository::get_workspace_schedules(
            pool,
            workspace_id,
            start_date,
            end_date,
            Some(employee_ids),
        )
        .await
        .map_err(|e| format!("Failed to fetch schedules: {}", e))?;

        // 2. Get workspace config from cache (cache-aside pattern)
        let workspace_config = cache
            .workspace
            .get(workspace_id)
            .await
            .map_err(|e| format!("Failed to get workspace config: {}", e))?;

        let tz: Tz = workspace_config
            .timezone
            .parse()
            .unwrap_or(chrono_tz::UTC);

        // 3. Convert dates to UTC for time entry query
        let start_dt = tz
            .from_local_datetime(
                &start_date
                    .and_hms_opt(0, 0, 0)
                    .ok_or("Invalid start date")?,
            )
            .single()
            .ok_or("Ambiguous start datetime")?
            .with_timezone(&Utc);

        let end_dt = tz
            .from_local_datetime(
                &end_date
                    .and_hms_opt(23, 59, 59)
                    .ok_or("Invalid end date")?,
            )
            .single()
            .ok_or("Ambiguous end datetime")?
            .with_timezone(&Utc);

        // 4. Batch fetch time entries (dynamic data, always from DB)
        let entries_map = time_entry_repository::get_workspace_time_entries_batch(
            pool,
            employee_ids,
            start_dt,
            end_dt,
        )
        .await
        .map_err(|e| format!("Failed to fetch time entries: {}", e))?;

        // 5. Calculate based on report type
        let result_json = match job.config.report_type {
            ReportType::Summary => {
                Self::calculate_summary(
                    pool,
                    &schedules,
                    &entries_map,
                    workspace_id,
                    start_date,
                    end_date,
                    &workspace_config,
                    tz,
                )
                .await?
            }
            ReportType::Detail => {
                Self::calculate_detail(
                    pool,
                    &schedules,
                    &entries_map,
                    workspace_id,
                    &workspace_config,
                    tz,
                )
                .await?
            }
        };

        // 6. Store result in Redis
        queue
            .store_result(job.parent_job_id, job.batch_index, &result_json)
            .await
            .map_err(|e| format!("Failed to store result: {}", e))?;

        info!(
            job_id = %job.job_id,
            result_len = result_json.len(),
            "Batch result stored"
        );

        Ok(())
    }

    /// Calculate summary attendance for batch of employees.
    async fn calculate_summary(
        pool: &PgPool,
        schedules: &[crate::models::EmployeeSchedule],
        entries_map: &HashMap<Uuid, Vec<TimeEntry>>,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        workspace_config: &CachedWorkspaceConfig,
        tz: Tz,
    ) -> Result<String, String> {
        use sqlx::Row;

        let mut summaries: Vec<AttendanceSummary> = Vec::new();
        let emp_ids: Vec<Uuid> = schedules.iter().map(|s| s.employee_id).collect();

        // Batch fetch leave data
        let leave_data: HashMap<Uuid, i64> = sqlx::query(
            r#"
            SELECT
                lr.employee_id,
                SUM(lr.end_date - lr.start_date + 1)::bigint as leave_days
            FROM leave_requests lr
            WHERE lr.employee_id = ANY($1)
              AND lr.status = 'approved'
              AND lr.start_date >= $2::date
              AND lr.end_date <= $3::date
            GROUP BY lr.employee_id
            "#,
        )
        .bind(&emp_ids)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch leave data: {}", e))?
        .into_iter()
        .map(|row| {
            (
                row.get::<Uuid, _>("employee_id"),
                row.get::<i64, _>("leave_days"),
            )
        })
        .collect();

        for emp_sched in schedules {
            let entries = entries_map
                .get(&emp_sched.employee_id)
                .cloned()
                .unwrap_or_default();
            let mut total_units = 0i32;

            for daily in &emp_sched.schedules {
                let date = daily.date;

                let day_entries: Vec<_> = entries
                    .iter()
                    .filter(|e| e.punch_in_time.with_timezone(&tz).date_naive() == date)
                    .collect();

                let actual_punch_in = day_entries
                    .first()
                    .map(|e| e.punch_in_time.with_timezone(&tz));
                let actual_punch_out = day_entries
                    .last()
                    .and_then(|e| e.punch_out_time.map(|t| t.with_timezone(&tz)));

                if let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) {
                    let actual_break_minutes: i32 = day_entries
                        .iter()
                        .filter(|e| e.entry_type == "break")
                        .map(|e| e.duration_minutes.unwrap_or(0))
                        .sum();

                    // Resolve policy rules
                    let mut rules = PolicyResolver::resolve_attendance_rules(
                        pool,
                        emp_sched.employee_id,
                        workspace_id,
                        date,
                    )
                    .await
                    .unwrap_or(PolicyRules {
                        grace_period_minutes: daily
                            .early_punch_in_grace_period_minutes
                            .unwrap_or(0),
                        authorized_early_leave_minutes: 0,
                        is_early_leave_paid: false,
                        early_arrival_strategy: workspace_config.early_arrival_strategy,
                        is_break_auto_deduct: daily.is_break_auto_deduct.unwrap_or(false),
                        is_break_overtime: false,
                        allow_early_ot: daily.allow_early_ot.unwrap_or(false),
                    });

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
                        .and_then(|t| tz.from_local_datetime(&date.and_time(t)).single());
                    let shift_end_dt = daily
                        .end_time
                        .and_then(|t| tz.from_local_datetime(&date.and_time(t)).single());

                    let (shift_start, shift_end) =
                        if let (Some(start), Some(mut end)) = (shift_start_dt, shift_end_dt) {
                            if daily.end_time < daily.start_time {
                                end = end + Duration::days(1);
                            }
                            (Some(start), Some(end))
                        } else {
                            (None, None)
                        };

                    let (std_units, ot_units) = AttendanceCalculator::calculate_daily_units(
                        punch_in,
                        punch_out,
                        shift_start,
                        shift_end,
                        daily.break_duration_minutes.unwrap_or(0),
                        actual_break_minutes,
                        workspace_config.work_unit_minutes,
                        &workspace_config.rounding_policy,
                        &rules,
                    );

                    total_units += std_units + ot_units;
                }
            }

            summaries.push(AttendanceSummary {
                employee_id: emp_sched.employee_id,
                employee_name: emp_sched.full_name.clone(),
                total_work_units: total_units,
                total_leave_days: leave_data.get(&emp_sched.employee_id).copied().unwrap_or(0),
            });
        }

        serde_json::to_string(&summaries).map_err(|e| format!("Failed to serialize: {}", e))
    }

    /// Calculate detailed daily attendance for batch of employees.
    async fn calculate_detail(
        pool: &PgPool,
        schedules: &[crate::models::EmployeeSchedule],
        entries_map: &HashMap<Uuid, Vec<TimeEntry>>,
        workspace_id: Uuid,
        workspace_config: &CachedWorkspaceConfig,
        tz: Tz,
    ) -> Result<String, String> {
        let mut reports: Vec<DailyAttendanceReport> = Vec::new();

        for emp_sched in schedules {
            let entries = entries_map
                .get(&emp_sched.employee_id)
                .cloned()
                .unwrap_or_default();

            for daily in &emp_sched.schedules {
                let date = daily.date;

                let day_entries: Vec<_> = entries
                    .iter()
                    .filter(|e| e.punch_in_time.with_timezone(&tz).date_naive() == date)
                    .collect();

                let actual_punch_in = day_entries
                    .first()
                    .map(|e| e.punch_in_time.with_timezone(&tz));
                let actual_punch_out = day_entries
                    .last()
                    .and_then(|e| e.punch_out_time.map(|t| t.with_timezone(&tz)));

                let mut total_payable_units = 0;

                if let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) {
                    let actual_break_minutes: i32 = day_entries
                        .iter()
                        .filter(|e| e.entry_type == "break")
                        .map(|e| e.duration_minutes.unwrap_or(0))
                        .sum();

                    let shift_start_dt = daily
                        .start_time
                        .and_then(|t| tz.from_local_datetime(&date.and_time(t)).single());
                    let shift_end_dt = daily
                        .end_time
                        .and_then(|t| tz.from_local_datetime(&date.and_time(t)).single());

                    let mut rules = PolicyResolver::resolve_attendance_rules(
                        pool,
                        emp_sched.employee_id,
                        workspace_id,
                        date,
                    )
                    .await
                    .unwrap_or(PolicyRules {
                        grace_period_minutes: daily
                            .early_punch_in_grace_period_minutes
                            .unwrap_or(0),
                        authorized_early_leave_minutes: 0,
                        is_early_leave_paid: false,
                        early_arrival_strategy: workspace_config.early_arrival_strategy,
                        is_break_auto_deduct: daily.is_break_auto_deduct.unwrap_or(false),
                        is_break_overtime: false,
                        allow_early_ot: daily.allow_early_ot.unwrap_or(false),
                    });

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
                                end = end + Duration::days(1);
                            }
                            (Some(start), Some(end))
                        } else {
                            (None, None)
                        };

                    let (std_units, ot_units) = AttendanceCalculator::calculate_daily_units(
                        punch_in,
                        punch_out,
                        shift_start,
                        shift_end,
                        daily.break_duration_minutes.unwrap_or(0),
                        actual_break_minutes,
                        workspace_config.work_unit_minutes,
                        &workspace_config.rounding_policy,
                        &rules,
                    );

                    total_payable_units = std_units + ot_units;
                }

                let mut status = "absent".to_string();
                if actual_punch_in.is_some() {
                    status = "present".to_string();

                    if let (Some(punch_in), Some(sched_start)) = (actual_punch_in, daily.start_time)
                    {
                        if punch_in.time() > sched_start + Duration::minutes(5) {
                            status = "late".to_string();
                        }
                    }
                } else if daily.shift_id.is_none() {
                    status = "off".to_string();
                }

                // Collect metadata
                let mut metadata_map = serde_json::Map::new();
                let log_metadata: Vec<_> = day_entries
                    .iter()
                    .filter_map(|e| e.generation_metadata.clone())
                    .collect();

                if !log_metadata.is_empty() {
                    metadata_map.insert("time_entries".to_string(), serde_json::json!(log_metadata));
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
                    scheduled_start: daily.start_time,
                    scheduled_end: daily.end_time,
                    break_duration_minutes: daily.break_duration_minutes,
                    break_start: daily.break_start,
                    break_end: daily.break_end,
                    is_break_auto_deduct: daily.is_break_auto_deduct,
                    actual_punch_in: actual_punch_in.map(|t| t.with_timezone(&Utc)),
                    actual_punch_out: actual_punch_out.map(|t| t.with_timezone(&Utc)),
                    work_units: total_payable_units,
                    status,
                    generation_metadata: combined_metadata,
                });
            }
        }

        serde_json::to_string(&reports).map_err(|e| format!("Failed to serialize: {}", e))
    }
}
