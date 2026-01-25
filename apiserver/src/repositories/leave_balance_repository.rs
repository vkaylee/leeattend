use crate::models::{EmployeeLeaveBalance, LeaveBalance, WorkspaceLeaveBalance};
use chrono::Datelike;
use sqlx::{query, query_as, PgPool};
use uuid::Uuid;

pub async fn get_employee_balances(
    pool: &PgPool,
    employee_id: Uuid,
    year: i32,
) -> Result<Vec<EmployeeLeaveBalance>, sqlx::Error> {
    let balances = query_as!(
        EmployeeLeaveBalance,
        r#"
        SELECT
            lb.leave_type_id as "leave_type_id!",
            lt.code as "leave_type_code!",
            lt.name as "leave_type_name!",
            lt.is_paid as "is_paid!",
            lb.entitlement as "entitlement!",
            lb.accrued_amount::FLOAT8 as "accrued_amount!",
            lb.carried_over as "carried_over!",
            COALESCE(
                (
                    SELECT SUM(end_date - start_date + 1)
                    FROM leave_requests
                    WHERE employee_id = lb.employee_id
                    AND leave_type_id = lb.leave_type_id
                    AND status = 'approved'
                    AND EXTRACT(YEAR FROM start_date) = lb.year
                ),
                0
            )::INT as "used!",
            (
                lb.entitlement + lb.accrued_amount + lb.carried_over - COALESCE(
                    (
                        SELECT SUM(end_date - start_date + 1)
                        FROM leave_requests
                        WHERE employee_id = lb.employee_id
                        AND leave_type_id = lb.leave_type_id
                        AND status = 'approved'
                        AND EXTRACT(YEAR FROM start_date) = lb.year
                    ),
                    0
                )
            )::FLOAT8 as "remaining!"
        FROM leave_balances lb
        JOIN leave_types lt ON lb.leave_type_id = lt.id
        WHERE lb.employee_id = $1 AND lb.year = $2
        "#,
        employee_id,
        year
    )
    .fetch_all(pool)
    .await?;

    Ok(balances)
}

