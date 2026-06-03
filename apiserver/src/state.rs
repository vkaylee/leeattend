use std::sync::Arc;

use crate::agent_manager::AgentManager;

use crate::repositories::assignment_rule_repository::AssignmentRuleRepository;
use crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository;
use crate::repositories::audit_log_repository::AuditLogRepository;
use crate::repositories::department_repository::DepartmentRepository;
use crate::repositories::feature_flag_repository::FeatureFlagRepository;
use crate::repositories::holiday_repository::HolidayRepository;
use crate::repositories::import_job_repository::ImportJobRepository;
use crate::repositories::leave_policy_repository::LeavePolicyRepository;
use crate::repositories::leave_request_repository::LeaveRequestRepository;
use crate::repositories::overtime_policy_repository::OvertimePolicyRepositoryTrait;
use crate::repositories::payroll_repository::PayrollRepository;
use crate::repositories::payroll_rule_repository::PayrollRuleRepository;
use crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::shift_repository::ShiftRepository;
use crate::repositories::shift_swap_repository::ShiftSwapRepository;
use crate::repositories::staging_log_repository::StagingLogRepository;
use crate::repositories::time_entry_repository::TimeEntryRepository;
use crate::repositories::user_repository::UserRepository;
use crate::repositories::workspace_stats_repository::WorkspaceStatsRepository;
use crate::repositories::{
    AgentRepository, BehalfAssignmentRepository, DieselEmployeeContractRepository,
    EmployeeRepository, WorkspaceRepository,
};
use crate::services::attendance_adjustment_service::AttendanceAdjustmentService;
use crate::services::attendance_import_service::AttendanceImportService;
use crate::services::auth_service::AuthService;
use crate::services::holiday_leave_service::HolidayLeaveService;
use crate::services::leave_accrual_service::LeaveAccrualService;
use crate::services::leave_service::LeaveService;
use crate::services::pairing_service::PairingService;
use crate::services::payroll_service::PayrollService;
use crate::services::shift_service::ShiftService;
use crate::services::shift_swap_service::ShiftSwapService;
use crate::services::workspace_service::WorkspaceService;
use crate::services::{
    AgentService, AttendanceLogService, AttendanceSummaryService, EmployeeService,
    RecruitmentService, ReportCoordinator, ReportService, TimeTrackingService,
};

pub use crate::queue::RedisPool;

// ── Domain Sub-Structs ─────────────────────────────────────────────

/// Employee management — service + repository
pub struct EmployeeState {
    pub service: Arc<EmployeeService>,
    pub recruitment_service: Arc<RecruitmentService>,
    pub repo: Arc<dyn EmployeeRepository>,
    pub contract_repo: Arc<DieselEmployeeContractRepository>,
    pub employment_phase_repo: Arc<crate::repositories::DieselEmploymentPhaseRepository>,
}

/// Time & attendance tracking, summaries, logs, staging, adjustments, imports
pub struct AttendanceState {
    pub time_tracking_service: Arc<TimeTrackingService>,
    pub summary_service: Arc<AttendanceSummaryService>,
    pub log_service: Arc<AttendanceLogService>,
    pub pairing_service: Arc<PairingService>,
    pub adjustment_service: Arc<AttendanceAdjustmentService>,
    pub import_service: Arc<AttendanceImportService>,
    pub time_entry_repo: Arc<dyn TimeEntryRepository>,
    pub staging_repo: Arc<dyn StagingLogRepository>,
    pub raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
    pub import_repo: Arc<dyn ImportJobRepository>,
    pub error_repo: Arc<dyn AttendanceLogErrorRepository>,
    pub policy_repo:
        Arc<dyn crate::repositories::attendance_policy_repository::AttendancePolicyRepository>,
    pub attendance_daily_summary_repo:
        Arc<dyn crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository>,
    pub behalf_assignment_repo: Arc<dyn BehalfAssignmentRepository>,
}

/// Shift management — shifts, rosters, swaps, patterns
pub struct ShiftState {
    pub service: Arc<ShiftService>,
    pub swap_service: Arc<ShiftSwapService>,
    pub roster_repo: Arc<dyn RosterRepository>,
    pub shift_repo: Arc<dyn ShiftRepository>,
    pub swap_repo: Arc<dyn ShiftSwapRepository>,
    pub assignment_rule_repo: Arc<dyn AssignmentRuleRepository>,
    pub working_pattern_repo:
        Arc<dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait>,
}

