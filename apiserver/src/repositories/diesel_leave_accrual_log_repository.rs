use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::repositories::leave_accrual_log_repository::*;

use crate::models::diesel::leave::LeaveAccrualLogEntity;
use crate::models::LeaveAccrualLog;
use crate::schema::*;
use async_trait::async_trait;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

pub struct DieselLeaveAccrualLogRepository {
    diesel_pools: DieselPools,
}

impl DieselLeaveAccrualLogRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[allow(clippy::too_many_lines)]
#[async_trait]
impl LeaveAccrualLogRepository for DieselLeaveAccrualLogRepository {
    async fn find_by_employment_date_type(
        &self,
        employment_id: Uuid,
        leave_type_id: Uuid,
        accrual_date: NaiveDate,
        workspace_id: Uuid,
    ) -> Result<Option<ExistingAccrualLog>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let row = leave_accrual_logs::table
            .filter(leave_accrual_logs::employment_id.eq(employment_id))
            .filter(leave_accrual_logs::leave_type_id.eq(leave_type_id))
            .filter(leave_accrual_logs::accrual_date.eq(accrual_date))
            .filter(leave_accrual_logs::workspace_id.eq(workspace_id))
            .select((leave_accrual_logs::id, leave_accrual_logs::amount))
            .first::<(Uuid, bigdecimal::BigDecimal)>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        let row = row
            .map(|(id, amount)| -> Result<_, AppError> {
                Ok(ExistingAccrualLog {
                    id,
                    amount: Decimal::from_str(&amount.to_string())
                        .map_err(|e| AppError::Internal(format!("Decimal parse failed: {e}")))?,
                })
            })
            .transpose()?;

