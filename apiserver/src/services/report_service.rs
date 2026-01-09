use axum::http::StatusCode;
use chrono::{DateTime, Utc, NaiveDate, Local, TimeZone};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{AttendanceSummary, DailyAttendanceReport};
use crate::repositories::{roster_repository, time_entry_repository};

pub struct ReportService;

impl ReportService {
    pub async fn get_daily_attendance_report(
        pool: &PgPool,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<DailyAttendanceReport>, StatusCode> {
        // 1. Get all schedules for the range
        let schedules = roster_repository::get_workspace_schedules(pool, workspace_id, start_date, end_date)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 2. Get all time entries for the range
        // Convert NaiveDate to DateTime<Utc> for query
        let start_dt = Local.from_local_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap()).single().unwrap().with_timezone(&Utc);
        let end_dt = Local.from_local_datetime(&end_date.and_hms_opt(23, 59, 59).unwrap()).single().unwrap().with_timezone(&Utc);

        let mut reports = Vec::new();

        for emp_sched in schedules {
            let entries = time_entry_repository::get_employee_time_entries(pool, emp_sched.employee_id, start_dt, end_dt)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            for daily in emp_sched.schedules {
                let date = daily.date;
                
                // Find entries for this day
                let day_entries: Vec<_> = entries.iter()
                    .filter(|e| e.punch_in_time.with_timezone(&Local).date_naive() == date)
                    .collect();

                let actual_punch_in = day_entries.first().map(|e| e.punch_in_time);
                let actual_punch_out = day_entries.last().and_then(|e| e.punch_out_time);
                let total_minutes: i32 = day_entries.iter().map(|e| e.duration_minutes.unwrap_or(0)).sum();

                let mut status = "absent".to_string();
                if actual_punch_in.is_some() {
                    status = "present".to_string();
                    
                    if let (Some(punch_in), Some(sched_start)) = (actual_punch_in, daily.start_time) {
                        if punch_in.with_timezone(&Local).time() > sched_start + chrono::Duration::minutes(5) {
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
                    actual_punch_in,
                    actual_punch_out,
                    total_minutes,
                    status,
                });
            }
        }

        Ok(reports)
    }

    pub async fn get_workspace_attendance_summary(
        pool: &PgPool,
        workspace_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<AttendanceSummary>, StatusCode> {
        let summaries = sqlx::query!(
            r#"
            SELECT 
                e.id as employee_id,
                e.first_name || ' ' || e.last_name as employee_name,
                COALESCE(SUM(te.duration_minutes), 0)::float / 60.0 as total_hours,
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
                AND te.punch_in_time >= $2 
                AND te.punch_in_time <= $3
            WHERE e.workspace_id = $1
            GROUP BY e.id, e.first_name, e.last_name
            "#,
            workspace_id,
            start_date,
            end_date
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            eprintln!("Error generating report: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok(summaries
            .into_iter()
            .map(|row| AttendanceSummary {
                employee_id: row.employee_id,
                employee_name: row.employee_name.unwrap_or_default(),
                total_hours: row.total_hours.unwrap_or(0.0),
                total_leave_days: row.total_leave_days.unwrap_or(0),
            })
            .collect())
    }
}
