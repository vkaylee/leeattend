#![allow(clippy::clone_on_ref_ptr)]
use super::repositories::Repositories;
use super::services_attendance::AttendanceServices;
use super::services_core::CoreServices;
use crate::diesel_pool::DieselPools;
use crate::queue::RedisPool;
use std::sync::Arc;

pub struct LeaveServices {
    pub leave: Arc<crate::services::leave_service::LeaveService>,
    pub shift_swap: Arc<crate::services::shift_swap_service::ShiftSwapService>,
    pub holiday_leave: Arc<crate::services::holiday_leave_service::HolidayLeaveService>,
    pub leave_accrual: Arc<crate::services::leave_accrual_service::LeaveAccrualService>,
    pub overtime_request: Arc<crate::services::OvertimeRequestService>,
    pub workspace_config: Arc<crate::services::WorkspaceConfigService>,
    pub approval_workflow: Arc<crate::services::ApprovalWorkflowService>,
}

pub fn build_leave_services(
    diesel_pools: &DieselPools,
    redis_pool: &Option<RedisPool>,
    repos: &Repositories,
    core: &CoreServices,
    attendance: &AttendanceServices,
) -> Result<LeaveServices, Box<dyn std::error::Error>> {
    let cache_repo = redis_pool.clone().map(|r| {
        Arc::new(crate::cache::RedisCache::new(r)) as Arc<dyn crate::repositories::CacheRepository>
    });

    let leave = Arc::new(
        crate::services::leave_service::LeaveService::new(
            crate::services::leave_service::LeaveServiceParams {
                leave_balance_repo: Arc::clone(&repos.leave_balance),
                leave_request_repo: Arc::clone(&repos.leave_request),
                employee_repo: Arc::clone(&repos.employee),
                employment_repo: Arc::clone(&repos.employment),
                roster_repo: Arc::clone(&repos.roster),
                leave_accrual_log_repo: Arc::clone(&repos.leave_accrual_log),
                workspace_config_repo: Arc::clone(&repos.workspace_config),
                user_repo: Arc::clone(&repos.user),
                workspace_repo: Arc::clone(&repos.workspace),
                cache: cache_repo,
            },
        )
        .with_diesel_pools(diesel_pools.clone()),
    );

    let shift_swap = Arc::new(crate::services::shift_swap_service::ShiftSwapService::new(
        Arc::clone(&repos.shift_swap),
        Arc::clone(&repos.roster),
        Arc::clone(&repos.employee),
        Arc::clone(&repos.payroll),
    ));

    let holiday_leave = Arc::new(
        crate::services::holiday_leave_service::HolidayLeaveService::new(
            Arc::clone(&repos.employee),
            Arc::clone(&repos.employment),
            Arc::clone(&repos.leave_balance),
            Arc::clone(&repos.leave_request),
            Arc::clone(&repos.roster),
            Arc::clone(&repos.holiday),
            Arc::clone(&repos.leave_policy),
        ),
    );

    let leave_accrual = Arc::new(
        crate::services::leave_accrual_service::LeaveAccrualService::new(
            crate::services::leave_accrual_service::LeaveAccrualServiceParams {
                employee_repo: Arc::clone(&repos.employee),
                department_repo: Arc::clone(&repos.department),
                leave_policy_repo: Arc::clone(&repos.leave_policy),
                accrual_log_repo: Arc::clone(&repos.leave_accrual_log),
                time_entry_repo: Arc::clone(&repos.time_entry),
                leave_request_repo: Arc::clone(&repos.leave_request),
                daily_summary_repo: Arc::clone(&repos.attendance_daily_summary),
                contract_repo: repos.contract.clone()
                    as Arc<dyn crate::repositories::EmployeeContractRepository>,
            },
        ),
    );

    let overtime_request = Arc::new(crate::services::OvertimeRequestService::new(
        repos.overtime_request.clone(),
    ));
    let workspace_config = Arc::new(crate::services::WorkspaceConfigService::new(
        repos.workspace_config.clone(),
    ));

    let mut workflow_handlers: std::collections::HashMap<
        String,
        Arc<dyn crate::services::approval_workflow::WorkflowCallbackHandler>,
    > = std::collections::HashMap::new();
    workflow_handlers.insert(
        "attendance_adjustment".to_string(),
        attendance.adjustment.clone(),
    );
    workflow_handlers.insert("leave_request".to_string(), leave.clone());
    workflow_handlers.insert("ot_request".to_string(), overtime_request.clone());
    workflow_handlers.insert(
        "ot_claim".to_string(),
        attendance.attendance_summary.clone(),
    );

    let approval_workflow = Arc::new(
        crate::services::ApprovalWorkflowService::new(
            crate::services::ApprovalWorkflowServiceParams {
                approval_workflow_repo: repos.approval_workflow.clone(),
                workspace_config_repo: repos.workspace_config.clone(),
                workspace_repo: repos.workspace.clone(),
                employee_repo: repos.employee.clone(),
                employment_repo: repos.employment.clone(),
                department_repo: repos.department.clone(),
                leave_request_repo: repos.leave_request.clone(),
                overtime_request_repo: repos.overtime_request.clone(),
                daily_summary_repo: repos.attendance_daily_summary.clone(),
            },
        )
        .with_handlers(workflow_handlers)
        .with_outbox_service(Arc::clone(&core.outbox)),
    );

    Ok(LeaveServices {
        leave,
        shift_swap,
        holiday_leave,
        leave_accrual,
        overtime_request,
        workspace_config,
        approval_workflow,
    })
}
