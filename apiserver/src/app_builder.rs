#![allow(clippy::clone_on_ref_ptr)]
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::agent_manager::AgentManager;

use crate::queue::RedisPool;
use crate::services::{AttendanceLogService, ReportCoordinator};
use crate::state::AppState;
use crate::worker::{HousekeepingScheduler, JobWorker, WorkerConfig};

pub struct AppWorkers {
    pub log_worker_handle: JoinHandle<()>,
    pub job_worker_handle: Option<JoinHandle<()>>,
    pub scheduler_handle: Option<JoinHandle<()>>,
    pub outbox_sweep_handle: Option<JoinHandle<()>>,
    pub job_worker_token: CancellationToken,
}

impl AppState {
    #[allow(clippy::too_many_lines, clippy::similar_names)]
    pub async fn build(
        diesel_pools: crate::diesel_pool::DieselPools,
        redis_pool: Option<RedisPool>,
    ) -> Result<(Arc<Self>, AppWorkers), Box<dyn std::error::Error>> {
        // Use API pool for application-level initialization (HTTP handlers, CRUD)
        // Use Worker pool for background services (patient timeout, won't starve API)
        // Use Worker pool for background services (patient timeout, won't starve API)

        // Dev seed data removed — first registered user gets platform_admin automatically.

        let crypto_service = if let Ok(keys_json) = std::env::var("ENCRYPTION_KEYS_JSON") {
            let crypto = crate::crypto::CryptoService::from_json(&keys_json)
                .map_err(|e| format!("Invalid ENCRYPTION_KEYS_JSON: {e}"))?;
            Some(Arc::new(crypto))
        } else {
            let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
            if app_env == "production" {
                return Err("ENCRYPTION_KEYS_JSON must be set in production".into());
            }
            None
        };

        let agent_manager = AgentManager::new(
            redis_pool
                .clone()
                .map(|p| Arc::new(p) as Arc<dyn crate::agent_manager::PresenceStore>),
        );
        let agent_repo: Arc<dyn crate::repositories::agent_repository::AgentRepository> = Arc::new(
            crate::repositories::DieselAgentRepository::new(diesel_pools.clone()),
        );
        let workspace_repo: Arc<
            dyn crate::repositories::workspace_repository::WorkspaceRepository,
        > = Arc::new(crate::repositories::DieselWorkspaceRepository::new(
            diesel_pools.clone(),
        ));
        let audit_repo: Arc<dyn crate::repositories::audit_log_repository::AuditLogRepository> =
            Arc::new(crate::repositories::DieselAuditLogRepository::new(
                diesel_pools.clone(),
            ));
        let agent_service = Arc::new(crate::services::AgentService::new(
            Arc::clone(&agent_repo),
            agent_manager.clone(),
        ));
        let employee_repo: Arc<dyn crate::repositories::employee::EmployeeRepository> = Arc::new(
            crate::repositories::diesel_employee_repository::DieselEmployeeRepository::new_with_crypto(
                diesel_pools.clone(),
                crypto_service.clone(),
            ),
        );
        let employment_repo: Arc<dyn crate::repositories::EmploymentRepository> = Arc::new(
            crate::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
        );
        let department_repo: Arc<
            dyn crate::repositories::department_repository::DepartmentRepository,
        > = Arc::new(
            crate::repositories::diesel_department_repository::DieselDepartmentRepository::new(
                diesel_pools.clone(),
            ),
        );
        let user_repo: Arc<dyn crate::repositories::user_repository::UserRepository> = Arc::new(
            crate::repositories::DieselUserRepository::new(diesel_pools.clone()),
        );
        let staging_repo: Arc<
            dyn crate::repositories::staging_log_repository::StagingLogRepository,
        > = Arc::new(crate::repositories::DieselStagingLogRepository::new(
            diesel_pools.clone(),
        ));
        let leave_balance_repo: Arc<
            dyn crate::repositories::leave_balance_repository::LeaveBalanceRepository,
        > = Arc::new(crate::repositories::DieselLeaveBalanceRepository::new(
            diesel_pools.clone(),
        ));
        let employee_service = Arc::new(crate::services::EmployeeService::new(
            Arc::clone(&employee_repo),
            Arc::clone(&employment_repo),
            Arc::clone(&department_repo),
            Arc::clone(&user_repo),
            Arc::clone(&staging_repo),
            Arc::clone(&leave_balance_repo),
        ));
        let contract_repo = Arc::new(crate::repositories::DieselEmployeeContractRepository::new(
            diesel_pools.clone(),
        ));
        let employment_phase_repo = Arc::new(
            crate::repositories::DieselEmploymentPhaseRepository::new(diesel_pools.clone()),
        );
        let time_entry_repo: Arc<
            dyn crate::repositories::time_entry_repository::TimeEntryRepository,
        > = Arc::new(crate::repositories::DieselTimeEntryRepository::new(
            diesel_pools.clone(),
        ));
        let attendance_daily_summary_repo: Arc<
            dyn crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository,
        > = Arc::new(
            crate::repositories::diesel_attendance_daily_summary_repository::DieselAttendanceDailySummaryRepository::new(
                diesel_pools.clone(),
            ),
        );
        let behalf_assignment_repo: Arc<
            dyn crate::repositories::behalf_assignment_repository::BehalfAssignmentRepository,
        > = Arc::new(
            crate::repositories::diesel_behalf_assignment_repository::DieselBehalfAssignmentRepository::new(
                diesel_pools.clone(),
            ),
        );
        let roster_repo: Arc<dyn crate::repositories::roster::RosterRepository> = Arc::new(
            crate::repositories::DieselRosterRepository::new(diesel_pools.clone()),
        );
        let policy_repo: Arc<dyn crate::repositories::AttendancePolicyRepository> = Arc::new(
            crate::repositories::DieselAttendancePolicyRepository::new(diesel_pools.clone()),
        );
        let leave_request_repo: Arc<
            dyn crate::repositories::leave_request_repository::LeaveRequestRepository,
        > = Arc::new(crate::repositories::DieselLeaveRequestRepository::new(
            diesel_pools.clone(),
        ));
        let payroll_repo: Arc<dyn crate::repositories::payroll_repository::PayrollRepository> =
            Arc::new(crate::repositories::DieselPayrollRepository::new(
                diesel_pools.clone(),
            ));
        let holiday_repo: Arc<dyn crate::repositories::holiday_repository::HolidayRepository> =
            Arc::new(crate::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            ));
        let shift_repo: Arc<dyn crate::repositories::shift_repository::ShiftRepository> = Arc::new(
            crate::repositories::DieselShiftRepository::new(diesel_pools.clone()),
        );

