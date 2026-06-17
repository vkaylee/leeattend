use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::leave::*;
use crate::models::{LeaveRequest, LeaveStatus};
use crate::repositories::leave_request_repository::{
    LeaveBreakdown, LeaveHistoryRequestItem, LeaveRequestRepository,
};

use crate::schema::*;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub struct DieselLeaveRequestRepository {
    diesel_pools: DieselPools,
}

impl DieselLeaveRequestRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[allow(clippy::too_many_lines)]
#[async_trait]
impl LeaveRequestRepository for DieselLeaveRequestRepository {
    async fn create_leave_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request: &LeaveRequest,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // Check if employment exists and belongs to the workspace of this tenant
        let employment_exists = employments::table
            .filter(employments::id.eq(request.employment_id()))
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(employments::deleted_at.is_null())
            .select(employments::id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db check employment: {e}")))?;

        if employment_exists.is_none() {
            return Err(AppError::NotFound("error.employment_not_found".into()));
        }

        diesel::insert_into(leave_requests::table)
            .values((
                leave_requests::id.eq(request.id()),
                leave_requests::employment_id.eq(request.employment_id()),
                leave_requests::leave_type_id.eq(request.leave_type_id()),
                leave_requests::start_date.eq(request.start_date()),
                leave_requests::end_date.eq(request.end_date()),
                leave_requests::reason.eq(&request.reason()),
                leave_requests::status.eq(request.status()),
                leave_requests::duration_type.eq(request.duration_type().to_string()),
                leave_requests::start_time.eq(request.start_time()),
                leave_requests::end_time.eq(request.end_time()),
                leave_requests::requested_by_user_id.eq(request.requested_by_user_id()),
                leave_requests::approved_by_user_id.eq(request.approved_by_user_id()),
                leave_requests::created_at.eq(request.created_at()),
                leave_requests::updated_at.eq(request.updated_at()),
                leave_requests::attachment_file_ids.eq(request.attachment_file_ids()),
            ))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_leave_request_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<LeaveRequest>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = leave_requests::table
            .inner_join(employments::table)
            .filter(leave_requests::id.eq(id))
            .filter(employments::workspace_id.eq(workspace_id))
            .select(LeaveRequestEntity::as_select())
            .first::<LeaveRequestEntity>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }

