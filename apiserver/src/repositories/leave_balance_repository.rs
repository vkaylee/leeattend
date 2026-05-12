use crate::errors::AppError;
use crate::models::{EmployeeLeaveBalance, LeaveBalance, WorkspaceLeaveBalance};
use async_trait::async_trait;
use chrono::Datelike;
use sqlx::{query, query_as, PgPool};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: LeaveBalanceRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait LeaveBalanceRepository: super::PoolBacked + Send + Sync {
    async fn get_employee_balances(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
    ) -> Result<Vec<EmployeeLeaveBalance>, AppError>;
    async fn get_workspace_balances(
        &self,
        workspace_id: Uuid,
        year: i32,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<WorkspaceLeaveBalance>, i64), AppError>;
    async fn upsert_leave_balance(&self, balance: &LeaveBalance) -> Result<(), AppError>;
    async fn adjust_balance(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        leave_type_id: Uuid,
        amount: rust_decimal::Decimal,
        transaction_date: chrono::NaiveDate,
        reason: &str,
    ) -> Result<(), AppError>;
}

// ---------------------------------------------------------------------------
// SqlxLeaveBalanceRepository — production implementation backed by PgPool
// ---------------------------------------------------------------------------
pub struct SqlxLeaveBalanceRepository {
    pub pool: PgPool,
}

impl SqlxLeaveBalanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LeaveBalanceRepository for SqlxLeaveBalanceRepository {
    async fn get_employee_balances(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
    ) -> Result<Vec<EmployeeLeaveBalance>, AppError> {
        get_employee_balances(&self.pool, workspace_id, employee_id, year).await.map_err(Into::into)
    }
    async fn get_workspace_balances(
        &self,
        workspace_id: Uuid,
        year: i32,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<WorkspaceLeaveBalance>, i64), AppError> {
        get_workspace_balances(&self.pool, workspace_id, year, limit, offset, search).await.map_err(Into::into)
    }
    async fn upsert_leave_balance(&self, balance: &LeaveBalance) -> Result<(), AppError> {
        upsert_leave_balance(&self.pool, balance).await.map_err(Into::into)
    }
    async fn adjust_balance(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        leave_type_id: Uuid,
        amount: rust_decimal::Decimal,
        transaction_date: chrono::NaiveDate,
        reason: &str,
    ) -> Result<(), AppError> {
        adjust_balance(
            &self.pool,
            workspace_id,
            employee_id,
            leave_type_id,
            amount,
            transaction_date,
            reason,
        )
        .await
        .map_err(Into::into)
    }
}

pub async fn get_employee_balances(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
    year: i32,
) -> Result<Vec<EmployeeLeaveBalance>, sqlx::Error> {
    let balances = query_as!(
        EmployeeLeaveBalance,
        r#"
        WITH leave_usage AS (
            SELECT
                lr.leave_type_id,
                SUM(
                    (SELECT COUNT(*)
                     FROM generate_series(lr.start_date, lr.end_date, '1 day'::interval) d
                     WHERE EXTRACT(DOW FROM d) NOT IN (0, 6))
                )::INT as used_days
            FROM leave_requests lr
            JOIN employees e_usage ON lr.employee_id = e_usage.id
            WHERE lr.employee_id = $1
              AND e_usage.workspace_id = $3
              AND lr.status IN ('approved', 'pre_approved')
              AND EXTRACT(YEAR FROM lr.start_date)::INT = $2
            GROUP BY lr.leave_type_id
        )
        SELECT
            lb.leave_type_id as "leave_type_id!",
            lt.code as "leave_type_code!",
            lt.name as "leave_type_name!",
            lt.is_paid as "is_paid!",
            lb.entitlement as "entitlement!",
            lb.accrued_amount::FLOAT8 as "accrued_amount!",
            lb.carried_over as "carried_over!",
            COALESCE(lu.used_days, 0)::INT as "used!",
            (
                lb.entitlement + lb.accrued_amount + lb.carried_over
                - COALESCE(lu.used_days, 0)
            )::FLOAT8 as "remaining!"
        FROM leave_balances lb
        JOIN leave_types lt ON lb.leave_type_id = lt.id
        JOIN employees e ON lb.employee_id = e.id
        LEFT JOIN leave_usage lu ON lb.leave_type_id = lu.leave_type_id
        WHERE lb.employee_id = $1 AND lb.year = $2 AND e.workspace_id = $3
        "#,
        employee_id,
        year,
        workspace_id
    )
    .fetch_all(pool)
    .await?;

    Ok(balances)
}

