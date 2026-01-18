use crate::models::{
    BulkAssignResponse, EmployeeWorkingPattern, WorkingPattern, WorkingPatternDetail,
};
use chrono::NaiveDate;
use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;

pub async fn create_pattern<'a, E>(
    executor: E,
    workspace_id: Uuid,
    name: &str,
    description: Option<&str>,
) -> Result<WorkingPattern, sqlx::Error>
where
    E: Executor<'a, Database = Postgres>,
{
    sqlx::query_as::<_, WorkingPattern>(
        "INSERT INTO working_patterns (workspace_id, name, description) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(workspace_id)
    .bind(name)
    .bind(description)
    .fetch_one(executor)
    .await
}

pub async fn add_pattern_detail<'a, E>(
    executor: E,
    pattern_id: Uuid,
    day_of_week: i32,
    shift_id: Option<Uuid>,
) -> Result<WorkingPatternDetail, sqlx::Error>
where
    E: Executor<'a, Database = Postgres>,
{
    sqlx::query_as::<_, WorkingPatternDetail>(
        "INSERT INTO working_pattern_details (pattern_id, day_of_week, shift_id) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(pattern_id)
    .bind(day_of_week)
    .bind(shift_id)
    .fetch_one(executor)
    .await
}

pub async fn get_patterns(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<WorkingPattern>, sqlx::Error> {
    sqlx::query_as::<_, WorkingPattern>(
        "SELECT * FROM working_patterns WHERE workspace_id = $1 AND is_active = true ORDER BY name",
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await
}

pub async fn get_pattern_details(
    pool: &PgPool,
    pattern_id: Uuid,
) -> Result<Vec<WorkingPatternDetail>, sqlx::Error> {
    sqlx::query_as::<_, WorkingPatternDetail>(
        "SELECT * FROM working_pattern_details WHERE pattern_id = $1 ORDER BY day_of_week",
    )
    .bind(pattern_id)
    .fetch_all(pool)
    .await
}

pub async fn assign_pattern(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
    pattern_id: Uuid,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
) -> Result<EmployeeWorkingPattern, sqlx::Error> {
    sqlx::query_as::<_, EmployeeWorkingPattern>(
        r#"
        INSERT INTO employee_working_patterns (workspace_id, employee_id, pattern_id, start_date, end_date)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(workspace_id)
    .bind(employee_id)
    .bind(pattern_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_one(pool)
    .await
}

pub async fn get_employee_pattern_history(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Vec<EmployeeWorkingPattern>, sqlx::Error> {
    sqlx::query_as::<_, EmployeeWorkingPattern>(
        "SELECT * FROM employee_working_patterns WHERE employee_id = $1 ORDER BY start_date DESC",
    )
    .bind(employee_id)
    .fetch_all(pool)
    .await
}

pub async fn bulk_assign_pattern(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_ids: &[Uuid],
    pattern_id: Uuid,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
) -> Result<BulkAssignResponse, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // 1. Validation: Check if any employee already has a pattern starting on the same date
    let conflicts = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT employee_id FROM employee_working_patterns 
        WHERE workspace_id = $1 AND start_date = $2 AND employee_id = ANY($3)
        "#,
    )
    .bind(workspace_id)
    .bind(start_date)
    .bind(employee_ids)
    .fetch_all(&mut *tx)
    .await?;

    if !conflicts.is_empty() {
        return Ok(BulkAssignResponse {
            success_count: 0,
            failed_employees: conflicts,
        });
    }

    for &employee_id in employee_ids {
        sqlx::query(
            r#"
            INSERT INTO employee_working_patterns (workspace_id, employee_id, pattern_id, start_date, end_date)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(workspace_id)
        .bind(employee_id)
        .bind(pattern_id)
        .bind(start_date)
        .bind(end_date)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(BulkAssignResponse {
        success_count: employee_ids.len(),
        failed_employees: vec![],
    })
}

pub async fn delete_pattern(pool: &PgPool, pattern_id: Uuid) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // 1. Delete details first
    sqlx::query("DELETE FROM working_pattern_details WHERE pattern_id = $1")
        .bind(pattern_id)
        .execute(&mut *tx)
        .await?;

    // 2. Delete the pattern itself
    sqlx::query("DELETE FROM working_patterns WHERE id = $1")
        .bind(pattern_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}
