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
    let query =
        format!("{EMPLOYEE_SELECT_QUERY} WHERE e.workspace_id = $1 AND e.employee_id_number = $2");
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
    let result = sqlx::query("DELETE FROM working_conditions WHERE id = $1 AND employee_id = $2")
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
    let query =
        format!("{EMPLOYEE_SELECT_QUERY} WHERE e.employee_id_number = $1 AND e.workspace_id = $2");
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
    let query = format!("{EMPLOYEE_SELECT_QUERY} WHERE e.id = $1 AND e.workspace_id = $2");
    query_as::<_, Employee>(&query)
        .bind(id)
        .bind(workspace_id)
        .fetch_optional(pool)
        .await
}

/// Build the SQL components for employee search.
/// Returns (where_clause, order_clause, has_search).
/// Extracted for testability.
fn build_employee_search_sql(search_query: &Option<String>) -> (String, &'static str, bool) {
    let has_search = search_query
        .as_ref()
        .is_some_and(|q| !q.trim().is_empty());

    let mut where_clause = "WHERE e.workspace_id = $1".to_string();
    if has_search {
        where_clause.push_str(" AND (e.full_name ILIKE $2 OR e.employee_id_number ILIKE $2)");
    }

    // When searching, rank results: exact id > prefix id > exact name > prefix name > alpha
    let order_clause = if has_search {
        "ORDER BY \
            (e.employee_id_number = $5) DESC, \
            (e.employee_id_number ILIKE $5 || '%') DESC, \
            (e.full_name ILIKE $5) DESC, \
            (e.full_name ILIKE $5 || '%') DESC, \
            e.full_name ASC"
    } else {
        "ORDER BY e.full_name ASC"
    };

    (where_clause, order_clause, has_search)
}

