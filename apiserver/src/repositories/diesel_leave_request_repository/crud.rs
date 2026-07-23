use super::*;
use crate::errors::AppError;
use crate::models::{LeaveRequest, LeaveStatus};
use crate::schema::*;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

impl DieselLeaveRequestRepository {
    pub(super) async fn create_leave_request_impl(
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
        self.create_leave_request_tx_impl(&mut conn, auth_token, request)
            .await
    }

    pub(super) async fn create_leave_request_tx_impl(
        &self,
        conn: &mut diesel_async::AsyncPgConnection,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request: &LeaveRequest,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();

        // Check if employment exists and belongs to the workspace of this tenant
        let employment_exists = employments::table
            .filter(employments::id.eq(request.employment_id()))
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(employments::deleted_at.is_null())
            .select(employments::id)
            .first::<Uuid>(conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db check employment: {e}")))?;

        if employment_exists.is_none() {
            return Err(AppError::not_found(
                crate::utils::i18n::I18nKey::EmploymentNotFound,
            ));
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
            .execute(conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    pub(super) async fn update_leave_request_status_impl(
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
            .map_err(AppError::from)?;

        Ok(())
    }

    pub(super) async fn cancel_leave_request_impl(
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
            .map_err(AppError::from)?;

        Ok(())
    }

    pub(super) async fn bulk_cancel_leave_requests_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        ids: &[Uuid],
    ) -> Result<u64, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );
        self.bulk_cancel_leave_requests_tx_impl(&mut conn, auth_token, ids)
            .await
    }

    pub(super) async fn bulk_cancel_leave_requests_tx_impl(
        &self,
        conn: &mut diesel_async::AsyncPgConnection,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        ids: &[Uuid],
    ) -> Result<u64, AppError> {
        if ids.is_empty() {
            return Ok(0);
        }

        let workspace_id = auth_token.workspace_id();
        let affected = diesel::update(leave_requests::table)
            .filter(leave_requests::id.eq_any(ids))
            .filter(diesel::dsl::exists(
                employments::table
                    .filter(employments::id.eq(leave_requests::employment_id))
                    .filter(employments::workspace_id.eq(workspace_id)),
            ))
            .set((
                leave_requests::status.eq(LeaveStatus::Cancelled),
                leave_requests::updated_at.eq(Utc::now()),
            ))
            .execute(conn)
            .await
            .map_err(AppError::from)?;

        Ok(u64::try_from(affected).unwrap_or(0))
    }
}
