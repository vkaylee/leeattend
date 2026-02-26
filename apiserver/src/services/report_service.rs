use axum::http::StatusCode;
use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::policy::resolver::PolicyResolver;
use crate::models::{
    AttendanceReportSort, AttendanceSummary, DailyAttendanceReport, PaginatedResponse, PolicyRules,
};
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::repositories::{
    employee_repository, roster_repository, time_entry_repository, SqlxWorkspaceRepository,
};
use crate::services::AttendanceCalculator;

pub struct ReportService;

impl ReportService {
    pub async fn get_daily_attendance_report(
        pool: &PgPool,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        page: Option<i64>,
        limit: Option<i64>,
        employee_ids: Option<Vec<Uuid>>,
    ) -> Result<PaginatedResponse<DailyAttendanceReport>, StatusCode> {
        // 0. Get Workspace Settings for Timezone & Rounding Logic
        let workspace_repo = SqlxWorkspaceRepository { pool: pool.clone() };
        let workspace = workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(|e| {
                tracing::error!("Error fetching workspace: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::NOT_FOUND)?;

        let tz: Tz = workspace.timezone.parse().unwrap_or(chrono_tz::UTC);

        // 1. PAGINATION AT DATABASE LEVEL
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(20);

        // Daily report always uses ASC sort (no sort parameter exposed)
        let (paginated_employees, total_employees) =
            employee_repository::get_paginated_employee_ids(
                pool,
                workspace_id,
                employee_ids.as_ref(),
                page,
                limit,
                false, // sort_desc = false -> ORDER BY full_name ASC
            )
            .await
            .map_err(|e| {
                tracing::error!("Error fetching paginated employees: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        // Early return if no employees
        if paginated_employees.is_empty() {
            return Ok(PaginatedResponse {
                data: Vec::new(),
                total: total_employees as usize,
                page,
                limit,
            });
        }

        // 2. Get employee IDs for filtered queries
        let emp_ids: Vec<Uuid> = paginated_employees.iter().map(|e| e.id).collect();

        // 3. Get schedules ONLY for paginated employees
        let schedules = roster_repository::get_workspace_schedules(
            pool,
            None,
            workspace_id,
            start_date,
            end_date,
            Some(&emp_ids),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 4. Convert NaiveDate to DateTime<Utc> using Workspace Timezone
        let start_dt = tz
            .from_local_datetime(
                &start_date
                    .and_hms_opt(0, 0, 0)
                    .ok_or(StatusCode::BAD_REQUEST)?,
            )
            .single()
            .ok_or(StatusCode::BAD_REQUEST)?
            .with_timezone(&Utc);

        let end_dt = tz
            .from_local_datetime(
                &end_date
                    .and_hms_opt(23, 59, 59)
                    .ok_or(StatusCode::BAD_REQUEST)?,
            )
            .single()
            .ok_or(StatusCode::BAD_REQUEST)?
            .with_timezone(&Utc);

        // 5. BATCH fetch time entries (eliminates N+1)
        let entries_map = time_entry_repository::get_workspace_time_entries_batch(
            pool, &emp_ids, start_dt, end_dt,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 6. Calculate reports (same logic as before)
        let mut reports = Vec::new();

        for emp_sched in schedules {
            // Get entries from HashMap (O(1) lookup instead of DB query)
            let entries = entries_map
                .get(&emp_sched.employee_id)
                .cloned()
                .unwrap_or_default();

            for daily in emp_sched.schedules {
                let date = daily.date;

                // Find entries for this day in Workspace Local Time
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

                // --- CALCULATION LOGIC ---
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

                    let mut rules = PolicyResolver::resolve_attendance_rules(
                        pool,
                        emp_sched.employee_id,
                        workspace.id,
                        date,
                    )
                    .await
                    .unwrap_or(PolicyRules {
                        grace_period_minutes: daily
                            .early_punch_in_grace_period_minutes
                            .unwrap_or(0),
                        authorized_early_leave_minutes: 0,
                        is_early_leave_paid: false,
                        early_arrival_strategy: workspace.early_arrival_strategy,
                        is_break_auto_deduct: daily.is_break_auto_deduct.unwrap_or(false),
                        is_break_overtime: false,
                        allow_early_ot: daily.allow_early_ot.unwrap_or(false),
                    });

                    // Override rules with shift-specific settings if they exist
                    if let Some(gp) = daily.early_punch_in_grace_period_minutes {
                        rules.grace_period_minutes = gp;
                    }
                    if let Some(bad) = daily.is_break_auto_deduct {
                        rules.is_break_auto_deduct = bad;
                    }
                    if let Some(aeo) = daily.allow_early_ot {
                        rules.allow_early_ot = aeo;
                    }

                    // Handle shift boundaries (with overnight shift support)
                    let (shift_start, shift_end) =
                        if let (Some(start), Some(mut end)) = (shift_start_dt, shift_end_dt) {
                            if daily.end_time < daily.start_time {
                                end += chrono::Duration::days(1);
                            }
                            (Some(start), Some(end))
                        } else {
                            // No shift assigned - calculate as raw duration
                            (None, None)
                        };

                    let (std_units, ot_units) = AttendanceCalculator::calculate_daily_units(
                        punch_in,
                        punch_out,
                        shift_start,
                        shift_end,
                        daily.break_duration_minutes.unwrap_or(0),
                        actual_break_minutes,
                        workspace.work_unit_minutes,
                        &workspace.rounding_policy,
                        &rules,
                    );

                    std_work_units = std_units;
                    ot_work_units = ot_units;
                }
                // -----------------------------

                let mut status = "absent".to_string();
                if actual_punch_in.is_some() {
                    status = "present".to_string();

                    if let (Some(punch_in), Some(sched_start)) = (actual_punch_in, daily.start_time)
                    {
                        if punch_in.time() > sched_start + chrono::Duration::minutes(5) {
                            status = "late".to_string();
                        }
                    }
                } else if daily.shift_id.is_none() {
                    status = "off".to_string();
                }

                // Collect metadata from all entries for this day
                let mut metadata_map = serde_json::Map::new();

                let log_metadata: Vec<_> = day_entries
                    .iter()
                    .filter_map(|e| e.generation_metadata.clone())
                    .collect();

                if !log_metadata.is_empty() {
                    metadata_map
                        .insert("time_entries".to_string(), serde_json::json!(log_metadata));
                }

                // Add schedule traces from effective schedule
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
                    status,
                    generation_metadata: combined_metadata,
                });
            }
        }

        // 7. Return paginated response (NO in-memory skip/take)
        Ok(PaginatedResponse {
            data: reports,
            total: total_employees as usize, // Total from DB pagination
            page,
            limit,
        })
    }

    /// Generates monthly summary attendance report.
    ///
    /// **IMPORTANT**: This method uses the same calculation logic as `get_daily_attendance_report`
    /// to ensure data consistency between Summary and Detail reports.
    ///
    /// Process:
    /// 1. DB-level pagination via get_paginated_employee_ids
    /// 2. Batch fetch schedules and time entries (eliminates N+1)
    /// 3. Apply AttendanceCalculator for each day (same as detail report)
    /// 4. Aggregate daily results per employee
    #[allow(clippy::too_many_arguments)]
    pub async fn get_workspace_attendance_summary(
        pool: &PgPool,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        sort: Option<AttendanceReportSort>,
        page: Option<i64>,
        limit: Option<i64>,
        employee_ids: Option<Vec<Uuid>>,
    ) -> Result<PaginatedResponse<AttendanceSummary>, StatusCode> {
        use sqlx::Row;
        use std::collections::HashMap;

        // 0. Get Workspace Settings
        let workspace_repo = SqlxWorkspaceRepository { pool: pool.clone() };
        let workspace = workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(|e| {
                tracing::error!("Error fetching workspace: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::NOT_FOUND)?;

        let tz: Tz = workspace.timezone.parse().unwrap_or(chrono_tz::UTC);

        // 1. PAGINATION AT DATABASE LEVEL
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(20);

        // Determine sort direction for name sorting at DB level
        // NameDesc -> ORDER BY full_name DESC; all others -> ORDER BY full_name ASC
        let sort_desc = matches!(sort, Some(AttendanceReportSort::NameDesc));

        let (paginated_employees, total_employees) =
            employee_repository::get_paginated_employee_ids(
                pool,
                workspace_id,
                employee_ids.as_ref(),
                page,
                limit,
                sort_desc,
            )
            .await
            .map_err(|e| {
                tracing::error!("Error fetching paginated employees: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        // Early return if no employees
        if paginated_employees.is_empty() {
            return Ok(PaginatedResponse {
                data: Vec::new(),
                total: total_employees as usize,
                page,
                limit,
            });
        }

        // 2. Get employee IDs for filtered queries
        let emp_ids: Vec<Uuid> = paginated_employees.iter().map(|e| e.id).collect();

        // 3. Get schedules ONLY for paginated employees
        let schedules = roster_repository::get_workspace_schedules(
            pool,
            None,
            workspace_id,
            start_date,
            end_date,
            Some(&emp_ids),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 4. Convert dates to UTC for time entry query
        let start_dt = tz
            .from_local_datetime(
                &start_date
                    .and_hms_opt(0, 0, 0)
                    .ok_or(StatusCode::BAD_REQUEST)?,
            )
            .single()
            .ok_or(StatusCode::BAD_REQUEST)?
            .with_timezone(&Utc);

        let end_dt = tz
            .from_local_datetime(
                &end_date
                    .and_hms_opt(23, 59, 59)
                    .ok_or(StatusCode::BAD_REQUEST)?,
            )
            .single()
            .ok_or(StatusCode::BAD_REQUEST)?
            .with_timezone(&Utc);

        // 5. BATCH fetch time entries (eliminates N+1)
        let entries_map = time_entry_repository::get_workspace_time_entries_batch(
            pool, &emp_ids, start_dt, end_dt,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 6. Fetch leave days for paginated employees (batch query)
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
        .map_err(|e| {
            tracing::error!("Error fetching leave data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_iter()
        .map(|row| {
            (
                row.get::<Uuid, _>("employee_id"),
                row.get::<i64, _>("leave_days"),
            )
        })
        .collect();

        // 7. Calculate work units per employee using the SAME logic as daily report
        let mut employee_totals: HashMap<Uuid, i32> = HashMap::new();

        for emp_sched in schedules {
            // Get entries from HashMap (O(1) lookup instead of DB query)
            let entries = entries_map
                .get(&emp_sched.employee_id)
                .cloned()
                .unwrap_or_default();
            let mut employee_total_units = 0i32;

            // Calculate for each day (same logic as get_daily_attendance_report)
            for daily in &emp_sched.schedules {
                let date = daily.date;

                // Filter entries for this day in workspace local time
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

                // Apply calculation only if we have both punch in and out
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

                    // Resolve policy rules (same as daily report)
                    let mut rules = PolicyResolver::resolve_attendance_rules(
                        pool,
                        emp_sched.employee_id,
                        workspace.id,
                        date,
                    )
                    .await
                    .unwrap_or(PolicyRules {
                        grace_period_minutes: daily
                            .early_punch_in_grace_period_minutes
                            .unwrap_or(0),
                        authorized_early_leave_minutes: 0,
                        is_early_leave_paid: false,
                        early_arrival_strategy: workspace.early_arrival_strategy,
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

                    // Handle shift boundaries (with overnight shift support)
                    let (shift_start, shift_end) =
                        if let (Some(start), Some(mut end)) = (shift_start_dt, shift_end_dt) {
                            if daily.end_time < daily.start_time {
                                end += chrono::Duration::days(1);
                            }
                            (Some(start), Some(end))
                        } else {
                            // No shift assigned - calculate as raw duration
                            (None, None)
                        };

                    // Calculate units using AttendanceCalculator
                    let (std_units, ot_units) = AttendanceCalculator::calculate_daily_units(
                        punch_in,
                        punch_out,
                        shift_start,
                        shift_end,
                        daily.break_duration_minutes.unwrap_or(0),
                        actual_break_minutes,
                        workspace.work_unit_minutes,
                        &workspace.rounding_policy,
                        &rules,
                    );

                    employee_total_units += std_units + ot_units;
                }
            }

            employee_totals.insert(emp_sched.employee_id, employee_total_units);
        }

        // 8. Build results from paginated employees (preserve order)
        let mut results: Vec<AttendanceSummary> = paginated_employees
            .iter()
            .map(|emp| AttendanceSummary {
                employee_id: emp.id,
                employee_name: emp.full_name.clone(),
                total_work_units: *employee_totals.get(&emp.id).unwrap_or(&0),
                total_leave_days: leave_data.get(&emp.id).copied().unwrap_or(0),
            })
            .collect();

        // 9. Sort within batch (for calculated fields only)
        // Note: Name sorting (ASC/DESC) is already handled by DB ORDER BY via sort_desc parameter
        // Calculated field sorts (WorkHours, LeaveDays) only affect order within current page
        if let Some(s) = &sort {
            match s {
                AttendanceReportSort::NameAsc | AttendanceReportSort::NameDesc => {
                    // Already sorted from DB - no action needed
                }
                AttendanceReportSort::WorkHoursAsc => {
                    results.sort_by(|a, b| a.total_work_units.cmp(&b.total_work_units))
                }
                AttendanceReportSort::WorkHoursDesc => {
                    results.sort_by(|a, b| b.total_work_units.cmp(&a.total_work_units))
                }
                AttendanceReportSort::LeaveDaysAsc => {
                    results.sort_by(|a, b| a.total_leave_days.cmp(&b.total_leave_days))
                }
                AttendanceReportSort::LeaveDaysDesc => {
                    results.sort_by(|a, b| b.total_leave_days.cmp(&a.total_leave_days))
                }
            }
        }

        // 10. Return response (NO in-memory skip/take - pagination done at DB level)
        Ok(PaginatedResponse {
            data: results,
            total: total_employees as usize,
            page,
            limit,
        })
    }
}
