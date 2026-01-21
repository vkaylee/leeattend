use crate::models::{EmployeeLeaveBalance, LeaveBalance, WorkspaceLeaveBalance};
use chrono::Datelike;
use sqlx::{query, query_as, PgPool};
use uuid::Uuid;

pub async fn get_employee_balances(
    pool: &PgPool,
    employee_id: Uuid,
    year: i32,
) -> Result<Vec<EmployeeLeaveBalance>, sqlx::Error> {
    // We calculate "used" days by summing up approved leave requests in the given year
    // Note: This is a simple implementation that counts days between start and end.
    // In a production system, you'd need to exclude weekends or holidays depending on policy.

    let balances = query_as!(
        EmployeeLeaveBalance,
        r#"
        SELECT 
            lb.leave_type as "leave_type!",
            lb.entitlement as "entitlement!",
            lb.accrued_amount::FLOAT8 as "accrued_amount!",
            lb.carried_over as "carried_over!",
            COALESCE(
                (
                    SELECT SUM(end_date - start_date + 1)
                    FROM leave_requests
                    WHERE employee_id = lb.employee_id 
                    AND leave_type = lb.leave_type
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
                        AND leave_type = lb.leave_type
                        AND status = 'approved'
                        AND EXTRACT(YEAR FROM start_date) = lb.year
                    ), 
                    0
                )
            )::FLOAT8 as "remaining!"
        FROM leave_balances lb
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
) -> Result<Vec<WorkspaceLeaveBalance>, sqlx::Error> {
    let balances = query_as!(
        WorkspaceLeaveBalance,
        r#"
        SELECT 
            e.id as "employee_id!",
            (e.first_name || ' ' || e.last_name) as "employee_name!",
            COALESCE(lb.leave_type, 'vacation') as "leave_type!",
            COALESCE(lb.entitlement, 0) as "entitlement!",
            COALESCE(lb.accrued_amount::FLOAT8, 0.0) as "accrued_amount!",
            COALESCE(lb.carried_over, 0) as "carried_over!",
            COALESCE(
                (
                    SELECT SUM(end_date - start_date + 1)
                    FROM leave_requests
                    WHERE employee_id = e.id 
                    AND leave_type = COALESCE(lb.leave_type, 'vacation')
                    AND status = 'approved'
                    AND EXTRACT(YEAR FROM start_date) = $2
                ), 
                0
            )::INT as "used!",
            (
                COALESCE(lb.entitlement, 0) + COALESCE(lb.accrued_amount, 0) + COALESCE(lb.carried_over, 0) - COALESCE(
                    (
                        SELECT SUM(end_date - start_date + 1)
                        FROM leave_requests
                        WHERE employee_id = e.id 
                        AND leave_type = COALESCE(lb.leave_type, 'vacation')
                        AND status = 'approved'
                        AND EXTRACT(YEAR FROM start_date) = $2
                    ), 
                    0
                )
            )::FLOAT8 as "remaining!",
            (
                SELECT d.name 
                FROM leave_policy_assignments a
                JOIN leave_policy_definitions d ON a.policy_id = d.id
                WHERE a.workspace_id = $1
                  AND d.leave_type = COALESCE(lb.leave_type, 'vacation')
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
            ) as "policy_name",
            (
                SELECT a.assignee_type 
                FROM leave_policy_assignments a
                JOIN leave_policy_definitions d ON a.policy_id = d.id
                WHERE a.workspace_id = $1
                  AND d.leave_type = COALESCE(lb.leave_type, 'vacation')
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
            ) as "assignment_type"
        FROM employees e
        LEFT JOIN leave_balances lb ON e.id = lb.employee_id AND lb.year = $2
        WHERE e.workspace_id = $1 AND e.is_active = TRUE
        ORDER BY e.first_name, e.last_name, lb.leave_type
        "#,
        workspace_id,
        year
    )
    .fetch_all(pool)
    .await?;

    Ok(balances)
}

pub async fn upsert_leave_balance(
    pool: &PgPool,
    balance: &LeaveBalance,
) -> Result<(), sqlx::Error> {
    query!(
        r#"
        INSERT INTO leave_balances (id, employee_id, leave_type, year, entitlement, carried_over)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (employee_id, leave_type, year)
        DO UPDATE SET 
            entitlement = EXCLUDED.entitlement,
            carried_over = EXCLUDED.carried_over,
            updated_at = CURRENT_TIMESTAMP
        "#,
        balance.id,
        balance.employee_id,
        balance.leave_type,
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
    leave_type: &str,
    amount: rust_decimal::Decimal,
    transaction_date: chrono::NaiveDate,
    reason: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // 1. Log the adjustment (is_manual implied by null policy_id or explicit flag if we had one)
    sqlx::query!(
        r#"
        INSERT INTO leave_accrual_logs (workspace_id, employee_id, policy_assignment_id, accrual_date, amount, leave_type, notes)
        VALUES ($1, $2, NULL, $3, $4, $5, $6)
        "#,
        workspace_id,
        employee_id,
        transaction_date,
        amount,
        leave_type,
        reason
    )
    .execute(&mut *tx)
    .await?;

    // 2. Update the balance
    // We treat 'adjustment' as modifying 'entitlement' for simplicity in this model,
    // or we could have a separate 'adjustments' column.
    // Given the current schema has `entitlement` and `accrued_amount` (from previous migrations),
    // let's check `leave_balances` schema again.
    // Based on previous context, `leave_balances` has `entitlement`, `accrued_amount`, `carried_over`.
    // Let's add to `accrued_amount` as that's the most logical place for dynamic additions.

    // However, looking at the `upsert_leave_balance` above, it only updates `entitlement` and `carried_over`.
    // Wait, I recall a migration added `accrued_amount`. Let me verify `models.rs` doesn't show it in `LeaveBalance` struct yet?
    // Checking `models.rs`... `LeaveBalance` has `entitlement` and `carried_over`.
    // Ah, `LeaveAccrualService` uses `accrued_amount`.
    // I need to be consistent. If `accrued_amount` exists in DB, I should use it.
    // The previous `upsert_leave_balance` in this file might be outdated or for initial setup.

    // Let's assume `accrued_amount` exists as per `LeaveAccrualService`.
    // I will update `accrued_amount` + amount.

    let year = transaction_date.year();

    sqlx::query!(
        r#"
        INSERT INTO leave_balances (employee_id, leave_type, year, entitlement, accrued_amount, carried_over)
        VALUES ($1, $2, $3, 0, $4, 0)
        ON CONFLICT (employee_id, leave_type, year)
        DO UPDATE SET 
            accrued_amount = leave_balances.accrued_amount + EXCLUDED.accrued_amount,
            updated_at = CURRENT_TIMESTAMP
        "#,
        employee_id,
        leave_type,
        year,
        amount
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
