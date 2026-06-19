#![allow(clippy::clone_on_ref_ptr)]
use super::repositories::Repositories;
pub use super::services_attendance::{build_attendance_services, AttendanceServices};
pub use super::services_core::{build_core_services, CoreServices};
pub use super::services_leave::{build_leave_services, LeaveServices};
use crate::agent_manager::AgentManager;
use crate::crypto::CryptoService;
use crate::diesel_pool::DieselPools;
use crate::queue::RedisPool;
use std::sync::Arc;
use tokio::task::JoinHandle;

pub struct Services {
    pub lifecycle: Arc<crate::services::LifecycleService>,
    pub employee: Arc<crate::services::EmployeeService>,
    pub outbox: Arc<crate::services::OutboxJobService>,
    pub time_tracking: Arc<crate::services::TimeTrackingService>,
    pub attendance_summary: Arc<crate::services::AttendanceSummaryService>,
    pub report: Arc<crate::services::ReportService>,
    pub pairing: Arc<crate::services::pairing_service::PairingService>,
    pub adjustment:
        Arc<crate::services::attendance_adjustment_service::AttendanceAdjustmentService>,
    pub auth: Arc<crate::services::auth_service::AuthService>,
    pub shift: Arc<crate::services::shift_service::ShiftService>,
    pub tenant_provisioner: Arc<dyn crate::services::tenant_provisioner::TenantProvisionerTrait>,
    pub workspace: Arc<crate::services::workspace_service::WorkspaceService>,
    pub leave: Arc<crate::services::leave_service::LeaveService>,
    pub shift_swap: Arc<crate::services::shift_swap_service::ShiftSwapService>,
    pub holiday_leave: Arc<crate::services::holiday_leave_service::HolidayLeaveService>,
    pub leave_accrual: Arc<crate::services::leave_accrual_service::LeaveAccrualService>,
    pub overtime_request: Arc<crate::services::OvertimeRequestService>,
    pub workspace_config: Arc<crate::services::WorkspaceConfigService>,
    pub approval_workflow: Arc<crate::services::ApprovalWorkflowService>,
    pub location: Arc<crate::services::LocationService>,
    pub employer: Arc<crate::services::EmployerService>,
    pub custom_field: Arc<crate::services::CustomFieldService>,
    pub payroll: Arc<crate::services::payroll_service::PayrollService>,
    pub recruitment: Arc<crate::services::RecruitmentService>,
    pub storage_resolver: Arc<crate::services::storage::resolver::StorageResolver>,
    pub file: Arc<crate::services::storage::file_service::FileService>,
    pub signature: Arc<crate::services::signature_service::SignatureService>,
    pub scan: Arc<dyn crate::services::storage::VirusScanner>,
    pub agent: Arc<crate::services::AgentService>,
    pub attendance: Arc<crate::services::AttendanceLogService>,
    pub import: Arc<crate::services::attendance_import_service::AttendanceImportService>,
}

pub async fn build_services(
    diesel_pools: &DieselPools,
    redis_pool: &Option<RedisPool>,
    crypto_service: &Option<Arc<CryptoService>>,
    agent_manager: &AgentManager,
    repos: &Repositories,
) -> Result<(Services, JoinHandle<()>), Box<dyn std::error::Error>> {
    let core = build_core_services(
        diesel_pools,
        redis_pool,
        crypto_service,
        agent_manager,
        repos,
    )
    .await?;
    let (attendance, log_worker_handle) =
        build_attendance_services(diesel_pools, redis_pool, crypto_service, repos).await?;
    let leave = build_leave_services(redis_pool, repos, &core, &attendance)?;

    let roster_provider: Arc<dyn crate::services::payroll_service::RosterDataProvider> = Arc::new(
        crate::services::payroll_service::DirectRosterDataProvider(Arc::clone(&repos.roster)),
    );

    let payroll = Arc::new(crate::services::payroll_service::PayrollService::new(
        crate::services::payroll_service::PayrollServiceArgs {
            payroll_repo: Arc::clone(&repos.payroll),
            department_repo: Arc::clone(&repos.department),
            time_entry_repo: Arc::clone(&repos.time_entry),
            workspace_repo: Arc::clone(&repos.workspace),
            holiday_repo: repos.holiday.clone(),
            roster_repo: repos.roster.clone(),
            leave_repo: repos.leave_request.clone(),
            policy_repo: repos.attendance_policy.clone(),
            employee_repo: Arc::clone(&repos.employee),
            overtime_policy_repo: Arc::clone(&repos.overtime_policy),
            salary_base_repo: Arc::clone(&repos.salary_base),
            roster_provider,
            contract_repo: Arc::clone(&repos.contract)
                as Arc<dyn crate::repositories::EmployeeContractRepository>,
            employment_repo: Arc::clone(&repos.employment),
        },
    ));

    Ok((
        Services {
            lifecycle: core.lifecycle,
            employee: core.employee,
            outbox: core.outbox,
            time_tracking: attendance.time_tracking,
            attendance_summary: attendance.attendance_summary,
            report: attendance.report,
            pairing: attendance.pairing,
            adjustment: attendance.adjustment,
            auth: core.auth,
            shift: core.shift,
            tenant_provisioner: core.tenant_provisioner,
            workspace: core.workspace,
            leave: leave.leave,
            shift_swap: leave.shift_swap,
            holiday_leave: leave.holiday_leave,
            leave_accrual: leave.leave_accrual,
            overtime_request: leave.overtime_request,
            workspace_config: leave.workspace_config,
            approval_workflow: leave.approval_workflow,
            location: core.location,
            employer: core.employer,
            custom_field: core.custom_field,
            payroll,
            recruitment: core.recruitment,
            storage_resolver: core.storage_resolver,
            file: core.file,
            signature: core.signature,
            scan: core.scan,
            agent: core.agent,
            attendance: attendance.attendance,
            import: attendance.import,
        },
        log_worker_handle,
    ))
}
