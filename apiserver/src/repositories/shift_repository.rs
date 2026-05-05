use chrono::NaiveDate;
use sqlx::{query, query_as, Executor, PgPool, Postgres};
use uuid::Uuid;

use crate::models::Shift;

pub async fn create_shift<'a, E>(
    executor: E,
    workspace_id: Uuid,
    req: &crate::models::CreateShiftRequest,
    is_active: bool,
    parent_shift_id: Option<Uuid>,
) -> Result<Shift, sqlx::Error>
where
    E: Executor<'a, Database = Postgres>,
{
    query_as!(
        Shift,
        r#"INSERT INTO shifts (
            workspace_id, name, report_symbol, start_time, end_time, 
            break_duration_minutes, break_start, break_end, 
            is_break_auto_deduct, break_is_overtime, 
            early_punch_in_grace_period_minutes, allow_early_ot, is_active, parent_shift_id
        )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
         RETURNING 
            id, workspace_id, name, report_symbol, start_time, end_time, 
            break_duration_minutes, break_start, break_end, 
            is_break_auto_deduct, break_is_overtime, 
            early_punch_in_grace_period_minutes, allow_early_ot, is_active, parent_shift_id,
            created_at as "created_at!", updated_at as "updated_at!""#,
        workspace_id,
        &req.name,
        req.report_symbol.as_deref(),
        req.start_time,
        req.end_time,
        req.break_duration_minutes,
        req.break_start,
        req.break_end,
        req.is_break_auto_deduct,
        req.break_is_overtime,
        req.early_punch_in_grace_period_minutes,
        req.allow_early_ot,
        is_active,
        parent_shift_id
    )
    .fetch_one(executor)
    .await
}

pub async fn get_shift_by_id(
    pool: &PgPool,
    workspace_id: Uuid,
    id: Uuid,
) -> Result<Option<Shift>, sqlx::Error> {
    query_as!(
        Shift,
        r#"SELECT 
            id, workspace_id, name, report_symbol, start_time, end_time, 
            break_duration_minutes, break_start, break_end, 
            is_break_auto_deduct, break_is_overtime, 
            early_punch_in_grace_period_minutes, allow_early_ot, is_active, parent_shift_id,
            created_at as "created_at!", updated_at as "updated_at!" 
         FROM shifts WHERE id = $1 AND workspace_id = $2"#,
        id,
        workspace_id
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
            id, workspace_id, name, report_symbol, start_time, end_time, 
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
    workspace_id: Uuid,
    id: Uuid,
    req: &crate::models::UpdateShiftRequest,
) -> Result<Shift, sqlx::Error> {
    query_as!(
        Shift,
        r#"UPDATE shifts 
         SET name = $2, report_symbol = $3, start_time = $4, end_time = $5, 
             break_duration_minutes = $6, break_start = $7, break_end = $8, 
             is_break_auto_deduct = $9, break_is_overtime = $10, 
             early_punch_in_grace_period_minutes = $11, allow_early_ot = $12, updated_at = CURRENT_TIMESTAMP
         WHERE id = $1 AND workspace_id = $13
         RETURNING 
            id, workspace_id, name, report_symbol, start_time, end_time, 
            break_duration_minutes, break_start, break_end, 
            is_break_auto_deduct, break_is_overtime, 
            early_punch_in_grace_period_minutes, allow_early_ot, is_active, parent_shift_id,
            created_at as "created_at!", updated_at as "updated_at!""#,
        id,
        &req.name,
        req.report_symbol.as_deref(),
        req.start_time,
        req.end_time,
        req.break_duration_minutes,
        req.break_start,
        req.break_end,
        req.is_break_auto_deduct,
        req.break_is_overtime,
        req.early_punch_in_grace_period_minutes,
        req.allow_early_ot,
        workspace_id
    )
    .fetch_one(pool)
    .await
}

pub async fn archive_shift<'a, E>(
    executor: E,
    workspace_id: Uuid,
    id: Uuid,
) -> Result<(), sqlx::Error>
where
    E: Executor<'a, Database = Postgres>,
{
    query!(
        "UPDATE shifts SET is_active = FALSE, updated_at = CURRENT_TIMESTAMP WHERE id = $1 AND workspace_id = $2",
        id,
        workspace_id
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
            s.id, s.workspace_id, s.name, s.report_symbol, s.start_time, s.end_time, 
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
    let yesterday = effective_date.pred_opt().unwrap_or(effective_date);

    // 1. Close old assignments FIRST
    //    CRITICAL: Must happen before INSERT to prevent unique constraint violation
    //    on employee_shifts(employee_id, start_date) when effective_date = start_date
    sqlx::query!(
        "UPDATE employee_shifts
         SET end_date = $2, updated_at = CURRENT_TIMESTAMP
         WHERE shift_id = $1 AND (end_date IS NULL OR end_date >= $3)",
        old_shift_id,
        yesterday,
        effective_date
    )
    .execute(&mut **tx)
    .await?;

    // 2. Create new assignments for employees whose assignments were just closed
    sqlx::query!(
        "INSERT INTO employee_shifts (employee_id, shift_id, start_date, end_date)
         SELECT employee_id, $2, $3, NULL
         FROM employee_shifts
         WHERE shift_id = $1 AND end_date = $4",
        old_shift_id,
        new_shift_id,
        effective_date,
        yesterday
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}
