use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Utc};
use diesel::prelude::*;
use diesel::upsert::excluded;
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::collections::HashMap;
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::holiday::HolidayEntity;
use crate::models::Holiday;
use crate::repositories::holiday_repository::HolidayRepository;
use crate::schema::holidays;

pub struct DieselHolidayRepository {
    pub diesel_pools: DieselPools,
}

impl DieselHolidayRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[allow(clippy::too_many_lines)]
#[async_trait]
impl HolidayRepository for DieselHolidayRepository {
    async fn list(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        year: Option<i32>,
    ) -> Result<Vec<Holiday>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = holidays::table
            .filter(holidays::workspace_id.eq(workspace_id))
            .filter(holidays::deleted_at.is_null())
            .into_boxed();

        if let Some(y) = year {
            if let (Some(start), Some(end)) = (
                NaiveDate::from_ymd_opt(y, 1, 1),
                NaiveDate::from_ymd_opt(y, 12, 31),
            ) {
                query = query
                    .filter(holidays::date.ge(start))
                    .filter(holidays::date.le(end));
            }
        }

        let entities: Vec<HolidayEntity> = query
            .order(holidays::date.asc())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<Holiday>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity: Option<HolidayEntity> = holidays::table
            .filter(holidays::id.eq(id))
            .filter(holidays::workspace_id.eq(workspace_id))
            .filter(holidays::deleted_at.is_null())
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }

    async fn create_holiday(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &crate::models::CreateHolidayRequest,
    ) -> Result<Holiday, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let new_holiday = HolidayEntity {
            id: Uuid::new_v4(),
            workspace_id,
            name: req.name.clone(),
            date: req.date,
            holiday_type: req.holiday_type.clone(),
            is_paid: req.is_paid,
            leave_type_id: req.leave_type_id,
            ot_multiplier: req.ot_multiplier,
            description: req.description.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            work_threshold_percent: req.work_threshold_percent,
            is_credit_on_work: req.is_credit_on_work,
            deleted_at: None,
        };

        let result: HolidayEntity = diesel::insert_into(holidays::table)
            .values(&new_holiday)
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn update_holiday(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        req: &crate::models::UpdateHolidayRequest,
    ) -> Result<Option<Holiday>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result: Option<HolidayEntity> = diesel::update(holidays::table)
            .filter(holidays::id.eq(id))
            .filter(holidays::workspace_id.eq(workspace_id))
            .filter(holidays::deleted_at.is_null())
            .set((
                req.name.as_ref().map(|n| holidays::name.eq(n)),
                req.date.map(|d| holidays::date.eq(d)),
                req.holiday_type
                    .as_ref()
                    .map(|t| holidays::holiday_type.eq(t)),
                req.is_paid.map(|p| holidays::is_paid.eq(p)),
                req.leave_type_id.map(|l| holidays::leave_type_id.eq(l)),
                req.ot_multiplier.map(|m| holidays::ot_multiplier.eq(m)),
                req.description
                    .as_ref()
                    .map(|d| holidays::description.eq(d)),
                req.work_threshold_percent
                    .map(|w| holidays::work_threshold_percent.eq(w)),
                req.is_credit_on_work
                    .map(|c| holidays::is_credit_on_work.eq(c)),
                Some(holidays::updated_at.eq(Utc::now())),
            ))
            .get_result(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn delete(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<bool, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let affected = diesel::update(holidays::table)
            .filter(holidays::id.eq(id))
            .filter(holidays::workspace_id.eq(workspace_id))
            .set(holidays::deleted_at.eq(Utc::now()))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected > 0)
    }

    async fn get_holidays_in_range(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<NaiveDate, Holiday>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entities: Vec<HolidayEntity> = holidays::table
            .filter(holidays::workspace_id.eq(workspace_id))
            .filter(holidays::deleted_at.is_null())
            .filter(holidays::date.ge(start_date))
            .filter(holidays::date.le(end_date))
            .order(holidays::date.asc())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(|e| (e.date, e.into())).collect())
    }

    async fn is_holiday(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
    ) -> Result<Option<Holiday>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity: Option<HolidayEntity> = holidays::table
            .filter(holidays::workspace_id.eq(workspace_id))
            .filter(holidays::deleted_at.is_null())
            .filter(holidays::date.eq(date))
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }

    async fn apply_holiday_leave_deductions(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        requests: &[crate::models::LeaveRequest],
        rosters: &[crate::models::ShiftRoster],
        logs: &[(
            Uuid,                  // employee_id
            Uuid,                  // leave_type_id
            rust_decimal::Decimal, // amount
            chrono::NaiveDate,     // date
            String,                // notes
        )],
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        use bigdecimal::BigDecimal;
        use std::str::FromStr;

        conn.transaction(|conn| {
            Box::pin(async move {
                // 1. Bulk insert leave requests
                if !requests.is_empty() {
                    let lr_entities: Vec<crate::models::diesel::leave::LeaveRequestEntity> =
                        requests
                            .iter()
                            .map(|r| crate::models::diesel::leave::LeaveRequestEntity {
                                id: r.id,
                                employee_id: r.employee_id,
                                leave_type_id: r.leave_type_id,
                                start_date: r.start_date,
                                end_date: r.end_date,
                                reason: r.reason.clone(),
                                status: r.status,
                                duration_type: r.duration_type.to_string(),
                                start_time: r.start_time,
                                end_time: r.end_time,
                                requested_by_user_id: r.requested_by_user_id,
                                approved_by_user_id: r.approved_by_user_id,
                                created_at: Some(r.created_at),
                                updated_at: Some(r.updated_at),
                            })
                            .collect();

                    diesel::insert_into(crate::schema::leave_requests::table)
                        .values(&lr_entities)
                        .execute(&mut *conn)
                        .await?;
                }

                // 2. Bulk insert/upsert rosters
                if !rosters.is_empty() {
                    let roster_entities: Vec<crate::models::diesel::roster::ShiftRosterEntity> =
                        rosters
                            .iter()
                            .map(|r| crate::models::diesel::roster::ShiftRosterEntity {
                                id: r.id,
                                employee_id: r.employee_id,
                                shift_id: r.shift_id,
                                date: r.date,
                                status: Some(r.status.clone()),
                                notes: r.notes.clone(),
                                created_at: Some(r.created_at),
                                updated_at: Some(r.updated_at),
                            })
                            .collect();

                    diesel::insert_into(crate::schema::shift_rosters::table)
                        .values(&roster_entities)
                        .on_conflict((
                            crate::schema::shift_rosters::employee_id,
                            crate::schema::shift_rosters::date,
                        ))
                        .do_update()
                        .set((
                            crate::schema::shift_rosters::status
                                .eq(excluded(crate::schema::shift_rosters::status)),
                            crate::schema::shift_rosters::notes
                                .eq(excluded(crate::schema::shift_rosters::notes)),
                            crate::schema::shift_rosters::updated_at.eq(diesel::dsl::now),
                        ))
                        .execute(&mut *conn)
                        .await?;
                }

                // 3. Insert accrual logs and update balances
                for (emp_id, leave_type_id, amount, date, notes) in logs {
                    let amount_bd = BigDecimal::from_str(&amount.to_string()).unwrap_or_default();
                    let year = date.year();

                    // Log adjustment
                    diesel::insert_into(crate::schema::leave_accrual_logs::table)
                        .values((
                            crate::schema::leave_accrual_logs::workspace_id.eq(workspace_id),
                            crate::schema::leave_accrual_logs::employee_id.eq(emp_id),
                            crate::schema::leave_accrual_logs::accrual_date.eq(date),
                            crate::schema::leave_accrual_logs::amount.eq(&amount_bd),
                            crate::schema::leave_accrual_logs::leave_type_id.eq(leave_type_id),
                            crate::schema::leave_accrual_logs::notes.eq(notes),
                        ))
                        .execute(&mut *conn)
                        .await?;

                    // Update balance
                    diesel::insert_into(crate::schema::leave_balances::table)
                        .values((
                            crate::schema::leave_balances::id.eq(Uuid::new_v4()),
                            crate::schema::leave_balances::employee_id.eq(emp_id),
                            crate::schema::leave_balances::leave_type_id.eq(leave_type_id),
                            crate::schema::leave_balances::year.eq(year),
                            crate::schema::leave_balances::entitlement.eq(0),
                            crate::schema::leave_balances::accrued_amount.eq(&amount_bd),
                            crate::schema::leave_balances::carried_over.eq(0),
                        ))
                        .on_conflict((
                            crate::schema::leave_balances::employee_id,
                            crate::schema::leave_balances::leave_type_id,
                            crate::schema::leave_balances::year,
                        ))
                        .do_update()
                        .set((
                            crate::schema::leave_balances::accrued_amount
                                .eq(crate::schema::leave_balances::accrued_amount + &amount_bd),
                            crate::schema::leave_balances::updated_at.eq(diesel::dsl::now),
                        ))
                        .execute(&mut *conn)
                        .await?;
                }

                Ok::<_, AppError>(())
            })
        })
        .await
    }
}
