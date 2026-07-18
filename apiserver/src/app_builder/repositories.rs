#![allow(clippy::clone_on_ref_ptr)]
use crate::crypto::CryptoService;
use crate::diesel_pool::DieselPools;
use std::sync::Arc;

pub struct Repositories {
    pub agent: Arc<dyn crate::repositories::agent_repository::AgentRepository>,
    pub workspace: Arc<dyn crate::repositories::workspace_repository::WorkspaceRepository>,
    pub audit: Arc<dyn crate::repositories::audit_log_repository::AuditLogRepository>,
    pub payroll: Arc<dyn crate::repositories::payroll_repository::PayrollRepository>,
    pub employer: Arc<dyn crate::repositories::EmployerRepository>,
    pub employee: Arc<dyn crate::repositories::employee::EmployeeRepository>,
    pub recruitment: Arc<dyn crate::repositories::employee::EmployeeRecruitmentRepository>,
    pub employment: Arc<dyn crate::repositories::EmploymentRepository>,
    pub department: Arc<dyn crate::repositories::department_repository::DepartmentRepository>,
    pub user: Arc<dyn crate::repositories::user_repository::UserRepository>,
    pub staging: Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository>,
    pub leave_balance: Arc<dyn crate::repositories::leave_balance_repository::LeaveBalanceRepository>,
    pub lifecycle: Arc<dyn crate::repositories::LifecycleChecklistRepository>,
    pub contract: Arc<dyn crate::repositories::EmployeeContractRepository>,
    pub employment_phase: Arc<dyn crate::repositories::EmploymentPhaseRepository>,
    pub time_entry: Arc<dyn crate::repositories::time_entry_repository::TimeEntryRepository>,
    pub attendance_daily_summary: Arc<dyn crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository>,
    pub behalf_assignment: Arc<dyn crate::repositories::behalf_assignment_repository::BehalfAssignmentRepository>,
    pub roster: Arc<dyn crate::repositories::roster::RosterRepository>,
    pub policy: Arc<dyn crate::repositories::AttendancePolicyRepository>,
    pub leave_request: Arc<dyn crate::repositories::leave_request_repository::LeaveRequestRepository>,
    pub holiday: Arc<dyn crate::repositories::holiday_repository::HolidayRepository>,
    pub shift: Arc<dyn crate::repositories::shift_repository::ShiftRepository>,
    pub outbox: Arc<dyn crate::repositories::outbox_job_repository::OutboxJobRepository>,
    pub shift_swap: Arc<dyn crate::repositories::shift_swap_repository::ShiftSwapRepository>,
    pub stats: Arc<dyn crate::repositories::workspace_stats_repository::WorkspaceStatsRepository>,
    pub raw_log: Arc<dyn crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository>,
    pub adj: Arc<dyn crate::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository>,
    pub import: Arc<dyn crate::repositories::import_job_repository::ImportJobRepository>,
    pub session: Arc<dyn crate::repositories::session_repository::SessionRepository>,
    pub storage_config: Arc<crate::repositories::storage_config_repo::StorageConfigRepository>,
    pub file: Arc<dyn crate::repositories::FileRepository>,
    pub signature: Arc<dyn crate::repositories::SignatureRepository>,
    pub system_db_cluster: Arc<dyn crate::repositories::system_db_cluster_repository::SystemDbClusterRepository>,
    pub system_settings: Arc<dyn crate::repositories::system_settings_repository::SystemSettingsRepository>,
    pub workspace_config: Arc<dyn crate::repositories::workspace_config_repository::WorkspaceConfigRepository>,
    pub approval_workflow: Arc<dyn crate::repositories::approval_workflow_repository::ApprovalWorkflowRepository>,
    pub overtime_request: Arc<dyn crate::repositories::overtime_request_repository::OvertimeRequestRepository>,
    pub attendance_policy: Arc<dyn crate::repositories::attendance_policy_repository::AttendancePolicyRepository>,
    pub payroll_rule: Arc<dyn crate::repositories::payroll_rule_repository::PayrollRuleRepository>,
    pub feature_flag: Arc<dyn crate::repositories::feature_flag_repository::FeatureFlagRepository>,
    pub assignment_rule: Arc<dyn crate::repositories::assignment_rule_repository::AssignmentRuleRepository>,
    pub overtime_policy: Arc<dyn crate::repositories::overtime_policy_repository::OvertimePolicyRepositoryTrait>,
    pub salary_base: Arc<dyn crate::repositories::salary_base_repository::SalaryBaseRepository>,
    pub working_pattern: Arc<dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait>,
    pub leave_policy: Arc<dyn crate::repositories::leave_policy_repository::LeavePolicyRepository>,
    pub leave_accrual_log: Arc<dyn crate::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository>,
    pub location: Arc<dyn crate::repositories::LocationRepository>,
    pub custom_field: Arc<dyn crate::repositories::CustomFieldRepository>,
    pub error: Arc<dyn crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository>,
    pub condition_type: Arc<dyn crate::repositories::ConditionTypeRepository>,
}

