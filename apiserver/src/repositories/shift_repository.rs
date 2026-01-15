use chrono::{NaiveDate, NaiveTime};
use sqlx::{query, query_as, PgPool, Postgres, Executor};
use uuid::Uuid;

use crate::models::Shift;

pub async fn create_shift<'a, E>(
    executor: E,
    workspace_id: Uuid,
    name: String,
    start_time: NaiveTime,
    end_time: NaiveTime,
    break_duration_minutes: i32,
    break_start: Option<NaiveTime>,
    break_end: Option<NaiveTime>,
    is_break_auto_deduct: bool,
    break_is_overtime: bool,
    early_punch_in_grace_period_minutes: i32,
    allow_early_ot: bool,
    is_active: bool,
    parent_shift_id: Option<Uuid>,
) -> Result<Shift, sqlx::Error> 
where E: Executor<'a, Database = Postgres>
{
    query_as!(
        Shift,
        r#"INSERT INTO shifts (
            workspace_id, name, start_time, end_time, 
            break_duration_minutes, break_start, break_end, 
            is_break_auto_deduct, break_is_overtime, 
            early_punch_in_grace_period_minutes, allow_early_ot, is_active, parent_shift_id
        )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
         RETURNING 
            id, workspace_id, name, start_time, end_time, 
            break_duration_minutes, break_start, break_end, 
            is_break_auto_deduct, break_is_overtime, 
            early_punch_in_grace_period_minutes, allow_early_ot, is_active, parent_shift_id,
            created_at as "created_at!", updated_at as "updated_at!""#,
        workspace_id,
        name,
        start_time,
        end_time,
        break_duration_minutes,
        break_start,
        break_end,
        is_break_auto_deduct,
        break_is_overtime,
        early_punch_in_grace_period_minutes,
        allow_early_ot,
        is_active,
        parent_shift_id
    )
    .fetch_one(executor)
    .await
}

pub async fn get_shift_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Shift>, sqlx::Error> {
    query_as!(
        Shift,
        r#"SELECT 
            id, workspace_id, name, start_time, end_time, 
            break_duration_minutes, break_start, break_end, 
            is_break_auto_deduct, break_is_overtime, 
            early_punch_in_grace_period_minutes, allow_early_ot, is_active, parent_shift_id,
            created_at as "created_at!", updated_at as "updated_at!" 
         FROM shifts WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_workspace_shifts(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<Shift>, sqlx::Error> {
    query_as!(
        Shift,
        r#"SELECT 
            id, workspace_id, name, start_time, end_time, 
            break_duration_minutes, break_start, break_end, 
            is_break_auto_deduct, break_is_overtime, 
            early_punch_in_grace_period_minutes, allow_early_ot, is_active, parent_shift_id,
            created_at as "created_at!", updated_at as "updated_at!" 
         FROM shifts 
         WHERE workspace_id = $1 AND is_active = TRUE 
         ORDER BY name"#,
        workspace_id
    )
    .fetch_all(pool)
    .await
}

pub async fn update_shift_direct(
    pool: &PgPool,
    id: Uuid,
    name: String,
    start_time: NaiveTime,
    end_time: NaiveTime,
    break_duration_minutes: i32,
    break_start: Option<NaiveTime>,
    break_end: Option<NaiveTime>,
    is_break_auto_deduct: bool,
    break_is_overtime: bool,
    early_punch_in_grace_period_minutes: i32,
    allow_early_ot: bool,
) -> Result<Shift, sqlx::Error> {
    query_as!(
        Shift,
        r#"UPDATE shifts 
         SET name = $2, start_time = $3, end_time = $4, 
             break_duration_minutes = $5, break_start = $6, break_end = $7, 
             is_break_auto_deduct = $8, break_is_overtime = $9, 
             early_punch_in_grace_period_minutes = $10, allow_early_ot = $11, updated_at = CURRENT_TIMESTAMP
         WHERE id = $1
         RETURNING 
            id, workspace_id, name, start_time, end_time, 
            break_duration_minutes, break_start, break_end, 
            is_break_auto_deduct, break_is_overtime, 
            early_punch_in_grace_period_minutes, allow_early_ot, is_active, parent_shift_id,
            created_at as "created_at!", updated_at as "updated_at!""#,
        id,
        name,
        start_time,
        end_time,
        break_duration_minutes,
        break_start,
        break_end,
        is_break_auto_deduct,
        break_is_overtime,
        early_punch_in_grace_period_minutes,
        allow_early_ot
    )
    .fetch_one(pool)
    .await
}

pub async fn archive_shift<'a, E>(
    executor: E,
    id: Uuid,
) -> Result<(), sqlx::Error> 
where E: Executor<'a, Database = Postgres>
{
    query!(
        "UPDATE shifts SET is_active = FALSE, updated_at = CURRENT_TIMESTAMP WHERE id = $1",
        id
    )
    .execute(executor)
    .await?;
    Ok(())
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
        r#"SELECT 
            s.id, s.workspace_id, s.name, s.start_time, s.end_time, 
            s.break_duration_minutes, s.break_start, s.break_end, 
            s.is_break_auto_deduct, s.break_is_overtime, 
            s.early_punch_in_grace_period_minutes, s.allow_early_ot, s.is_active, s.parent_shift_id,
            s.created_at as "created_at!", s.updated_at as "updated_at!"
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

pub async fn update_assignments_to_new_version_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    old_shift_id: Uuid,
    new_shift_id: Uuid,
    effective_date: NaiveDate,
) -> Result<(), sqlx::Error> {
    let _yesterday = effective_date.pred_opt().unwrap_or(effective_date);

    // 1. Create new assignments based on existing active ones
    sqlx::query!(
        "INSERT INTO employee_shifts (employee_id, shift_id, start_date, end_date)
         SELECT employee_id, $2, $3, end_date 
         FROM employee_shifts 
         WHERE shift_id = $1 AND (end_date IS NULL OR end_date >= $3)",
        old_shift_id,
        new_shift_id,
        effective_date
    )
    .execute(&mut **tx)
    .await?;

    // 2. Close old assignments
    sqlx::query!(
        "UPDATE employee_shifts 
         SET end_date = $2, updated_at = CURRENT_TIMESTAMP 
         WHERE shift_id = $1 AND (end_date IS NULL OR end_date >= $3)",
        old_shift_id,
        _yesterday,
        effective_date
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

