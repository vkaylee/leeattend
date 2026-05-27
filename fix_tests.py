import re

with open("apiserver/tests/integration/common/app.rs", "r") as f:
    content = f.read()

init_block = """
    let workspace_config_repo: Arc<dyn apiserver::repositories::workspace_config_repository::WorkspaceConfigRepository> =
        Arc::new(apiserver::repositories::diesel_workspace_config_repository::DieselWorkspaceConfigRepository::new(diesel_pools.clone()));
    let approval_workflow_repo: Arc<dyn apiserver::repositories::approval_workflow_repository::ApprovalWorkflowRepository> =
        Arc::new(apiserver::repositories::diesel_approval_workflow_repository::DieselApprovalWorkflowRepository::new(diesel_pools.clone()));
    let overtime_request_repo: Arc<dyn apiserver::repositories::overtime_request_repository::OvertimeRequestRepository> =
        Arc::new(apiserver::repositories::diesel_overtime_request_repository::DieselOvertimeRequestRepository::new(diesel_pools.clone()));
    let workspace_config_service = Arc::new(apiserver::services::WorkspaceConfigService::new(
        workspace_config_repo.clone(),
    ));
    let approval_workflow_service = Arc::new(apiserver::services::ApprovalWorkflowService::new(
        approval_workflow_repo.clone(),
    ));
    let overtime_request_service = Arc::new(apiserver::services::OvertimeRequestService::new(
        overtime_request_repo.clone(),
    ));

    let payroll_service = Arc::new(apiserver::services::payroll_service::PayrollService::new("""

content = content.replace("    let payroll_service = Arc::new(apiserver::services::payroll_service::PayrollService::new(", init_block)

payroll_state_target = """            salary_base_repo: std::sync::Arc::new(
                apiserver::repositories::DieselSalaryBaseRepository::new(diesel_pools.clone()),
            ),
        },"""
payroll_state_replacement = """            salary_base_repo: std::sync::Arc::new(
                apiserver::repositories::DieselSalaryBaseRepository::new(diesel_pools.clone()),
            ),
            overtime_request_repo: overtime_request_repo.clone(),
            overtime_request_service: overtime_request_service.clone(),
        },"""
content = content.replace(payroll_state_target, payroll_state_replacement)

core_state_target = """            session_repo: session_repo.clone(),
            jwt_secret: "test_secret_tower".to_string(),
        },"""
core_state_replacement = """            session_repo: session_repo.clone(),
            jwt_secret: "test_secret_tower".to_string(),
            workspace_config_repo: workspace_config_repo.clone(),
            workspace_config_service: workspace_config_service.clone(),
            approval_workflow_repo: approval_workflow_repo.clone(),
            approval_workflow_service: approval_workflow_service.clone(),
        },"""
content = content.replace(core_state_target, core_state_replacement)

with open("apiserver/tests/integration/common/app.rs", "w") as f:
    f.write(content)

print("Replaced")