        let outbox_repo: Arc<dyn crate::repositories::outbox_job_repository::OutboxJobRepository> =
            Arc::new(crate::repositories::DieselOutboxJobRepository::new(
                diesel_pools.clone(),
            ));
        let outbox_service = Arc::new(crate::services::OutboxJobService::new(outbox_repo.clone()));

        let time_tracking_service = Arc::new(crate::services::TimeTrackingService::new(
            Arc::clone(&time_entry_repo),
            Arc::clone(&roster_repo),
            Arc::clone(&policy_repo),
            Arc::clone(&employee_repo),
            Arc::clone(&employment_repo),
            Arc::clone(&workspace_repo),
            Arc::clone(&shift_repo),
            Arc::clone(&holiday_repo),
            Arc::clone(&leave_request_repo),
            Arc::clone(&attendance_daily_summary_repo),
        ));

        let attendance_summary_service = Arc::new(crate::services::AttendanceSummaryService::new(
            crate::services::attendance_summary_service::AttendanceSummaryServiceArgs {
                time_entry_repo: Arc::clone(&time_entry_repo),
                roster_repo: Arc::clone(&roster_repo),
                policy_repo: Arc::clone(&policy_repo),
                workspace_repo: Arc::clone(&workspace_repo),
                employee_repo: Arc::clone(&employee_repo),
                employment_repo: Arc::clone(&employment_repo),
                holiday_repo: Arc::clone(&holiday_repo),
                leave_request_repo: Arc::clone(&leave_request_repo),
                daily_summary_repo: Arc::clone(&attendance_daily_summary_repo),
            },
        ));
        let report_service = Arc::new(crate::services::ReportService::new(
            crate::services::report_service::ReportServiceArgs {
                time_entry_repo: Arc::clone(&time_entry_repo),
                roster_repo: Arc::clone(&roster_repo),
                policy_repo: Arc::clone(&policy_repo),
                employee_repo: Arc::clone(&employee_repo),
                employment_repo: Arc::clone(&employment_repo),
                attendance_daily_summary_repo: Arc::clone(&attendance_daily_summary_repo),
                leave_repo: Arc::clone(&leave_request_repo),
                holiday_repo: Arc::clone(&holiday_repo),
                workspace_repo: Arc::clone(&workspace_repo),
            },
        ));
        let stats_repo: Arc<
            dyn crate::repositories::workspace_stats_repository::WorkspaceStatsRepository,
        > = Arc::new(
            crate::repositories::workspace_stats_repository::DieselWorkspaceStatsRepository::new(
                diesel_pools.clone(),
            ),
        );
        let pairing_service = Arc::new(crate::services::pairing_service::PairingService::new(
            crate::services::pairing_service::PairingServiceArgs {
                time_entry_repo: Arc::clone(&time_entry_repo),
                roster_repo: Arc::clone(&roster_repo),
                policy_repo: Arc::clone(&policy_repo),
                workspace_repo: Arc::clone(&workspace_repo),
                employee_repo: Arc::clone(&employee_repo),
                employment_repo: Arc::clone(&employment_repo),
                holiday_repo: Arc::clone(&holiday_repo),
                leave_request_repo: Arc::clone(&leave_request_repo),
                daily_summary_repo: Arc::clone(&attendance_daily_summary_repo),
                raw_log_repo: Arc::new(crate::repositories::DieselRawAttendanceLogRepository::new(
                    diesel_pools.clone(),
                )),
                shift_repo: Arc::new(crate::repositories::DieselShiftRepository::new(
                    diesel_pools.clone(),
                )),
            },
        ));
        let adj_repo: Arc<
            dyn crate::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository,
        > = Arc::new(
            crate::repositories::DieselAttendanceAdjustmentRepository::new(
                diesel_pools.clone(),
            ),
        );
        let adjustment_service = Arc::new(
            crate::services::attendance_adjustment_service::AttendanceAdjustmentService::new(
                Arc::clone(&adj_repo),
                Arc::clone(&time_entry_repo),
            ),
        );
        let import_repo: Arc<dyn crate::repositories::import_job_repository::ImportJobRepository> =
            Arc::new(crate::repositories::DieselImportJobRepository::new(
                diesel_pools.clone(),
            ));

