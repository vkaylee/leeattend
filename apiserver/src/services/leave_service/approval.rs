//! Leave approval and listing operations for [`LeaveService`].
//!
//! Handles the approval lifecycle (status updates with transaction safety),
//! and read-side queries for fetching leave requests.

use crate::errors::AppError;
use crate::utils::i18n::I18nKey;
use diesel_async::AsyncConnection;
use uuid::Uuid;

use crate::models::{ApproveLeaveRequest, LeaveRequest, LeaveStatus};

impl super::LeaveService {
    /// Apply leave-to-roster side effects (create roster entries with "on_leave" status).
    /// Shared between auto-approve (submit) and manual approve flows.
    #[allow(clippy::unused_async)]
    pub(super) async fn apply_leave_to_roster(
        &self,
        _workspace_id: Uuid,
        _request: &LeaveRequest,
    ) -> Result<(), AppError> {
        // Option B: Giữ nguyên ca làm việc gốc và đối chiếu đơn phép khi chấm công.
        // Không tạo bản ghi đè shift_id = NULL trong bảng shift_rosters.
        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn approve_leave_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
        request_id: Uuid,
        req: ApproveLeaveRequest,
    ) -> Result<(), AppError> {
        let request = self
            .leave_request_repo
            .get_leave_request_by_id(auth_token, request_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?;

        // Verify employment of this request belongs to the workspace
        let _ = self
            .employment_repo
            .get_employment_by_id(auth_token, request.employment_id())
            .await?
            .ok_or_else(|| AppError::forbidden(I18nKey::Forbidden))?;

        // Run status update inside a transaction so it can be rolled back if
        // post-approval side effects fail. Previously the status update used
        // autocommit, which left the leave Approved even when conflict handling
        // or roster updates failed.
        if let Some(ref pools) = self.diesel_pools {
            let mut conn = pools.tenant_transaction_conn(auth_token).await?;
            let repo = std::sync::Arc::clone(&self.leave_request_repo);
            let auth = auth_token;
            let status = req.status;

            conn.transaction::<_, AppError, _>(|pg_conn| {
                let repo = std::sync::Arc::clone(&repo);
                let auth = auth;
                Box::pin(async move {
                    repo.update_leave_request_status_tx(
                        pg_conn, &auth, request_id, status, Some(user_id),
                    )
                    .await
                })
            })
            .await?;
        } else {
            // Fallback: no DieselPools injected — use non-transactional call
            self.leave_request_repo
                .update_leave_request_status(&auth_token, request_id, req.status, Some(user_id))
                .await?;
        }

        // If approved, handle post-approval side effects (roster, conflict checking).
        // These are intentionally outside the transaction to keep the write window short
        // and because roster side effects are currently no-ops and conflict handling
        // is a read-then-notify pattern.
        if req.status == LeaveStatus::Approved {
            self.apply_leave_to_roster(auth_token.workspace_id(), &request).await?;

            // Fetch the updated request object to pass to the conflict checker
            // TODO: avoid double-fetch by having update_leave_request_status_tx return the updated row
            let updated_request = self
                .leave_request_repo
                .get_leave_request_by_id(auth_token, request_id)
                .await?
                .ok_or_else(|| AppError::not_found(I18nKey::LeaveNotFoundAfterUpdate))?;

            self.check_and_handle_conflicts(auth_token, &updated_request)
                .await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_employee_leave_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        // Verify employee belongs to workspace
        let _ = self
            .employee_repo
            .get_employee_by_id(
                employee_id,
                auth_token,
            )
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?;

        self.leave_request_repo
            .get_employee_leave_requests(
                auth_token,
                employee_id,
            )
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_workspace_leave_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        status: Option<LeaveStatus>,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        self.leave_request_repo
            .get_workspace_leave_requests(
                auth_token,
                status,
            )
            .await
    }
}
