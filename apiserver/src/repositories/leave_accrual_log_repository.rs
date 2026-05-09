use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::LeaveAccrualLog;

// ---------------------------------------------------------------------------
// Trait: LeaveAccrualLogRepository — DIP abstraction for accrual log persistence
// ---------------------------------------------------------------------------
#[async_trait]
pub trait LeaveAccrualLogRepository: super::PoolBacked + Send + Sync {
    /// Find an existing accrual log for a specific employee/leave-type/date.
    async fn find_by_employee_date_type(
        &self,
        employee_id: Uuid,
        leave_type_id: Uuid,
        accrual_date: NaiveDate,
        workspace_id: Uuid,
    ) -> Result<Option<ExistingAccrualLog>, sqlx::Error>;

    /// Fetch the `effective_from` date for a leave policy assignment.
    async fn get_assignment_effective_from(
        &self,
        assignment_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<NaiveDate, sqlx::Error>;

    /// Fetch the workspace timezone string.
    async fn get_workspace_timezone(&self, workspace_id: Uuid) -> Result<String, sqlx::Error>;
}

/// Minimal view of an existing accrual log (only the fields the orchestrator needs).
pub struct ExistingAccrualLog {
    pub id: Uuid,
    pub amount: Decimal,
}

// ---------------------------------------------------------------------------
// SqlxLeaveAccrualLogRepository — production implementation
// ---------------------------------------------------------------------------
pub struct SqlxLeaveAccrualLogRepository {
    pub pool: PgPool,
}

impl SqlxLeaveAccrualLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LeaveAccrualLogRepository for SqlxLeaveAccrualLogRepository {
    async fn find_by_employee_date_type(
        &self,
        employee_id: Uuid,
        leave_type_id: Uuid,
        accrual_date: NaiveDate,
        workspace_id: Uuid,
    ) -> Result<Option<ExistingAccrualLog>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, amount FROM leave_accrual_logs
            WHERE employee_id = $1 AND leave_type_id = $2 AND accrual_date = $3 AND workspace_id = $4
            "#,
            employee_id,
            leave_type_id,
            accrual_date,
            workspace_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ExistingAccrualLog {
            id: r.id,
            amount: r.amount,
        }))
    }

    async fn get_assignment_effective_from(
        &self,
        assignment_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<NaiveDate, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT effective_from FROM leave_policy_assignments WHERE id = $1 AND workspace_id = $2",
            assignment_id,
            workspace_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.effective_from)
    }

    async fn get_workspace_timezone(&self, workspace_id: Uuid) -> Result<String, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT timezone FROM workspaces WHERE id = $1",
            workspace_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.timezone)
    }
}

pub async fn update_accrual_amount(
    executor: impl sqlx::PgExecutor<'_>,
    log_id: Uuid,
    workspace_id: Uuid,
    new_amount: Decimal,
) -> Result<LeaveAccrualLog, sqlx::Error> {
    sqlx::query_as!(
        LeaveAccrualLog,
        r#"
        UPDATE leave_accrual_logs
        SET amount = $1, notes = COALESCE(notes, '') || ' (Re-calculated)'
        WHERE id = $2 AND workspace_id = $3
        RETURNING id, workspace_id, employee_id, policy_assignment_id,
                  accrual_date, amount, leave_type_id, notes,
                  created_at as "created_at!"
        "#,
        new_amount,
        log_id,
        workspace_id
    )
    .fetch_one(executor)
    .await
}

pub async fn insert_accrual_log(
    executor: impl sqlx::PgExecutor<'_>,
    workspace_id: Uuid,
    employee_id: Uuid,
    assignment_id: Option<Uuid>,
    accrual_date: NaiveDate,
    amount: Decimal,
    leave_type_id: Uuid,
) -> Result<Option<LeaveAccrualLog>, sqlx::Error> {
    sqlx::query_as!(
        LeaveAccrualLog,
        r#"
        INSERT INTO leave_accrual_logs
            (workspace_id, employee_id, policy_assignment_id, accrual_date, amount, leave_type_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (employee_id, leave_type_id, accrual_date) DO NOTHING
        RETURNING id, workspace_id, employee_id, policy_assignment_id,
                  accrual_date, amount, leave_type_id, notes,
                  created_at as "created_at!"
        "#,
        workspace_id,
        employee_id,
        assignment_id,
        accrual_date,
        amount,
        leave_type_id
    )
    .fetch_optional(executor)
    .await
}

pub async fn upsert_balance_accrual(
    executor: impl sqlx::PgExecutor<'_>,
    employee_id: Uuid,
    year: i32,
    leave_type_id: Uuid,
    amount: Decimal,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO leave_balances (employee_id, year, leave_type_id, entitlement, accrued_amount, carried_over)
        VALUES ($1, $2, $3, 0, $4, 0)
        ON CONFLICT (employee_id, leave_type_id, year)
        DO UPDATE SET
            accrued_amount = leave_balances.accrued_amount + EXCLUDED.accrued_amount,
            updated_at = NOW()
        "#,
        employee_id,
        year,
        leave_type_id,
        amount
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn adjust_balance_accrual(
    executor: impl sqlx::PgExecutor<'_>,
    employee_id: Uuid,
    year: i32,
    leave_type_id: Uuid,
    diff: Decimal,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE leave_balances
        SET accrued_amount = accrued_amount + $3, updated_at = NOW()
        WHERE employee_id = $1 AND year = $2 AND leave_type_id = $4
        "#,
        employee_id,
        year,
        diff,
        leave_type_id
    )
    .execute(executor)
    .await?;
    Ok(())
}

impl super::PoolBacked for SqlxLeaveAccrualLogRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self {
        Self::new(pool)
    }
    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::PoolBacked;

    #[tokio::test]
    async fn test_pool_backed() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://fake:fake@localhost:5432/fake")
            .unwrap();
        let repo = SqlxLeaveAccrualLogRepository::from_pool(pool);
        assert!(!repo.pool().is_closed());
    }
}