#[allow(clippy::too_many_lines)]
pub fn build_repositories(
    diesel_pools: &DieselPools,
    crypto_service: &Option<Arc<CryptoService>>,
) -> Result<Repositories, Box<dyn std::error::Error>> {
    let agent = Arc::new(crate::repositories::DieselAgentRepository::new(
        diesel_pools.clone(),
    ));
    let workspace = Arc::new(crate::repositories::DieselWorkspaceRepository::new(
        diesel_pools.clone(),
    ));
    let audit = Arc::new(crate::repositories::DieselAuditLogRepository::new(
        diesel_pools.clone(),
    ));
    let payroll = Arc::new(crate::repositories::DieselPayrollRepository::new(
        diesel_pools.clone(),
    ));
    let employer = Arc::new(crate::repositories::DieselEmployerRepository::new(
        diesel_pools.clone(),
    ));
    let employee = Arc::new(
        crate::repositories::diesel_employee_repository::DieselEmployeeRepository::new_with_crypto(
            diesel_pools.clone(),
            crypto_service.clone(),
        ),
    );
    let recruitment = Arc::new(
        crate::repositories::diesel_employee_repository::DieselEmployeeRepository::new_with_crypto(
            diesel_pools.clone(),
            crypto_service.clone(),
        ),
    );
    let employment = Arc::new(crate::repositories::DieselEmploymentRepository::new(
        diesel_pools.clone(),
    ));
    let department = Arc::new(
        crate::repositories::diesel_department_repository::DieselDepartmentRepository::new(
            diesel_pools.clone(),
        ),
    );
    let user = Arc::new(crate::repositories::DieselUserRepository::new(
        diesel_pools.clone(),
    ));
    let staging = Arc::new(crate::repositories::DieselStagingLogRepository::new(
        diesel_pools.clone(),
    ));
    let leave_balance = Arc::new(crate::repositories::DieselLeaveBalanceRepository::new(
        diesel_pools.clone(),
    ));
    let lifecycle = Arc::new(
        crate::repositories::DieselLifecycleChecklistRepository::new(diesel_pools.clone()),
    );
    let contract = Arc::new(crate::repositories::DieselEmployeeContractRepository::new(
        diesel_pools.clone(),
    ));
    let employment_phase = Arc::new(crate::repositories::DieselEmploymentPhaseRepository::new(
        diesel_pools.clone(),
    ));
    let time_entry = Arc::new(crate::repositories::DieselTimeEntryRepository::new(
        diesel_pools.clone(),
    ));
    let attendance_daily_summary = Arc::new(crate::repositories::diesel_attendance_daily_summary_repository::DieselAttendanceDailySummaryRepository::new(diesel_pools.clone()));
    let behalf_assignment = Arc::new(crate::repositories::diesel_behalf_assignment_repository::DieselBehalfAssignmentRepository::new(diesel_pools.clone()));
    let roster = Arc::new(crate::repositories::DieselRosterRepository::new(
        diesel_pools.clone(),
    ));
    let policy = Arc::new(crate::repositories::DieselAttendancePolicyRepository::new(
        diesel_pools.clone(),
    ));
    let leave_request = Arc::new(crate::repositories::DieselLeaveRequestRepository::new(
        diesel_pools.clone(),
    ));
    let holiday = Arc::new(crate::repositories::DieselHolidayRepository::new(
        diesel_pools.clone(),
    ));
    let shift = Arc::new(crate::repositories::DieselShiftRepository::new(
        diesel_pools.clone(),
    ));
    let shift_swap = Arc::new(crate::repositories::DieselShiftSwapRepository::new(
        diesel_pools.clone(),
    ));
    let outbox = Arc::new(crate::repositories::DieselOutboxJobRepository::new(
        diesel_pools.clone(),
    ));
    let stats = Arc::new(
        crate::repositories::workspace_stats_repository::DieselWorkspaceStatsRepository::new(
            diesel_pools.clone(),
        ),
    );
    let raw_log = Arc::new(crate::repositories::DieselRawAttendanceLogRepository::new(
        diesel_pools.clone(),
    ));
    let adj = Arc::new(
        crate::repositories::DieselAttendanceAdjustmentRepository::new(diesel_pools.clone()),
    );
    let import = Arc::new(crate::repositories::DieselImportJobRepository::new(
        diesel_pools.clone(),
    ));
    let session = Arc::new(crate::repositories::DieselSessionRepository::new(
        diesel_pools.clone(),
    ));

    let crypto_inner = crypto_service
        .clone()
        .ok_or("CryptoService is required for StorageConfig")?;
    let storage_config = Arc::new(
        crate::repositories::storage_config_repo::StorageConfigRepository::new(
            diesel_pools.api.clone(),
            crypto_inner,
        ),
    );
    let file = Arc::new(
        crate::repositories::diesel_file_repository::DieselFileRepository::new(
            diesel_pools.clone(),
        ),
    );
    let signature = Arc::new(
        crate::repositories::diesel_signature_repository::DieselSignatureRepository::new(Arc::new(
            diesel_pools.clone(),
        )),
    );
    let system_db_cluster = Arc::new(crate::repositories::diesel_system_db_cluster_repository::DieselSystemDbClusterRepository::new(diesel_pools.clone()));
    let system_settings = Arc::new(
        crate::repositories::diesel_system_settings_repository::DieselSystemSettingsRepository::new(
            diesel_pools.clone(),
        ),
    );
    let workspace_config = Arc::new(crate::repositories::diesel_workspace_config_repository::DieselWorkspaceConfigRepository::new(diesel_pools.clone()));
    let approval_workflow = Arc::new(crate::repositories::diesel_approval_workflow_repository::DieselApprovalWorkflowRepository::new(diesel_pools.clone()));
    let overtime_request = Arc::new(crate::repositories::diesel_overtime_request_repository::DieselOvertimeRequestRepository::new(diesel_pools.clone()));
    let attendance_policy = Arc::new(crate::repositories::DieselAttendancePolicyRepository::new(
        diesel_pools.clone(),
    ));
    let payroll_rule = Arc::new(crate::repositories::DieselPayrollRuleRepository::new(
        diesel_pools.clone(),
    ));
    let feature_flag = Arc::new(crate::repositories::DieselFeatureFlagRepository::new(
        diesel_pools.clone(),
    ));
    let assignment_rule = Arc::new(crate::repositories::DieselAssignmentRuleRepository::new(
        diesel_pools.clone(),
    ));
    let overtime_policy = Arc::new(crate::repositories::DieselOvertimePolicyRepository::new(
        diesel_pools.clone(),
    ));
    let salary_base = Arc::new(crate::repositories::DieselSalaryBaseRepository::new(
        diesel_pools.clone(),
    ));
    let working_pattern = Arc::new(crate::repositories::DieselWorkingPatternRepository::new(
        diesel_pools.clone(),
    ));
    let leave_policy = Arc::new(crate::repositories::DieselLeavePolicyRepository::new(
        diesel_pools.clone(),
    ));
    let leave_accrual_log = Arc::new(crate::repositories::DieselLeaveAccrualLogRepository::new(
        diesel_pools.clone(),
    ));
    let location = Arc::new(crate::repositories::DieselLocationRepository::new(
        diesel_pools.clone(),
    ));
    let custom_field = Arc::new(crate::repositories::DieselCustomFieldRepository::new(
        diesel_pools.clone(),
    ));
    let error = Arc::new(
        crate::repositories::DieselAttendanceLogErrorRepository::new(diesel_pools.clone()),
    );
    let condition_type = Arc::new(
        crate::repositories::DieselConditionTypeRepository::new(diesel_pools.clone()),
    );

    Ok(Repositories {
        agent,
        workspace,
        audit,
        payroll,
        employer,
        employee,
        recruitment,
        employment,
        department,
        user,
        staging,
        leave_balance,
        lifecycle,
        contract,
        employment_phase,
        time_entry,
        attendance_daily_summary,
        behalf_assignment,
        roster,
        policy,
        leave_request,
        holiday,
        shift,
        outbox,
        shift_swap,
        stats,
        raw_log,
        adj,
        import,
        session,
        storage_config,
        file,
        signature,
        system_db_cluster,
        system_settings,
        workspace_config,
        approval_workflow,
        overtime_request,
        attendance_policy,
        payroll_rule,
        feature_flag,
        assignment_rule,
        overtime_policy,
        salary_base,
        working_pattern,
        leave_policy,
        leave_accrual_log,
        location,
        custom_field,
        error,
        condition_type,
    })
}