        let session_repo: Arc<dyn crate::repositories::session_repository::SessionRepository> =
            Arc::new(crate::repositories::DieselSessionRepository::new(
                diesel_pools.clone(),
            ));
        let auth_service = Arc::new(crate::services::auth_service::AuthService::new(
            user_repo.clone(),
            session_repo.clone(),
            redis_pool.clone(),
        ));
        let shift_service = Arc::new(crate::services::shift_service::ShiftService::new(
            shift_repo.clone(),
        ));
        let tenant_provisioner =
            Arc::new(crate::services::tenant_provisioner::TenantMigrator::new());

        let system_db_cluster_repo: Arc<
            dyn crate::repositories::system_db_cluster_repository::SystemDbClusterRepository,
        > = Arc::new(
            crate::repositories::diesel_system_db_cluster_repository::DieselSystemDbClusterRepository::new(
                diesel_pools.clone(),
            ),
        );

        let workspace_service =
            Arc::new(crate::services::workspace_service::WorkspaceService::new(
                workspace_repo.clone(),
                user_repo.clone(),
                tenant_provisioner.clone(),
                system_db_cluster_repo.clone(),
                diesel_pools.clone(),
            ));
        // leave_balance_repo already instantiated earlier in this function
        let leave_accrual_log_repo: Arc<
            dyn crate::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository,
        > = Arc::new(crate::repositories::DieselLeaveAccrualLogRepository::new(
            diesel_pools.clone(),
        ));
        let workspace_config_repo: Arc<
            dyn crate::repositories::workspace_config_repository::WorkspaceConfigRepository,
        > = Arc::new(
            crate::repositories::diesel_workspace_config_repository::DieselWorkspaceConfigRepository::new(
                diesel_pools.clone(),
            ),
        );