        Ok(row)
    }

    async fn get_assignment_effective_from(
        &self,
        assignment_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<NaiveDate, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let date = leave_policy_assignments::table
            .filter(leave_policy_assignments::id.eq(assignment_id))
            .filter(leave_policy_assignments::workspace_id.eq(workspace_id))
            .filter(leave_policy_assignments::deleted_at.is_null())
            .select(leave_policy_assignments::effective_from)
            .first::<NaiveDate>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(date)
    }

    async fn get_workspace_timezone(&self, workspace_id: Uuid) -> Result<String, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let tz = workspaces::table
            .filter(workspaces::id.eq(workspace_id))
            .select(workspaces::timezone)
            .first::<String>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(tz)
    }

    async fn get_employment_leave_history_logs(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        year: i32,
        leave_type_id: Option<Uuid>,
    ) -> Result<Vec<LeaveHistoryLogItem>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = leave_accrual_logs::table
            .inner_join(leave_types::table)
            .inner_join(employments::table)
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(leave_accrual_logs::employment_id.eq(employment_id))
            .into_boxed();

        if let Some(lt_id) = leave_type_id {
            query = query.filter(leave_accrual_logs::leave_type_id.eq(lt_id));
        }

        query = query.filter(
            diesel::dsl::sql::<diesel::sql_types::Bool>("EXTRACT(YEAR FROM accrual_date) = ")
                .bind::<diesel::sql_types::Double, _>(f64::from(year)),
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
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities
            .into_iter()
            .map(|row| -> Result<_, AppError> {
                Ok(LeaveHistoryLogItem {
                    id: row.0,
                    accrual_date: row.1,
                    amount: Decimal::from_str(&row.2.to_string())
                        .map_err(|e| AppError::Internal(format!("Decimal parse failed: {e}")))?,
                    leave_type_name: row.3,
                    notes: row.4,
                    created_at: row.5,
                    policy_assignment_id: row.6,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?)
    }

    async fn get_employment_leave_history_logs_batch(
        &self,
        workspace_id: Uuid,
        employment_ids: &[Uuid],
        year: i32,
        leave_type_id: Option<Uuid>,
    ) -> Result<Vec<LeaveHistoryLogItem>, AppError> {
        if employment_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = leave_accrual_logs::table
            .inner_join(leave_types::table)
            .inner_join(employments::table)
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(leave_accrual_logs::employment_id.eq_any(employment_ids))
            .into_boxed();

        if let Some(lt_id) = leave_type_id {
            query = query.filter(leave_accrual_logs::leave_type_id.eq(lt_id));
        }

        query = query.filter(
            diesel::dsl::sql::<diesel::sql_types::Bool>("EXTRACT(YEAR FROM accrual_date) = ")
                .bind::<diesel::sql_types::Double, _>(f64::from(year)),
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
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities
            .into_iter()
            .map(|row| -> Result<_, AppError> {
                Ok(LeaveHistoryLogItem {
                    id: row.0,
                    accrual_date: row.1,
                    amount: Decimal::from_str(&row.2.to_string())
                        .map_err(|e| AppError::Internal(format!("Decimal parse failed: {e}")))?,
                    leave_type_name: row.3,
                    notes: row.4,
                    created_at: row.5,
                    policy_assignment_id: row.6,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?)
    }

    async fn update_accrual_amount(
        &self,
        log_id: Uuid,
        workspace_id: Uuid,
        new_amount: Decimal,
    ) -> Result<LeaveAccrualLog, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

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
            .get_result::<LeaveAccrualLogEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(entity.into())
    }

    async fn insert_accrual_log(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        assignment_id: Option<Uuid>,
        accrual_date: NaiveDate,
        amount: Decimal,
        leave_type_id: Uuid,
    ) -> Result<Option<LeaveAccrualLog>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let amount_bd = bigdecimal::BigDecimal::from_str(&amount.to_string())
            .map_err(|e| AppError::Internal(format!("BigDecimal conversion failed: {e}")))?;

        let res = diesel::insert_into(leave_accrual_logs::table)
            .values((
                leave_accrual_logs::id.eq(Uuid::new_v4()),
                leave_accrual_logs::workspace_id.eq(workspace_id),
                leave_accrual_logs::employment_id.eq(employment_id),
                leave_accrual_logs::policy_assignment_id.eq(assignment_id),
                leave_accrual_logs::accrual_date.eq(accrual_date),
                leave_accrual_logs::amount.eq(amount_bd),
                leave_accrual_logs::leave_type_id.eq(leave_type_id),
            ))
            .on_conflict((
                leave_accrual_logs::employment_id,
                leave_accrual_logs::leave_type_id,
                leave_accrual_logs::accrual_date,
            ))
            .do_nothing()
            .get_result::<LeaveAccrualLogEntity>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(res.map(Into::into))
    }

    async fn upsert_balance_accrual(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        year: i32,
        leave_type_id: Uuid,
        amount: Decimal,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let amount_bd = bigdecimal::BigDecimal::from_str(&amount.to_string())
            .map_err(|e| AppError::Internal(format!("BigDecimal conversion failed: {e}")))?;

        diesel::insert_into(leave_balances::table)
            .values((
                leave_balances::id.eq(Uuid::new_v4()),
                leave_balances::employment_id.eq(employment_id),
                leave_balances::year.eq(year),
                leave_balances::leave_type_id.eq(leave_type_id),
                leave_balances::entitlement.eq(0),
                leave_balances::accrued_amount.eq(&amount_bd),
                leave_balances::carried_over.eq(0),
            ))
            .on_conflict((
                leave_balances::employment_id,
                leave_balances::leave_type_id,
                leave_balances::year,
            ))
            .do_update()
            .set((
                leave_balances::accrued_amount.eq(leave_balances::accrued_amount + &amount_bd),
                leave_balances::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    async fn adjust_balance_accrual(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        year: i32,
        leave_type_id: Uuid,
        diff: Decimal,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let diff_bd = bigdecimal::BigDecimal::from_str(&diff.to_string())
            .map_err(|e| AppError::Internal(format!("BigDecimal conversion failed: {e}")))?;

        diesel::update(leave_balances::table)
            .filter(leave_balances::employment_id.eq(employment_id))
            .filter(leave_balances::year.eq(year))
            .filter(leave_balances::leave_type_id.eq(leave_type_id))
            .set((
                leave_balances::accrued_amount.eq(leave_balances::accrued_amount + &diff_bd),
                leave_balances::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    async fn persist_accrual(
        &self,
        params: PersistAccrualParams<'_>,
    ) -> Result<Option<LeaveAccrualLog>, AppError> {
        let PersistAccrualParams {
            auth_token,
            workspace_id,
            employment_id,
            assignment_id,
            leave_type_id,
            accrual_date,
            amount,
            force,
        } = params;
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        use rust_decimal::prelude::FromPrimitive;
        let decimal_amount = rust_decimal::Decimal::from_f64(amount)
            .ok_or_else(|| AppError::Internal("Invalid f64 for Decimal conversion".into()))?;
        let amount_bd = bigdecimal::BigDecimal::from_str(&decimal_amount.to_string())
            .map_err(|e| AppError::Internal(format!("BigDecimal conversion failed: {e}")))?;
        let year = accrual_date.year();

        conn.transaction(|conn| {
            Box::pin(async move {
                let row = leave_accrual_logs::table
                    .filter(leave_accrual_logs::employment_id.eq(employment_id))
                    .filter(leave_accrual_logs::leave_type_id.eq(leave_type_id))
                    .filter(leave_accrual_logs::accrual_date.eq(accrual_date))
                    .filter(leave_accrual_logs::workspace_id.eq(workspace_id))
                    .select((leave_accrual_logs::id, leave_accrual_logs::amount))
                    .first::<(Uuid, bigdecimal::BigDecimal)>(&mut *conn)
                    .await
                    .optional()
                    .map_err(AppError::from)?;

                if let Some((log_id, old_amount_bd)) = row {
                    let old_amount = Decimal::from_str(&old_amount_bd.to_string())
                        .map_err(|e| AppError::Internal(format!("Decimal parse failed: {e}")))?;
                    if force && old_amount != decimal_amount {
                        let diff = decimal_amount - old_amount;
                        let diff_bd =
                            bigdecimal::BigDecimal::from_str(&diff.to_string()).map_err(|e| {
                                AppError::Internal(format!("BigDecimal conversion failed: {e}"))
                            })?;

                        let entity = diesel::update(leave_accrual_logs::table)
                            .filter(leave_accrual_logs::id.eq(log_id))
                            .filter(leave_accrual_logs::workspace_id.eq(workspace_id))
                            .set((
                                leave_accrual_logs::amount.eq(amount_bd.clone()),
                                leave_accrual_logs::notes.eq(diesel::dsl::sql::<
                                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                                >(
                                    "COALESCE(notes, '') || ' (Re-calculated)'",
                                )),
                            ))
                            .get_result::<LeaveAccrualLogEntity>(&mut *conn)
                            .await
                            .map_err(AppError::from)?;

                        diesel::update(leave_balances::table)
                            .filter(leave_balances::employment_id.eq(employment_id))
                            .filter(leave_balances::year.eq(year))
                            .filter(leave_balances::leave_type_id.eq(leave_type_id))
                            .set((
                                leave_balances::accrued_amount
                                    .eq(leave_balances::accrued_amount + diff_bd),
                                leave_balances::updated_at.eq(diesel::dsl::now),
                            ))
                            .execute(&mut *conn)
                            .await
                            .map_err(AppError::from)?;

                        return Ok(Some(entity.into()));
                    }
                    Ok(None)
                } else {
                    let log_id = Uuid::new_v4();
                    let entity = diesel::insert_into(leave_accrual_logs::table)
                        .values((
                            leave_accrual_logs::id.eq(log_id),
                            leave_accrual_logs::workspace_id.eq(workspace_id),
                            leave_accrual_logs::employment_id.eq(employment_id),
                            leave_accrual_logs::policy_assignment_id.eq(assignment_id),
                            leave_accrual_logs::accrual_date.eq(accrual_date),
                            leave_accrual_logs::amount.eq(amount_bd.clone()),
                            leave_accrual_logs::leave_type_id.eq(leave_type_id),
                        ))
                        .on_conflict((
                            leave_accrual_logs::employment_id,
                            leave_accrual_logs::leave_type_id,
                            leave_accrual_logs::accrual_date,
                        ))
                        .do_nothing()
                        .get_result::<LeaveAccrualLogEntity>(&mut *conn)
                        .await
                        .optional()
                        .map_err(AppError::from)?;

                    if entity.is_some() {
                        diesel::insert_into(leave_balances::table)
                            .values((
                                leave_balances::id.eq(Uuid::new_v4()),
                                leave_balances::employment_id.eq(employment_id),
                                leave_balances::year.eq(year),
                                leave_balances::leave_type_id.eq(leave_type_id),
                                leave_balances::entitlement.eq(0),
                                leave_balances::accrued_amount.eq(amount_bd.clone()),
                                leave_balances::carried_over.eq(0),
                            ))
                            .on_conflict((
                                leave_balances::employment_id,
                                leave_balances::leave_type_id,
                                leave_balances::year,
                            ))
                            .do_update()
                            .set((
                                leave_balances::accrued_amount
                                    .eq(leave_balances::accrued_amount + amount_bd),
                                leave_balances::updated_at.eq(diesel::dsl::now),
                            ))
                            .execute(&mut *conn)
                            .await
                            .map_err(AppError::from)?;
                    }

                    Ok(entity.map(Into::into))
                }
            })
        })
        .await
    }

    async fn persist_accruals_batch(
        &self,
        params: PersistAccrualsBatchParams<'_>,
    ) -> Result<Vec<LeaveAccrualLog>, AppError> {
        let PersistAccrualsBatchParams {
            auth_token,
            workspace_id,
            entries,
            force,
        } = params;
        if entries.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        // Clone data into owned structures for the async move block
        let entries_owned: Vec<_> = entries
            .iter()
            .map(|e| {
                (
                    e.employment_id,
                    e.assignment_id,
                    e.leave_type_id,
                    e.accrual_date,
                    e.amount,
                )
            })
            .collect();

        conn.transaction(|conn| {
            Box::pin(async move {
                let mut all_logs = Vec::new();

                for (employment_id, assignment_id, leave_type_id, accrual_date, amount) in
                    &entries_owned
                {
                    let active_emp_id = *employment_id;

                    use rust_decimal::prelude::FromPrimitive;
                    let decimal_amount =
                        rust_decimal::Decimal::from_f64(*amount).ok_or_else(|| {
                            AppError::Internal("Invalid f64 for Decimal conversion".into())
                        })?;
                    let amount_bd = bigdecimal::BigDecimal::from_str(&decimal_amount.to_string())
                        .map_err(|e| {
                        AppError::Internal(format!("BigDecimal conversion failed: {e}"))
                    })?;
                    let year = accrual_date.year();

                    // Check for existing log
                    let row = leave_accrual_logs::table
                        .filter(leave_accrual_logs::employment_id.eq(employment_id))
                        .filter(leave_accrual_logs::leave_type_id.eq(leave_type_id))
                        .filter(leave_accrual_logs::accrual_date.eq(accrual_date))
                        .filter(leave_accrual_logs::workspace_id.eq(workspace_id))
                        .select((leave_accrual_logs::id, leave_accrual_logs::amount))
                        .first::<(Uuid, bigdecimal::BigDecimal)>(&mut *conn)
                        .await
                        .optional()
                        .map_err(AppError::from)?;

                    if let Some((log_id, old_amount_bd)) = row {
                        let old_amount =
                            Decimal::from_str(&old_amount_bd.to_string()).map_err(|e| {
                                AppError::Internal(format!("Decimal parse failed: {e}"))
                            })?;
                        if force && old_amount != decimal_amount {
                            let diff = decimal_amount - old_amount;
                            let diff_bd = bigdecimal::BigDecimal::from_str(&diff.to_string())
                                .map_err(|e| {
                                    AppError::Internal(format!("BigDecimal conversion failed: {e}"))
                                })?;

                            let entity = diesel::update(leave_accrual_logs::table)
                                .filter(leave_accrual_logs::id.eq(log_id))
                                .filter(leave_accrual_logs::workspace_id.eq(workspace_id))
                                .set((
                                    leave_accrual_logs::amount.eq(amount_bd.clone()),
                                    leave_accrual_logs::notes.eq(diesel::dsl::sql::<
                                        diesel::sql_types::Nullable<diesel::sql_types::Text>,
                                    >(
                                        "COALESCE(notes, '') || ' (Re-calculated)'",
                                    )),
                                ))
                                .get_result::<LeaveAccrualLogEntity>(&mut *conn)
                                .await
                                .map_err(AppError::from)?;

                            diesel::update(leave_balances::table)
                                .filter(leave_balances::employment_id.eq(active_emp_id))
                                .filter(leave_balances::year.eq(year))
                                .filter(leave_balances::leave_type_id.eq(leave_type_id))
                                .set((
                                    leave_balances::accrued_amount
                                        .eq(leave_balances::accrued_amount + diff_bd),
                                    leave_balances::updated_at.eq(diesel::dsl::now),
                                ))
                                .execute(&mut *conn)
                                .await
                                .map_err(AppError::from)?;

                            all_logs.push(entity.into());
                        }
                        // else: already exists with same amount, skip
                    } else {
                        // New entry
                        let log_id = Uuid::new_v4();
                        let entity = diesel::insert_into(leave_accrual_logs::table)
                            .values((
                                leave_accrual_logs::id.eq(log_id),
                                leave_accrual_logs::workspace_id.eq(workspace_id),
                                leave_accrual_logs::employment_id.eq(active_emp_id),
                                leave_accrual_logs::policy_assignment_id.eq(assignment_id),
                                leave_accrual_logs::accrual_date.eq(accrual_date),
                                leave_accrual_logs::amount.eq(amount_bd.clone()),
                                leave_accrual_logs::leave_type_id.eq(leave_type_id),
                            ))
                            .on_conflict((
                                leave_accrual_logs::employment_id,
                                leave_accrual_logs::leave_type_id,
                                leave_accrual_logs::accrual_date,
                            ))
                            .do_nothing()
                            .get_result::<LeaveAccrualLogEntity>(&mut *conn)
                            .await
                            .optional()
                            .map_err(AppError::from)?;

                        if let Some(e) = entity {
                            diesel::insert_into(leave_balances::table)
                                .values((
                                    leave_balances::id.eq(Uuid::new_v4()),
                                    leave_balances::employment_id.eq(active_emp_id),
                                    leave_balances::year.eq(year),
                                    leave_balances::leave_type_id.eq(leave_type_id),
                                    leave_balances::entitlement.eq(0),
                                    leave_balances::accrued_amount.eq(amount_bd.clone()),
                                    leave_balances::carried_over.eq(0),
                                ))
                                .on_conflict((
                                    leave_balances::employment_id,
                                    leave_balances::leave_type_id,
                                    leave_balances::year,
                                ))
                                .do_update()
                                .set((
                                    leave_balances::accrued_amount
                                        .eq(leave_balances::accrued_amount + amount_bd),
                                    leave_balances::updated_at.eq(diesel::dsl::now),
                                ))
                                .execute(&mut *conn)
                                .await
                                .map_err(AppError::from)?;

                            all_logs.push(e.into());
                        }
                    }
                }

                Ok(all_logs)
            })
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;
    use rust_decimal::prelude::FromPrimitive;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselLeaveAccrualLogRepository::new(pools);
        // DI connection test passes if it compiles
    }

    #[tokio::test]
    async fn test_repo_methods_error_handling() {
        // Create a repo with a bad connection string
        let bad_url = "postgres://invalid:invalid@localhost:12345/invalid_db";
        let pools = DieselPools::for_test(bad_url);
        let repo = DieselLeaveAccrualLogRepository::new(pools);
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let lt_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let dec = Decimal::new(1, 0);

        let err = repo
            .find_by_employment_date_type(emp_id, lt_id, date, ws_id)
            .await;
        assert!(err.is_err());

        let err = repo
            .get_assignment_effective_from(Uuid::new_v4(), ws_id)
            .await;
        assert!(err.is_err());

        let err = repo.get_workspace_timezone(ws_id).await;
        assert!(err.is_err());

        let err = repo
            .get_employment_leave_history_logs(ws_id, emp_id, 2024, None)
            .await;
        assert!(err.is_err());

        let err = repo.update_accrual_amount(Uuid::new_v4(), ws_id, dec).await;
        assert!(err.is_err());

        let err = repo
            .insert_accrual_log(ws_id, emp_id, None, date, dec, lt_id)
            .await;
        assert!(err.is_err());

        let err = repo
            .upsert_balance_accrual(ws_id, emp_id, 2024, lt_id, dec)
            .await;
        assert!(err.is_err());

        let err = repo
            .adjust_balance_accrual(ws_id, emp_id, 2024, lt_id, dec)
            .await;
        assert!(err.is_err());

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(ws_id);
        let err = repo
            .persist_accrual(PersistAccrualParams {
                auth_token: &auth_token,
                workspace_id: ws_id,
                employment_id: emp_id,
                assignment_id: Uuid::new_v4(),
                leave_type_id: lt_id,
                accrual_date: date,
                amount: 1.0,
                force: false,
            })
            .await;
        assert!(err.is_err());
    }

    // --- Decimal safety regression tests (Debt #2) ---
    // Verify that from_f64(NaN/Infinity) and from_str(invalid) never silently zero out data

    /// Empty input must return empty output before acquiring a DB connection.
    #[tokio::test]
    async fn test_get_employment_leave_history_logs_batch_empty_input() {
        let pools = DieselPools::for_test("postgres://invalid:invalid@localhost:12345/invalid_db");
        let repo = DieselLeaveAccrualLogRepository::new(pools);

        let result = repo
            .get_employment_leave_history_logs_batch(Uuid::new_v4(), &[], 2026, None)
            .await;

        assert!(
            matches!(result, Ok(ref logs) if logs.is_empty()),
            "Empty input should return Ok([])"
        );
    }

    #[test]
    fn test_decimal_from_f64_nan_returns_none() {
        let result = Decimal::from_f64(f64::NAN);
        assert!(
            result.is_none(),
            "from_f64(NaN) must return None, not silently default to zero"
        );
    }

    #[test]
    fn test_decimal_from_f64_infinity_returns_none() {
        let result = Decimal::from_f64(f64::INFINITY);
        assert!(result.is_none(), "from_f64(Infinity) must return None");
    }

    #[test]
    fn test_decimal_from_f64_neg_infinity_returns_none() {
        let result = Decimal::from_f64(f64::NEG_INFINITY);
        assert!(result.is_none(), "from_f64(-Infinity) must return None");
    }

    #[test]
    fn test_decimal_from_f64_valid_returns_some() {
        let result = Decimal::from_f64(42.5);
        assert!(result.is_some(), "from_f64(42.5) should return Some");
        assert_eq!(result.unwrap(), Decimal::new(425, 1));
    }

    #[test]
    fn test_decimal_from_str_invalid_returns_error() {
        let result = Decimal::from_str("not_a_number");
        assert!(
            result.is_err(),
            "from_str('not_a_number') must return Err, not silently default to zero"
        );
    }

    #[test]
    fn test_decimal_from_str_empty_returns_error() {
        let result = Decimal::from_str("");
        assert!(result.is_err(), "from_str('') must return Err");
    }

    #[test]
    fn test_decimal_error_propagation_pattern() {
        // Verify the pattern we use: .ok_or_else(|| AppError::Internal(...))?
        let nan_result = Decimal::from_f64(f64::NAN)
            .ok_or_else(|| AppError::Internal("Invalid f64 for Decimal conversion".into()));
        assert!(nan_result.is_err());
        match nan_result {
            Err(AppError::Internal(msg)) => {
                assert!(
                    msg.contains("Invalid f64"),
                    "Error message should describe the problem"
                );
            }
            _ => panic!("Expected AppError::Internal"),
        }
    }
}