/// Leave management — requests, policies, holidays, accrual
pub struct LeaveState {
    pub service: Arc<LeaveService>,
    pub holiday_leave_service: Arc<HolidayLeaveService>,
    pub accrual_service: Arc<LeaveAccrualService>,
    pub request_repo: Arc<dyn LeaveRequestRepository>,
    pub balance_repo:
        Arc<dyn crate::repositories::leave_balance_repository::LeaveBalanceRepository>,
    pub policy_repo: Arc<dyn LeavePolicyRepository>,
    pub holiday_repo: Arc<dyn HolidayRepository>,
}

/// Payroll — calculation, settings, repos
pub struct PayrollState {
    pub service: Arc<PayrollService>,
    pub repo: Arc<dyn PayrollRepository>,
    pub rule_repo: Arc<dyn PayrollRuleRepository>,
    pub department_repo: Arc<dyn DepartmentRepository>,
    pub feature_flag_repo: Arc<dyn FeatureFlagRepository>,
    pub overtime_policy_repo: Arc<dyn OvertimePolicyRepositoryTrait>,
    pub salary_base_repo:
        Arc<dyn crate::repositories::salary_base_repository::SalaryBaseRepository>,
    pub overtime_request_repo:
        Arc<dyn crate::repositories::overtime_request_repository::OvertimeRequestRepository>,
    pub overtime_request_service: Arc<crate::services::OvertimeRequestService>,
}

/// Reports & async coordination
pub struct ReportState {
    pub service: Arc<ReportService>,
    pub coordinator: Option<Arc<ReportCoordinator>>,
    pub stats_repo: Arc<dyn WorkspaceStatsRepository>,
}

/// Auth, workspace, user management
pub struct CoreState {
    pub auth_service: Arc<AuthService>,
    pub workspace_service: Arc<WorkspaceService>,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
    pub audit_repo: Arc<dyn AuditLogRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub session_repo: Arc<dyn crate::repositories::session_repository::SessionRepository>,
    pub jwt_secret: String,

    pub workspace_config_repo:
        Arc<dyn crate::repositories::workspace_config_repository::WorkspaceConfigRepository>,
    pub approval_workflow_repo:
        Arc<dyn crate::repositories::approval_workflow_repository::ApprovalWorkflowRepository>,

    pub workspace_config_service: Arc<crate::services::WorkspaceConfigService>,
    pub approval_workflow_service: Arc<crate::services::ApprovalWorkflowService>,
    pub location_service: Arc<crate::services::LocationService>,
    pub location_repo: Arc<dyn crate::repositories::LocationRepository>,
    pub employer_service: Arc<crate::services::EmployerService>,
    pub employer_repo: Arc<dyn crate::repositories::EmployerRepository>,
}

/// AI agent management
pub struct AgentState {
    pub manager: AgentManager,
    pub service: Arc<AgentService>,
    pub repo: Arc<dyn AgentRepository>,
}

/// Infrastructure — DB, Redis, caches
pub struct InfraState {
    pub diesel: crate::diesel_pool::DieselPools,
    pub redis: Option<RedisPool>,
    pub cache: Option<Arc<dyn crate::repositories::CacheRepository>>,
    pub roster_cache: Option<Arc<crate::cache::RosterCache>>,
    pub queue: Option<Arc<dyn crate::repositories::queue_repository::QueueRepository>>,
    pub tenant_manager: Arc<dyn crate::supavisor::TenantManager>,
    pub tenant_provisioner: Arc<dyn crate::services::tenant_provisioner::TenantProvisionerTrait>,
    pub crypto_service: Option<Arc<crate::crypto::CryptoService>>,
    pub system_db_cluster_repo: Arc<dyn crate::repositories::SystemDbClusterRepository>,
}

// ── Composed AppState ──────────────────────────────────────────────

pub struct AppState {
    pub employees: EmployeeState,
    pub attendance: AttendanceState,
    pub shifts: ShiftState,
    pub leaves: LeaveState,
    pub payroll: PayrollState,
    pub reports: ReportState,
    pub core: CoreState,
    pub agents: AgentState,
    pub infra: InfraState,
}