        let leave_service = Arc::new(crate::services::leave_service::LeaveService::new(
            Arc::clone(&leave_balance_repo),
            Arc::clone(&leave_request_repo),
            Arc::clone(&employee_repo),
            Arc::clone(&employment_repo),
            Arc::clone(&roster_repo),
            Arc::clone(&leave_accrual_log_repo),
            Arc::clone(&workspace_config_repo),
            Arc::clone(&user_repo),
            Arc::clone(&workspace_repo),
        ));
        let shift_swap_repo: Arc<
            dyn crate::repositories::shift_swap_repository::ShiftSwapRepository,
        > = Arc::new(crate::repositories::DieselShiftSwapRepository::new(
            diesel_pools.clone(),
        ));
        let shift_swap_service =
            Arc::new(crate::services::shift_swap_service::ShiftSwapService::new(
                Arc::clone(&shift_swap_repo),
                Arc::clone(&roster_repo),
                Arc::clone(&employee_repo),
                Arc::clone(&shift_repo),
                Arc::clone(&payroll_repo),
            ));

        let working_pattern_repo: Arc<
            dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait,
        > = Arc::new(crate::repositories::DieselWorkingPatternRepository::new(
            diesel_pools.clone(),
        ));

        let leave_policy_repo: Arc<
            dyn crate::repositories::leave_policy_repository::LeavePolicyRepository,
        > = Arc::new(crate::repositories::DieselLeavePolicyRepository::new(
            diesel_pools.clone(),
        ));

        let holiday_leave_service = Arc::new(
            crate::services::holiday_leave_service::HolidayLeaveService::new(
                Arc::clone(&employee_repo),
                Arc::clone(&employment_repo),
                Arc::clone(&leave_balance_repo),
                Arc::clone(&leave_request_repo),
                Arc::clone(&roster_repo),
                Arc::clone(&holiday_repo),
                Arc::clone(&leave_policy_repo),
            ),
        );

        let leave_accrual_service = Arc::new(
            crate::services::leave_accrual_service::LeaveAccrualService::new(
                Arc::clone(&employee_repo),
                Arc::clone(&department_repo),
                Arc::clone(&leave_policy_repo),
                Arc::clone(&leave_accrual_log_repo),
                Arc::clone(&time_entry_repo),
                Arc::clone(&leave_request_repo),
                Arc::clone(&attendance_daily_summary_repo),
                contract_repo.clone() as Arc<dyn crate::repositories::EmployeeContractRepository>,
            ),
        );

        let attendance_policy_repo: Arc<
            dyn crate::repositories::attendance_policy_repository::AttendancePolicyRepository,
        > = Arc::new(crate::repositories::DieselAttendancePolicyRepository::new(
            diesel_pools.clone(),
        ));
        let payroll_rule_repo: Arc<
            dyn crate::repositories::payroll_rule_repository::PayrollRuleRepository,
        > = Arc::new(crate::repositories::DieselPayrollRuleRepository::new(
            diesel_pools.clone(),
        ));
        let feature_flag_repo: Arc<
            dyn crate::repositories::feature_flag_repository::FeatureFlagRepository,
        > = Arc::new(crate::repositories::DieselFeatureFlagRepository::new(
            diesel_pools.clone(),
        ));
        let assignment_rule_repo: Arc<
            dyn crate::repositories::assignment_rule_repository::AssignmentRuleRepository,
        > = Arc::new(crate::repositories::DieselAssignmentRuleRepository::new(
            diesel_pools.clone(),
        ));

        let overtime_policy_repo: Arc<
            dyn crate::repositories::overtime_policy_repository::OvertimePolicyRepositoryTrait,
        > = Arc::new(crate::repositories::DieselOvertimePolicyRepository::new(
            diesel_pools.clone(),
        ));

        let salary_base_repo: Arc<
            dyn crate::repositories::salary_base_repository::SalaryBaseRepository,
        > = Arc::new(crate::repositories::DieselSalaryBaseRepository::new(
            diesel_pools.clone(),
        ));

        let approval_workflow_repo: Arc<dyn crate::repositories::approval_workflow_repository::ApprovalWorkflowRepository> =
            Arc::new(crate::repositories::diesel_approval_workflow_repository::DieselApprovalWorkflowRepository::new(diesel_pools.clone()));

        let overtime_request_repo: Arc<dyn crate::repositories::overtime_request_repository::OvertimeRequestRepository> =
            Arc::new(crate::repositories::diesel_overtime_request_repository::DieselOvertimeRequestRepository::new(diesel_pools.clone()));

        let overtime_request_service = Arc::new(crate::services::OvertimeRequestService::new(
            overtime_request_repo.clone(),
        ));

        let workspace_config_service = Arc::new(crate::services::WorkspaceConfigService::new(
            workspace_config_repo.clone(),
        ));

