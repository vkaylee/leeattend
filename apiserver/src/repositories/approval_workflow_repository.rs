use crate::errors::AppError;
use crate::models::{
    ApprovalActionLog, ApprovalWorkflow, ApprovalWorkflowInstance, ApprovalWorkflowStep,
    ApprovalWorkflowTask,
};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ApprovalWorkflowRepository: Send + Sync {
    async fn create_workflow(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow: ApprovalWorkflow,
    ) -> Result<ApprovalWorkflow, AppError>;

    async fn create_steps(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        steps: Vec<ApprovalWorkflowStep>,
    ) -> Result<Vec<ApprovalWorkflowStep>, AppError>;

    async fn create_workflow_transactional(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow: ApprovalWorkflow,
        steps: Vec<ApprovalWorkflowStep>,
    ) -> Result<(ApprovalWorkflow, Vec<ApprovalWorkflowStep>), AppError>;

    async fn update_workflow_definition_transactional(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow: ApprovalWorkflow,
        steps: Vec<ApprovalWorkflowStep>,
    ) -> Result<(ApprovalWorkflow, Vec<ApprovalWorkflowStep>), AppError>;

    async fn get_active_workflow(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        entity_type: &str,
    ) -> Result<Option<ApprovalWorkflow>, AppError>;

    async fn get_steps(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow_id: Uuid,
    ) -> Result<Vec<ApprovalWorkflowStep>, AppError>;

    async fn log_action(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        log: ApprovalActionLog,
    ) -> Result<ApprovalActionLog, AppError>;

    async fn get_action_logs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_id: Uuid,
    ) -> Result<Vec<ApprovalActionLog>, AppError>;

    async fn create_instance(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        instance: ApprovalWorkflowInstance,
    ) -> Result<ApprovalWorkflowInstance, AppError>;

    async fn update_instance(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        instance: ApprovalWorkflowInstance,
    ) -> Result<ApprovalWorkflowInstance, AppError>;

    async fn get_instance_by_entity(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Option<ApprovalWorkflowInstance>, AppError>;

    async fn get_instance_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        instance_id: Uuid,
    ) -> Result<Option<ApprovalWorkflowInstance>, AppError>;

    async fn get_instances_by_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        instance_ids: &[Uuid],
    ) -> Result<Vec<ApprovalWorkflowInstance>, AppError>;

    async fn get_workflow_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow_id: Uuid,
    ) -> Result<Option<ApprovalWorkflow>, AppError>;

    async fn list_workflows(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        entity_type: Option<String>,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<ApprovalWorkflow>, i64), AppError>;

    async fn update_workflow(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow: ApprovalWorkflow,
    ) -> Result<ApprovalWorkflow, AppError>;

    async fn delete_workflow(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow_id: Uuid,
    ) -> Result<(), AppError>;

    async fn delete_steps_by_workflow_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow_id: Uuid,
    ) -> Result<(), AppError>;

    async fn get_steps_for_workflows(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow_ids: Vec<Uuid>,
    ) -> Result<Vec<ApprovalWorkflowStep>, AppError>;

    async fn create_tasks(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        tasks: Vec<ApprovalWorkflowTask>,
    ) -> Result<Vec<ApprovalWorkflowTask>, AppError>;

    async fn update_task(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        task: ApprovalWorkflowTask,
    ) -> Result<ApprovalWorkflowTask, AppError>;

    async fn get_active_tasks_for_instance(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        instance_id: Uuid,
    ) -> Result<Vec<ApprovalWorkflowTask>, AppError>;

    async fn get_pending_tasks_for_user(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
        role_ids: Vec<Uuid>,
        permissions: Vec<String>,
        managed_dept_ids: Vec<Uuid>,
    ) -> Result<Vec<ApprovalWorkflowTask>, AppError>;
}