pub async fn get_all_employees(
    pool: &PgPool,
    workspace_id: Uuid,
    search_query: Option<String>,
    page: i64,
    limit: i64,
) -> Result<(Vec<Employee>, i64), sqlx::Error> {
    let (where_clause, order_clause, has_search) = build_employee_search_sql(&search_query);

    let count_sql = format!("SELECT COUNT(*) FROM employees e {where_clause}");
    let mut count_query = query_scalar::<_, i64>(&count_sql).bind(workspace_id);

    let sql = format!(
        "{EMPLOYEE_SELECT_QUERY} {where_clause} {order_clause} LIMIT $3 OFFSET $4"
    );
    let mut db_query = query_as::<_, Employee>(&sql).bind(workspace_id);

    if has_search {
        let raw_q = search_query.as_ref().expect("search_query verified non-empty by has_search").trim().to_string();
        let bound_q = format!("%{raw_q}%");
        count_query = count_query.bind(bound_q.clone());
        db_query = db_query.bind(bound_q);
    } else {
        db_query = db_query.bind(Option::<String>::None);
    }

    let total = count_query.fetch_one(pool).await?;

    let offset = (page - 1) * limit;
    db_query = db_query.bind(limit).bind(offset);

    // Bind $5 (raw search term for ranking) only when searching
    if has_search {
        let raw_q = search_query.as_ref().expect("search_query verified non-empty by has_search").trim().to_string();
        db_query = db_query.bind(raw_q);
    }

    let employees = db_query.fetch_all(pool).await?;

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

    let normalized_id = employee_id_number
        .as_ref()
        .map(|id| normalize_employee_id(id));

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
    let result: sqlx::postgres::PgQueryResult = query!(
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
        "{EMPLOYEE_SELECT_QUERY} WHERE e.workspace_id = $1 AND e.is_active = true ORDER BY e.full_name"
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

    let normalized_ids: Vec<String> = id_numbers
        .iter()
        .map(|id| normalize_employee_id(id))
        .collect();

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
/// * `department_id` - Optional: filter employees belonging to this department
#[allow(clippy::too_many_arguments)]
pub async fn get_paginated_employee_ids(
    pool: &PgPool,
    workspace_id: Uuid,
    filter_ids: Option<&Vec<Uuid>>,
    department_id: Option<Uuid>,
    page: i64,
    limit: i64,
    sort_desc: bool,
) -> Result<(Vec<EmployeeBasicInfo>, i64), sqlx::Error> {
    // Count query
    let total: i64 = query_scalar(
        r#"SELECT COUNT(*) FROM employees
           WHERE workspace_id = $1 AND is_active = true
           AND ($2::uuid[] IS NULL OR id = ANY($2))
           AND ($3::uuid IS NULL OR EXISTS (
               SELECT 1 FROM employee_department_history edh
               WHERE edh.employee_id = employees.id AND edh.department_id = $3
           ))"#,
    )
    .bind(workspace_id)
    .bind(filter_ids)
    .bind(department_id)
    .fetch_one(pool)
    .await?;

    // Data query with pagination and dynamic ORDER BY
    // SAFETY: order_dir is a hardcoded &str ("DESC" or "ASC"), never user input.
    // sqlx doesn't support binding ORDER direction as a parameter.
    let offset = (page - 1) * limit;
    let order_dir = if sort_desc { "DESC" } else { "ASC" };
    let sql = format!(
        r#"SELECT id, full_name FROM employees
           WHERE workspace_id = $1 AND is_active = true
           AND ($2::uuid[] IS NULL OR id = ANY($2))
           AND ($5::uuid IS NULL OR EXISTS (
               SELECT 1 FROM employee_department_history edh
               WHERE edh.employee_id = employees.id AND edh.department_id = $5
           ))
           ORDER BY full_name {order_dir}
           LIMIT $3 OFFSET $4"#
    );

    let employees = query_as::<_, EmployeeBasicInfo>(&sql)
        .bind(workspace_id)
        .bind(filter_ids)
        .bind(limit)
        .bind(offset)
        .bind(department_id)
        .fetch_all(pool)
        .await?;

    Ok((employees, total))
}

// ── Recruitment Source Repository ──────────────────────────────────

use crate::models::{RecruitmentSource, RecruitmentSourceBreakdown};

pub async fn create_recruitment_source(
    pool: &PgPool,
    source: &RecruitmentSource,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO recruitment_sources
           (id, workspace_id, source_type, name,
            referrer_employee_id, referral_bonus_amount, referral_bonus_paid,
            agency_contact_name, agency_contact_email, agency_fee_amount, agency_fee_paid,
            notes, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"#,
    )
    .bind(source.id)
    .bind(source.workspace_id)
    .bind(source.source_type)
    .bind(&source.name)
    .bind(source.referrer_employee_id)
    .bind(source.referral_bonus_amount)
    .bind(source.referral_bonus_paid)
    .bind(&source.agency_contact_name)
    .bind(&source.agency_contact_email)
    .bind(source.agency_fee_amount)
    .bind(source.agency_fee_paid)
    .bind(&source.notes)
    .bind(source.created_at)
    .bind(source.updated_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_recruitment_sources(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<RecruitmentSource>, sqlx::Error> {
    sqlx::query_as::<_, RecruitmentSource>(
        "SELECT * FROM recruitment_sources WHERE workspace_id = $1 ORDER BY created_at DESC",
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await
}

pub async fn get_recruitment_source_by_id(
    pool: &PgPool,
    id: Uuid,
    workspace_id: Uuid,
) -> Result<Option<RecruitmentSource>, sqlx::Error> {
    sqlx::query_as::<_, RecruitmentSource>(
        "SELECT * FROM recruitment_sources WHERE id = $1 AND workspace_id = $2",
    )
    .bind(id)
    .bind(workspace_id)
    .fetch_optional(pool)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn update_recruitment_source(
    pool: &PgPool,
    id: Uuid,
    workspace_id: Uuid,
    name: Option<String>,
    referrer_employee_id: Option<Uuid>,
    referral_bonus_amount: Option<Decimal>,
    referral_bonus_paid: Option<bool>,
    agency_contact_name: Option<String>,
    agency_contact_email: Option<String>,
    agency_fee_amount: Option<Decimal>,
    agency_fee_paid: Option<bool>,
    notes: Option<String>,
) -> Result<Option<RecruitmentSource>, sqlx::Error> {
    sqlx::query_as::<_, RecruitmentSource>(
        r#"UPDATE recruitment_sources SET
            name = COALESCE($3, name),
            referrer_employee_id = COALESCE($4, referrer_employee_id),
            referral_bonus_amount = COALESCE($5, referral_bonus_amount),
            referral_bonus_paid = COALESCE($6, referral_bonus_paid),
            agency_contact_name = COALESCE($7, agency_contact_name),
            agency_contact_email = COALESCE($8, agency_contact_email),
            agency_fee_amount = COALESCE($9, agency_fee_amount),
            agency_fee_paid = COALESCE($10, agency_fee_paid),
            notes = COALESCE($11, notes),
            updated_at = NOW()
        WHERE id = $1 AND workspace_id = $2
        RETURNING *"#,
    )
    .bind(id)
    .bind(workspace_id)
    .bind(name)
    .bind(referrer_employee_id)
    .bind(referral_bonus_amount)
    .bind(referral_bonus_paid)
    .bind(agency_contact_name)
    .bind(agency_contact_email)
    .bind(agency_fee_amount)
    .bind(agency_fee_paid)
    .bind(notes)
    .fetch_optional(pool)
    .await
}

pub async fn delete_recruitment_source(
    pool: &PgPool,
    id: Uuid,
    workspace_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM recruitment_sources WHERE id = $1 AND workspace_id = $2",
    )
    .bind(id)
    .bind(workspace_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Dashboard chart: count employees by recruitment source type/name
pub async fn get_recruitment_source_breakdown(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<RecruitmentSourceBreakdown>, sqlx::Error> {
    #[derive(Debug, sqlx::FromRow)]
    struct Row {
        source_type: String,
        source_name: String,
        count: i64,
    }

    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT
            COALESCE(rs.source_type::text, 'direct') AS source_type,
            COALESCE(rs.name, 'Direct Hire') AS source_name,
            COUNT(*) AS count
        FROM employees e
        LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
        WHERE e.workspace_id = $1 AND e.is_active = TRUE
        GROUP BY rs.source_type, rs.name
        ORDER BY count DESC
        "#,
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    Ok(rows
        .into_iter()
        .map(|r| RecruitmentSourceBreakdown {
            source_type: r.source_type,
            source_name: r.source_name,
            count: r.count,
        })
        .collect())
}

pub async fn set_employee_recruitment_source(
    pool: &PgPool,
    employee_id: Uuid,
    recruitment_source_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE employees SET recruitment_source_id = $2, updated_at = NOW() WHERE id = $1",
    )
    .bind(employee_id)
    .bind(recruitment_source_id)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_sql_without_query_uses_alpha_order() {
        let (where_clause, order_clause, has_search) = build_employee_search_sql(&None);

        assert!(!has_search);
        assert_eq!(where_clause, "WHERE e.workspace_id = $1");
        assert!(order_clause.contains("ORDER BY e.full_name ASC"));
        assert!(!order_clause.contains("$5"));
    }

    #[test]
    fn test_search_sql_with_empty_query_uses_alpha_order() {
        let q = Some("   ".to_string());
        let (where_clause, order_clause, has_search) = build_employee_search_sql(&q);

        assert!(!has_search);
        assert_eq!(where_clause, "WHERE e.workspace_id = $1");
        assert!(order_clause.contains("ORDER BY e.full_name ASC"));
        assert!(!order_clause.contains("$5"));
    }

    #[test]
    fn test_search_sql_with_query_adds_relevance_ranking() {
        let q = Some("16".to_string());
        let (where_clause, order_clause, has_search) = build_employee_search_sql(&q);

        assert!(has_search);
        // WHERE should include ILIKE filter
        assert!(where_clause.contains("ILIKE $2"));
        // ORDER should include exact match ranking ($5)
        assert!(order_clause.contains("employee_id_number = $5"));
        assert!(order_clause.contains("employee_id_number ILIKE $5"));
        assert!(order_clause.contains("full_name ILIKE $5"));
        assert!(order_clause.contains("e.full_name ASC"));
    }

    #[test]
    fn test_search_sql_ranking_order_exact_before_prefix_before_alpha() {
        let q = Some("test".to_string());
        let (_, order_clause, _) = build_employee_search_sql(&q);

        // Verify the ranking tiers appear in correct order:
        // 1. exact id match, 2. prefix id match, 3. exact name match, 4. prefix name, 5. alpha
        let exact_id_pos = order_clause.find("employee_id_number = $5").unwrap();
        let prefix_id_pos = order_clause.find("employee_id_number ILIKE $5 || '%'").unwrap();
        let exact_name_pos = order_clause.find("full_name ILIKE $5) DESC").unwrap();
        let prefix_name_pos = order_clause.find("full_name ILIKE $5 || '%'").unwrap();
        let alpha_pos = order_clause.find("e.full_name ASC").unwrap();

        assert!(exact_id_pos < prefix_id_pos, "Exact ID should come before prefix ID");
        assert!(prefix_id_pos < exact_name_pos, "Prefix ID should come before exact name");
        assert!(exact_name_pos < prefix_name_pos, "Exact name should come before prefix name");
        assert!(prefix_name_pos < alpha_pos, "Prefix name should come before alphabetical");
    }
}
