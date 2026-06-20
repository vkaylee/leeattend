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
use crate::services::pairing::PairingProcessor;
use crate::services::payroll_service::PayrollService;
use crate::services::shift_service::ShiftService;
use crate::services::shift_swap_service::ShiftSwapService;
use crate::services::signature_service::SignatureService;
use crate::services::workspace_service::WorkspaceService;
use crate::services::{
    AgentService, AttendanceLogService, AttendanceSummaryService, EmployeeService,
    RecruitmentService, ReportCoordinator, ReportService, TimeTrackingService,
};

pub use crate::queue::RedisPool;

// ── Domain Sub-Structs ─────────────────────────────────────────────

/// Employee management — service + repository
pub struct EmployeeState {
    service: Arc<EmployeeService>,
    recruitment_service: Arc<RecruitmentService>,
    repo: Arc<dyn EmployeeRepository>,
    employment_repo: Arc<dyn crate::repositories::EmploymentRepository>,
    contract_repo: Arc<DieselEmployeeContractRepository>,
    employment_phase_repo: Arc<crate::repositories::DieselEmploymentPhaseRepository>,
}

impl EmployeeState {
    pub fn new(
        service: Arc<EmployeeService>,
        recruitment_service: Arc<RecruitmentService>,
        repo: Arc<dyn EmployeeRepository>,
        employment_repo: Arc<dyn crate::repositories::EmploymentRepository>,
        contract_repo: Arc<DieselEmployeeContractRepository>,
        employment_phase_repo: Arc<crate::repositories::DieselEmploymentPhaseRepository>,
    ) -> Self {
        Self {
            service,
            recruitment_service,
            repo,
            employment_repo,
            contract_repo,
            employment_phase_repo,
        }
    }

    pub fn service(&self) -> &Arc<EmployeeService> {
        &self.service
    }

    pub fn recruitment_service(&self) -> &Arc<RecruitmentService> {
        &self.recruitment_service
    }

    pub fn repo(&self) -> &Arc<dyn EmployeeRepository> {
        &self.repo
    }

    pub fn employment_repo(&self) -> &Arc<dyn crate::repositories::EmploymentRepository> {
        &self.employment_repo
    }

    pub fn contract_repo(&self) -> &Arc<DieselEmployeeContractRepository> {
        &self.contract_repo
    }

    pub fn employment_phase_repo(
        &self,
    ) -> &Arc<crate::repositories::DieselEmploymentPhaseRepository> {
        &self.employment_phase_repo
    }
}

/// Time & attendance tracking, summaries, logs, staging, adjustments, imports
pub struct AttendanceState {
    time_tracking_service: Arc<TimeTrackingService>,
    summary_service: Arc<AttendanceSummaryService>,
    log_service: Arc<AttendanceLogService>,
    pairing_service: Arc<dyn PairingProcessor>,
    adjustment_service: Arc<AttendanceAdjustmentService>,
    import_service: Arc<AttendanceImportService>,
    time_entry_repo: Arc<dyn TimeEntryRepository>,
    staging_repo: Arc<dyn StagingLogRepository>,
    raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
    import_repo: Arc<dyn ImportJobRepository>,
    error_repo: Arc<dyn AttendanceLogErrorRepository>,
    policy_repo: Arc<dyn crate::repositories::attendance_policy_repository::AttendancePolicyRepository>,
    attendance_daily_summary_repo: Arc<dyn crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository>,
    behalf_assignment_repo: Arc<dyn BehalfAssignmentRepository>,
}

impl AttendanceState {
    pub fn new(
        time_tracking_service: Arc<TimeTrackingService>,
        summary_service: Arc<AttendanceSummaryService>,
        log_service: Arc<AttendanceLogService>,
        pairing_service: Arc<dyn PairingProcessor>,
        adjustment_service: Arc<AttendanceAdjustmentService>,
        import_service: Arc<AttendanceImportService>,
        time_entry_repo: Arc<dyn TimeEntryRepository>,
        staging_repo: Arc<dyn StagingLogRepository>,
        raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
        import_repo: Arc<dyn ImportJobRepository>,
        error_repo: Arc<dyn AttendanceLogErrorRepository>,
        policy_repo: Arc<
            dyn crate::repositories::attendance_policy_repository::AttendancePolicyRepository,
        >,
        attendance_daily_summary_repo: Arc<dyn crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository>,
        behalf_assignment_repo: Arc<dyn BehalfAssignmentRepository>,
    ) -> Self {
        Self {
            time_tracking_service,
            summary_service,
            log_service,
            pairing_service,
            adjustment_service,
            import_service,
            time_entry_repo,
            staging_repo,
            raw_log_repo,
            import_repo,
            error_repo,
            policy_repo,
            attendance_daily_summary_repo,
            behalf_assignment_repo,
        }
    }