pub async fn get_workspace_balances(
    pool: &PgPool,
    workspace_id: Uuid,
    year: i32,
    limit: i64,
    offset: i64,
    search: Option<String>,
) -> Result<(Vec<WorkspaceLeaveBalance>, i64), sqlx::Error> {
    let search_term = format!("%{}%", search.unwrap_or_default());

    // Get total count (per employee-leave_type pair)
    let count_sql = r#"
        SELECT COUNT(*)
        FROM employees e
        CROSS JOIN leave_types lt
        WHERE e.workspace_id = $1 AND e.is_active = TRUE AND lt.workspace_id = $1
          AND ($3 = '%%' OR e.full_name ILIKE $3 OR e.employee_id_number ILIKE $3 OR lt.name ILIKE $3 OR lt.code ILIKE $3)
    "#;

    let total: i64 = sqlx::query_scalar(count_sql)
        .bind(workspace_id)
        .bind(year)
        .bind(&search_term)
        .fetch_one(pool)
        .await?;

    let sql = r#"
        SELECT
            e.id as employee_id,
            e.full_name as employee_name,
            lt.id as leave_type_id,
            lt.code as leave_type_code,
            lt.name as leave_type_name,
            lt.is_paid as is_paid,
            COALESCE(lb.entitlement, 0) as entitlement,
            COALESCE(lb.accrued_amount::FLOAT8, 0.0) as accrued_amount,
            COALESCE(lb.carried_over, 0) as carried_over,
            COALESCE(
                (
                    SELECT SUM(end_date - start_date + 1)
                    FROM leave_requests
                    WHERE employee_id = e.id
                    AND leave_type_id = lt.id
                    AND status = 'approved'
                    AND EXTRACT(YEAR FROM start_date) = $2
                ),
                0
            )::INT as used,
            (
                COALESCE(lb.entitlement, 0) + COALESCE(lb.accrued_amount, 0) + COALESCE(lb.carried_over, 0) - COALESCE(
                    (
                        SELECT SUM(end_date - start_date + 1)
                        FROM leave_requests
                        WHERE employee_id = e.id
                        AND leave_type_id = lt.id
                        AND status = 'approved'
                        AND EXTRACT(YEAR FROM start_date) = $2
                    ),
                    0
                )
            )::FLOAT8 as remaining,
            (
                SELECT d.name
                FROM leave_policy_assignments a
                JOIN leave_policy_definitions d ON a.policy_id = d.id
                WHERE a.workspace_id = $1
                  AND d.leave_type_id = lt.id
                  AND (
                    (a.assignee_type = 'employee' AND a.assignee_id = e.id) OR
                    (a.assignee_type = 'department' AND a.assignee_id = (SELECT department_id FROM employee_department_history WHERE employee_id = e.id ORDER BY effective_date DESC LIMIT 1)) OR
                    (a.assignee_type = 'workspace')
                  )
                  AND a.effective_from <= NOW()
                  AND (a.effective_to IS NULL OR a.effective_to > NOW())
                ORDER BY
                  CASE a.assignee_type
                    WHEN 'employee' THEN 1
                    WHEN 'department' THEN 2
                    WHEN 'workspace' THEN 3
                  END ASC
                LIMIT 1
            ) as policy_name,
            (
                SELECT a.assignee_type
                FROM leave_policy_assignments a
                JOIN leave_policy_definitions d ON a.policy_id = d.id
                WHERE a.workspace_id = $1
                  AND d.leave_type_id = lt.id
                  AND (
                    (a.assignee_type = 'employee' AND a.assignee_id = e.id) OR
                    (a.assignee_type = 'department' AND a.assignee_id = (SELECT department_id FROM employee_department_history WHERE employee_id = e.id ORDER BY effective_date DESC LIMIT 1)) OR
                    (a.assignee_type = 'workspace')
                  )
                  AND a.effective_from <= NOW()
                  AND (a.effective_to IS NULL OR a.effective_to > NOW())
                ORDER BY
                  CASE a.assignee_type
                    WHEN 'employee' THEN 1
                    WHEN 'department' THEN 2
                    WHEN 'workspace' THEN 3
                  END ASC
                LIMIT 1
            ) as assignment_type
        FROM employees e
        CROSS JOIN leave_types lt
        LEFT JOIN leave_balances lb ON e.id = lb.employee_id AND lt.id = lb.leave_type_id AND lb.year = $2
        WHERE e.workspace_id = $1 AND e.is_active = TRUE AND lt.workspace_id = $1
          AND ($3 = '%%' OR e.full_name ILIKE $3 OR e.employee_id_number ILIKE $3 OR lt.name ILIKE $3 OR lt.code ILIKE $3)
        ORDER BY e.full_name ASC, lt.name ASC
        LIMIT $4 OFFSET $5
    "#;

    let data = sqlx::query_as::<_, WorkspaceLeaveBalance>(sql)
        .bind(workspace_id)
        .bind(year)
        .bind(search_term)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok((data, total))
}

pub async fn upsert_leave_balance(
    pool: &PgPool,
    balance: &LeaveBalance,
) -> Result<(), sqlx::Error> {
    query!(
        r#"
        INSERT INTO leave_balances (id, employee_id, leave_type_id, year, entitlement, carried_over)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (employee_id, leave_type_id, year)
        DO UPDATE SET
            entitlement = EXCLUDED.entitlement,
            carried_over = EXCLUDED.carried_over,
            updated_at = CURRENT_TIMESTAMP
        "#,
        balance.id,
        balance.employee_id,
        balance.leave_type_id,
        balance.year,
        balance.entitlement,
        balance.carried_over
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn adjust_balance(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
    leave_type_id: Uuid,
    amount: rust_decimal::Decimal,
    transaction_date: chrono::NaiveDate,
    reason: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Log the adjustment
    sqlx::query!(
        r#"
        INSERT INTO leave_accrual_logs (workspace_id, employee_id, policy_assignment_id, accrual_date, amount, leave_type_id, notes)
        VALUES ($1, $2, NULL, $3, $4, $5, $6)
        "#,
        workspace_id,
        employee_id,
        transaction_date,
        amount,
        leave_type_id,
        reason
    )
    .execute(&mut *tx)
    .await?;

    let year = transaction_date.year();

    sqlx::query!(
        r#"
        INSERT INTO leave_balances (employee_id, leave_type_id, year, entitlement, accrued_amount, carried_over)
        VALUES ($1, $2, $3, 0, $4, 0)
        ON CONFLICT (employee_id, leave_type_id, year)
        DO UPDATE SET
            accrued_amount = leave_balances.accrued_amount + EXCLUDED.accrued_amount,
            updated_at = CURRENT_TIMESTAMP
        "#,
        employee_id,
        leave_type_id,
        year,
        amount
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
