use chrono::{NaiveDate, NaiveTime};
use sqlx::{query, query_as, PgPool};
use uuid::Uuid;

use crate::models::Shift;

pub async fn create_shift(
    pool: &PgPool,
    workspace_id: Uuid,
    name: String,
    start_time: NaiveTime,
    end_time: NaiveTime,
    break_duration_minutes: i32,
    break_is_overtime: bool,
    early_punch_in_grace_period_minutes: i32,
) -> Result<Shift, sqlx::Error> {
    query_as!(
        Shift,
        r#"INSERT INTO shifts (workspace_id, name, start_time, end_time, break_duration_minutes, break_is_overtime, early_punch_in_grace_period_minutes)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id, workspace_id, name, start_time, end_time, break_duration_minutes, break_is_overtime, early_punch_in_grace_period_minutes, created_at as "created_at!", updated_at as "updated_at!""#,
        workspace_id,
        name,
        start_time,
        end_time,
        break_duration_minutes,
        break_is_overtime,
        early_punch_in_grace_period_minutes
    )
    .fetch_one(pool)
    .await
}

pub async fn get_workspace_shifts(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<Shift>, sqlx::Error> {
    query_as!(
        Shift,
        r#"SELECT id, workspace_id, name, start_time, end_time, break_duration_minutes, break_is_overtime, early_punch_in_grace_period_minutes, created_at as "created_at!", updated_at as "updated_at!" FROM shifts WHERE workspace_id = $1 ORDER BY name"#,
        workspace_id
    )
    .fetch_all(pool)
    .await
}

pub async fn assign_employee_shift(
    pool: &PgPool,
    employee_id: Uuid,
    shift_id: Uuid,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
) -> Result<(), sqlx::Error> {
    query!(
        "INSERT INTO employee_shifts (employee_id, shift_id, start_date, end_date)
         VALUES ($1, $2, $3, $4)",
        employee_id,
        shift_id,
        start_date,
        end_date
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_employee_active_shift(
    pool: &PgPool,
    employee_id: Uuid,
    date: NaiveDate,
) -> Result<Option<Shift>, sqlx::Error> {
    query_as!(
        Shift,
        r#"SELECT s.id, s.workspace_id, s.name, s.start_time, s.end_time, s.break_duration_minutes, s.break_is_overtime, s.early_punch_in_grace_period_minutes, s.created_at as "created_at!", s.updated_at as "updated_at!"
         FROM shifts s
         JOIN employee_shifts es ON s.id = es.shift_id
         WHERE es.employee_id = $1 
           AND es.start_date <= $2 
           AND (es.end_date IS NULL OR es.end_date >= $2)
         ORDER BY es.created_at DESC
         LIMIT 1"#,
        employee_id,
        date
    )
    .fetch_optional(pool)
    .await
}