    pub fn time_tracking_service(&self) -> &Arc<TimeTrackingService> {
        &self.time_tracking_service
    }

    pub fn summary_service(&self) -> &Arc<AttendanceSummaryService> {
        &self.summary_service
    }

    pub fn log_service(&self) -> &Arc<AttendanceLogService> {
        &self.log_service
    }

    pub fn pairing_service(&self) -> &Arc<dyn PairingProcessor> {
        &self.pairing_service
    }

    pub fn adjustment_service(&self) -> &Arc<AttendanceAdjustmentService> {
        &self.adjustment_service
    }

    pub fn import_service(&self) -> &Arc<AttendanceImportService> {
        &self.import_service
    }

    pub fn time_entry_repo(&self) -> &Arc<dyn TimeEntryRepository> {
        &self.time_entry_repo
    }

    pub fn staging_repo(&self) -> &Arc<dyn StagingLogRepository> {
        &self.staging_repo
    }

    pub fn raw_log_repo(&self) -> &Arc<dyn RawAttendanceLogRepository> {
        &self.raw_log_repo
    }

    pub fn import_repo(&self) -> &Arc<dyn ImportJobRepository> {
        &self.import_repo
    }

    pub fn error_repo(&self) -> &Arc<dyn AttendanceLogErrorRepository> {
        &self.error_repo
    }

    pub fn policy_repo(
        &self,
    ) -> &Arc<dyn crate::repositories::attendance_policy_repository::AttendancePolicyRepository>
    {
        &self.policy_repo
    }

    pub fn attendance_daily_summary_repo(&self) -> &Arc<dyn crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository>{
        &self.attendance_daily_summary_repo
    }

    pub fn behalf_assignment_repo(&self) -> &Arc<dyn BehalfAssignmentRepository> {
        &self.behalf_assignment_repo
    }
}

/// Shift management — shifts, rosters, swaps, patterns
pub struct ShiftState {
    service: Arc<ShiftService>,
    swap_service: Arc<ShiftSwapService>,
    roster_repo: Arc<dyn RosterRepository>,
    shift_repo: Arc<dyn ShiftRepository>,
    swap_repo: Arc<dyn ShiftSwapRepository>,
    assignment_rule_repo: Arc<dyn AssignmentRuleRepository>,
    working_pattern_repo:
        Arc<dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait>,
}

impl ShiftState {
    pub fn new(
        service: Arc<ShiftService>,
        swap_service: Arc<ShiftSwapService>,
        roster_repo: Arc<dyn RosterRepository>,
        shift_repo: Arc<dyn ShiftRepository>,
        swap_repo: Arc<dyn ShiftSwapRepository>,
        assignment_rule_repo: Arc<dyn AssignmentRuleRepository>,
        working_pattern_repo: Arc<
            dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait,
        >,
    ) -> Self {
        Self {
            service,
            swap_service,
            roster_repo,
            shift_repo,
            swap_repo,
            assignment_rule_repo,
            working_pattern_repo,
        }
    }

    pub fn service(&self) -> &Arc<ShiftService> {
        &self.service
    }

    pub fn swap_service(&self) -> &Arc<ShiftSwapService> {
        &self.swap_service
    }

    pub fn roster_repo(&self) -> &Arc<dyn RosterRepository> {
        &self.roster_repo
    }

    pub fn shift_repo(&self) -> &Arc<dyn ShiftRepository> {
        &self.shift_repo
    }

    pub fn swap_repo(&self) -> &Arc<dyn ShiftSwapRepository> {
        &self.swap_repo
    }

    pub fn assignment_rule_repo(&self) -> &Arc<dyn AssignmentRuleRepository> {
        &self.assignment_rule_repo
    }

    pub fn working_pattern_repo(
        &self,
    ) -> &Arc<dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait>
    {
        &self.working_pattern_repo
    }
}

/// Leave management — requests, policies, holidays, accrual
pub struct LeaveState {
    service: Arc<LeaveService>,
    holiday_leave_service: Arc<HolidayLeaveService>,
    accrual_service: Arc<LeaveAccrualService>,
    request_repo: Arc<dyn LeaveRequestRepository>,
    balance_repo: Arc<dyn crate::repositories::leave_balance_repository::LeaveBalanceRepository>,
    policy_repo: Arc<dyn LeavePolicyRepository>,
    holiday_repo: Arc<dyn HolidayRepository>,
}

