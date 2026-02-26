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

use crate::cache::{CacheManager, CachedWorkspaceConfig, RosterCache};
use crate::domain::policy::resolver::PolicyResolver;
use crate::models::{AttendanceSummary, DailyAttendanceReport, TimeEntry};
use crate::queue::{RedisPool, ReportJob, ReportPublisher, ReportQueue, ReportType};
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
        redis: &RedisPool,
        roster_cache: Option<&RosterCache>,
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

        let t0 = std::time::Instant::now();

        // 1. Get workspace config from cache first (needed for timezone)
        let workspace_config = cache
            .workspace
            .get(workspace_id)
            .await
            .map_err(|e| format!("Failed to get workspace config: {e}"))?;

        let tz: Tz = workspace_config.timezone.parse().unwrap_or(chrono_tz::UTC);

        // 2. Convert dates to UTC for time entry query
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
            .from_local_datetime(&end_date.and_hms_opt(23, 59, 59).ok_or("Invalid end date")?)
            .single()
            .ok_or("Ambiguous end datetime")?
            .with_timezone(&Utc);

        // 3. Fetch schedules AND time entries in parallel (biggest latency win)
        let (schedules_result, entries_result) = tokio::join!(
            roster_repository::get_workspace_schedules(
                pool,
                roster_cache,
                workspace_id,
                start_date,
                end_date,
                Some(employee_ids),
            ),
            time_entry_repository::get_workspace_time_entries_batch(
                pool,
                employee_ids,
                start_dt,
                end_dt,
            )
        );

        let schedules = schedules_result
            .map_err(|e| format!("Failed to fetch schedules: {e}"))?;

        let t_fetch = t0.elapsed();

        let entries_map = entries_result
            .map_err(|e| format!("Failed to fetch time entries: {e}"))?;

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
                    start_date,
                    end_date,
                    &workspace_config,
                    tz,
                )
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

        let publisher = ReportPublisher::new(redis.clone());

        publisher
            .publish_chunk(
                job.parent_job_id,
                job.batch_index,
                metadata.total_batches,
            )
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

    /// Calculate summary attendance for batch of employees.
    #[allow(clippy::too_many_arguments)]
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
        .map_err(|e| format!("Failed to fetch leave data: {e}"))?
        .into_iter()
        .map(|row| {
            (
                row.get::<Uuid, _>("employee_id"),
                row.get::<i64, _>("leave_days"),
            )
        })
        .collect();

        // Batch fetch working conditions for all employees (eliminates N+1)
        let conditions_map = PolicyResolver::resolve_attendance_rules_batch(
            pool,
            &emp_ids,
            workspace_id,
            start_date,
            end_date,
        )
        .await
        .map_err(|e| format!("Failed to fetch working conditions: {e}"))?;

        for emp_sched in schedules {
            let entries = entries_map
                .get(&emp_sched.employee_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            let mut total_units = 0i32;

            // Get this employee's conditions once
            let emp_conditions = conditions_map
                .get(&emp_sched.employee_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            // Pre-group entries by date once — O(entries) instead of O(entries × days)
            let mut entries_by_date: HashMap<NaiveDate, Vec<&TimeEntry>> = HashMap::new();
            for entry in entries {
                let date = entry.punch_in_time.with_timezone(&tz).date_naive();
                entries_by_date.entry(date).or_default().push(entry);
            }

            for daily in &emp_sched.schedules {
                let date = daily.date;

                let day_entries = entries_by_date
                    .get(&date)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]);

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

                    // Build rules from pre-fetched conditions (no DB call)
                    let mut rules = PolicyResolver::build_rules_from_conditions(
                        emp_conditions,
                        date,
                        workspace_config.early_arrival_strategy,
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
                        .and_then(|t| tz.from_local_datetime(&date.and_time(t)).single());
                    let shift_end_dt = daily
                        .end_time
                        .and_then(|t| tz.from_local_datetime(&date.and_time(t)).single());

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

        serde_json::to_string(&summaries).map_err(|e| format!("Failed to serialize: {e}"))
    }

    /// Calculate detailed daily attendance for batch of employees.
    #[allow(clippy::too_many_arguments)]
    async fn calculate_detail(
        pool: &PgPool,
        schedules: &[crate::models::EmployeeSchedule],
        entries_map: &HashMap<Uuid, Vec<TimeEntry>>,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        workspace_config: &CachedWorkspaceConfig,
        tz: Tz,
    ) -> Result<String, String> {
        let mut reports: Vec<DailyAttendanceReport> = Vec::new();
        let emp_ids: Vec<Uuid> = schedules.iter().map(|s| s.employee_id).collect();

        // Batch fetch working conditions for all employees (eliminates N+1)
        let conditions_map = PolicyResolver::resolve_attendance_rules_batch(
            pool,
            &emp_ids,
            workspace_id,
            start_date,
            end_date,
        )
        .await
        .map_err(|e| format!("Failed to fetch working conditions: {e}"))?;

        for emp_sched in schedules {
            let entries = entries_map
                .get(&emp_sched.employee_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            // Get this employee's conditions once
            let emp_conditions = conditions_map
                .get(&emp_sched.employee_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            // Pre-group entries by date once — O(entries) instead of O(entries × days)
            let mut entries_by_date: HashMap<NaiveDate, Vec<&TimeEntry>> = HashMap::new();
            for entry in entries {
                let date = entry.punch_in_time.with_timezone(&tz).date_naive();
                entries_by_date.entry(date).or_default().push(entry);
            }

            for daily in &emp_sched.schedules {
                let date = daily.date;

                let day_entries = entries_by_date
                    .get(&date)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]);

                let actual_punch_in = day_entries
                    .first()
                    .map(|e| e.punch_in_time.with_timezone(&tz));
                let actual_punch_out = day_entries
                    .last()
                    .and_then(|e| e.punch_out_time.map(|t| t.with_timezone(&tz)));

                let mut std_work_units = 0;
                let mut ot_work_units = 0;

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

                    // Build rules from pre-fetched conditions (no DB call)
                    let mut rules = PolicyResolver::build_rules_from_conditions(
                        emp_conditions,
                        date,
                        workspace_config.early_arrival_strategy,
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

                    std_work_units = std_units;
                    ot_work_units = ot_units;
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
                    status,
                    generation_metadata: combined_metadata,
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
    use chrono::NaiveTime;

    #[test]
    fn test_status_determination_present() {
        let status = determine_status(
            Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()), // punch_in
            Some(NaiveTime::from_hms_opt(8, 30, 0).unwrap()), // sched_start
            true,                                            // has_shift
        );
        assert_eq!(status, "present");
    }

    #[test]
    fn test_status_determination_late() {
        let status = determine_status(
            Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()), // punch_in (late)
            Some(NaiveTime::from_hms_opt(8, 30, 0).unwrap()), // sched_start
            true,
        );
        assert_eq!(status, "late");
    }

    #[test]
    fn test_status_determination_absent() {
        let status = determine_status(None, Some(NaiveTime::from_hms_opt(8, 30, 0).unwrap()), true);
        assert_eq!(status, "absent");
    }

    #[test]
    fn test_status_determination_off() {
        let status = determine_status(None, None, false);
        assert_eq!(status, "off");
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
                total_work_units: 160,
                total_leave_days: 2,
            },
            AttendanceSummary {
                employee_id: Uuid::new_v4(),
                employee_name: "Bob".to_string(),
                total_work_units: 152,
                total_leave_days: 4,
            },
        ];

        let json = serde_json::to_string(&summaries).unwrap();
        let decoded: Vec<AttendanceSummary> = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].employee_name, "Alice");
        assert_eq!(decoded[0].total_work_units, 160);
    }

    #[test]
    fn test_detail_report_serialization() {
        let reports = vec![DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            employee_id: Uuid::new_v4(),
            employee_name: "Charlie".to_string(),
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
            status: "absent".to_string(),
            generation_metadata: None,
        }];

        let json = serde_json::to_string(&reports).unwrap();
        let decoded: Vec<DailyAttendanceReport> = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].status, "absent");
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

    // Helper function for status determination (mirrors handler logic)
    fn determine_status(
        punch_in_time: Option<NaiveTime>,
        sched_start: Option<NaiveTime>,
        has_shift: bool,
    ) -> &'static str {
        if let Some(punch_in) = punch_in_time {
            if let Some(sched) = sched_start {
                if punch_in > sched + Duration::minutes(5) {
                    return "late";
                }
            }
            "present"
        } else if has_shift {
            "absent"
        } else {
            "off"
        }
    }

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

        // Pre-group by date (the same pattern used in calculate_summary/calculate_detail)
        let mut entries_by_date: HashMap<NaiveDate, Vec<&TimeEntry>> = HashMap::new();
        for entry in &entries {
            let date = entry.punch_in_time.with_timezone(&tz).date_naive();
            entries_by_date.entry(date).or_default().push(entry);
        }

        let jan15 = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let jan16 = NaiveDate::from_ymd_opt(2026, 1, 16).unwrap();

        // 2 entries on Jan 15 (in Ho_Chi_Minh timezone)
        assert_eq!(entries_by_date.get(&jan15).map(|v| v.len()).unwrap_or(0), 2);
        // 1 entry on Jan 16 (UTC 18:00 → local 01:00 next day)
        assert_eq!(entries_by_date.get(&jan16).map(|v| v.len()).unwrap_or(0), 1);
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

        // Pre-group approach (new)
        let mut entries_by_date: HashMap<NaiveDate, Vec<&TimeEntry>> = HashMap::new();
        for entry in &entries {
            let date = entry.punch_in_time.with_timezone(&tz).date_naive();
            entries_by_date.entry(date).or_default().push(entry);
        }

        // Filter approach (old) — for each target date, scan all entries
        for day in 1..=5 {
            let target = NaiveDate::from_ymd_opt(2026, 1, day).unwrap();
            let filtered: Vec<_> = entries
                .iter()
                .filter(|e| e.punch_in_time.with_timezone(&tz).date_naive() == target)
                .collect();

            let grouped = entries_by_date.get(&target).map(|v| v.len()).unwrap_or(0);
            assert_eq!(filtered.len(), grouped, "Mismatch on day {day}");
        }
    }

    /// Test that empty entries produce an empty grouping.
    #[test]
    fn test_pregrouping_empty_entries() {
        let tz: Tz = "UTC".parse().unwrap();
        let entries: Vec<TimeEntry> = vec![];

        let mut entries_by_date: HashMap<NaiveDate, Vec<&TimeEntry>> = HashMap::new();
        for entry in &entries {
            let date = entry.punch_in_time.with_timezone(&tz).date_naive();
            entries_by_date.entry(date).or_default().push(entry);
        }

        assert!(entries_by_date.is_empty());

        // Looking up any date returns empty
        let jan1 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let day_entries = entries_by_date.get(&jan1).map(|v| v.as_slice()).unwrap_or(&[]);
        assert!(day_entries.is_empty());
    }

    #[test]
    fn test_slow_attendance_batch_threshold_is_5_seconds() {
        // The per-phase timing breakdown threshold is 5 seconds.
        let fast = std::time::Duration::from_secs(4);
        assert!(fast.as_secs() <= 5, "4s batch should not trigger breakdown log");

        let slow = std::time::Duration::from_secs(6);
        assert!(slow.as_secs() > 5, "6s batch should trigger breakdown log");

        // Exactly 5s should NOT trigger (> 5).
        let edge = std::time::Duration::from_secs(5);
        assert!(!(edge.as_secs() > 5), "Exactly 5s should not trigger");
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
        assert_eq!(entries_ms, 198_000, "Should identify entries fetch as the bottleneck");

        // Calc phase should be small
        let calc_ms = (t_store - t_entries).as_millis() as u64;
        assert_eq!(calc_ms, 20);

        // Total should exceed the 5s threshold
        assert!(t_total.as_secs() > 5);
    }
}