    async fn update_leave_request_status(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        status: LeaveStatus,
        approved_by: Option<Uuid>,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        diesel::update(leave_requests::table)
            .filter(leave_requests::id.eq(id))
            .filter(diesel::dsl::exists(
                employments::table
                    .filter(employments::id.eq(leave_requests::employment_id))
                    .filter(employments::workspace_id.eq(workspace_id)),
            ))
            .set((
                leave_requests::status.eq(status),
                leave_requests::approved_by_user_id.eq(approved_by),
                leave_requests::updated_at.eq(Utc::now()),
            ))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_employee_leave_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entities = leave_requests::table
            .inner_join(employments::table)
            .filter(employments::employee_id.eq(employee_id))
            .filter(employments::workspace_id.eq(workspace_id))
            .order(leave_requests::start_date.desc())
            .select(LeaveRequestEntity::as_select())
            .load::<LeaveRequestEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_workspace_leave_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        status: Option<LeaveStatus>,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = leave_requests::table
            .inner_join(employments::table)
            .filter(employments::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(s) = status {
            query = query.filter(leave_requests::status.eq(s));
        }

        let entities = query
            .order(leave_requests::created_at.desc())
            .select(LeaveRequestEntity::as_select())
            .load::<LeaveRequestEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_paginated_workspace_leave_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        status: Option<LeaveStatus>,
        employee_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<LeaveRequest>, i64), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = leave_requests::table
            .inner_join(employments::table)
            .filter(employments::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(s) = status {
            if s == LeaveStatus::Approved {
                query = query.filter(
                    leave_requests::status
                        .eq(LeaveStatus::Approved)
                        .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
                );
            } else {
                query = query.filter(leave_requests::status.eq(s));
            }
        }

        if let Some(emp_id) = employee_id {
            query = query.filter(employments::employee_id.eq(emp_id));
        }

        let total = query
            .select(diesel::dsl::count(leave_requests::id))
            .get_result::<i64>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db count: {e}")))?;

        let mut query = leave_requests::table
            .inner_join(employments::table)
            .filter(employments::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(s) = status {
            if s == LeaveStatus::Approved {
                query = query.filter(
                    leave_requests::status
                        .eq(LeaveStatus::Approved)
                        .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
                );
            } else {
                query = query.filter(leave_requests::status.eq(s));
            }
        }

        if let Some(emp_id) = employee_id {
            query = query.filter(employments::employee_id.eq(emp_id));
        }

        let entities = query
            .order(leave_requests::created_at.desc())
            .select(LeaveRequestEntity::as_select())
            .limit(limit)
            .offset(offset)
            .load::<LeaveRequestEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db load: {e}")))?;

        Ok((entities.into_iter().map(Into::into).collect(), total))
    }

    async fn get_approved_leave_breakdown(
        &self,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<LeaveBreakdown, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // Step 1: Get raw leave days per type
        let rows = leave_requests::table
            .inner_join(leave_types::table)
            .inner_join(employments::table)
            .filter(employments::employee_id.eq(employee_id))
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(
                leave_requests::status
                    .eq(LeaveStatus::Approved)
                    .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
            )
            .filter(leave_requests::start_date.le(end_date))
            .filter(leave_requests::end_date.ge(start_date))
            .group_by(leave_types::code)
            .select((
                leave_types::code,
                diesel::dsl::sql::<diesel::sql_types::BigInt>(&format!(
                    "SUM(LEAST(leave_requests.end_date, '{end_date}') - GREATEST(leave_requests.start_date, '{start_date}') + 1)"
                )),
            ))
            .load::<(String, i64)>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // Step 2: Count paid holidays in the period to subtract from total
        let paid_holiday_count: i64 = holidays::table
            .filter(holidays::workspace_id.eq(workspace_id))
            .filter(holidays::is_paid.eq(true))
            .filter(holidays::date.ge(start_date))
            .filter(holidays::date.le(end_date))
            .filter(diesel::dsl::exists(
                leave_requests::table
                    .inner_join(employments::table)
                    .filter(employments::employee_id.eq(employee_id))
                    .filter(employments::workspace_id.eq(workspace_id))
                    .filter(
                        leave_requests::status
                            .eq(LeaveStatus::Approved)
                            .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
                    )
                    .filter(leave_requests::start_date.le(holidays::date))
                    .filter(leave_requests::end_date.ge(holidays::date)),
            ))
            .count()
            .get_result::<i64>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut breakdown = LeaveBreakdown {
            total_days: 0.0,
            unpaid_days: 0.0,
            sick_days: 0.0,
        };

        for (code, days) in rows {
            let days_f = days as f64;
            breakdown.total_days += days_f;
            if code == "unpaid" || code == "personal" {
                breakdown.unpaid_days += days_f;
            } else if code == "sick" {
                breakdown.sick_days += days_f;
            }
        }

        breakdown.total_days -= paid_holiday_count as f64;
        if breakdown.total_days < 0.0 {
            breakdown.total_days = 0.0;
        }

        Ok(breakdown)
    }

    async fn get_employee_department_for_request(
        &self,
        request_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Uuid>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let dept_id = leave_requests::table
            .inner_join(employments::table)
            .filter(leave_requests::id.eq(request_id))
            .filter(employments::workspace_id.eq(workspace_id))
            .select(employments::department_id)
            .first::<Option<Uuid>>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?
            .flatten();

        Ok(dept_id)
    }

    async fn get_leave_types(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<(String, String)>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let rows = leave_types::table
            .filter(leave_types::workspace_id.eq(workspace_id))
            .order(leave_types::code.asc())
            .select((leave_types::code, leave_types::name))
            .load::<(String, String)>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows)
    }

    async fn get_system_generated_holiday_leave(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<(Uuid, Uuid, String)>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let row = leave_requests::table
            .inner_join(leave_types::table)
            .inner_join(employments::table)
            .filter(employments::employee_id.eq(employee_id))
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(
                leave_requests::status
                    .eq(LeaveStatus::Approved)
                    .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
            )
            .filter(leave_requests::start_date.le(date))
            .filter(leave_requests::end_date.ge(date))
            .filter(leave_requests::requested_by_user_id.is_null())
            .select((
                leave_requests::id,
                leave_requests::leave_type_id,
                leave_types::code,
            ))
            .first::<(Uuid, Uuid, String)>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(row)
    }

    async fn cancel_leave_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        diesel::update(leave_requests::table)
            .filter(leave_requests::id.eq(id))
            .filter(diesel::dsl::exists(
                employments::table
                    .filter(employments::id.eq(leave_requests::employment_id))
                    .filter(employments::workspace_id.eq(workspace_id)),
            ))
            .set((
                leave_requests::status.eq(LeaveStatus::Cancelled),
                leave_requests::updated_at.eq(Utc::now()),
            ))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_employee_leave_history_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        year: i32,
        leave_type_id: Option<Uuid>,
    ) -> Result<Vec<LeaveHistoryRequestItem>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = leave_requests::table
            .inner_join(leave_types::table)
            .inner_join(employments::table)
            .filter(employments::employee_id.eq(employee_id))
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(
                leave_requests::status
                    .eq(LeaveStatus::Approved)
                    .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
            )
            .filter(
                diesel::dsl::sql::<diesel::sql_types::Bool>(
                    "EXTRACT(YEAR FROM leave_requests.start_date) = ",
                )
                .bind::<diesel::sql_types::Double, _>(f64::from(year)),
            )
            .into_boxed();

