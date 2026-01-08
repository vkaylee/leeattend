use sqlx::{PgPool, query, query_as};
use uuid::Uuid;
use chrono::{NaiveTime, NaiveDate};

use crate::models::Shift;

pub async fn create_shift(pool: &PgPool, workspace_id: Uuid, name: String, start_time: NaiveTime, end_time: NaiveTime, break_duration_minutes: i32) -> Result<Shift, sqlx::Error> {
    query_as!(
        Shift,
        r#"INSERT INTO shifts (workspace_id, name, start_time, end_time, break_duration_minutes)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, workspace_id, name, start_time, end_time, break_duration_minutes, created_at as "created_at!", updated_at as "updated_at!""#,
        workspace_id,
        name,
        start_time,
        end_time,
        break_duration_minutes
    )
    .fetch_one(pool)
    .await
}

pub async fn get_workspace_shifts(pool: &PgPool, workspace_id: Uuid) -> Result<Vec<Shift>, sqlx::Error> {
    query_as!(
        Shift,
        r#"SELECT id, workspace_id, name, start_time, end_time, break_duration_minutes, created_at as "created_at!", updated_at as "updated_at!" FROM shifts WHERE workspace_id = $1 ORDER BY name"#,
        workspace_id
    )
    .fetch_all(pool)
    .await
}

pub async fn assign_employee_shift(pool: &PgPool, employee_id: Uuid, shift_id: Uuid, effective_date: NaiveDate) -> Result<(), sqlx::Error> {
    query!(
        "INSERT INTO employee_shifts (employee_id, shift_id, effective_date)
         VALUES ($1, $2, $3)
         ON CONFLICT (employee_id, shift_id, effective_date) DO NOTHING",
        employee_id,
        shift_id,
        effective_date
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_employee_active_shift(pool: &PgPool, employee_id: Uuid, date: NaiveDate) -> Result<Option<Shift>, sqlx::Error> {
    query_as!(
        Shift,
        r#"SELECT s.id, s.workspace_id, s.name, s.start_time, s.end_time, s.break_duration_minutes, s.created_at as "created_at!", s.updated_at as "updated_at!"
         FROM shifts s
         JOIN employee_shifts es ON s.id = es.shift_id
         WHERE es.employee_id = $1 AND es.effective_date <= $2
         ORDER BY es.effective_date DESC
         LIMIT 1"#,
        employee_id,
        date
    )
    .fetch_optional(pool)
    .await
}