impl LeaveState {
    pub fn new(
        service: Arc<LeaveService>,
        holiday_leave_service: Arc<HolidayLeaveService>,
        accrual_service: Arc<LeaveAccrualService>,
        request_repo: Arc<dyn LeaveRequestRepository>,
        balance_repo: Arc<
            dyn crate::repositories::leave_balance_repository::LeaveBalanceRepository,
        >,
        policy_repo: Arc<dyn LeavePolicyRepository>,
        holiday_repo: Arc<dyn HolidayRepository>,
    ) -> Self {
        Self {
            service,
            holiday_leave_service,
            accrual_service,
            request_repo,
            balance_repo,
            policy_repo,
            holiday_repo,
        }
    }

    pub fn service(&self) -> &Arc<LeaveService> {
        &self.service
    }

    pub fn holiday_leave_service(&self) -> &Arc<HolidayLeaveService> {
        &self.holiday_leave_service
    }

    pub fn accrual_service(&self) -> &Arc<LeaveAccrualService> {
        &self.accrual_service
    }

    pub fn request_repo(&self) -> &Arc<dyn LeaveRequestRepository> {
        &self.request_repo
    }

    pub fn balance_repo(
        &self,
    ) -> &Arc<dyn crate::repositories::leave_balance_repository::LeaveBalanceRepository> {
        &self.balance_repo
    }

    pub fn policy_repo(&self) -> &Arc<dyn LeavePolicyRepository> {
        &self.policy_repo
    }

    pub fn holiday_repo(&self) -> &Arc<dyn HolidayRepository> {
        &self.holiday_repo
    }
}

/// Payroll — calculation, settings, repos
pub struct PayrollState {
    service: Arc<PayrollService>,
    repo: Arc<dyn PayrollRepository>,
    rule_repo: Arc<dyn PayrollRuleRepository>,
    department_repo: Arc<dyn DepartmentRepository>,
    feature_flag_repo: Arc<dyn FeatureFlagRepository>,
    overtime_policy_repo: Arc<dyn OvertimePolicyRepositoryTrait>,
    salary_base_repo: Arc<dyn crate::repositories::salary_base_repository::SalaryBaseRepository>,
    overtime_request_repo:
        Arc<dyn crate::repositories::overtime_request_repository::OvertimeRequestRepository>,
    overtime_request_service: Arc<crate::services::OvertimeRequestService>,
}

impl PayrollState {
    pub fn new(
        service: Arc<PayrollService>,
        repo: Arc<dyn PayrollRepository>,
        rule_repo: Arc<dyn PayrollRuleRepository>,
        department_repo: Arc<dyn DepartmentRepository>,
        feature_flag_repo: Arc<dyn FeatureFlagRepository>,
        overtime_policy_repo: Arc<dyn OvertimePolicyRepositoryTrait>,
        salary_base_repo: Arc<
            dyn crate::repositories::salary_base_repository::SalaryBaseRepository,
        >,
        overtime_request_repo: Arc<
            dyn crate::repositories::overtime_request_repository::OvertimeRequestRepository,
        >,
        overtime_request_service: Arc<crate::services::OvertimeRequestService>,
    ) -> Self {
        Self {
            service,
            repo,
            rule_repo,
            department_repo,
            feature_flag_repo,
            overtime_policy_repo,
            salary_base_repo,
            overtime_request_repo,
            overtime_request_service,
        }
    }

    pub fn service(&self) -> &Arc<PayrollService> {
        &self.service
    }

    pub fn repo(&self) -> &Arc<dyn PayrollRepository> {
        &self.repo
    }

    pub fn rule_repo(&self) -> &Arc<dyn PayrollRuleRepository> {
        &self.rule_repo
    }

    pub fn department_repo(&self) -> &Arc<dyn DepartmentRepository> {
        &self.department_repo
    }

    pub fn feature_flag_repo(&self) -> &Arc<dyn FeatureFlagRepository> {
        &self.feature_flag_repo
    }

    pub fn overtime_policy_repo(&self) -> &Arc<dyn OvertimePolicyRepositoryTrait> {
        &self.overtime_policy_repo
    }

    pub fn salary_base_repo(
        &self,
    ) -> &Arc<dyn crate::repositories::salary_base_repository::SalaryBaseRepository> {
        &self.salary_base_repo
    }

