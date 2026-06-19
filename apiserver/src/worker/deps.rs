//! Worker dependency container for pre-built repository instances.
//!
//! Centralises `Arc<dyn Trait>` construction so that every handler
//! receives its dependencies through DI instead of instantiating
//! concrete Diesel repositories inline.

use std::sync::Arc;

use crate::repositories::outbox_job_repository::OutboxJobRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::AttendancePolicyRepository;
use crate::repositories::TimeEntryRepository;

/// Shared, pre-built repository instances for worker handlers.
///
/// Constructed once in [`super::JobWorker::new`] and shared across
/// all handler invocations via `Arc<WorkerDeps>`.
#[derive(Clone)]
pub struct WorkerDeps {
    pub time_entry_repo: Arc<dyn TimeEntryRepository>,
    pub roster_repo: Arc<dyn RosterRepository>,
    pub policy_repo: Arc<dyn AttendancePolicyRepository>,
    pub employee_repo: Arc<dyn crate::repositories::EmployeeRepository>,
    pub raw_log_repo: Arc<dyn crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository>,
    pub staging_repo: Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository>,
    pub error_repo: Arc<dyn crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository>,

    // Additional Repositories
    pub workspace_repo: Arc<dyn crate::repositories::workspace_repository::WorkspaceRepository>,
    pub holiday_repo: Arc<dyn crate::repositories::holiday_repository::HolidayRepository>,
    pub leave_request_repo: Arc<dyn crate::repositories::leave_request_repository::LeaveRequestRepository>,
    pub daily_summary_repo: Arc<dyn crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository>,
    pub shift_repo: Arc<dyn crate::repositories::shift_repository::ShiftRepository>,
    pub adjustment_repo: Arc<dyn crate::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository>,
    pub department_repo: Arc<dyn crate::repositories::department_repository::DepartmentRepository>,
    pub import_job_repo: Arc<dyn crate::repositories::import_job_repository::ImportJobRepository>,
    pub session_repo: Arc<dyn crate::repositories::SessionRepository>,
    pub leave_balance_repo: Arc<dyn crate::repositories::leave_balance_repository::LeaveBalanceRepository>,
    pub contract_repo: Arc<dyn crate::repositories::EmployeeContractRepository>,
    pub employment_repo: Arc<dyn crate::repositories::EmploymentRepository>,
    pub leave_accrual_log_repo: Arc<dyn crate::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository>,
    pub leave_policy_repo: Arc<dyn crate::repositories::leave_policy_repository::LeavePolicyRepository>,
    pub user_repo: Arc<dyn crate::repositories::UserRepository>,
    pub system_repo: Arc<dyn crate::repositories::SystemRepository>,
    pub outbox_repo: Arc<dyn OutboxJobRepository>,
    pub workspace_config_repo: Arc<dyn crate::repositories::workspace_config_repository::WorkspaceConfigRepository>,
    pub custom_field_repo: Arc<dyn crate::repositories::CustomFieldRepository>,
    pub employer_repo: Arc<dyn crate::repositories::EmployerRepository>,
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl WorkerDeps {
    /// Build all worker-scoped repositories from Diesel connection pools.
    pub fn new(diesel: &crate::diesel_pool::DieselPools) -> Self {
        Self::new_with_crypto(diesel, None)
    }

    /// Build all worker-scoped repositories with custom crypto service.
    pub fn new_with_crypto(
        diesel: &crate::diesel_pool::DieselPools,
        crypto_service: Option<std::sync::Arc<crate::crypto::CryptoService>>,
    ) -> Self {
        Self {
            diesel_pools: diesel.clone(),
            time_entry_repo: Arc::new(crate::repositories::DieselTimeEntryRepository::new(
                diesel.clone(),
            )),
            roster_repo: Arc::new(crate::repositories::DieselRosterRepository::new(
                diesel.clone(),
            )),
            policy_repo: Arc::new(crate::repositories::DieselAttendancePolicyRepository::new(
                diesel.clone(),
            )),
            employee_repo: Arc::new(
                crate::repositories::DieselEmployeeRepository::new_with_crypto(
                    diesel.clone(),
                    crypto_service,
                ),
            ),
            raw_log_repo: Arc::new(crate::repositories::DieselRawAttendanceLogRepository::new(
                diesel.clone(),
            )),
            staging_repo: Arc::new(crate::repositories::DieselStagingLogRepository::new(
                diesel.clone(),
            )),
            error_repo: Arc::new(
                crate::repositories::DieselAttendanceLogErrorRepository::new(diesel.clone()),
            ),

            workspace_repo: Arc::new(crate::repositories::DieselWorkspaceRepository::new(
                diesel.clone(),
            )),
            holiday_repo: Arc::new(crate::repositories::DieselHolidayRepository::new(
                diesel.clone(),
            )),
            leave_request_repo: Arc::new(crate::repositories::DieselLeaveRequestRepository::new(
                diesel.clone(),
            )),
            daily_summary_repo: Arc::new(
                crate::repositories::DieselAttendanceDailySummaryRepository::new(diesel.clone()),
            ),
            shift_repo: Arc::new(crate::repositories::DieselShiftRepository::new(
                diesel.clone(),
            )),
            adjustment_repo: Arc::new(
                crate::repositories::DieselAttendanceAdjustmentRepository::new(diesel.clone()),
            ),
            department_repo: Arc::new(crate::repositories::DieselDepartmentRepository::new(
                diesel.clone(),
            )),
            import_job_repo: Arc::new(crate::repositories::DieselImportJobRepository::new(
                diesel.clone(),
            )),
            session_repo: Arc::new(crate::repositories::DieselSessionRepository::new(
                diesel.clone(),
            )),
            leave_balance_repo: Arc::new(crate::repositories::DieselLeaveBalanceRepository::new(
                diesel.clone(),
            )),
            contract_repo: Arc::new(crate::repositories::DieselEmployeeContractRepository::new(
                diesel.clone(),
            )),
            employment_repo: Arc::new(crate::repositories::DieselEmploymentRepository::new(
                diesel.clone(),
            )),
            leave_accrual_log_repo: Arc::new(
                crate::repositories::DieselLeaveAccrualLogRepository::new(diesel.clone()),
            ),
            leave_policy_repo: Arc::new(crate::repositories::DieselLeavePolicyRepository::new(
                diesel.clone(),
            )),
            user_repo: Arc::new(crate::repositories::DieselUserRepository::new(
                diesel.clone(),
            )),
            system_repo: Arc::new(crate::repositories::DieselSystemRepository::new(
                diesel.clone(),
            )),
            outbox_repo: Arc::new(crate::repositories::DieselOutboxJobRepository::new(
                diesel.clone(),
            )),
            workspace_config_repo: Arc::new(
                crate::repositories::diesel_workspace_config_repository::DieselWorkspaceConfigRepository::new(diesel.clone()),
            ),
            custom_field_repo: Arc::new(crate::repositories::DieselCustomFieldRepository::new(
                diesel.clone(),
            )),
            employer_repo: Arc::new(crate::repositories::DieselEmployerRepository::new(
                diesel.clone(),
            )),
        }
    }

    /// Build a [`AttendanceSummaryService`](crate::services::AttendanceSummaryService)
    /// using the shared repository instances.
    pub fn summary_service(&self) -> Arc<crate::services::AttendanceSummaryService> {
        Arc::new(crate::services::AttendanceSummaryService::new(
            crate::services::attendance_summary_service::AttendanceSummaryServiceArgs {
                time_entry_repo: Arc::clone(&self.time_entry_repo),
                roster_repo: Arc::clone(&self.roster_repo),
                policy_repo: Arc::clone(&self.policy_repo),
                workspace_repo: Arc::clone(&self.workspace_repo),
                employee_repo: Arc::clone(&self.employee_repo),
                employment_repo: Arc::clone(&self.employment_repo),
                holiday_repo: Arc::clone(&self.holiday_repo),
                leave_request_repo: Arc::clone(&self.leave_request_repo),
                daily_summary_repo: Arc::clone(&self.daily_summary_repo),
            },
        ))
    }

    /// Build a [`PairingService`](crate::services::pairing_service::PairingService)
    /// using the shared repository instances.
    pub fn pairing_service(&self) -> Arc<crate::services::pairing_service::PairingService> {
        Arc::new(crate::services::pairing_service::PairingService::new(
            crate::services::pairing_service::PairingServiceArgs {
                time_entry_repo: Arc::clone(&self.time_entry_repo),
                roster_repo: Arc::clone(&self.roster_repo),
                employee_repo: Arc::clone(&self.employee_repo),
                raw_log_repo: Arc::clone(&self.raw_log_repo),
                shift_repo: Arc::clone(&self.shift_repo),
                summary_service: self.summary_service() as Arc<dyn crate::services::SummaryRecalculator>,
            },
        ))
    }

    /// Build an [`AttendanceLogService`](crate::services::AttendanceLogService)
    /// using the shared repository instances.
    pub fn log_service(
        &self,
    ) -> (
        crate::services::AttendanceLogService,
        tokio::task::JoinHandle<()>,
    ) {
        let pairing_svc = self.pairing_service();
        crate::services::AttendanceLogService::new(
            pairing_svc,
            crate::services::attendance_log_service::AttendanceLogServiceConfig {
                buffer_capacity: 1,
                batch_size: 1,
                flush_interval_ms: 1000,
            },
            crate::services::attendance_log_service::AttendanceLogServiceDeps {
                employee_repo: Arc::clone(&self.employee_repo),
                raw_log_repo: Arc::clone(&self.raw_log_repo),
                staging_repo: Arc::clone(&self.staging_repo),
                error_repo: Arc::clone(&self.error_repo),
            },
        )
    }
}
