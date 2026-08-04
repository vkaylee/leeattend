use crate::errors::AppError;
use crate::repositories::{
    department_repository::DepartmentRepository, leave_request_repository::LeaveRequestRepository,
    overtime_request_repository::OvertimeRequestRepository, ApprovalWorkflowRepository,
    EmployeeRepository, EmploymentRepository, WorkspaceConfigRepository, WorkspaceRepository,
};
use crate::services::OutboxJobService;
use std::sync::Arc;
use uuid::Uuid;

pub mod definition;
pub mod engine;
pub mod task_resolver;

#[cfg(test)]
mod tests;

#[async_trait::async_trait]
pub trait WorkflowCallbackHandler: Send + Sync {
    async fn on_workflow_completed(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_id: Uuid,
        status: &str,
        user_id: Uuid,
    ) -> Result<(), AppError>;
}

pub struct ApprovalWorkflowService {
    pub(crate) approval_workflow_repo: Arc<dyn ApprovalWorkflowRepository>,
    pub(crate) workspace_config_repo: Arc<dyn WorkspaceConfigRepository>,
    pub(crate) workspace_repo: Arc<dyn WorkspaceRepository>,
    pub(crate) employee_repo: Arc<dyn EmployeeRepository>,
    pub(crate) employment_repo: Arc<dyn EmploymentRepository>,
    pub(crate) department_repo: Arc<dyn DepartmentRepository>,
    pub(crate) leave_request_repo: Arc<dyn LeaveRequestRepository>,
    pub(crate) overtime_request_repo: Arc<dyn OvertimeRequestRepository>,
    pub(crate) daily_summary_repo: Arc<dyn crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository>,
    pub(crate) handlers: std::collections::HashMap<String, Arc<dyn WorkflowCallbackHandler>>,
    pub(crate) outbox_service: Option<Arc<OutboxJobService>>,
}

pub struct ApprovalWorkflowServiceParams {
    pub approval_workflow_repo: Arc<dyn ApprovalWorkflowRepository>,
    pub workspace_config_repo: Arc<dyn WorkspaceConfigRepository>,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
    pub employee_repo: Arc<dyn EmployeeRepository>,
    pub employment_repo: Arc<dyn EmploymentRepository>,
    pub department_repo: Arc<dyn DepartmentRepository>,
    pub leave_request_repo: Arc<dyn LeaveRequestRepository>,
    pub overtime_request_repo: Arc<dyn OvertimeRequestRepository>,
    pub daily_summary_repo: Arc<dyn crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository>,
}

impl ApprovalWorkflowService {
    pub fn new(params: ApprovalWorkflowServiceParams) -> Self {
        Self {
            approval_workflow_repo: params.approval_workflow_repo,
            workspace_config_repo: params.workspace_config_repo,
            workspace_repo: params.workspace_repo,
            employee_repo: params.employee_repo,
            employment_repo: params.employment_repo,
            department_repo: params.department_repo,
            leave_request_repo: params.leave_request_repo,
            overtime_request_repo: params.overtime_request_repo,
            daily_summary_repo: params.daily_summary_repo,
            handlers: std::collections::HashMap::new(),
            outbox_service: None,
        }
    }

    #[must_use]
    pub fn with_handlers(
        mut self,
        handlers: std::collections::HashMap<String, Arc<dyn WorkflowCallbackHandler>>,
    ) -> Self {
        self.handlers = handlers;
        self
    }

    #[must_use]
    pub fn with_outbox_service(mut self, outbox_service: Arc<OutboxJobService>) -> Self {
        self.outbox_service = Some(outbox_service);
        self
    }
}