#[allow(clippy::too_many_lines)]
pub async fn get_workspace_balances(
    pool: &PgPool,
    workspace_id: Uuid,
    year: i32,
    limit: i64,
    offset: i64,
    search: Option<String>,
) -> Result<(Vec<WorkspaceLeaveBalance>, i64), sqlx::Error> {
    let search_term = format!("%{}%", search.unwrap_or_default());

    // Get total count (per employee-leave_type pair, only those with balance or policy)
    let count_sql = r#"
        SELECT COUNT(*)
        FROM employees e
        CROSS JOIN leave_types lt
        LEFT JOIN leave_balances lb ON e.id = lb.employee_id AND lt.id = lb.leave_type_id AND lb.year = $2
        WHERE e.workspace_id = $1 AND e.is_active = TRUE AND lt.workspace_id = $1
          AND ($3 = '%%' OR e.full_name ILIKE $3 OR e.employee_id_number ILIKE $3 OR lt.name ILIKE $3 OR lt.code ILIKE $3)
          AND (
            lb.employee_id IS NOT NULL
            OR EXISTS (
              SELECT 1 FROM leave_policy_assignments a
              JOIN leave_policy_definitions d ON a.policy_id = d.id
              WHERE d.leave_type_id = lt.id AND a.workspace_id = $1
                AND a.effective_from <= NOW()
                AND (a.effective_to IS NULL OR a.effective_to > NOW())
                AND (
                  (a.assignee_type = 'employee' AND a.assignee_id = e.id)
                  OR (a.assignee_type = 'workspace')
                  OR (a.assignee_type = 'department' AND a.assignee_id = (
                    SELECT department_id FROM employee_department_history
                    WHERE employee_id = e.id ORDER BY effective_date DESC LIMIT 1
                  ))
                )
            )
          )
    "#;

    let total: i64 = sqlx::query_scalar(count_sql)
        .bind(workspace_id)
        .bind(year)
        .bind(&search_term)
        .fetch_one(pool)
        .await?;

    // Optimized query with CTEs to avoid duplicated correlated subqueries
    let sql = r#"
        WITH leave_usage AS (
            -- Pre-calculate used working days per employee/leave_type (exclude weekends)
            SELECT
                lr.employee_id,
                lr.leave_type_id,
                SUM(
                    (SELECT COUNT(*)
                     FROM generate_series(lr.start_date, lr.end_date, '1 day'::interval) d
                     WHERE EXTRACT(DOW FROM d) NOT IN (0, 6))
                )::INT as used_days
            FROM leave_requests lr
            WHERE lr.status IN ('approved', 'pre_approved')
              AND EXTRACT(YEAR FROM lr.start_date)::INT = $2
            GROUP BY lr.employee_id, lr.leave_type_id
        ),
        best_policy AS (
            -- Pre-calculate best policy per leave_type/employee (priority: employee > department > workspace)
            SELECT DISTINCT ON (lt_id, emp_id)
                a.assignee_id as emp_id_raw,
                d.leave_type_id as lt_id,
                e_inner.id as emp_id,
                d.name as policy_name,
                a.assignee_type as assignment_type
            FROM leave_policy_assignments a
            JOIN leave_policy_definitions d ON a.policy_id = d.id
            CROSS JOIN employees e_inner
            WHERE a.workspace_id = $1
              AND e_inner.workspace_id = $1
              AND e_inner.is_active = TRUE
              AND (
                (a.assignee_type = 'employee' AND a.assignee_id = e_inner.id) OR
                (a.assignee_type = 'department' AND a.assignee_id = (
                    SELECT department_id FROM employee_department_history
                    WHERE employee_id = e_inner.id ORDER BY effective_date DESC LIMIT 1
                )) OR
                (a.assignee_type = 'workspace')
              )
              AND a.effective_from <= NOW()
              AND (a.effective_to IS NULL OR a.effective_to > NOW())
            ORDER BY lt_id, emp_id,
              CASE a.assignee_type
                WHEN 'employee' THEN 1
                WHEN 'department' THEN 2
                WHEN 'workspace' THEN 3
              END ASC
        )
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
            COALESCE(lu.used_days, 0) as used,
            (
                COALESCE(lb.entitlement, 0) + COALESCE(lb.accrued_amount, 0) + COALESCE(lb.carried_over, 0)
                - COALESCE(lu.used_days, 0)
            )::FLOAT8 as remaining,
            bp.policy_name as policy_name,
            bp.assignment_type as assignment_type
        FROM employees e
        CROSS JOIN leave_types lt
        LEFT JOIN leave_balances lb ON e.id = lb.employee_id AND lt.id = lb.leave_type_id AND lb.year = $2
        LEFT JOIN leave_usage lu ON e.id = lu.employee_id AND lt.id = lu.leave_type_id
        LEFT JOIN best_policy bp ON e.id = bp.emp_id AND lt.id = bp.lt_id
        WHERE e.workspace_id = $1 AND e.is_active = TRUE AND lt.workspace_id = $1
          AND ($3 = '%%' OR e.full_name ILIKE $3 OR e.employee_id_number ILIKE $3 OR lt.name ILIKE $3 OR lt.code ILIKE $3)
          AND (lb.employee_id IS NOT NULL OR bp.emp_id IS NOT NULL)
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

impl super::PoolBacked for SqlxLeaveBalanceRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self {
        Self::new(pool)
    }
    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}
