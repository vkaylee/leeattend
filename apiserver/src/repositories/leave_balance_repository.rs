use sqlx::{query, query_as, PgPool};
use uuid::Uuid;
use crate::models::{LeaveBalance, EmployeeLeaveBalance};

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
            lb.leave_type,
            lb.entitlement,
            lb.carried_over,
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
                lb.entitlement + lb.carried_over - COALESCE(
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
            )::INT as "remaining!"
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