    pub fn overtime_request_repo(
        &self,
    ) -> &Arc<dyn crate::repositories::overtime_request_repository::OvertimeRequestRepository> {
        &self.overtime_request_repo
    }

    pub fn overtime_request_service(&self) -> &Arc<crate::services::OvertimeRequestService> {
        &self.overtime_request_service
    }
}

/// Reports & async coordination
pub struct ReportState {
    service: Arc<ReportService>,
    coordinator: Option<Arc<ReportCoordinator>>,
    stats_repo: Arc<dyn WorkspaceStatsRepository>,
}

impl ReportState {
    pub fn new(
        service: Arc<ReportService>,
        coordinator: Option<Arc<ReportCoordinator>>,
        stats_repo: Arc<dyn WorkspaceStatsRepository>,
    ) -> Self {
        Self {
            service,
            coordinator,
            stats_repo,
        }
    }

    pub fn service(&self) -> &Arc<ReportService> {
        &self.service
    }

    pub fn coordinator(&self) -> Option<&Arc<ReportCoordinator>> {
        self.coordinator.as_ref()
    }

    pub fn stats_repo(&self) -> &Arc<dyn WorkspaceStatsRepository> {
        &self.stats_repo
    }
}

/// Auth, workspace, user management
pub struct CoreState {
    auth_service: Arc<AuthService>,
    workspace_service: Arc<WorkspaceService>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
    audit_repo: Arc<dyn AuditLogRepository>,
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn crate::repositories::session_repository::SessionRepository>,
    jwt_secret: String,
    workspace_config_repo:
        Arc<dyn crate::repositories::workspace_config_repository::WorkspaceConfigRepository>,
    approval_workflow_repo:
        Arc<dyn crate::repositories::approval_workflow_repository::ApprovalWorkflowRepository>,
    workspace_config_service: Arc<crate::services::WorkspaceConfigService>,
    approval_workflow_service: Arc<crate::services::ApprovalWorkflowService>,
    location_service: Arc<crate::services::LocationService>,
    location_repo: Arc<dyn crate::repositories::LocationRepository>,
    employer_service: Arc<crate::services::EmployerService>,
    employer_repo: Arc<dyn crate::repositories::EmployerRepository>,
    custom_field_service: Arc<crate::services::CustomFieldService>,
    custom_field_repo: Arc<dyn crate::repositories::CustomFieldRepository>,
    lifecycle_service: Arc<crate::services::LifecycleService>,
    lifecycle_repo: Arc<dyn crate::repositories::LifecycleChecklistRepository>,
    signature_service: Arc<SignatureService>,
}

