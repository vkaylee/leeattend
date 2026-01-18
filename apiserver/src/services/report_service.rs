use axum::http::StatusCode;
use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{AttendanceSummary, DailyAttendanceReport};
use crate::repositories::workspace_repository::{SqlxWorkspaceRepository, WorkspaceRepository};
use crate::repositories::{roster_repository, time_entry_repository};
use crate::services::AttendanceCalculator;

pub struct ReportService;

impl ReportService {
    pub async fn get_daily_attendance_report(
        pool: &PgPool,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<DailyAttendanceReport>, StatusCode> {
        // 0. Get Workspace Settings for Timezone & Rounding Logic
        let workspace_repo = SqlxWorkspaceRepository { pool: pool.clone() };
        let workspace = workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        let tz: Tz = workspace.timezone.parse().unwrap_or(chrono_tz::UTC);

        // 1. Get all schedules for the range
        let schedules =
            roster_repository::get_workspace_schedules(pool, workspace_id, start_date, end_date)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 2. Get all time entries for the range
        // Convert NaiveDate to DateTime<Utc> using Workspace Timezone
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

        let mut reports = Vec::new();

        for emp_sched in schedules {
            let entries = time_entry_repository::get_employee_time_entries(
                pool,
                emp_sched.employee_id,
                start_dt,
                end_dt,
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
                let mut total_payable_units = 0;

                if let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) {
                    let actual_break_minutes: i32 = day_entries
                        .iter()
                        .filter(|e| e.entry_type == "break")
                        .map(|e| e.duration_minutes.unwrap_or(0))
                        .sum();

                    let start_dt = daily
                        .start_time
                        .and_then(|t| tz.from_local_datetime(&date.and_time(t)).single());
                    let end_dt = daily
                        .end_time
                        .and_then(|t| tz.from_local_datetime(&date.and_time(t)).single());

                    if let (Some(start), Some(mut end)) = (start_dt, end_dt) {
                        if daily.end_time < daily.start_time {
                            end += chrono::Duration::days(1);
                        }

                        let (std_units, ot_units) = AttendanceCalculator::calculate_daily_units(
                            punch_in,
                            punch_out,
                            Some(start),
                            Some(end),
                            daily.break_duration_minutes.unwrap_or(0),
                            daily.early_punch_in_grace_period_minutes.unwrap_or(0),
                            daily.is_break_auto_deduct.unwrap_or(false),
                            false,
                            actual_break_minutes,
                            daily.allow_early_ot.unwrap_or(false),
                            workspace.work_unit_minutes,
                            &workspace.rounding_policy,
                            0,
                            false,
                        );

                        total_payable_units = std_units + ot_units;
                    }
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

                reports.push(DailyAttendanceReport {
                    date,
                    employee_id: emp_sched.employee_id,
                    employee_name: format!("{} {}", emp_sched.first_name, emp_sched.last_name),
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
                });
            }
        }

        Ok(reports)
    }

    pub async fn get_workspace_attendance_summary(
        pool: &PgPool,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<AttendanceSummary>, StatusCode> {
        // 0. Get Workspace Settings for Timezone
        let workspace_repo = SqlxWorkspaceRepository { pool: pool.clone() };
        let workspace = workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        let tz: Tz = workspace.timezone.parse().unwrap_or(chrono_tz::UTC);

        // 1. Convert NaiveDate to DateTime<Utc> using Workspace Timezone
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

        let summaries = sqlx::query!(
            r#"
            SELECT 
                e.id as employee_id,
                e.first_name || ' ' || e.last_name as employee_name,
                COALESCE(SUM(te.duration_minutes), 0)::float as total_minutes,
                COALESCE((
                    SELECT SUM(lr.end_date - lr.start_date + 1)
                    FROM leave_requests lr
                    WHERE lr.employee_id = e.id 
                      AND lr.status = 'approved'
                      AND lr.start_date >= $2::date
                      AND lr.end_date <= $3::date
                ), 0)::bigint as total_leave_days
            FROM employees e
            LEFT JOIN time_entries te ON e.id = te.employee_id 
                AND te.punch_in_time >= $4 
                AND te.punch_in_time <= $5
            WHERE e.workspace_id = $1
            GROUP BY e.id, e.first_name, e.last_name
            "#,
            workspace_id,
            start_date,
            end_date,
            start_dt,
            end_dt
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            eprintln!("Error generating report: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok(summaries
            .into_iter()
            .map(|row| {
                let total_minutes = row.total_minutes.unwrap_or(0.0) as i32;
                let work_units = AttendanceCalculator::calculate_units(
                    total_minutes,
                    workspace.work_unit_minutes,
                    &workspace.rounding_policy,
                );

                AttendanceSummary {
                    employee_id: row.employee_id,
                    employee_name: row.employee_name.unwrap_or_default(),
                    total_work_units: work_units,
                    total_leave_days: row.total_leave_days.unwrap_or(0),
                }
            })
            .collect())
    }
}