        let mut workflow_handlers: std::collections::HashMap<
            String,
            Arc<dyn crate::services::approval_workflow::WorkflowCallbackHandler>,
        > = std::collections::HashMap::new();
        workflow_handlers.insert(
            "attendance_adjustment".to_string(),
            adjustment_service.clone(),
        );
        workflow_handlers.insert("leave_request".to_string(), leave_service.clone());
        workflow_handlers.insert("ot_request".to_string(), overtime_request_service.clone());
        workflow_handlers.insert("ot_claim".to_string(), attendance_summary_service.clone());

        let approval_workflow_service = Arc::new(
            crate::services::ApprovalWorkflowService::new(
                approval_workflow_repo.clone(),
                workspace_config_repo.clone(),
                workspace_repo.clone(),
                employee_repo.clone(),
                employment_repo.clone(),
                department_repo.clone(),
                leave_request_repo.clone(),
                overtime_request_repo.clone(),
                attendance_daily_summary_repo.clone(),
            )
            .with_handlers(workflow_handlers),
        );

        let location_repo: Arc<dyn crate::repositories::LocationRepository> = Arc::new(
            crate::repositories::DieselLocationRepository::new(diesel_pools.clone()),
        );

        let location_service = Arc::new(crate::services::LocationService::new(
            location_repo.clone(),
            audit_repo.clone(),
        ));

        let employer_repo: Arc<dyn crate::repositories::EmployerRepository> = Arc::new(
            crate::repositories::DieselEmployerRepository::new(diesel_pools.clone()),
        );

        let employer_service = Arc::new(crate::services::EmployerService::new(
            employer_repo.clone(),
            audit_repo.clone(),
        ));

        let roster_provider: Arc<dyn crate::services::payroll_service::RosterDataProvider> =
            Arc::new(crate::services::payroll_service::DirectRosterDataProvider(
                Arc::clone(&roster_repo),
            ));

