use chrono::{NaiveDate, Utc};
use sqlx::{query, query_as, query_scalar, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::{Employee, EmploymentPeriod, WorkingCondition};
use crate::utils::normalize_employee_id;

/// Minimal employee info for paginated reports
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EmployeeBasicInfo {
    pub id: Uuid,
    pub full_name: String,
}

pub async fn find_by_number(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id_number: &str,
) -> Result<Option<Employee>, sqlx::Error> {
    let normalized_id = normalize_employee_id(employee_id_number);
    let query = format!("{} WHERE e.workspace_id = $1 AND e.employee_id_number = $2", EMPLOYEE_SELECT_QUERY);
    sqlx::query_as::<_, Employee>(&query)
        .bind(workspace_id)
        .bind(normalized_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_working_conditions(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Vec<WorkingCondition>, sqlx::Error> {
    sqlx::query_as!(
        WorkingCondition,
        r#"SELECT id, employee_id, type as "type_", minute_allowance, is_paid, start_date, end_date
           FROM working_conditions WHERE employee_id = $1 ORDER BY start_date DESC"#,
        employee_id
    )
    .fetch_all(pool)
    .await
}

pub async fn create_working_condition(
    pool: &PgPool,
    condition: &WorkingCondition,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO working_conditions (id, employee_id, type, minute_allowance, is_paid, start_date, end_date)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
    )
    .bind(condition.id)
    .bind(condition.employee_id)
    .bind(&condition.type_)
    .bind(condition.minute_allowance)
    .bind(condition.is_paid)
    .bind(condition.start_date)
    .bind(condition.end_date)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_working_condition(
    pool: &PgPool,
    id: Uuid,
    employee_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM working_conditions WHERE id = $1 AND employee_id = $2",
    )
    .bind(id)
    .bind(employee_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn create_employee_tx(
    tx: &mut Transaction<'_, Postgres>,
    employee: &Employee,
) -> Result<(), sqlx::Error> {
    let normalized_id = normalize_employee_id(&employee.employee_id_number);
    sqlx::query(
        "INSERT INTO employees (id, workspace_id, user_id, full_name, employee_id_number, tags, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(employee.id)
    .bind(employee.workspace_id)
    .bind(employee.user_id)
    .bind(&employee.full_name)
    .bind(normalized_id)
    .bind(&employee.tags)
    .bind(employee.is_active)
    .bind(employee.created_at)
    .bind(employee.updated_at)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "INSERT INTO employment_periods (employee_id, hire_date, termination_date) VALUES ($1, $2, $3)",
    )
    .bind(employee.id)
    .bind(employee.hire_date)
    .bind(employee.termination_date)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn create_employee_tx_skip_duplicates(
    tx: &mut Transaction<'_, Postgres>,
    employee: &Employee,
) -> Result<bool, sqlx::Error> {
    let normalized_id = normalize_employee_id(&employee.employee_id_number);
    let result = sqlx::query(
        "INSERT INTO employees (id, workspace_id, user_id, full_name, employee_id_number, tags, is_active, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         ON CONFLICT (workspace_id, employee_id_number) DO NOTHING",
    )
    .bind(employee.id)
    .bind(employee.workspace_id)
    .bind(employee.user_id)
    .bind(&employee.full_name)
    .bind(normalized_id)
    .bind(&employee.tags)
    .bind(employee.is_active)
    .bind(employee.created_at)
    .bind(employee.updated_at)
    .execute(&mut **tx)
    .await?;

    if result.rows_affected() > 0 {
        sqlx::query(
            "INSERT INTO employment_periods (employee_id, hire_date, termination_date) VALUES ($1, $2, $3)",
        )
        .bind(employee.id)
        .bind(employee.hire_date)
        .bind(employee.termination_date)
        .execute(&mut **tx)
        .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

const EMPLOYEE_SELECT_QUERY: &str = r#"
    SELECT 
        e.id, e.workspace_id, e.user_id, e.full_name, e.employee_id_number, 
        COALESCE(ep.hire_date, CURRENT_DATE) as hire_date, ep.termination_date, 
        e.tags, e.is_active, e.created_at, e.updated_at
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
    let normalized_id = normalize_employee_id(id_number);
    let query = format!(
        "{} WHERE e.employee_id_number = $1 AND e.workspace_id = $2",
        EMPLOYEE_SELECT_QUERY
    );
    query_as::<_, Employee>(&query)
        .bind(normalized_id)
        .bind(workspace_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_employee_by_id(
    pool: &PgPool,
    id: Uuid,
    workspace_id: Uuid,
) -> Result<Option<Employee>, sqlx::Error> {
    let query = format!(
        "{} WHERE e.id = $1 AND e.workspace_id = $2",
        EMPLOYEE_SELECT_QUERY
    );
    query_as::<_, Employee>(&query)
        .bind(id)
        .bind(workspace_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_all_employees(
    pool: &PgPool,
    workspace_id: Uuid,
    search_query: Option<String>,
    page: i64,
    limit: i64,
) -> Result<(Vec<Employee>, i64), sqlx::Error> {
    let mut where_clause = "WHERE e.workspace_id = $1".to_string();
    if let Some(ref q) = search_query {
        if !q.trim().is_empty() {
            where_clause.push_str(" AND (e.full_name ILIKE $2 OR e.employee_id_number ILIKE $2)");
        }
    }

    let count_sql = format!("SELECT COUNT(*) FROM employees e {}", where_clause);
    let mut count_query = query_scalar::<_, i64>(&count_sql).bind(workspace_id);

    let sql = format!(
        "{} {} ORDER BY e.full_name LIMIT $3 OFFSET $4",
        EMPLOYEE_SELECT_QUERY, where_clause
    );
    let mut db_query = query_as::<_, Employee>(&sql).bind(workspace_id);

    if let Some(ref q) = search_query {
        if !q.trim().is_empty() {
            let bound_q = format!("%{}%", q.trim());
            count_query = count_query.bind(bound_q.clone());
            db_query = db_query.bind(bound_q);
        } else {
            db_query = db_query.bind(Option::<String>::None);
        }
    } else {
        db_query = db_query.bind(Option::<String>::None);
    }

    let total = count_query.fetch_one(pool).await?;

    let offset = (page - 1) * limit;
    let employees = db_query.bind(limit).bind(offset).fetch_all(pool).await?;

    Ok((employees, total))
}

#[allow(clippy::too_many_arguments)]
pub async fn update_employee(
    pool: &PgPool,
    id: Uuid,
    workspace_id: Uuid,
    user_id: Option<Uuid>,
    full_name: Option<String>,
    employee_id_number: Option<String>,
    hire_date: Option<NaiveDate>,
    termination_date: Option<NaiveDate>,
    tags: Option<Vec<String>>,
    is_active: Option<bool>,
) -> Result<Option<Employee>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let normalized_id = employee_id_number.as_ref().map(|id| normalize_employee_id(id));

    // Update base employee info
    sqlx::query(
        "UPDATE employees SET
            user_id = COALESCE($3, user_id),
            full_name = COALESCE($4, full_name),
            employee_id_number = COALESCE($5, employee_id_number),
            tags = COALESCE($6, tags),
            is_active = COALESCE($7, is_active),
            updated_at = $8
        WHERE id = $1 AND workspace_id = $2",
    )
    .bind(id)
    .bind(workspace_id)
    .bind(user_id)
    .bind(full_name)
    .bind(normalized_id)
    .bind(tags.as_deref())
    .bind(is_active)
    .bind(Utc::now())
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
    let query = format!(
        "{} WHERE e.workspace_id = $1 AND e.is_active = true ORDER BY e.full_name",
        EMPLOYEE_SELECT_QUERY
    );
    query_as::<_, Employee>(&query)
        .bind(workspace_id)
        .fetch_all(pool)
        .await
}

pub async fn get_existing_employee_ids_by_numbers(
    pool: &PgPool,
    workspace_id: Uuid,
    id_numbers: &[String],
) -> Result<Vec<String>, sqlx::Error> {
    if id_numbers.is_empty() {
        return Ok(Vec::new());
    }

    let normalized_ids: Vec<String> = id_numbers.iter().map(|id| normalize_employee_id(id)).collect();

    let query = "SELECT employee_id_number FROM employees WHERE workspace_id = $1 AND employee_id_number = ANY($2)";
    let result: Vec<String> = sqlx::query_scalar(query)
        .bind(workspace_id)
        .bind(normalized_ids)
        .fetch_all(pool)
        .await?;

    Ok(result)
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

use crate::models::EmployeePayrollConfig;
use rust_decimal::Decimal;

pub async fn upsert_employee_payroll_config(
    pool: &PgPool,
    employee_id: Uuid,
    std_working_days: Decimal,
    hours_per_day: Decimal,
    effective_date: NaiveDate,
) -> Result<EmployeePayrollConfig, sqlx::Error> {
    sqlx::query_as!(
        EmployeePayrollConfig,
        r#"INSERT INTO employee_payroll_configs (employee_id, std_working_days, hours_per_day, effective_date)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (employee_id, effective_date) DO UPDATE 
           SET std_working_days = EXCLUDED.std_working_days,
               hours_per_day = EXCLUDED.hours_per_day
           RETURNING id, employee_id, std_working_days, hours_per_day, effective_date, created_at as "created_at!""#,
        employee_id,
        std_working_days,
        hours_per_day,
        effective_date
    )
    .fetch_one(pool)
    .await
}

pub async fn get_employee_payroll_config_at(
    pool: &PgPool,
    employee_id: Uuid,
    date: NaiveDate,
) -> Result<Option<EmployeePayrollConfig>, sqlx::Error> {
    sqlx::query_as!(
        EmployeePayrollConfig,
        r#"SELECT id, employee_id, std_working_days, hours_per_day, effective_date, created_at as "created_at!"
           FROM employee_payroll_configs
           WHERE employee_id = $1 AND effective_date <= $2
           ORDER BY effective_date DESC LIMIT 1"#,
        employee_id,
        date
    )
    .fetch_optional(pool)
    .await
}

/// Get active employee IDs for distributed report processing.
/// Returns only IDs (lightweight) for batching into micro-jobs.
///
/// # Arguments
/// * `workspace_id` - Workspace to query
/// * `filter_ids` - Optional: subset of employee IDs to filter by
pub async fn get_active_employee_ids(
    pool: &PgPool,
    workspace_id: Uuid,
    filter_ids: Option<&Vec<Uuid>>,
) -> Result<Vec<Uuid>, sqlx::Error> {
    query_scalar(
        r#"SELECT id FROM employees
           WHERE workspace_id = $1 AND is_active = true
           AND ($2::uuid[] IS NULL OR id = ANY($2))
           ORDER BY full_name"#,
    )
    .bind(workspace_id)
    .bind(filter_ids)
    .fetch_all(pool)
    .await
}

/// Get paginated employee IDs with optional filter for reports
/// Returns: (employees, total_count)
///
/// # Arguments
/// * `sort_desc` - If true, ORDER BY full_name DESC; if false, ORDER BY full_name ASC
pub async fn get_paginated_employee_ids(
    pool: &PgPool,
    workspace_id: Uuid,
    filter_ids: Option<&Vec<Uuid>>,
    page: i64,
    limit: i64,
    sort_desc: bool,
) -> Result<(Vec<EmployeeBasicInfo>, i64), sqlx::Error> {
    // Count query
    let total: i64 = query_scalar(
        r#"SELECT COUNT(*) FROM employees
           WHERE workspace_id = $1 AND is_active = true
           AND ($2::uuid[] IS NULL OR id = ANY($2))"#,
    )
    .bind(workspace_id)
    .bind(filter_ids)
    .fetch_one(pool)
    .await?;

    // Data query with pagination and dynamic ORDER BY
    // Note: Using format!() for ORDER direction since sqlx doesn't support binding ORDER direction
    let offset = (page - 1) * limit;
    let order_dir = if sort_desc { "DESC" } else { "ASC" };
    let sql = format!(
        r#"SELECT id, full_name FROM employees
           WHERE workspace_id = $1 AND is_active = true
           AND ($2::uuid[] IS NULL OR id = ANY($2))
           ORDER BY full_name {}
           LIMIT $3 OFFSET $4"#,
        order_dir
    );

    let employees = query_as::<_, EmployeeBasicInfo>(&sql)
        .bind(workspace_id)
        .bind(filter_ids)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok((employees, total))
}