        if let Some(lt_id) = leave_type_id {
            query = query.filter(leave_requests::leave_type_id.eq(lt_id));
        }

        let entities = query
            .select((
                leave_requests::id,
                leave_requests::start_date,
                leave_requests::end_date,
                leave_types::name,
                leave_requests::reason,
                leave_requests::created_at,
            ))
            .load::<(
                Uuid,
                NaiveDate,
                NaiveDate,
                String,
                Option<String>,
                Option<DateTime<Utc>>,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities
            .into_iter()
            .map(|row| LeaveHistoryRequestItem {
                id: row.0,
                start_date: row.1,
                end_date: row.2,
                leave_type_name: row.3,
                reason: row.4,
                created_at: row.5,
            })
            .collect())
    }

    async fn check_leave_type_exists(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        leave_type_id: Uuid,
    ) -> Result<bool, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let exists = leave_types::table
            .filter(leave_types::id.eq(leave_type_id))
            .filter(leave_types::workspace_id.eq(workspace_id))
            .count()
            .get_result::<i64>(&mut *conn)
            .await
            .map(|c| c > 0)
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(exists)
    }

    async fn get_leave_type_name(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        leave_type_id: Uuid,
    ) -> Result<Option<String>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let name = leave_types::table
            .filter(leave_types::id.eq(leave_type_id))
            .filter(leave_types::workspace_id.eq(workspace_id))
            .select(leave_types::name)
            .first::<String>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(name)
    }

    async fn get_unpaid_holiday_days_without_leave(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<i64, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let count = holidays::table
            .filter(holidays::workspace_id.eq(workspace_id))
            .filter(holidays::is_paid.eq(false))
            .filter(holidays::date.ge(start_date))
            .filter(holidays::date.le(end_date))
            .filter(diesel::dsl::not(diesel::dsl::exists(
                leave_requests::table
                    .inner_join(employments::table)
                    .filter(employments::employee_id.eq(employee_id))
                    .filter(employments::workspace_id.eq(workspace_id))
                    .filter(
                        leave_requests::status
                            .eq(LeaveStatus::Approved)
                            .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
                    )
                    .filter(leave_requests::start_date.le(holidays::date))
                    .filter(leave_requests::end_date.ge(holidays::date)),
            )))
            .count()
            .get_result::<i64>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(count)
    }

    async fn get_batch_leave_days(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, i64>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let rows = leave_requests::table
            .inner_join(employments::table)
            .filter(employments::employee_id.eq_any(employee_ids))
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(
                leave_requests::status
                    .eq(LeaveStatus::Approved)
                    .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
            )
            .filter(leave_requests::start_date.ge(start_date))
            .filter(leave_requests::end_date.le(end_date))
            .group_by(employments::employee_id)
            .select((
                employments::employee_id,
                diesel::dsl::sql::<diesel::sql_types::BigInt>(
                    "SUM(leave_requests.end_date - leave_requests.start_date + 1)",
                ),
            ))
            .load::<(Uuid, i64)>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows.into_iter().collect())
    }

    async fn get_approved_leave_dates(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashSet<(Uuid, NaiveDate)>, AppError> {
        use diesel_async::RunQueryDsl;

        #[derive(diesel::QueryableByName)]
        struct LeaveRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            employee_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Date)]
            start_date: NaiveDate,
            #[diesel(sql_type = diesel::sql_types::Date)]
            end_date: NaiveDate,
        }

        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let leave_rows: Vec<LeaveRow> = diesel::sql_query(
            r"SELECT emp.employee_id, lr.start_date, lr.end_date
               FROM leave_requests lr
               JOIN employments emp ON lr.employment_id = emp.id
               WHERE emp.workspace_id = $1
                 AND emp.employee_id = ANY($2)
                 AND lr.status IN ('approved', 'pre_approved')
                 AND lr.start_date <= $4
                 AND lr.end_date >= $3",
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(employee_ids.to_vec())
        .bind::<diesel::sql_types::Date, _>(start_date)
        .bind::<diesel::sql_types::Date, _>(end_date)
        .load(&mut *conn)
        .await
        .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;

        let mut set = std::collections::HashSet::new();
        for lr in &leave_rows {
            let mut d = lr.start_date;
            while d <= lr.end_date {
                if d >= start_date && d <= end_date {
                    set.insert((lr.employee_id, d));
                }
                d = match d.succ_opt() {
                    Some(next) => next,
                    None => break,
                };
            }
        }

        Ok(set)
    }

    async fn get_approved_leave_details(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), bool>, AppError> {
        use diesel_async::RunQueryDsl;

        #[derive(diesel::QueryableByName)]
        struct LeaveRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            employee_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Date)]
            start_date: NaiveDate,
            #[diesel(sql_type = diesel::sql_types::Date)]
            end_date: NaiveDate,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_paid: bool,
        }

        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let leave_rows: Vec<LeaveRow> = diesel::sql_query(
            r"SELECT emp.employee_id, lr.start_date, lr.end_date, lt.is_paid
               FROM leave_requests lr
               JOIN employments emp ON lr.employment_id = emp.id
               JOIN leave_types lt ON lr.leave_type_id = lt.id
               WHERE emp.workspace_id = $1
                 AND emp.employee_id = ANY($2)
                 AND lr.status IN ('approved', 'pre_approved')
                 AND lr.start_date <= $4
                 AND lr.end_date >= $3",
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(employee_ids.to_vec())
        .bind::<diesel::sql_types::Date, _>(start_date)
        .bind::<diesel::sql_types::Date, _>(end_date)
        .load(&mut *conn)
        .await
        .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;

        let mut map = std::collections::HashMap::new();
        for lr in &leave_rows {
            let mut d = lr.start_date;
            while d <= lr.end_date {
                if d >= start_date && d <= end_date {
                    map.insert((lr.employee_id, d), lr.is_paid);
                }
                d = match d.succ_opt() {
                    Some(next) => next,
                    None => break,
                };
            }
        }

        Ok(map)
    }

    async fn get_approved_leave_periods(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<(NaiveDate, NaiveDate)>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let rows: Vec<(NaiveDate, NaiveDate)> = leave_requests::table
            .inner_join(employments::table)
            .filter(employments::employee_id.eq(employee_id))
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(
                leave_requests::status
                    .eq(LeaveStatus::Approved)
                    .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
            )
            .filter(leave_requests::start_date.le(end_date))
            .filter(leave_requests::end_date.ge(start_date))
            .select((leave_requests::start_date, leave_requests::end_date))
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::{DieselPools, TenantAuthToken};
    use crate::models::{LeaveRequest, LeaveStatus};
    use chrono::{NaiveDate, Utc};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_leave_request_repository_errors() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });
        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselLeaveRequestRepository::new(pools);
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let lt_id = Uuid::new_v4();
        let auth_token = TenantAuthToken(ws_id);

        let req = LeaveRequest::new(
            Uuid::new_v4(),
            emp_id,
            lt_id,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(),
            Some("test".to_string()),
            LeaveStatus::Pending,
            crate::models::LeaveDurationType::FullDay,
            None,
            None,
            None,
            None,
            Utc::now(),
            Utc::now(),
            None,
        );

        let _ = repo.create_leave_request(&auth_token, &req).await;
        let _ = repo
            .get_leave_request_by_id(&auth_token, Uuid::new_v4())
            .await;
        let _ = repo
            .update_leave_request_status(&auth_token, Uuid::new_v4(), LeaveStatus::Approved, None)
            .await;
        let _ = repo.get_employee_leave_requests(&auth_token, emp_id).await;
        let _ = repo.get_workspace_leave_requests(&auth_token, None).await;
        let _ = repo
            .get_approved_leave_breakdown(
                emp_id,
                NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
                &auth_token,
            )
            .await;
        let _ = repo
            .get_employee_department_for_request(Uuid::new_v4(), &auth_token)
            .await;
        let _ = repo.get_leave_types(&auth_token).await;
        let _ = repo
            .get_system_generated_holiday_leave(
                &auth_token,
                emp_id,
                NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            )
            .await;
        let _ = repo.cancel_leave_request(&auth_token, Uuid::new_v4()).await;
        let _ = repo
            .get_employee_leave_history_requests(&auth_token, emp_id, 2023, None)
            .await;
        let _ = repo.check_leave_type_exists(&auth_token, lt_id).await;
        let _ = repo.get_leave_type_name(&auth_token, lt_id).await;
        let _ = repo
            .get_unpaid_holiday_days_without_leave(
                &auth_token,
                emp_id,
                NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
            )
            .await;
        let _ = repo
            .get_batch_leave_days(
                &auth_token,
                &[emp_id],
                NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
            )
            .await;
        let _ = repo
            .get_approved_leave_dates(
                &auth_token,
                &[emp_id],
                NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
            )
            .await;
        let _ = repo
            .get_approved_leave_periods(
                &auth_token,
                emp_id,
                NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
            )
            .await;
    }
}
