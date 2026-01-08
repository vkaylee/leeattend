use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use axum::http::StatusCode;

use crate::models::AttendanceSummary;

pub struct ReportService;

impl ReportService {
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

        Ok(summaries.into_iter().map(|row| AttendanceSummary {
            employee_id: row.employee_id,
            employee_name: row.employee_name.unwrap_or_default(),
            total_hours: row.total_hours.unwrap_or(0.0),
            total_leave_days: row.total_leave_days.unwrap_or(0),
        }).collect())
    }
}