impl CoreState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        auth_service: Arc<AuthService>,
        workspace_service: Arc<WorkspaceService>,
        workspace_repo: Arc<dyn WorkspaceRepository>,
        audit_repo: Arc<dyn AuditLogRepository>,
        user_repo: Arc<dyn UserRepository>,
        session_repo: Arc<dyn crate::repositories::session_repository::SessionRepository>,
        jwt_secret: String,
        workspace_config_repo: Arc<
            dyn crate::repositories::workspace_config_repository::WorkspaceConfigRepository,
        >,
        approval_workflow_repo: Arc<
            dyn crate::repositories::approval_workflow_repository::ApprovalWorkflowRepository,
        >,
        workspace_config_service: Arc<crate::services::WorkspaceConfigService>,
        approval_workflow_service: Arc<crate::services::ApprovalWorkflowService>,
        location_service: Arc<crate::services::LocationService>,
        location_repo: Arc<dyn crate::repositories::LocationRepository>,
        employer_service: Arc<crate::services::EmployerService>,
        employer_repo: Arc<dyn crate::repositories::EmployerRepository>,
        custom_field_service: Arc<crate::services::CustomFieldService>,
        custom_field_repo: Arc<dyn crate::repositories::CustomFieldRepository>,
        lifecycle_service: Arc<crate::services::LifecycleService>,
        lifecycle_repo: Arc<dyn crate::repositories::LifecycleChecklistRepository>,
        signature_service: Arc<SignatureService>,
    ) -> Self {
        Self {
            auth_service,
            workspace_service,
            workspace_repo,
            audit_repo,
            user_repo,
            session_repo,
            jwt_secret,
            workspace_config_repo,
            approval_workflow_repo,
            workspace_config_service,
            approval_workflow_service,
            location_service,
            location_repo,
            employer_service,
            employer_repo,
            custom_field_service,
            custom_field_repo,
            lifecycle_service,
            lifecycle_repo,
            signature_service,
        }
    }

    pub fn auth_service(&self) -> &Arc<AuthService> {
        &self.auth_service
    }

    pub fn workspace_service(&self) -> &Arc<WorkspaceService> {
        &self.workspace_service
    }

    pub fn workspace_repo(&self) -> &Arc<dyn WorkspaceRepository> {
        &self.workspace_repo
    }

    pub fn audit_repo(&self) -> &Arc<dyn AuditLogRepository> {
        &self.audit_repo
    }

    pub fn user_repo(&self) -> &Arc<dyn UserRepository> {
        &self.user_repo
    }

    pub fn session_repo(
        &self,
    ) -> &Arc<dyn crate::repositories::session_repository::SessionRepository> {
        &self.session_repo
    }

    pub fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }

    pub fn workspace_config_repo(
        &self,
    ) -> &Arc<dyn crate::repositories::workspace_config_repository::WorkspaceConfigRepository> {
        &self.workspace_config_repo
    }

    pub fn approval_workflow_repo(
        &self,
    ) -> &Arc<dyn crate::repositories::approval_workflow_repository::ApprovalWorkflowRepository>
    {
        &self.approval_workflow_repo
    }

    pub fn workspace_config_service(&self) -> &Arc<crate::services::WorkspaceConfigService> {
        &self.workspace_config_service
    }

    pub fn approval_workflow_service(&self) -> &Arc<crate::services::ApprovalWorkflowService> {
        &self.approval_workflow_service
    }

    pub fn location_service(&self) -> &Arc<crate::services::LocationService> {
        &self.location_service
    }

    pub fn location_repo(&self) -> &Arc<dyn crate::repositories::LocationRepository> {
        &self.location_repo
    }

    pub fn employer_service(&self) -> &Arc<crate::services::EmployerService> {
        &self.employer_service
    }

    pub fn employer_repo(&self) -> &Arc<dyn crate::repositories::EmployerRepository> {
        &self.employer_repo
    }

    pub fn custom_field_service(&self) -> &Arc<crate::services::CustomFieldService> {
        &self.custom_field_service
    }

    pub fn custom_field_repo(&self) -> &Arc<dyn crate::repositories::CustomFieldRepository> {
        &self.custom_field_repo
    }

    pub fn lifecycle_service(&self) -> &Arc<crate::services::LifecycleService> {
        &self.lifecycle_service
    }

    pub fn lifecycle_repo(&self) -> &Arc<dyn crate::repositories::LifecycleChecklistRepository> {
        &self.lifecycle_repo
    }

    pub fn signature_service(&self) -> &Arc<SignatureService> {
        &self.signature_service
    }
}

/// AI agent management
pub struct AgentState {
    manager: AgentManager,
    service: Arc<AgentService>,
    repo: Arc<dyn AgentRepository>,
}

impl AgentState {
    pub fn new(
        manager: AgentManager,
        service: Arc<AgentService>,
        repo: Arc<dyn AgentRepository>,
    ) -> Self {
        Self {
            manager,
            service,
            repo,
        }
    }

    pub fn manager(&self) -> &AgentManager {
        &self.manager
    }

    pub fn service(&self) -> &Arc<AgentService> {
        &self.service
    }

    pub fn repo(&self) -> &Arc<dyn AgentRepository> {
        &self.repo
    }
}

/// Infrastructure — DB, Redis, caches
pub struct InfraState {
    diesel: crate::diesel_pool::DieselPools,
    redis: Option<RedisPool>,
    cache: Option<Arc<dyn crate::repositories::CacheRepository>>,
    roster_cache: Option<Arc<crate::cache::RosterCache>>,
    queue: Option<Arc<dyn crate::repositories::queue_repository::QueueRepository>>,
    tenant_provisioner: Arc<dyn crate::services::tenant_provisioner::TenantProvisionerTrait>,
    crypto_service: Option<Arc<crate::crypto::CryptoService>>,
    system_db_cluster_repo: Arc<dyn crate::repositories::SystemDbClusterRepository>,
}

impl InfraState {
    pub fn new(
        diesel: crate::diesel_pool::DieselPools,
        redis: Option<RedisPool>,
        cache: Option<Arc<dyn crate::repositories::CacheRepository>>,
        roster_cache: Option<Arc<crate::cache::RosterCache>>,
        queue: Option<Arc<dyn crate::repositories::queue_repository::QueueRepository>>,
        tenant_provisioner: Arc<dyn crate::services::tenant_provisioner::TenantProvisionerTrait>,
        crypto_service: Option<Arc<crate::crypto::CryptoService>>,
        system_db_cluster_repo: Arc<dyn crate::repositories::SystemDbClusterRepository>,
    ) -> Self {
        Self {
            diesel,
            redis,
            cache,
            roster_cache,
            queue,
            tenant_provisioner,
            crypto_service,
            system_db_cluster_repo,
        }
    }

