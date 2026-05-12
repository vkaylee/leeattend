#![allow(clippy::clone_on_ref_ptr)]
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::agent_manager::AgentManager;
use crate::db_pool::DbPools;
use crate::queue::RedisPool;
use crate::services::{AttendanceLogService, ReportCoordinator};
use crate::state::AppState;
use crate::worker::{HousekeepingScheduler, JobWorker, WorkerConfig};

pub struct AppWorkers {
    pub log_worker_handle: JoinHandle<()>,
    pub job_worker_handle: Option<JoinHandle<()>>,
    pub scheduler_handle: Option<JoinHandle<()>>,
    pub job_worker_token: CancellationToken,
}

impl AppState {
    #[allow(clippy::too_many_lines)]
    pub async fn build(
        db_pools: DbPools,
        diesel_pools: crate::diesel_pool::DieselPools,
        redis_pool: Option<RedisPool>,
    ) -> Result<(Arc<Self>, AppWorkers), Box<dyn std::error::Error>> {
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
        let agent_repo: Arc<dyn crate::repositories::agent_repository::AgentRepository> = Arc::new(
            crate::repositories::DieselAgentRepository::new(pool.clone(), diesel_pools.clone()),
        );
        let workspace_repo: Arc<
            dyn crate::repositories::workspace_repository::WorkspaceRepository,
        > = Arc::new(crate::repositories::DieselWorkspaceRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let audit_repo: Arc<dyn crate::repositories::audit_log_repository::AuditLogRepository> =
            Arc::new(crate::repositories::DieselAuditLogRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            ));
        let agent_service = Arc::new(crate::services::AgentService::new(
            Arc::clone(&agent_repo),
            agent_manager.clone(),
        ));
        let employee_repo: Arc<dyn crate::repositories::employee::EmployeeRepository> = Arc::new(
            crate::repositories::DieselEmployeeRepository::new(pool.clone(), diesel_pools.clone()),
        );
        let employee_service = Arc::new(crate::services::EmployeeService::new(
            Arc::clone(&employee_repo),
            pool.clone(),
        ));
        let contract_repo = Arc::new(crate::repositories::DieselEmployeeContractRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let employment_phase_repo =
            Arc::new(crate::repositories::DieselEmploymentPhaseRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            ));
        let time_entry_repo: Arc<
            dyn crate::repositories::time_entry_repository::TimeEntryRepository,
        > = Arc::new(crate::repositories::DieselTimeEntryRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let roster_repo: Arc<dyn crate::repositories::roster::RosterRepository> = Arc::new(
            crate::repositories::DieselRosterRepository::new(pool.clone(), diesel_pools.clone()),
        );
        let time_tracking_service = Arc::new(crate::services::TimeTrackingService::new(
            Arc::clone(&time_entry_repo),
            Arc::clone(&roster_repo),
            pool.clone(),
            diesel_pools.clone(),
        ));
        let attendance_summary_service = Arc::new(crate::services::AttendanceSummaryService::new(
            Arc::clone(&time_entry_repo),
            Arc::clone(&roster_repo),
            pool.clone(),
            diesel_pools.clone(),
        ));
        let report_service = Arc::new(crate::services::ReportService::new(
            Arc::clone(&time_entry_repo),
            Arc::clone(&roster_repo),
            pool.clone(),
            diesel_pools.clone(),
        ));
        let stats_repo: Arc<
            dyn crate::repositories::workspace_stats_repository::WorkspaceStatsRepository,
        > = Arc::new(
            crate::repositories::workspace_stats_repository::DieselWorkspaceStatsRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            ),
        );
        let pairing_service = Arc::new(crate::services::pairing_service::PairingService::new(
            Arc::clone(&time_entry_repo),
            Arc::clone(&roster_repo),
            pool.clone(),
            diesel_pools.clone(),
        ));
        let adj_repo: Arc<dyn crate::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository> = Arc::new(
            crate::repositories::DieselAttendanceAdjustmentRepository::new(pool.clone(), diesel_pools.clone()),
        );
        let adjustment_service = Arc::new(
            crate::services::attendance_adjustment_service::AttendanceAdjustmentService::new(
                Arc::clone(&adj_repo),
                Arc::clone(&time_entry_repo),
            ),
        );
        let import_repo: Arc<dyn crate::repositories::import_job_repository::ImportJobRepository> =
            Arc::new(crate::repositories::DieselImportJobRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            ));
        let import_service = Arc::new(
            crate::services::attendance_import_service::AttendanceImportService::new(
                Arc::clone(&import_repo),
                Arc::clone(&adj_repo),
                Arc::clone(&employee_repo),
                Arc::clone(&time_entry_repo),
                Arc::clone(&workspace_repo),
                worker_pool.clone(),
                diesel_pools.clone(),
            ),
        );
        import_service.resume_all_pending_jobs().await;

