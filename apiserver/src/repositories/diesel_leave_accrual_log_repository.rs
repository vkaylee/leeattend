use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::repositories::leave_accrual_log_repository::*;

use crate::models::diesel::leave::LeaveAccrualLogEntity;
use crate::models::LeaveAccrualLog;
use crate::schema::*;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

pub struct DieselLeaveAccrualLogRepository {
    pub diesel_pools: DieselPools,
}

impl DieselLeaveAccrualLogRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
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
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
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
            amount: Decimal::from_str(&amount.to_string()).unwrap_or(Decimal::ZERO),
        }))
    }

    async fn get_assignment_effective_from(
        &self,
        assignment_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<NaiveDate, AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
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
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
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
    ) -> Result<Vec<LeaveHistoryLogItem>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut query = leave_accrual_logs::table
            .inner_join(leave_types::table)
            .inner_join(employees::table)
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(leave_accrual_logs::employee_id.eq(employee_id))
            .into_boxed();

        if let Some(lt_id) = leave_type_id {
            query = query.filter(leave_accrual_logs::leave_type_id.eq(lt_id));
        }

        query = query.filter(
            diesel::dsl::sql::<diesel::sql_types::Bool>("EXTRACT(YEAR FROM accrual_date) = ")
                .bind::<diesel::sql_types::Double, _>(year as f64),
        );

        let entities = query
            .select((
                leave_accrual_logs::id,
                leave_accrual_logs::accrual_date,
                leave_accrual_logs::amount,
                leave_types::name,
                leave_accrual_logs::notes,
                leave_accrual_logs::created_at,
                leave_accrual_logs::policy_assignment_id,
            ))
            .load::<(
                Uuid,
                NaiveDate,
                bigdecimal::BigDecimal,
                String,
                Option<String>,
                Option<DateTime<Utc>>,
                Option<Uuid>,
            )>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities
            .into_iter()
            .map(|row| LeaveHistoryLogItem {
                id: row.0,
                accrual_date: row.1,
                amount: Decimal::from_str(&row.2.to_string()).unwrap_or(Decimal::ZERO),
                leave_type_name: row.3,
                notes: row.4,
                created_at: row.5,
                policy_assignment_id: row.6,
            })
            .collect())
    }

    async fn update_accrual_amount(
        &self,
        log_id: Uuid,
        workspace_id: Uuid,
        new_amount: Decimal,
    ) -> Result<LeaveAccrualLog, AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let amount_bd = bigdecimal::BigDecimal::from_str(&new_amount.to_string())
            .map_err(|e| AppError::Internal(format!("BigDecimal conversion failed: {e}")))?;

        let entity = diesel::update(leave_accrual_logs::table)
            .filter(leave_accrual_logs::id.eq(log_id))
            .filter(leave_accrual_logs::workspace_id.eq(workspace_id))
            .set((
                leave_accrual_logs::amount.eq(amount_bd),
                leave_accrual_logs::notes.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >(
                    "COALESCE(notes, '') || ' (Re-calculated)'"
                )),
            ))
            .get_result::<LeaveAccrualLogEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.into())
    }

    async fn insert_accrual_log(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        assignment_id: Option<Uuid>,
        accrual_date: NaiveDate,
        amount: Decimal,
        leave_type_id: Uuid,
    ) -> Result<Option<LeaveAccrualLog>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let amount_bd = bigdecimal::BigDecimal::from_str(&amount.to_string())
            .map_err(|e| AppError::Internal(format!("BigDecimal conversion failed: {e}")))?;

        let res = diesel::insert_into(leave_accrual_logs::table)
            .values((
                leave_accrual_logs::id.eq(Uuid::new_v4()),
                leave_accrual_logs::workspace_id.eq(workspace_id),
                leave_accrual_logs::employee_id.eq(employee_id),
                leave_accrual_logs::policy_assignment_id.eq(assignment_id),
                leave_accrual_logs::accrual_date.eq(accrual_date),
                leave_accrual_logs::amount.eq(amount_bd),
                leave_accrual_logs::leave_type_id.eq(leave_type_id),
            ))
            .on_conflict((
                leave_accrual_logs::employee_id,
                leave_accrual_logs::leave_type_id,
                leave_accrual_logs::accrual_date,
            ))
            .do_nothing()
            .get_result::<LeaveAccrualLogEntity>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(res.map(Into::into))
    }

    async fn upsert_balance_accrual(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
        leave_type_id: Uuid,
        amount: Decimal,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let amount_bd = bigdecimal::BigDecimal::from_str(&amount.to_string())
            .map_err(|e| AppError::Internal(format!("BigDecimal conversion failed: {e}")))?;

        diesel::insert_into(leave_balances::table)
            .values((
                leave_balances::id.eq(Uuid::new_v4()),
                leave_balances::employee_id.eq(employee_id),
                leave_balances::year.eq(year),
                leave_balances::leave_type_id.eq(leave_type_id),
                leave_balances::entitlement.eq(0),
                leave_balances::accrued_amount.eq(&amount_bd),
                leave_balances::carried_over.eq(0),
            ))
            .on_conflict((
                leave_balances::employee_id,
                leave_balances::leave_type_id,
                leave_balances::year,
            ))
            .do_update()
            .set((
                leave_balances::accrued_amount.eq(leave_balances::accrued_amount + &amount_bd),
                leave_balances::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn adjust_balance_accrual(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
        leave_type_id: Uuid,
        diff: Decimal,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let diff_bd = bigdecimal::BigDecimal::from_str(&diff.to_string())
            .map_err(|e| AppError::Internal(format!("BigDecimal conversion failed: {e}")))?;

        diesel::update(leave_balances::table)
            .filter(leave_balances::employee_id.eq(employee_id))
            .filter(leave_balances::year.eq(year))
            .filter(leave_balances::leave_type_id.eq(leave_type_id))
            .set((
                leave_balances::accrued_amount.eq(leave_balances::accrued_amount + &diff_bd),
                leave_balances::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }
}
