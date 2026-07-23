use super::*;
use crate::errors::AppError;
use crate::models::diesel::leave::*;
use crate::models::{LeaveRequest, LeaveStatus};
use crate::repositories::leave_request_repository::LeaveHistoryRequestItem;
use crate::schema::*;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

impl DieselLeaveRequestRepository {
    pub(super) async fn get_leave_request_by_id_impl(
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
            .map_err(AppError::from)?;

        Ok(entity.map(Into::into))
    }

    pub(super) async fn get_employee_leave_requests_impl(
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
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    pub(super) async fn get_workspace_leave_requests_impl(
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
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    pub(super) async fn get_paginated_workspace_leave_requests_impl(
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

    pub(super) async fn get_leave_types_impl(
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
            .map_err(AppError::from)?;

        Ok(rows)
    }

    pub(super) async fn check_leave_type_exists_impl(
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
            .map_err(AppError::from)?;

        Ok(exists)
    }

    pub(super) async fn get_leave_type_name_impl(
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
            .map_err(AppError::from)?;

        Ok(name)
    }

    pub(super) async fn get_employee_leave_history_requests_impl(
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

    pub(super) async fn get_employee_department_for_request_impl(
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
            .map_err(AppError::from)?
            .flatten();

        Ok(dept_id)
    }

    pub(super) async fn get_system_generated_holiday_leave_impl(
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
            .map_err(AppError::from)?;

        Ok(row)
    }

    pub(super) async fn get_leave_requests_by_ids_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        ids: &[Uuid],
    ) -> Result<Vec<LeaveRequest>, AppError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

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
            .filter(leave_requests::id.eq_any(ids))
            .filter(employments::workspace_id.eq(workspace_id))
            .select(LeaveRequestEntity::as_select())
            .load::<LeaveRequestEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }
}
