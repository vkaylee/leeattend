use crate::errors::AppResult;
use crate::models::BehalfAssignment;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait BehalfAssignmentRepository: Send + Sync {
    async fn create_assignment(
        &self,
        workspace_id: Uuid,
        assignee_employee_id: Uuid,
        target_employee_id: Option<Uuid>,
        target_department_id: Option<Uuid>,
        assignment_type: &str,
    ) -> AppResult<BehalfAssignment>;

    async fn delete_assignment(&self, workspace_id: Uuid, id: Uuid) -> AppResult<u64>;

    async fn list_all_assignments(
        &self,
        workspace_id: Uuid,
        assignment_type: &str,
    ) -> AppResult<Vec<BehalfAssignment>>;

    async fn check_behalf_permission(
        &self,
        workspace_id: Uuid,
        assignee_employee_id: Uuid,
        target_employee_id: Uuid,
        target_department_id: Option<Uuid>,
        assignment_type: &str,
    ) -> AppResult<bool>;

    /// List assignments where the given employee is the **assignee** (i.e. the person
    /// who is allowed to act on behalf of others).
    async fn list_assignments_for_assignee(
        &self,
        workspace_id: Uuid,
        assignee_employee_id: Uuid,
        assignment_type: &str,
    ) -> AppResult<Vec<BehalfAssignment>>;
}
