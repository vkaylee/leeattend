use chrono::{NaiveDate, Utc};
use sqlx::{query, query_as, query_scalar, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::{Employee, EmploymentPeriod};

pub async fn create_employee_tx(
    tx: &mut Transaction<'_, Postgres>,
    employee: &Employee,
) -> Result<(), sqlx::Error> {
    query!(
        "INSERT INTO employees (id, workspace_id, user_id, first_name, last_name, employee_id_number, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        employee.id,
        employee.workspace_id,
        employee.user_id,
        employee.first_name,
        employee.last_name,
        employee.employee_id_number,
        employee.is_active,
        employee.created_at,
        employee.updated_at,
    )
    .execute(&mut **tx)
    .await?;

    query!(
        "INSERT INTO employment_periods (employee_id, hire_date, termination_date) VALUES ($1, $2, $3)",
        employee.id,
        employee.hire_date,
        employee.termination_date
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

const EMPLOYEE_SELECT_QUERY: &str = r#"
    SELECT 
        e.id, e.workspace_id, e.user_id, e.first_name, e.last_name, e.employee_id_number, 
        COALESCE(ep.hire_date, CURRENT_DATE) as hire_date, ep.termination_date, 
        e.is_active, e.created_at, e.updated_at
    FROM employees e
    LEFT JOIN (
        SELECT DISTINCT ON (employee_id) employee_id, hire_date, termination_date
        FROM employment_periods
        ORDER BY employee_id, hire_date DESC
    ) ep ON e.id = ep.employee_id
"#;

pub async fn get_employee_by_id_number(
    pool: &PgPool,
    id_number: &str,
    workspace_id: Uuid,
) -> Result<Option<Employee>, sqlx::Error> {
    let query = format!("{} WHERE e.employee_id_number = $1 AND e.workspace_id = $2", EMPLOYEE_SELECT_QUERY);
    query_as::<_, Employee>(&query)
        .bind(id_number)
        .bind(workspace_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_employee_by_id(
    pool: &PgPool,
    id: Uuid,
    workspace_id: Uuid,
) -> Result<Option<Employee>, sqlx::Error> {
    let query = format!("{} WHERE e.id = $1 AND e.workspace_id = $2", EMPLOYEE_SELECT_QUERY);
    query_as::<_, Employee>(&query)
        .bind(id)
        .bind(workspace_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_all_employees(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<Employee>, sqlx::Error> {
    let query = format!("{} WHERE e.workspace_id = $1 ORDER BY e.last_name, e.first_name", EMPLOYEE_SELECT_QUERY);
    query_as::<_, Employee>(&query)
        .bind(workspace_id)
        .fetch_all(pool)
        .await
}

#[allow(clippy::too_many_arguments)]
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
    let mut tx = pool.begin().await?;

    // Update base employee info
    query!(
        "UPDATE employees SET
            user_id = COALESCE($3, user_id),
            first_name = COALESCE($4, first_name),
            last_name = COALESCE($5, last_name),
            employee_id_number = COALESCE($6, employee_id_number),
            is_active = COALESCE($7, is_active),
            updated_at = $8
        WHERE id = $1 AND workspace_id = $2",
        id,
        workspace_id,
        user_id,
        first_name,
        last_name,
        employee_id_number,
        is_active,
        Utc::now()
    )
    .execute(&mut *tx)
    .await?;

    // Update latest employment period if hire_date or termination_date is provided
    if hire_date.is_some() || termination_date.is_some() {
        // Find the latest period id
        let latest_period_id = query_scalar!(
            "SELECT id FROM employment_periods WHERE employee_id = $1 ORDER BY hire_date DESC LIMIT 1",
            id
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(pid) = latest_period_id {
            query!(
                "UPDATE employment_periods SET
                    hire_date = COALESCE($2, hire_date),
                    termination_date = COALESCE($3, termination_date),
                    updated_at = $4
                WHERE id = $1",
                pid,
                hire_date,
                termination_date,
                Utc::now()
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    get_employee_by_id(pool, id, workspace_id).await
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
    let query = format!("{} WHERE e.workspace_id = $1 AND e.is_active = true ORDER BY e.last_name, e.first_name", EMPLOYEE_SELECT_QUERY);
    query_as::<_, Employee>(&query)
        .bind(workspace_id)
        .fetch_all(pool)
        .await
}

pub async fn get_employment_history(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Vec<EmploymentPeriod>, sqlx::Error> {
    query_as!(
        EmploymentPeriod,
        "SELECT id, employee_id, hire_date, termination_date, is_rehire, created_at as \"created_at!\", updated_at as \"updated_at!\" 
         FROM employment_periods WHERE employee_id = $1 ORDER BY hire_date DESC",
        employee_id
    )
    .fetch_all(pool)
    .await
}

pub async fn create_employment_period(
    pool: &PgPool,
    period: &EmploymentPeriod,
) -> Result<(), sqlx::Error> {
    query!(
        "INSERT INTO employment_periods (id, employee_id, hire_date, termination_date, is_rehire, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        period.id,
        period.employee_id,
        period.hire_date,
        period.termination_date,
        period.is_rehire,
        period.created_at,
        period.updated_at
    )
    .execute(pool)
    .await?;
    Ok(())
}