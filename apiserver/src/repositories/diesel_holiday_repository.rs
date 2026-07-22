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
    diesel_pools: DieselPools,
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
        location_id: Option<Uuid>,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Holiday>, i64), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut count_query = holidays::table
            .filter(holidays::workspace_id.eq(workspace_id))
            .filter(holidays::deleted_at.is_null())
            .into_boxed();

        if let Some(y) = year {
            if let (Some(start), Some(end)) = (
                NaiveDate::from_ymd_opt(y, 1, 1),
                NaiveDate::from_ymd_opt(y, 12, 31),
            ) {
                count_query = count_query
                    .filter(holidays::date.ge(start))
                    .filter(holidays::date.le(end));
            }
        }

        if let Some(loc_id) = location_id {
            count_query = count_query.filter(
                holidays::location_id
                    .is_null()
                    .or(holidays::location_id.eq(loc_id)),
            );
        }

        let total: i64 = count_query
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db count: {e}")))?;

        let mut select_query = holidays::table
            .filter(holidays::workspace_id.eq(workspace_id))
            .filter(holidays::deleted_at.is_null())
            .into_boxed();

        if let Some(y) = year {
            if let (Some(start), Some(end)) = (
                NaiveDate::from_ymd_opt(y, 1, 1),
                NaiveDate::from_ymd_opt(y, 12, 31),
            ) {
                select_query = select_query
                    .filter(holidays::date.ge(start))
                    .filter(holidays::date.le(end));
            }
        }

        if let Some(loc_id) = location_id {
            select_query = select_query.filter(
                holidays::location_id
                    .is_null()
                    .or(holidays::location_id.eq(loc_id)),
            );
        }

        let entities: Vec<HolidayEntity> = select_query
            .order(holidays::date.asc())
            .limit(limit)
            .offset((page - 1) * limit)
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok((entities.into_iter().map(Into::into).collect(), total))
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
            .map_err(AppError::from)?;

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
            location_id: req.location_id,
        };

        let result: HolidayEntity = diesel::insert_into(holidays::table)
            .values(&new_holiday)
            .get_result(&mut *conn)
            .await
            .map_err(AppError::from)?;

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
                req.location_id.map(|l| holidays::location_id.eq(l)),
                Some(holidays::updated_at.eq(Utc::now())),
            ))
            .get_result(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

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
            .map_err(AppError::from)?;

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
        location_id: Option<Uuid>,
    ) -> Result<Option<Holiday>, AppError> {
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
            .filter(holidays::date.eq(date))
            .into_boxed();

        if let Some(loc_id) = location_id {
            query = query.filter(
                holidays::location_id
                    .is_null()
                    .or(holidays::location_id.eq(loc_id)),
            );
        }

        let entity: Option<HolidayEntity> = query
            .first(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

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

        use crate::schema::employments;
        use bigdecimal::BigDecimal;
        use std::str::FromStr;

        conn.transaction(|conn| {
            Box::pin(async move {
                // 0. Fetch all employments for these employees in bulk to map employee_id to employment_id
                let mut emp_ids: Vec<Uuid> =
                    logs.iter().map(|(emp_id, _, _, _, _)| *emp_id).collect();
                emp_ids.sort();
                emp_ids.dedup();

                let mut resolved_employments = HashMap::new();
                if !emp_ids.is_empty() {
                    let employments_list = employments::table
                        .filter(employments::workspace_id.eq(workspace_id))
                        .filter(employments::employee_id.eq_any(&emp_ids))
                        .filter(employments::deleted_at.is_null())
                        .load::<crate::models::diesel::employment::EmploymentEntity>(&mut *conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db fetch employments: {e}")))?;

                    let mut emp_groups: HashMap<
                        Uuid,
                        Vec<crate::models::diesel::employment::EmploymentEntity>,
                    > = HashMap::new();
                    for emp in employments_list {
                        emp_groups.entry(emp.employee_id).or_default().push(emp);
                    }

                    for (emp_id, mut list) in emp_groups {
                        list.sort_by(|a, b| b.start_date.cmp(&a.start_date));
                        if let Some(latest) = list.as_slice().first() {
                            resolved_employments.insert(emp_id, latest.id);
                        }
                    }
                }

                // 1. Bulk insert leave requests
                if !requests.is_empty() {
                    let lr_entities: Vec<crate::models::diesel::leave::LeaveRequestEntity> =
                        requests
                            .iter()
                            .map(|r| crate::models::diesel::leave::LeaveRequestEntity {
                                id: r.id(),
                                employment_id: r.employment_id(),
                                leave_type_id: r.leave_type_id(),
                                start_date: r.start_date(),
                                end_date: r.end_date(),
                                reason: r.reason().map(String::from),
                                status: r.status(),
                                duration_type: r.duration_type().to_string(),
                                start_time: r.start_time(),
                                end_time: r.end_time(),
                                requested_by_user_id: r.requested_by_user_id(),
                                approved_by_user_id: r.approved_by_user_id(),
                                created_at: Some(r.created_at()),
                                updated_at: Some(r.updated_at()),
                                attachment_file_ids: None,
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
                                employment_id: r.employment_id,
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
                            crate::schema::shift_rosters::employment_id,
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

                // 3. Bulk insert accrual logs
                if !logs.is_empty() {
                    let mut log_entities = Vec::new();
                    for (emp_id, leave_type_id, amount, date, notes) in logs {
                        let amount_bd =
                            BigDecimal::from_str(&amount.to_string()).unwrap_or_default();
                        let employment_id = match resolved_employments.get(emp_id) {
                            Some(id) => *id,
                            None => {
                                return Err(AppError::Internal(format!(
                                    "No employment found for employee {emp_id}"
                                )))
                            }
                        };
                        log_entities.push(crate::models::diesel::leave::LeaveAccrualLogEntity {
                            id: Uuid::new_v4(),
                            workspace_id,
                            policy_assignment_id: None,
                            accrual_date: *date,
                            amount: amount_bd,
                            notes: Some(notes.clone()),
                            created_at: Some(chrono::Utc::now()),
                            leave_type_id: *leave_type_id,
                            employment_id,
                        });
                    }

                    diesel::insert_into(crate::schema::leave_accrual_logs::table)
                        .values(&log_entities)
                        .execute(&mut *conn)
                        .await?;
                }

                // 4. Bulk insert/upsert leave balances
                if !logs.is_empty() {
                    let mut balance_aggregates: HashMap<(Uuid, Uuid, i32), BigDecimal> =
                        HashMap::new();

                    for (emp_id, leave_type_id, amount, date, _) in logs {
                        let amount_bd =
                            BigDecimal::from_str(&amount.to_string()).unwrap_or_default();
                        let year = date.year();
                        let employment_id = match resolved_employments.get(emp_id) {
                            Some(id) => *id,
                            None => {
                                return Err(AppError::Internal(format!(
                                    "No employment found for employee {emp_id}"
                                )))
                            }
                        };
                        let key = (employment_id, *leave_type_id, year);
                        let entry = balance_aggregates.entry(key).or_default();
                        *entry += amount_bd;
                    }

                    let balance_entities: Vec<crate::models::diesel::leave::LeaveBalanceEntity> =
                        balance_aggregates
                            .into_iter()
                            .map(|((employment_id, leave_type_id, year), amount_bd)| {
                                crate::models::diesel::leave::LeaveBalanceEntity {
                                    id: Uuid::new_v4(),
                                    employment_id,
                                    leave_type_id,
                                    year,
                                    entitlement: 0,
                                    accrued_amount: amount_bd,
                                    carried_over: 0,
                                    created_at: Some(chrono::Utc::now()),
                                    updated_at: Some(chrono::Utc::now()),
                                }
                            })
                            .collect();

                    diesel::insert_into(crate::schema::leave_balances::table)
                        .values(&balance_entities)
                        .on_conflict((
                            crate::schema::leave_balances::employment_id,
                            crate::schema::leave_balances::leave_type_id,
                            crate::schema::leave_balances::year,
                        ))
                        .do_update()
                        .set((
                            crate::schema::leave_balances::accrued_amount
                                .eq(crate::schema::leave_balances::accrued_amount
                                    + excluded(crate::schema::leave_balances::accrued_amount)),
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
