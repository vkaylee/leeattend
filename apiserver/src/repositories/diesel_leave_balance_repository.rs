use crate::diesel_pool::DieselPools;
use crate::models::diesel::leave::*;
use crate::models::{EmployeeLeaveBalance, LeaveBalance, WorkspaceLeaveBalance};
use crate::repositories::leave_balance_repository::{
    adjust_balance, get_employee_balances, get_workspace_balances, LeaveBalanceRepository,
};
use crate::repositories::PoolBacked;
use crate::schema::*;
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

pub struct DieselLeaveBalanceRepository {
    pub sqlx_pool: PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselLeaveBalanceRepository {
    pub fn new(sqlx_pool: PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

impl PoolBacked for DieselLeaveBalanceRepository {
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    fn from_pool(_pool: PgPool) -> Self {
        panic!(
            "DieselLeaveBalanceRepository requires diesel_pools to be manually injected via new()"
        );
    }

    fn pool(&self) -> &PgPool {
        &self.sqlx_pool
    }
}

#[async_trait]
impl LeaveBalanceRepository for DieselLeaveBalanceRepository {
    async fn get_employee_balances(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
    ) -> Result<Vec<EmployeeLeaveBalance>, sqlx::Error> {
        get_employee_balances(&self.sqlx_pool, workspace_id, employee_id, year).await
    }

    async fn get_workspace_balances(
        &self,
        workspace_id: Uuid,
        year: i32,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<WorkspaceLeaveBalance>, i64), sqlx::Error> {
        get_workspace_balances(&self.sqlx_pool, workspace_id, year, limit, offset, search).await
    }

    async fn upsert_leave_balance(&self, balance: &LeaveBalance) -> Result<(), sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let entity = LeaveBalanceEntity {
            id: balance.id,
            employee_id: balance.employee_id,
            year: balance.year,
            entitlement: balance.entitlement,
            carried_over: balance.carried_over,
            created_at: Some(balance.created_at),
            updated_at: Some(balance.updated_at),
            accrued_amount: BigDecimal::from_str(&balance.accrued_amount.to_string())
                .unwrap_or_default(),
            leave_type_id: balance.leave_type_id,
        };

        diesel::insert_into(leave_balances::table)
            .values(&entity)
            .on_conflict((
                leave_balances::employee_id,
                leave_balances::leave_type_id,
                leave_balances::year,
            ))
            .do_update()
            .set((
                leave_balances::entitlement.eq(entity.entitlement),
                leave_balances::carried_over.eq(entity.carried_over),
                leave_balances::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(())
    }

    async fn adjust_balance(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        leave_type_id: Uuid,
        amount: rust_decimal::Decimal,
        transaction_date: chrono::NaiveDate,
        reason: &str,
    ) -> Result<(), sqlx::Error> {
        // Complex transaction logic, fallback to SQLx
        adjust_balance(
            &self.sqlx_pool,
            workspace_id,
            employee_id,
            leave_type_id,
            amount,
            transaction_date,
            reason,
        )
        .await
    }
}
