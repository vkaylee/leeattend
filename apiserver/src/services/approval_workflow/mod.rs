use crate::errors::AppError;
use crate::repositories::{
    department_repository::DepartmentRepository, leave_request_repository::LeaveRequestRepository,
    overtime_request_repository::OvertimeRequestRepository, ApprovalWorkflowRepository,
    EmployeeRepository, EmploymentRepository, WorkspaceConfigRepository, WorkspaceRepository,
};
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
}

impl ApprovalWorkflowService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        approval_workflow_repo: Arc<dyn ApprovalWorkflowRepository>,
        workspace_config_repo: Arc<dyn WorkspaceConfigRepository>,
        workspace_repo: Arc<dyn WorkspaceRepository>,
        employee_repo: Arc<dyn EmployeeRepository>,
        employment_repo: Arc<dyn EmploymentRepository>,
        department_repo: Arc<dyn DepartmentRepository>,
        leave_request_repo: Arc<dyn LeaveRequestRepository>,
        overtime_request_repo: Arc<dyn OvertimeRequestRepository>,
        daily_summary_repo: Arc<dyn crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository>,
    ) -> Self {
        Self {
            approval_workflow_repo,
            workspace_config_repo,
            workspace_repo,
            employee_repo,
            employment_repo,
            department_repo,
            leave_request_repo,
            overtime_request_repo,
            daily_summary_repo,
            handlers: std::collections::HashMap::new(),
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
}
