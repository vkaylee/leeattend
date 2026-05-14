use crate::errors::AppError;
use crate::models::{LeavePolicyAssignment, LeavePolicyConfig, LeavePolicyDefinition};
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: LeavePolicyRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait LeavePolicyRepository: Send + Sync {
    async fn get_leave_types(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::LeaveType>, AppError>;
    async fn create_leave_type(
        &self,
        workspace_id: Uuid,
        code: String,
        name: String,
        is_paid: bool,
        policy_config: crate::models::LeaveTypeConfig,
    ) -> Result<crate::models::LeaveType, AppError>;
    async fn update_leave_type_config(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        policy_config: crate::models::LeaveTypeConfig,
    ) -> Result<crate::models::LeaveType, AppError>;
    async fn delete_leave_type(&self, id: Uuid, workspace_id: Uuid) -> Result<u64, AppError>;

    async fn create_policy_definition(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
        leave_type_id: Uuid,
        rule_config: LeavePolicyConfig,
    ) -> Result<LeavePolicyDefinition, AppError>;
    async fn get_policy_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<LeavePolicyDefinition>, AppError>;
    async fn update_policy_definition(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        leave_type_id: Option<Uuid>,
        rule_config: Option<LeavePolicyConfig>,
    ) -> Result<LeavePolicyDefinition, AppError>;
    async fn delete_policy_definition(&self, id: Uuid, workspace_id: Uuid)
        -> Result<u64, AppError>;
    async fn create_policy_assignment(
        &self,
        assignment: &LeavePolicyAssignment,
    ) -> Result<LeavePolicyAssignment, AppError>;
    async fn find_active_assignment_for_employee(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        date: NaiveDate,
    ) -> Result<Option<(LeavePolicyDefinition, Uuid, String)>, AppError>;
    async fn get_policy_assignments(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::LeavePolicyAssignmentDetail>, AppError>;
}
