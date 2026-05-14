//! Worker dependency container for pre-built repository instances.
//!
//! Centralises `Arc<dyn Trait>` construction so that every handler
//! receives its dependencies through DI instead of instantiating
//! concrete Diesel repositories inline.

use std::sync::Arc;

use crate::repositories::roster::RosterRepository;
use crate::repositories::AttendancePolicyRepository;
use crate::repositories::TimeEntryRepository;

/// Shared, pre-built repository instances for worker handlers.
///
/// Constructed once in [`super::JobWorker::new`] and shared across
/// all handler invocations via `Arc<WorkerDeps>`.
pub struct WorkerDeps {
    pub time_entry_repo: Arc<dyn TimeEntryRepository>,
    pub roster_repo: Arc<dyn RosterRepository>,
    pub policy_repo: Arc<dyn AttendancePolicyRepository>,
    pub employee_repo: Arc<dyn crate::repositories::EmployeeRepository>,
    pub raw_log_repo:
        Arc<dyn crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository>,
    pub staging_repo: Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository>,
    pub error_repo:
        Arc<dyn crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository>,
}

impl WorkerDeps {
    /// Build all worker-scoped repositories from Diesel connection pools.
    pub fn new(diesel: &crate::diesel_pool::DieselPools) -> Self {
        Self {
            time_entry_repo: Arc::new(
                crate::repositories::DieselTimeEntryRepository::new(diesel.clone()),
            ),
            roster_repo: Arc::new(crate::repositories::DieselRosterRepository::new(
                diesel.clone(),
            )),
            policy_repo: Arc::new(
                crate::repositories::DieselAttendancePolicyRepository::new(diesel.clone()),
            ),
            employee_repo: Arc::new(
                crate::repositories::DieselEmployeeRepository::new(diesel.clone()),
            ),
            raw_log_repo: Arc::new(
                crate::repositories::DieselRawAttendanceLogRepository::new(diesel.clone()),
            ),
            staging_repo: Arc::new(
                crate::repositories::DieselStagingLogRepository::new(diesel.clone()),
            ),
            error_repo: Arc::new(
                crate::repositories::DieselAttendanceLogErrorRepository::new(diesel.clone()),
            ),
        }
    }

    /// Build a [`PairingService`](crate::services::pairing_service::PairingService)
    /// using the shared repository instances.
    pub fn pairing_service(&self, pool: &sqlx::PgPool, diesel: &crate::diesel_pool::DieselPools) -> Arc<crate::services::pairing_service::PairingService> {
        Arc::new(crate::services::pairing_service::PairingService::new(
            Arc::clone(&self.time_entry_repo),
            Arc::clone(&self.roster_repo),
            Arc::clone(&self.policy_repo),
            pool.clone(),
            diesel.clone(),
        ))
    }

    /// Build an [`AttendanceLogService`](crate::services::AttendanceLogService)
    /// using the shared repository instances.
    pub fn log_service(
        &self,
        pool: &sqlx::PgPool,
        diesel: &crate::diesel_pool::DieselPools,
    ) -> (
        crate::services::AttendanceLogService,
        tokio::task::JoinHandle<()>,
    ) {
        let pairing_svc = self.pairing_service(pool, diesel);
        crate::services::AttendanceLogService::new(
            pool.clone(),
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
