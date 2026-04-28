use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::agent_manager::AgentManager;
use crate::db_pool::DbPools;
use crate::queue::RedisPool;
use crate::state::AppState;
use crate::services::{AttendanceLogService, ReportCoordinator};
use crate::worker::{JobWorker, WorkerConfig};

pub struct AppWorkers {
    pub log_worker_handle: JoinHandle<()>,
    pub job_worker_handle: Option<JoinHandle<()>>,
    pub job_worker_token: CancellationToken,
}

impl AppState {
    pub async fn build(db_pools: DbPools, redis_pool: Option<RedisPool>) -> Result<(Arc<Self>, AppWorkers), Box<dyn std::error::Error>> {
        // Use API pool for application-level initialization (HTTP handlers, CRUD)
        let pool = db_pools.api().clone();
        // Use Worker pool for background services (patient timeout, won't starve API)
        let worker_pool = db_pools.worker().clone();

        // Dev seed data removed — first registered user gets platform_admin automatically.

        let agent_manager = AgentManager::new(
            redis_pool
                .clone()
                .map(|p| Arc::new(p) as Arc<dyn crate::agent_manager::PresenceStore>),
        );
        let agent_repo = Arc::new(crate::repositories::SqlxAgentRepository::new(pool.clone()));
        let workspace_repo =
            Arc::new(crate::repositories::SqlxWorkspaceRepository::new(pool.clone()));
        let audit_repo =
            Arc::new(crate::repositories::audit_log_repository::SqlxAuditLogRepository::new(pool.clone()));
        let agent_service = Arc::new(crate::services::AgentService::new(
            agent_repo.clone(),
            agent_manager.clone(),
        ));
        let employee_repo =
            Arc::new(crate::repositories::SqlxEmployeeRepository::new(pool.clone()));
        let employee_service = Arc::new(crate::services::EmployeeService::new(
            employee_repo.clone(),
            pool.clone(),
            redis_pool.clone(),
        ));
        let time_entry_repo =
            Arc::new(crate::repositories::SqlxTimeEntryRepository::new(pool.clone()));
        let roster_repo: Arc<dyn crate::repositories::roster::RosterRepository> =
            Arc::new(crate::repositories::roster::SqlxRosterRepository::new(pool.clone()));
        let time_tracking_service = Arc::new(crate::services::TimeTrackingService::new(
            time_entry_repo.clone(),
            roster_repo.clone(),
            pool.clone(),
        ));
        let attendance_summary_service = Arc::new(
            crate::services::AttendanceSummaryService::new(
                time_entry_repo.clone(),
                roster_repo.clone(),
                pool.clone(),
            ),
        );
        let report_service = Arc::new(crate::services::ReportService::new(
            time_entry_repo.clone(),
            roster_repo.clone(),
            pool.clone(),
        ));
        let pairing_service = Arc::new(
            crate::services::pairing_service::PairingService::new(
                time_entry_repo.clone(),
                roster_repo.clone(),
                pool.clone(),
            ),
        );
        let adj_repo = Arc::new(
            crate::repositories::attendance_adjustment_repository::SqlxAttendanceAdjustmentRepository::new(pool.clone()),
        );
        let adjustment_service = Arc::new(
            crate::services::attendance_adjustment_service::AttendanceAdjustmentService::new(
                adj_repo.clone(),
                time_entry_repo.clone(),
                pool.clone(),
            ),
        );
        let import_repo = Arc::new(
            crate::repositories::import_job_repository::SqlxImportJobRepository::new(pool.clone()),
        );
        let import_service = Arc::new(
            crate::services::attendance_import_service::AttendanceImportService::new(
                import_repo.clone(),
                adj_repo.clone(),
                employee_repo.clone(),
                time_entry_repo.clone(),
                worker_pool.clone(),
                redis_pool.clone(),
            ),
        );
        import_service.resume_all_pending_jobs().await;

        let auth_service = Arc::new(
            crate::services::auth_service::AuthService::new(pool.clone()),
        );
        let shift_service = Arc::new(
            crate::services::shift_service::ShiftService::new(pool.clone()),
        );
        let workspace_service = Arc::new(
            crate::services::workspace_service::WorkspaceService::new(pool.clone()),
        );
        let leave_balance_repo = Arc::new(
            crate::repositories::leave_balance_repository::SqlxLeaveBalanceRepository::new(pool.clone()),
        );
        let leave_request_repo = Arc::new(
            crate::repositories::leave_request_repository::SqlxLeaveRequestRepository::new(pool.clone()),
        );
        let leave_service = Arc::new(
            crate::services::leave_service::LeaveService::new(
                leave_balance_repo.clone(),
                leave_request_repo.clone(),
                employee_repo.clone(),
                roster_repo.clone(),
                pool.clone(),
            ),
        );
        let shift_swap_repo = Arc::new(
            crate::repositories::shift_swap_repository::SqlxShiftSwapRepository::new(pool.clone()),
        );
        let shift_swap_service = Arc::new(
            crate::services::shift_swap_service::ShiftSwapService::new(shift_swap_repo.clone(), roster_repo.clone(), pool.clone()),
        );
        let holiday_leave_service = Arc::new(
            crate::services::holiday_leave_service::HolidayLeaveService::new(
                employee_repo.clone(), leave_balance_repo.clone(), leave_request_repo.clone(), roster_repo.clone(), pool.clone(),
            ),
        );
        let department_repo: Arc<dyn crate::repositories::department_repository::DepartmentRepository> =
            Arc::new(crate::repositories::department_repository::SqlxDepartmentRepository::new(pool.clone()));
        let leave_policy_repo: Arc<dyn crate::repositories::leave_policy_repository::LeavePolicyRepository> =
            Arc::new(crate::repositories::leave_policy_repository::SqlxLeavePolicyRepository::new(pool.clone()));
        let leave_accrual_service = Arc::new(
            crate::services::leave_accrual_service::LeaveAccrualService::new(
                employee_repo.clone(), department_repo.clone(), leave_policy_repo.clone(), worker_pool.clone(),
            ),
        );
        let payroll_repo: Arc<dyn crate::repositories::payroll_repository::PayrollRepository> =
            Arc::new(crate::repositories::payroll_repository::SqlxPayrollRepository::new(pool.clone()));
        let holiday_repo: Arc<dyn crate::repositories::holiday_repository::HolidayRepositoryTrait> =
            Arc::new(crate::repositories::holiday_repository::SqlxHolidayRepository::new(pool.clone()));
        let payroll_service = Arc::new(
            crate::services::payroll_service::PayrollService::new(
                payroll_repo.clone(), department_repo.clone(), time_entry_repo.clone(),
                workspace_repo.clone(), holiday_repo.clone(), roster_repo.clone(), pool.clone(),
            ),
        );
        let jwt_secret = if cfg!(debug_assertions) {
            std::env::var("JWT_SECRET").unwrap_or_else(|_| {
                tracing::warn!("⚠️ JWT_SECRET not set — using insecure dev fallback. DO NOT use in production!");
                "dev_secret_key_leeattend_2026".to_string()
            })
        } else {
            std::env::var("JWT_SECRET").map_err(|_| "JWT_SECRET must be set in production")?
        };

        let buffer_capacity: usize = std::env::var("AGENT_LOG_BUFFER_CAPACITY")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5000);
        let batch_size: usize = std::env::var("AGENT_LOG_BATCH_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);
        let flush_interval_ms: u64 = std::env::var("AGENT_LOG_FLUSH_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);

        let raw_log_repo: Arc<dyn crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository> =
            Arc::new(crate::repositories::raw_attendance_log_repository::SqlxRawAttendanceLogRepository::new(pool.clone()));
        let staging_repo: Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository> =
            Arc::new(crate::repositories::staging_log_repository::SqlxStagingLogRepository::new(pool.clone()));
        let error_repo: Arc<dyn crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository> =
            Arc::new(crate::repositories::attendance_log_error_repository::SqlxAttendanceLogErrorRepository::new(pool.clone()));

        let (attendance_service, log_worker_handle) =
            AttendanceLogService::new(worker_pool.clone(), pairing_service.clone(), buffer_capacity, batch_size, flush_interval_ms,
                employee_repo.clone(), raw_log_repo.clone(), staging_repo.clone(), error_repo.clone());
        let attendance_service = Arc::new(attendance_service);

        // Roster cache (requires Redis; shared between app state and worker)
        let roster_cache = redis_pool
            .as_ref()
            .map(|rpool| Arc::new(crate::cache::RosterCache::new(rpool.clone())));

        // Initialize job worker if Redis is available
        let worker_cancel_token = CancellationToken::new();
        let (coordinator, worker_handle) = redis_pool.as_ref().map_or_else(
            || (None, None),
            |redis| {
                let worker_config = WorkerConfig::from_env();
                tracing::info!(
                    max_concurrent = worker_config.max_concurrent_jobs,
                    poll_timeout = worker_config.poll_timeout_secs,
                    "Initializing job worker"
                );

                let coordinator = Arc::new(ReportCoordinator::new(redis.clone(), pool.clone()));

                let worker = JobWorker::new(
                    redis.clone(),
                    db_pools.worker().clone(),
                    worker_config,
                    coordinator.clone(),
                    roster_cache.clone(),
                );
                (
                    Some(coordinator),
                    Some(worker.start(worker_cancel_token.clone())),
                )
            },
        );

        let user_repo: Arc<dyn crate::repositories::user_repository::UserRepository> =
            Arc::new(crate::repositories::user_repository::SqlxUserRepository::new(pool.clone()));

        let state = Arc::new(AppState {
            employees: crate::EmployeeState {
                service: employee_service,
                repo: employee_repo,
            },
            attendance: crate::AttendanceState {
                time_tracking_service,
                summary_service: attendance_summary_service,
                log_service: attendance_service,
                pairing_service,
                adjustment_service,
                import_service,
                time_entry_repo,
                staging_repo,
                raw_log_repo,
                import_repo,
            },
            shifts: crate::ShiftState {
                service: shift_service,
                swap_service: shift_swap_service,
                roster_repo,
                swap_repo: shift_swap_repo,
            },
            leaves: crate::LeaveState {
                service: leave_service,
                holiday_leave_service,
                accrual_service: leave_accrual_service,
                request_repo: leave_request_repo,
                policy_repo: leave_policy_repo,
                holiday_repo,
            },
            payroll: crate::PayrollState {
                service: payroll_service,
                repo: payroll_repo,
                department_repo,
            },
            reports: crate::ReportState {
                service: report_service,
                coordinator,
            },
            core: crate::CoreState {
                auth_service,
                workspace_service,
                workspace_repo,
                audit_repo,
                user_repo,
                jwt_secret,
            },
            agents: crate::AgentState {
                manager: agent_manager,
                service: agent_service,
                repo: agent_repo,
            },
            infra: crate::InfraState {
                db: db_pools,
                redis: redis_pool,
                roster_cache,
            },
        });

        let workers = AppWorkers {
            log_worker_handle,
            job_worker_handle: worker_handle,
            job_worker_token: worker_cancel_token,
        };

        Ok((state, workers))
    }
}
