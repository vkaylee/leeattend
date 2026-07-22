#![allow(clippy::clone_on_ref_ptr)]
use super::repositories::Repositories;
use super::services_attendance::AttendanceServices;
use super::services_core::CoreServices;
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
    redis_pool: &Option<RedisPool>,
    repos: &Repositories,
    core: &CoreServices,
    attendance: &AttendanceServices,
) -> Result<LeaveServices, Box<dyn std::error::Error>> {
    let cache_repo = redis_pool.clone().map(|r| {
        Arc::new(crate::cache::RedisCache::new(r)) as Arc<dyn crate::repositories::CacheRepository>
    });

    let leave = Arc::new(crate::services::leave_service::LeaveService::new(
        Arc::clone(&repos.leave_balance),
        Arc::clone(&repos.leave_request),
        Arc::clone(&repos.employee),
        Arc::clone(&repos.employment),
        Arc::clone(&repos.roster),
        Arc::clone(&repos.leave_accrual_log),
        Arc::clone(&repos.workspace_config),
        Arc::clone(&repos.user),
        Arc::clone(&repos.workspace),
        cache_repo,
    ));

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
            Arc::clone(&repos.employee),
            Arc::clone(&repos.department),
            Arc::clone(&repos.leave_policy),
            Arc::clone(&repos.leave_accrual_log),
            Arc::clone(&repos.time_entry),
            Arc::clone(&repos.leave_request),
            Arc::clone(&repos.attendance_daily_summary),
            repos.contract.clone() as Arc<dyn crate::repositories::EmployeeContractRepository>,
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
            repos.approval_workflow.clone(),
            repos.workspace_config.clone(),
            repos.workspace.clone(),
            repos.employee.clone(),
            repos.employment.clone(),
            repos.department.clone(),
            repos.leave_request.clone(),
            repos.overtime_request.clone(),
            repos.attendance_daily_summary.clone(),
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
