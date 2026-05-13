use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::repositories::leave_accrual_log_repository::*;

use crate::schema::*;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

pub struct DieselLeaveAccrualLogRepository {
    pub sqlx_pool: PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselLeaveAccrualLogRepository {
    pub fn new(sqlx_pool: PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl LeaveAccrualLogRepository for DieselLeaveAccrualLogRepository {
    async fn find_by_employee_date_type(
        &self,
        employee_id: Uuid,
        leave_type_id: Uuid,
        accrual_date: NaiveDate,
        workspace_id: Uuid,
    ) -> Result<Option<ExistingAccrualLog>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let row = leave_accrual_logs::table
            .filter(leave_accrual_logs::employee_id.eq(employee_id))
            .filter(leave_accrual_logs::leave_type_id.eq(leave_type_id))
            .filter(leave_accrual_logs::accrual_date.eq(accrual_date))
            .filter(leave_accrual_logs::workspace_id.eq(workspace_id))
            .select((leave_accrual_logs::id, leave_accrual_logs::amount))
            .first::<(Uuid, bigdecimal::BigDecimal)>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(row.map(|(id, amount)| ExistingAccrualLog {
            id,
            amount: Decimal::from_str(&amount.to_string()).unwrap_or_default(),
        }))
    }

    async fn get_assignment_effective_from(
        &self,
        assignment_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<NaiveDate, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let date = leave_policy_assignments::table
            .filter(leave_policy_assignments::id.eq(assignment_id))
            .filter(leave_policy_assignments::workspace_id.eq(workspace_id))
            .select(leave_policy_assignments::effective_from)
            .first::<NaiveDate>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(date)
    }

    async fn get_workspace_timezone(&self, workspace_id: Uuid) -> Result<String, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let tz = workspaces::table
            .filter(workspaces::id.eq(workspace_id))
            .select(workspaces::timezone)
            .first::<String>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(tz)
    }

    async fn get_employee_leave_history_logs(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
        leave_type_id: Option<Uuid>,
    ) -> Result<Vec<crate::repositories::leave_accrual_log_repository::LeaveHistoryLogItem>, AppError>
    {
        let rows = sqlx::query!(
            r#"
            SELECT lal.id, lal.accrual_date, lal.amount, lt.name as leave_type_name,
                   lal.notes, lal.created_at, lal.policy_assignment_id
            FROM leave_accrual_logs lal
            JOIN leave_types lt ON lal.leave_type_id = lt.id
            JOIN employees e ON lal.employee_id = e.id
            WHERE lal.employee_id = $1
              AND e.workspace_id = $4
              AND EXTRACT(YEAR FROM lal.accrual_date) = $2
              AND (lt.id = $3 OR $3 IS NULL)
            "#,
            employee_id,
            year as f64,
            leave_type_id,
            workspace_id
        )
        .fetch_all(&self.sqlx_pool)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows
            .into_iter()
            .map(
                |row| crate::repositories::leave_accrual_log_repository::LeaveHistoryLogItem {
                    id: row.id,
                    accrual_date: row.accrual_date,
                    amount: row.amount,
                    leave_type_name: row.leave_type_name,
                    notes: row.notes,
                    created_at: row.created_at,
                    policy_assignment_id: row.policy_assignment_id,
                },
            )
            .collect())
    }
}