        let user_repo: Arc<dyn crate::repositories::user_repository::UserRepository> = Arc::new(
            crate::repositories::DieselUserRepository::new(pool.clone(), diesel_pools.clone()),
        );
        let session_repo: Arc<dyn crate::repositories::session_repository::SessionRepository> =
            Arc::new(crate::repositories::DieselSessionRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            ));
        let auth_service = Arc::new(crate::services::auth_service::AuthService::new(
            user_repo.clone(),
            session_repo.clone(),
        ));
        let shift_repo: Arc<dyn crate::repositories::shift_repository::ShiftRepository> = Arc::new(
            crate::repositories::DieselShiftRepository::new(pool.clone(), diesel_pools.clone()),
        );
        let shift_service = Arc::new(crate::services::shift_service::ShiftService::new(
            shift_repo.clone(),
        ));
        let workspace_service =
            Arc::new(crate::services::workspace_service::WorkspaceService::new(
                workspace_repo.clone(),
                user_repo.clone(),
            ));
        let leave_balance_repo: Arc<
            dyn crate::repositories::leave_balance_repository::LeaveBalanceRepository,
        > = Arc::new(crate::repositories::DieselLeaveBalanceRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let leave_request_repo: Arc<
            dyn crate::repositories::leave_request_repository::LeaveRequestRepository,
        > = Arc::new(crate::repositories::DieselLeaveRequestRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let leave_accrual_log_repo: Arc<
            dyn crate::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository,
        > = Arc::new(crate::repositories::DieselLeaveAccrualLogRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let leave_service = Arc::new(crate::services::leave_service::LeaveService::new(
            Arc::clone(&leave_balance_repo),
            Arc::clone(&leave_request_repo),
            Arc::clone(&employee_repo),
            Arc::clone(&roster_repo),
            Arc::clone(&leave_accrual_log_repo),
        ));
        let shift_swap_repo: Arc<
            dyn crate::repositories::shift_swap_repository::ShiftSwapRepository,
        > = Arc::new(crate::repositories::DieselShiftSwapRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let shift_swap_service =
            Arc::new(crate::services::shift_swap_service::ShiftSwapService::new(
                Arc::clone(&shift_swap_repo),
                Arc::clone(&roster_repo),
                Arc::clone(&employee_repo),
                pool.clone(),
            ));
        let holiday_leave_service = Arc::new(
            crate::services::holiday_leave_service::HolidayLeaveService::new(
                Arc::clone(&employee_repo),
                leave_balance_repo,
                Arc::clone(&leave_request_repo),
                Arc::clone(&roster_repo),
                pool.clone(),
            ),
        );
        let department_repo: Arc<
            dyn crate::repositories::department_repository::DepartmentRepository,
        > = Arc::new(
            crate::repositories::diesel_department_repository::DieselDepartmentRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            ),
        );
        let leave_policy_repo: Arc<
            dyn crate::repositories::leave_policy_repository::LeavePolicyRepository,
        > = Arc::new(crate::repositories::DieselLeavePolicyRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));

        let leave_accrual_service = Arc::new(
            crate::services::leave_accrual_service::LeaveAccrualService::new(
                Arc::clone(&employee_repo),
                Arc::clone(&department_repo),
                Arc::clone(&leave_policy_repo),
                leave_accrual_log_repo,
                worker_pool.clone(),
            ),
        );
        let payroll_repo: Arc<dyn crate::repositories::payroll_repository::PayrollRepository> =
            Arc::new(crate::repositories::DieselPayrollRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            ));
        let holiday_repo: Arc<dyn crate::repositories::holiday_repository::HolidayRepositoryTrait> =
            Arc::new(crate::repositories::DieselHolidayRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            ));
        let payroll_service = Arc::new(crate::services::payroll_service::PayrollService::new(
            crate::services::payroll_service::PayrollServiceArgs {
                payroll_repo: Arc::clone(&payroll_repo),
                department_repo: Arc::clone(&department_repo),
                time_entry_repo: Arc::clone(&time_entry_repo),
                workspace_repo: Arc::clone(&workspace_repo),
                holiday_repo: Arc::clone(&holiday_repo),
                roster_repo: Arc::clone(&roster_repo),
                pool: pool.clone(),
            },
        ));
        let jwt_secret = if cfg!(debug_assertions) {
            std::env::var("JWT_SECRET").unwrap_or_else(|_| {
                tracing::warn!("⚠️ JWT_SECRET not set — using insecure dev fallback. DO NOT use in production!");
                "dev_secret_key_leeattend_2026".to_string()
            })
        } else {
            std::env::var("JWT_SECRET").map_err(|_e| "JWT_SECRET must be set in production")?
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

        let raw_log_repo: Arc<
            dyn crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository,
        > = Arc::new(crate::repositories::DieselRawAttendanceLogRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let staging_repo: Arc<
            dyn crate::repositories::staging_log_repository::StagingLogRepository,
        > = Arc::new(crate::repositories::DieselStagingLogRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let error_repo: Arc<
            dyn crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository,
        > = Arc::new(
            crate::repositories::DieselAttendanceLogErrorRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            ),
        );

        let (attendance_service, log_worker_handle) = AttendanceLogService::new(
            worker_pool.clone(),
            Arc::clone(&pairing_service),
            crate::services::attendance_log_service::AttendanceLogServiceConfig {
                buffer_capacity,
                batch_size,
                flush_interval_ms,
            },
            crate::services::attendance_log_service::AttendanceLogServiceDeps {
                employee_repo: Arc::clone(&employee_repo),
                raw_log_repo: Arc::clone(&raw_log_repo),
                staging_repo: Arc::clone(&staging_repo),
                error_repo: Arc::clone(&error_repo),
            },
        );
        let attendance_service = Arc::new(attendance_service);

        // Roster cache (requires Redis; shared between app state and worker)
        let roster_cache = redis_pool
            .as_ref()
            .map(|rpool| Arc::new(crate::cache::RosterCache::new(rpool.clone())));

        // Initialize job worker if Redis is available
        let worker_cancel_token = CancellationToken::new();
        let (coordinator, worker_handle, scheduler_handle) = redis_pool.as_ref().map_or_else(
            || (None, None, None),
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
                    diesel_pools.clone(),
                    worker_config,
                    Arc::clone(&coordinator),
                    roster_cache.clone(),
                );

                let scheduler = HousekeepingScheduler::new(redis.clone());

                (
                    Some(coordinator),
                    Some(worker.start(worker_cancel_token.clone())),
                    Some(scheduler.start(worker_cancel_token.clone())),
                )
            },
        );

        let policy_repo: Arc<
            dyn crate::repositories::attendance_policy_repository::AttendancePolicyRepository,
        > = Arc::new(crate::repositories::DieselAttendancePolicyRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let payroll_rule_repo: Arc<
            dyn crate::repositories::payroll_rule_repository::PayrollRuleRepository,
        > = Arc::new(crate::repositories::DieselPayrollRuleRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let feature_flag_repo: Arc<
            dyn crate::repositories::feature_flag_repository::FeatureFlagRepositoryTrait,
        > = Arc::new(crate::repositories::DieselFeatureFlagRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let assignment_rule_repo: Arc<
            dyn crate::repositories::assignment_rule_repository::AssignmentRuleRepositoryTrait,
        > = Arc::new(crate::repositories::DieselAssignmentRuleRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let session_repo_core = session_repo.clone();
        let user_repo_core = user_repo.clone();

        let state = Arc::new(AppState {
            employees: crate::EmployeeState {
                service: employee_service,
                repo: employee_repo,
                contract_repo,
                employment_phase_repo,
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
                error_repo,
                policy_repo,
            },
            shifts: crate::ShiftState {
                service: shift_service,
                swap_service: shift_swap_service,
                roster_repo,
                shift_repo,
                swap_repo: shift_swap_repo,
                assignment_rule_repo: assignment_rule_repo.clone(),
            },
            leaves: crate::LeaveState {
                service: leave_service,
                holiday_leave_service,
                accrual_service: leave_accrual_service,
                request_repo: leave_request_repo,
                policy_repo: leave_policy_repo,
                holiday_repo: holiday_repo.clone(),
            },
            payroll: crate::PayrollState {
                service: payroll_service,
                repo: payroll_repo,
                rule_repo: payroll_rule_repo,
                department_repo,
                feature_flag_repo: feature_flag_repo.clone(),
            },
            reports: crate::ReportState {
                service: report_service,
                coordinator,
                stats_repo,
            },
            core: crate::CoreState {
                auth_service,
                workspace_service,
                workspace_repo,
                audit_repo,
                user_repo: user_repo_core,
                session_repo: session_repo_core,
                jwt_secret,
            },
            agents: crate::AgentState {
                manager: agent_manager,
                service: agent_service,
                repo: agent_repo,
            },
            infra: crate::InfraState {
                db: db_pools,
                diesel: diesel_pools,
                redis: redis_pool,
                roster_cache,
            },
        });

        let workers = AppWorkers {
            log_worker_handle,
            job_worker_handle: worker_handle,
            scheduler_handle,
            job_worker_token: worker_cancel_token,
        };

        Ok((state, workers))
    }
}