        let payroll_service = Arc::new(crate::services::payroll_service::PayrollService::new(
            crate::services::payroll_service::PayrollServiceArgs {
                payroll_repo: Arc::clone(&payroll_repo),
                department_repo: Arc::clone(&department_repo),
                time_entry_repo: Arc::clone(&time_entry_repo),
                workspace_repo: Arc::clone(&workspace_repo),
                holiday_repo: holiday_repo.clone(),
                roster_repo: roster_repo.clone(),
                leave_repo: leave_request_repo.clone(),
                policy_repo: attendance_policy_repo.clone(),
                employee_repo: Arc::clone(&employee_repo),
                overtime_policy_repo: Arc::clone(&overtime_policy_repo),
                salary_base_repo: Arc::clone(&salary_base_repo),
                roster_provider,
                contract_repo: Arc::clone(&contract_repo)
                    as Arc<dyn crate::repositories::EmployeeContractRepository>,
                employment_repo: Arc::clone(&employment_repo),
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
            diesel_pools.clone(),
        ));

        let error_repo: Arc<
            dyn crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository,
        > = Arc::new(
            crate::repositories::DieselAttendanceLogErrorRepository::new(diesel_pools.clone()),
        );

        let (attendance_service, log_worker_handle) = AttendanceLogService::new(
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

        let import_service = Arc::new(
            crate::services::attendance_import_service::AttendanceImportService::new(
                Arc::clone(&import_repo),
                Arc::clone(&adj_repo),
                Arc::clone(&employee_repo),
                Arc::clone(&time_entry_repo),
                Arc::clone(&workspace_repo),
                Arc::clone(&department_repo),
                Arc::clone(&staging_repo),
                crate::worker::deps::WorkerDeps::new_with_crypto(
                    &diesel_pools,
                    crypto_service.clone(),
                ),
                Some(Arc::clone(&attendance_service)),
            ),
        );
        import_service.resume_all_pending_jobs().await;

        // Roster cache (requires Redis; shared between app state and worker)
        let roster_cache = redis_pool
            .as_ref()
            .map(|rpool| Arc::new(crate::cache::RosterCache::new(rpool.clone())));

        // Initialize job worker if Redis is available
        let worker_cancel_token = CancellationToken::new();
        let (coordinator, worker_handle, scheduler_handle, outbox_sweep_handle) =
            redis_pool.as_ref().map_or_else(
                || (None, None, None, None),
                |redis| {
                    let worker_config = WorkerConfig::from_env();
                    tracing::info!(
                        max_concurrent = worker_config.max_concurrent_jobs,
                        poll_timeout = worker_config.poll_timeout_secs,
                        "Initializing job worker"
                    );

                    let queue_repo: Arc<
                        dyn crate::repositories::queue_repository::QueueRepository,
                    > = Arc::new(
                        crate::repositories::queue_repository::RedisQueueRepository::new(
                            redis.clone(),
                        ),
                    );

                    let coordinator = Arc::new(ReportCoordinator::new(
                        queue_repo.clone(),
                        Arc::clone(&employee_repo),
                        Arc::clone(&workspace_repo),
                        outbox_service.clone(),
                    ));

                    let worker = JobWorker::new(
                        redis.clone(),
                        &diesel_pools,
                        worker_config,
                        Arc::clone(&coordinator),
                        roster_cache.clone(),
                    );

                    let scheduler = HousekeepingScheduler::new(redis.clone());

                    let outbox_sweep = crate::worker::OutboxSweepScheduler::new(
                        Arc::clone(&workspace_repo),
                        outbox_service.clone(),
                        Arc::clone(&coordinator),
                    );

                    (
                        Some(coordinator),
                        Some(worker.start(worker_cancel_token.clone())),
                        Some(scheduler.start(worker_cancel_token.clone())),
                        Some(outbox_sweep.start(worker_cancel_token.clone())),
                    )
                },
            );

        let session_repo_core = session_repo.clone();
        let user_repo_core = user_repo.clone();

        // Create a separate reference for the recruitment repository trait
        let recruitment_repo: Arc<
            dyn crate::repositories::employee::EmployeeRecruitmentRepository,
        > = Arc::new(
            crate::repositories::diesel_employee_repository::DieselEmployeeRepository::new_with_crypto(
                diesel_pools.clone(),
                crypto_service.clone(),
            ),
        );
        let recruitment_service = Arc::new(crate::services::RecruitmentService::new(Arc::clone(
            &recruitment_repo,
        )));

        let state = Arc::new(AppState::new(
            crate::EmployeeState::new(
                employee_service,
                recruitment_service,
                employee_repo,
                employment_repo,
                contract_repo,
                employment_phase_repo,
            ),
            crate::AttendanceState::new(
                time_tracking_service,
                attendance_summary_service,
                attendance_service,
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
            ),
            crate::ShiftState::new(
                shift_service,
                shift_swap_service,
                roster_repo,
                shift_repo,
                shift_swap_repo,
                assignment_rule_repo.clone(),
                working_pattern_repo,
            ),
            crate::LeaveState::new(
                leave_service,
                holiday_leave_service,
                leave_accrual_service,
                leave_request_repo,
                leave_balance_repo,
                leave_policy_repo,
                holiday_repo.clone(),
            ),
            crate::PayrollState::new(
                payroll_service,
                payroll_repo,
                payroll_rule_repo,
                department_repo,
                feature_flag_repo.clone(),
                overtime_policy_repo,
                salary_base_repo,
                overtime_request_repo,
                overtime_request_service,
            ),
            crate::ReportState::new(report_service, coordinator, stats_repo),
            crate::CoreState::new(
                auth_service,
                workspace_service,
                workspace_repo,
                audit_repo,
                user_repo_core,
                session_repo_core,
                jwt_secret,
                workspace_config_repo,
                approval_workflow_repo,
                workspace_config_service,
                approval_workflow_service,
                location_service,
                location_repo,
                employer_service,
                employer_repo,
            ),
            crate::AgentState::new(agent_manager, agent_service, agent_repo),
            crate::InfraState::new(
                diesel_pools,
                redis_pool.clone(),
                redis_pool.clone().map(|r| {
                    Arc::new(crate::cache::RedisCache::new(r))
                        as Arc<dyn crate::repositories::CacheRepository>
                }),
                roster_cache,
                redis_pool.map(|r| {
                    Arc::new(crate::repositories::queue_repository::RedisQueueRepository::new(r))
                        as Arc<dyn crate::repositories::queue_repository::QueueRepository>
                }),
                tenant_provisioner.clone(),
                crypto_service,
                system_db_cluster_repo.clone(),
            ),
        ));

        let workers = AppWorkers {
            log_worker_handle,
            job_worker_handle: worker_handle,
            scheduler_handle,
            outbox_sweep_handle,
            job_worker_token: worker_cancel_token,
        };

        Ok((state, workers))
    }
}
