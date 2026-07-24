//! Approval workflow integration for [`LeaveService`].
//!
//! Implements [`WorkflowCallbackHandler`] so that when the approval workflow
//! engine completes a leave request, it routes back to `approve_leave_request`.

use crate::errors::AppError;
use crate::models::{ApproveLeaveRequest, LeaveStatus};
use uuid::Uuid;

#[async_trait::async_trait]
impl crate::services::approval_workflow::WorkflowCallbackHandler for super::LeaveService {
    async fn on_workflow_completed(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_id: Uuid,
        status: &str,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let leave_status = match status {
            "approved" => LeaveStatus::Approved,
            "rejected" => LeaveStatus::Rejected,
            _ => LeaveStatus::Pending,
        };
        self.approve_leave_request(
            workspace_id,
            user_id,
            entity_id,
            ApproveLeaveRequest {
                status: leave_status,
            },
        )
        .await
    }
}
