use crate::errors::AppResult;
use crate::models::OtClaimBehalfAssignment;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait OtClaimBehalfAssignmentRepository: Send + Sync {
    async fn create_assignment(
        &self,
        workspace_id: Uuid,
        assignee_employee_id: Uuid,
        target_employee_id: Option<Uuid>,
        target_department_id: Option<Uuid>,
    ) -> AppResult<OtClaimBehalfAssignment>;

    async fn delete_assignment(&self, workspace_id: Uuid, id: Uuid) -> AppResult<u64>;

    async fn list_all_assignments(
        &self,
        workspace_id: Uuid,
    ) -> AppResult<Vec<OtClaimBehalfAssignment>>;

    async fn check_behalf_permission(
        &self,
        workspace_id: Uuid,
        assignee_employee_id: Uuid,
        target_employee_id: Uuid,
        target_department_id: Option<Uuid>,
    ) -> AppResult<bool>;
}