    pub fn diesel(&self) -> &crate::diesel_pool::DieselPools {
        &self.diesel
    }

    pub fn redis(&self) -> Option<&RedisPool> {
        self.redis.as_ref()
    }

    pub fn cache(&self) -> Option<&Arc<dyn crate::repositories::CacheRepository>> {
        self.cache.as_ref()
    }

    pub fn roster_cache(&self) -> Option<&Arc<crate::cache::RosterCache>> {
        self.roster_cache.as_ref()
    }

    pub fn queue(
        &self,
    ) -> Option<&Arc<dyn crate::repositories::queue_repository::QueueRepository>> {
        self.queue.as_ref()
    }

    pub fn tenant_provisioner(
        &self,
    ) -> &Arc<dyn crate::services::tenant_provisioner::TenantProvisionerTrait> {
        &self.tenant_provisioner
    }

    pub fn crypto_service(&self) -> Option<&Arc<crate::crypto::CryptoService>> {
        self.crypto_service.as_ref()
    }

    pub fn system_db_cluster_repo(
        &self,
    ) -> &Arc<dyn crate::repositories::SystemDbClusterRepository> {
        &self.system_db_cluster_repo
    }

    /// Get Redis pool, returning `ServiceUnavailable` if Redis is not configured.
    pub fn require_redis(&self) -> crate::errors::AppResult<&RedisPool> {
        self.redis.as_ref().ok_or_else(|| {
            crate::errors::AppError::service_unavailable_args(
                crate::utils::i18n::I18nKey::ServiceUnavailable,
                [(
                    "details",
                    "This feature requires Redis for background processing",
                )],
            )
        })
    }

    /// Create a `JobQueue` from the Redis pool.
    pub fn job_queue(&self) -> crate::errors::AppResult<crate::queue::JobQueue> {
        let redis = self.require_redis()?;
        Ok(crate::queue::JobQueue::new(redis.clone()))
    }
}

use crate::services::storage::{FileService, StorageResolver};

// ── Composed AppState ──────────────────────────────────────────────

pub struct AppState {
    employees: EmployeeState,
    attendance: AttendanceState,
    shifts: ShiftState,
    leaves: LeaveState,
    payroll: PayrollState,
    reports: ReportState,
    core: CoreState,
    agents: AgentState,
    infra: InfraState,
    storage: Arc<StorageResolver>,
    file_service: Arc<FileService>,
    scanner: Arc<dyn crate::services::storage::VirusScanner>,
}

impl AppState {
    pub fn new(
        employees: EmployeeState,
        attendance: AttendanceState,
        shifts: ShiftState,
        leaves: LeaveState,
        payroll: PayrollState,
        reports: ReportState,
        core: CoreState,
        agents: AgentState,
        infra: InfraState,
        storage: Arc<StorageResolver>,
        file_service: Arc<FileService>,
        scanner: Arc<dyn crate::services::storage::VirusScanner>,
    ) -> Self {
        Self {
            employees,
            attendance,
            shifts,
            leaves,
            payroll,
            reports,
            core,
            agents,
            infra,
            storage,
            file_service,
            scanner,
        }
    }

    pub fn employees(&self) -> &EmployeeState {
        &self.employees
    }

    pub fn attendance(&self) -> &AttendanceState {
        &self.attendance
    }

    pub fn shifts(&self) -> &ShiftState {
        &self.shifts
    }

    pub fn leaves(&self) -> &LeaveState {
        &self.leaves
    }

    pub fn payroll(&self) -> &PayrollState {
        &self.payroll
    }

    pub fn reports(&self) -> &ReportState {
        &self.reports
    }

    pub fn core(&self) -> &CoreState {
        &self.core
    }

    pub fn agents(&self) -> &AgentState {
        &self.agents
    }

    pub fn infra(&self) -> &InfraState {
        &self.infra
    }

    pub fn storage(&self) -> &Arc<StorageResolver> {
        &self.storage
    }

    pub fn file_service(&self) -> &Arc<FileService> {
        &self.file_service
    }

    pub fn scanner(&self) -> &Arc<dyn crate::services::storage::VirusScanner> {
        &self.scanner
    }
}
