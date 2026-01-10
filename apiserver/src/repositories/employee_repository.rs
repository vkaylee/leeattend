use chrono::{NaiveDate, Utc};
use sqlx::{query, query_as, PgPool};
use uuid::Uuid;

use crate::models::Employee;

pub async fn create_employee(pool: &PgPool, employee: &Employee) -> Result<(), sqlx::Error> {
    query!(
        "INSERT INTO employees (id, workspace_id, user_id, first_name, last_name, employee_id_number, hire_date, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        employee.id,
        employee.workspace_id,
        employee.user_id,
        employee.first_name,
        employee.last_name,
        employee.employee_id_number,
        employee.hire_date,
        employee.is_active,
        employee.created_at,
        employee.updated_at,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_employee_by_id(
    pool: &PgPool,
    id: Uuid,
    workspace_id: Uuid,
) -> Result<Option<Employee>, sqlx::Error> {
    query_as!(
        Employee,
        "SELECT id, workspace_id, user_id, first_name, last_name, employee_id_number, hire_date, termination_date, is_active, created_at as \"created_at!\", updated_at as \"updated_at!\" FROM employees WHERE id = $1 AND workspace_id = $2",
        id,
        workspace_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_all_employees(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<Employee>, sqlx::Error> {
    query_as!(
        Employee,
        "SELECT id, workspace_id, user_id, first_name, last_name, employee_id_number, hire_date, termination_date, is_active, created_at as \"created_at!\", updated_at as \"updated_at!\" FROM employees WHERE workspace_id = $1 ORDER BY last_name, first_name",
        workspace_id
    )
    .fetch_all(pool)
    .await
}

pub async fn update_employee(
    pool: &PgPool,
    id: Uuid,
    workspace_id: Uuid,
    user_id: Option<Uuid>,
    first_name: Option<String>,
    last_name: Option<String>,
    employee_id_number: Option<String>,
    hire_date: Option<NaiveDate>,
    termination_date: Option<NaiveDate>,
    is_active: Option<bool>,
) -> Result<Option<Employee>, sqlx::Error> {
    let updated_employee = query_as!(
        Employee,
        "UPDATE employees SET
            user_id = COALESCE($3, user_id),
            first_name = COALESCE($4, first_name),
            last_name = COALESCE($5, last_name),
            employee_id_number = COALESCE($6, employee_id_number),
            hire_date = COALESCE($7, hire_date),
            termination_date = COALESCE($8, termination_date),
            is_active = COALESCE($9, is_active),
            updated_at = $10
        WHERE id = $1 AND workspace_id = $2
        RETURNING id, workspace_id, user_id, first_name, last_name, employee_id_number, hire_date, termination_date, is_active, created_at as \"created_at!\", updated_at as \"updated_at!\"",
        id,
        workspace_id,
        user_id,
        first_name,
        last_name,
        employee_id_number,
        hire_date,
        termination_date,
        is_active,
        Utc::now()
    )
    .fetch_optional(pool)
    .await?;

    Ok(updated_employee)
}

pub async fn delete_employee(
    pool: &PgPool,
    id: Uuid,
    workspace_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = query!(
        "DELETE FROM employees WHERE id = $1 AND workspace_id = $2",
        id,
        workspace_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn get_active_employees(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<Employee>, sqlx::Error> {
    query_as!(
        Employee,
        "SELECT id, workspace_id, user_id, first_name, last_name, employee_id_number, hire_date, termination_date, is_active, created_at as \"created_at!\", updated_at as \"updated_at!\" FROM employees WHERE workspace_id = $1 AND is_active = true ORDER BY last_name, first_name",
        workspace_id
    )
    .fetch_all(pool)
    .await
}

