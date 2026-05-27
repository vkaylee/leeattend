#![allow(warnings)]
#![allow(clippy::all)]
#![allow(clippy::clone_on_ref_ptr)]
//! Shared application spawning utilities for integration tests.
//!
//! This module provides various `spawn_app` variants to avoid duplication
//! across test files.

use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::services::tenant_provisioner::TenantProvisionerTrait;
use apiserver::services::AttendanceLogService;
use apiserver::supavisor::TenantManager;
use apiserver::AppState;
use std::sync::Arc;
use uuid::Uuid;

pub struct DummyTenantProvisioner;
#[async_trait::async_trait]
impl TenantProvisionerTrait for DummyTenantProvisioner {
    async fn provision_tenant_db(
        &self,
        _workspace_id: &uuid::Uuid,
        db_url: &str,
    ) -> Result<String, apiserver::errors::AppError> {
        Ok(db_url.to_string())
    }
}

pub struct DummySystemDbClusterRepository;
#[async_trait::async_trait]
impl apiserver::repositories::system_db_cluster_repository::SystemDbClusterRepository
    for DummySystemDbClusterRepository
{
    async fn get_least_loaded_cluster(
        &self,
    ) -> Result<
        Option<apiserver::models::system_db_cluster::SystemDbCluster>,
        apiserver::errors::AppError,
    > {
        Ok(None)
    }
    async fn increment_active_tenants(
        &self,
        _cluster_id: uuid::Uuid,
    ) -> Result<(), apiserver::errors::AppError> {
        Ok(())
    }
    async fn create_cluster(
        &self,
        _cluster: &apiserver::models::system_db_cluster::NewSystemDbCluster,
    ) -> Result<apiserver::models::system_db_cluster::SystemDbCluster, apiserver::errors::AppError>
    {
        unimplemented!()
    }
    async fn list_clusters(
        &self,
    ) -> Result<
        Vec<apiserver::models::system_db_cluster::SystemDbCluster>,
        apiserver::errors::AppError,
    > {
        Ok(vec![])
    }
    async fn delete_cluster(
        &self,
        _cluster_id: uuid::Uuid,
    ) -> Result<(), apiserver::errors::AppError> {
        Ok(())
    }
    async fn update_cluster_status(
        &self,
        _cluster_id: uuid::Uuid,
        _is_active: bool,
    ) -> Result<apiserver::models::system_db_cluster::SystemDbCluster, apiserver::errors::AppError>
    {
        unimplemented!()
    }
}

pub struct DummyTenantManager;
#[async_trait::async_trait]
impl TenantManager for DummyTenantManager {
    async fn init_tenant(&self) {}
    async fn delete_tenant(&self, _workspace_id: &str) -> Result<(), apiserver::errors::AppError> {
        Ok(())
    }
    async fn register_tenant_dynamic(
        &self,
        _workspace_id: &str,
        _db_config: &serde_json::Value,
    ) -> Result<(), apiserver::errors::AppError> {
        Ok(())
    }
    async fn get_tenant(
        &self,
        _tenant_id: &str,
    ) -> Result<serde_json::Value, apiserver::errors::AppError> {
        Ok(serde_json::json!({}))
    }
}

use super::db::TestDatabase;
use super::factory::TestFactory;

/// Helper to create an isolated Redis pool for tests
pub async fn create_test_redis_pool() -> Option<apiserver::queue::RedisPool> {
    if std::env::var("REDIS_URL").is_err() {
        return None;
    }

    let mut base_redis_url = std::env::var("REDIS_URL_FOR_TEST_GENERATION")
        .or_else(|_| std::env::var("REDIS_URL"))
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    if base_redis_url.ends_with('/') {
        base_redis_url.pop();
    }
    if let Some(pos) = base_redis_url.rfind('/') {
        if base_redis_url[pos + 1..]
            .chars()
            .all(|c| c.is_ascii_digit())
            && pos > base_redis_url.find("://").unwrap() + 2
        {
            base_redis_url.truncate(pos);
        }
    }
    let logical_db = uuid::Uuid::new_v4().as_u128() % 16;
    let url = format!("{}/{}", base_redis_url, logical_db);

    for attempt in 0..10 {
        match apiserver::queue::RedisPool::new(&url).await {
            Ok(p) => return Some(p),
            Err(e) if attempt < 9 => {
                tracing::warn!(
                    "Failed to connect to redis at {} (attempt {}): {}",
                    url,
                    attempt,
                    e
                );
                tokio::time::sleep(std::time::Duration::from_millis(500 * (attempt + 1))).await;
            }
            Err(e) => panic!(
                "Failed to create Redis pool for test factory after 10 attempts: {}",
                e
            ),
        }
    }
    None
}

/// Basic app spawn - returns address (URL) and `TestDatabase`
#[allow(dead_code)]
pub async fn spawn_app() -> (String, TestDatabase) {
    let test_db = TestDatabase::new().await;
    let shared_redis_pool = create_test_redis_pool().await;
    let pool = test_db.diesel_pools.clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let agent_manager = AgentManager::new(None);
    let staging_repo: Arc<
        dyn apiserver::repositories::staging_log_repository::StagingLogRepository,
    > = Arc::new(apiserver::repositories::DieselStagingLogRepository::new(
        diesel_pools.clone(),
    ));

    let workspace_repo: Arc<dyn apiserver::repositories::WorkspaceRepository> = Arc::new(
        apiserver::repositories::DieselWorkspaceRepository::new(diesel_pools.clone()),
    );
    let user_repo: Arc<dyn apiserver::repositories::UserRepository> = Arc::new(
        apiserver::repositories::DieselUserRepository::new(diesel_pools.clone()),
    );
    let session_repo: Arc<dyn apiserver::repositories::SessionRepository> = Arc::new(
        apiserver::repositories::DieselSessionRepository::new(diesel_pools.clone()),
    );
    let roster_repo: Arc<dyn apiserver::repositories::roster::RosterRepository> = Arc::new(
        apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone()),
    );
    let shift_repo: Arc<dyn apiserver::repositories::shift_repository::ShiftRepository> = Arc::new(
        apiserver::repositories::DieselShiftRepository::new(diesel_pools.clone()),
    );
    let holiday_repo: Arc<dyn apiserver::repositories::holiday_repository::HolidayRepository> =
        Arc::new(apiserver::repositories::DieselHolidayRepository::new(
            diesel_pools.clone(),
        ));

    let audit_repo: Arc<dyn apiserver::repositories::AuditLogRepository> = Arc::new(
        apiserver::repositories::DieselAuditLogRepository::new(diesel_pools.clone()),
    );

    let agent_repo = Arc::new(apiserver::repositories::DieselAgentRepository::new(
        diesel_pools.clone(),
    ));

    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let employee_repo = Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
        diesel_pools.clone(),
    ));
    let employee_service = Arc::new(apiserver::services::EmployeeService::new(
        employee_repo.clone(),
        Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            diesel_pools.clone(),
        )),
        user_repo.clone(),
        staging_repo.clone(),
    ));
    let time_entry_repo: Arc<
        dyn apiserver::repositories::time_entry_repository::TimeEntryRepository,
    > = Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
        diesel_pools.clone(),
    ));

    let policy_repo: Arc<
        dyn apiserver::repositories::attendance_policy_repository::AttendancePolicyRepository,
    > = Arc::new(
        apiserver::repositories::DieselAttendancePolicyRepository::new(diesel_pools.clone()),
    );
    let leave_request_repo: Arc<
        dyn apiserver::repositories::leave_request_repository::LeaveRequestRepository,
    > = Arc::new(apiserver::repositories::DieselLeaveRequestRepository::new(
        diesel_pools.clone(),
    ));

    let attendance_summary_repo: Arc<
        dyn apiserver::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository,
    > = Arc::new(
        apiserver::repositories::DieselAttendanceDailySummaryRepository::new(diesel_pools.clone()),
    );
    let time_tracking_service = Arc::new(apiserver::services::TimeTrackingService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        policy_repo.clone(),
        employee_repo.clone(),
        workspace_repo.clone(),
        shift_repo.clone(),
        holiday_repo.clone(),
        leave_request_repo.clone(),
        attendance_summary_repo.clone(),
    ));

    let report_service = Arc::new(apiserver::services::ReportService::new(
        apiserver::services::report_service::ReportServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
            roster_repo: roster_repo.clone(),
            policy_repo: policy_repo.clone(),
            employee_repo: employee_repo.clone(),
            attendance_daily_summary_repo: attendance_summary_repo.clone(),
            leave_repo: leave_request_repo.clone(),
            holiday_repo: Arc::new(apiserver::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            )),
            workspace_repo: Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
                diesel_pools.clone(),
            )),
        },
    ));
    let pairing_service = Arc::new(apiserver::services::pairing_service::PairingService::new(
        apiserver::services::pairing_service::PairingServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
            roster_repo: roster_repo.clone(),
            policy_repo: policy_repo.clone(),
            workspace_repo: Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
                diesel_pools.clone(),
            )),
            employee_repo: Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            holiday_repo: Arc::new(apiserver::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            )),
            leave_request_repo: Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            daily_summary_repo: Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            raw_log_repo: Arc::new(
                apiserver::repositories::DieselRawAttendanceLogRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            shift_repo: Arc::new(apiserver::repositories::DieselShiftRepository::new(
                diesel_pools.clone(),
            )),
        },
    ));
    let adj_repo: Arc<dyn apiserver::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository> = Arc::new(apiserver::repositories::DieselAttendanceAdjustmentRepository::new(diesel_pools.clone()));
    let import_repo: Arc<dyn apiserver::repositories::import_job_repository::ImportJobRepository> =
        Arc::new(apiserver::repositories::DieselImportJobRepository::new(
            diesel_pools.clone(),
        ));
    let leave_balance_repo: Arc<
        dyn apiserver::repositories::leave_balance_repository::LeaveBalanceRepository,
    > = Arc::new(apiserver::repositories::DieselLeaveBalanceRepository::new(
        diesel_pools.clone(),
    ));
    let shift_swap_repo: Arc<
        dyn apiserver::repositories::shift_swap_repository::ShiftSwapRepository,
    > = Arc::new(apiserver::repositories::DieselShiftSwapRepository::new(
        diesel_pools.clone(),
    ));
    let adjustment_service = Arc::new(
        apiserver::services::attendance_adjustment_service::AttendanceAdjustmentService::new(
            adj_repo.clone(),
            time_entry_repo.clone(),
        ),
    );
    let import_service = Arc::new(
        apiserver::services::attendance_import_service::AttendanceImportService::new(
            import_repo.clone(),
            adj_repo.clone(),
            employee_repo.clone(),
            time_entry_repo.clone(),
            workspace_repo.clone(),
            Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
                diesel_pools.clone(),
            )),
            staging_repo.clone(),
            apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone()),
            None, // log_svc
        ),
    );
    let payroll_repo: Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> =
        Arc::new(apiserver::repositories::DieselPayrollRepository::new(
            diesel_pools.clone(),
        ));
    let payroll_rule_repo: Arc<
        dyn apiserver::repositories::payroll_rule_repository::PayrollRuleRepository,
    > = Arc::new(apiserver::repositories::DieselPayrollRuleRepository::new(
        diesel_pools.clone(),
    ));
    let department_repo: Arc<
        dyn apiserver::repositories::department_repository::DepartmentRepository,
    > = Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
        diesel_pools.clone(),
    ));

    let workspace_config_repo: Arc<dyn apiserver::repositories::workspace_config_repository::WorkspaceConfigRepository> =
        Arc::new(apiserver::repositories::diesel_workspace_config_repository::DieselWorkspaceConfigRepository::new(diesel_pools.clone()));
    let approval_workflow_repo: Arc<dyn apiserver::repositories::approval_workflow_repository::ApprovalWorkflowRepository> =
        Arc::new(apiserver::repositories::diesel_approval_workflow_repository::DieselApprovalWorkflowRepository::new(diesel_pools.clone()));
    let overtime_request_repo: Arc<dyn apiserver::repositories::overtime_request_repository::OvertimeRequestRepository> =
        Arc::new(apiserver::repositories::diesel_overtime_request_repository::DieselOvertimeRequestRepository::new(diesel_pools.clone()));
    let workspace_config_service = Arc::new(apiserver::services::WorkspaceConfigService::new(
        workspace_config_repo.clone(),
    ));
    let approval_workflow_service = Arc::new(apiserver::services::ApprovalWorkflowService::new(
        approval_workflow_repo.clone(),
    ));
    let overtime_request_service = Arc::new(apiserver::services::OvertimeRequestService::new(
        overtime_request_repo.clone(),
    ));

    let payroll_service = Arc::new(apiserver::services::payroll_service::PayrollService::new(
        apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: payroll_repo.clone(),
            department_repo: department_repo.clone(),
            time_entry_repo: time_entry_repo.clone(),
            workspace_repo: workspace_repo.clone(),
            holiday_repo: holiday_repo.clone(),
            roster_repo: roster_repo.clone(),
            leave_repo: leave_request_repo.clone(),
            employee_repo: employee_repo.clone(),
            policy_repo: policy_repo.clone(),
            overtime_policy_repo: Arc::new(
                apiserver::repositories::DieselOvertimePolicyRepository::new(diesel_pools.clone()),
            ),
            salary_base_repo: std::sync::Arc::new(
                apiserver::repositories::DieselSalaryBaseRepository::new(diesel_pools.clone()),
            ),

            roster_provider: std::sync::Arc::new(
                apiserver::services::payroll_service::DirectRosterDataProvider(roster_repo.clone()),
            ),
        },
    ));
    let auth_service = Arc::new(apiserver::services::auth_service::AuthService::new(
        user_repo.clone(),
        session_repo.clone(),
    ));
    let shift_service = Arc::new(apiserver::services::shift_service::ShiftService::new(
        shift_repo.clone(),
    ));
    let workspace_service = Arc::new(
        apiserver::services::workspace_service::WorkspaceService::new(
            workspace_repo.clone(),
            user_repo.clone(),
            std::sync::Arc::new(DummyTenantProvisioner),
            std::sync::Arc::new(DummyTenantManager),
            std::sync::Arc::new(DummySystemDbClusterRepository),
        ),
    );
    let leave_accrual_log_repo: Arc<
        dyn apiserver::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository,
    > = Arc::new(
        apiserver::repositories::DieselLeaveAccrualLogRepository::new(diesel_pools.clone()),
    );
    let leave_service = Arc::new(apiserver::services::leave_service::LeaveService::new(
        leave_balance_repo.clone(),
        leave_request_repo.clone(),
        employee_repo.clone(),
        roster_repo.clone(),
        leave_accrual_log_repo.clone(),
    ));
    let shift_swap_service = Arc::new(
        apiserver::services::shift_swap_service::ShiftSwapService::new(
            shift_swap_repo.clone(),
            roster_repo.clone(),
            employee_repo.clone(),
            shift_repo.clone(),
            payroll_repo.clone(),
        ),
    );
    let feature_flag_repo: Arc<
        dyn apiserver::repositories::feature_flag_repository::FeatureFlagRepository,
    > = Arc::new(apiserver::repositories::DieselFeatureFlagRepository::new(
        diesel_pools.clone(),
    ));
    let assignment_rule_repo: Arc<
        dyn apiserver::repositories::assignment_rule_repository::AssignmentRuleRepository,
    > = Arc::new(
        apiserver::repositories::DieselAssignmentRuleRepository::new(diesel_pools.clone()),
    );
    let leave_policy_repo: Arc<
        dyn apiserver::repositories::leave_policy_repository::LeavePolicyRepository,
    > = Arc::new(apiserver::repositories::DieselLeavePolicyRepository::new(
        diesel_pools.clone(),
    ));
    let working_pattern_repo: Arc<
        dyn apiserver::repositories::working_pattern_repository::WorkingPatternRepositoryTrait,
    > = Arc::new(
        apiserver::repositories::DieselWorkingPatternRepository::new(diesel_pools.clone()),
    );
    let holiday_leave_service = Arc::new(
        apiserver::services::holiday_leave_service::HolidayLeaveService::new(
            employee_repo.clone(),
            leave_balance_repo.clone(),
            leave_request_repo.clone(),
            roster_repo.clone(),
            holiday_repo.clone(),
            leave_policy_repo.clone(),
        ),
    );
    let leave_accrual_service = Arc::new(
        apiserver::services::leave_accrual_service::LeaveAccrualService::new(
            employee_repo.clone(),
            department_repo.clone(),
            leave_policy_repo.clone(),
            Arc::new(
                apiserver::repositories::DieselLeaveAccrualLogRepository::new(diesel_pools.clone()),
            ),
            time_entry_repo.clone(),
            leave_request_repo.clone(),
            Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            Arc::new(
                apiserver::repositories::DieselEmployeeContractRepository::new(
                    diesel_pools.clone(),
                ),
            ),
        ),
    );

    let attendance_summary_service = Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
            roster_repo: roster_repo.clone(),
            policy_repo: policy_repo.clone(),
            workspace_repo: Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
                diesel_pools.clone(),
            )),
            employee_repo: Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            holiday_repo: Arc::new(apiserver::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            )),
            leave_request_repo: Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            daily_summary_repo: Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
        },
    ));

    let raw_log_repo: Arc<
        dyn apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository,
    > = Arc::new(
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone()),
    );
    let error_repo: Arc<
        dyn apiserver::repositories::attendance_log_error_repository::AttendanceLogErrorRepository,
    > = Arc::new(
        apiserver::repositories::DieselAttendanceLogErrorRepository::new(diesel_pools.clone()),
    );

    let recruitment_repo: Arc<dyn apiserver::repositories::employee::EmployeeRecruitmentRepository> = employee_repo.clone();
    let recruitment_service = Arc::new(apiserver::services::RecruitmentService::new(
        recruitment_repo,
    ));

    let state = Arc::new(AppState {
        employees: apiserver::EmployeeState {
            service: employee_service,
            recruitment_service: recruitment_service.clone(),
            repo: employee_repo.clone(),
            contract_repo: Arc::new(
                apiserver::repositories::DieselEmployeeContractRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            employment_phase_repo: Arc::new(
                apiserver::repositories::DieselEmploymentPhaseRepository::new(diesel_pools.clone()),
            ),
        },
        attendance: apiserver::AttendanceState {
            time_tracking_service,
            summary_service: attendance_summary_service,
            log_service: Arc::new(
                AttendanceLogService::new(
                    pairing_service.clone(),
                    apiserver::services::attendance_log_service::AttendanceLogServiceConfig {
                        buffer_capacity: 100,
                        batch_size: 10,
                        flush_interval_ms: 1000,
                    },
                    apiserver::services::attendance_log_service::AttendanceLogServiceDeps {
                        employee_repo: employee_repo.clone(),
                        raw_log_repo: raw_log_repo.clone(),
                        staging_repo: staging_repo.clone(),
                        error_repo: error_repo.clone(),
                    },
                )
                .0,
            ),
            pairing_service,
            adjustment_service,
            import_service,
            time_entry_repo: time_entry_repo.clone(),
            staging_repo: staging_repo.clone(),
            raw_log_repo: raw_log_repo.clone(),
            import_repo: import_repo.clone(),
            error_repo: error_repo.clone(),
            policy_repo: policy_repo.clone(),
            attendance_daily_summary_repo: attendance_summary_repo.clone(),
        },
        shifts: apiserver::ShiftState {
            service: shift_service,
            swap_service: shift_swap_service,
            roster_repo: roster_repo.clone(),
            shift_repo: shift_repo.clone(),
            swap_repo: shift_swap_repo.clone(),
            assignment_rule_repo: assignment_rule_repo.clone(),
            working_pattern_repo: working_pattern_repo.clone(),
        },
        leaves: apiserver::LeaveState {
            service: leave_service,
            holiday_leave_service,
            accrual_service: leave_accrual_service,
            request_repo: leave_request_repo.clone(),
            policy_repo: leave_policy_repo.clone(),
            holiday_repo: holiday_repo.clone(),
            balance_repo: leave_balance_repo.clone(),
        },
        payroll: apiserver::PayrollState {
            service: payroll_service,
            repo: payroll_repo.clone(),
            rule_repo: payroll_rule_repo.clone(),
            department_repo: department_repo.clone(),
            feature_flag_repo: feature_flag_repo.clone(),
            overtime_policy_repo: Arc::new(
                apiserver::repositories::DieselOvertimePolicyRepository::new(diesel_pools.clone()),
            ),
            salary_base_repo: std::sync::Arc::new(
                apiserver::repositories::DieselSalaryBaseRepository::new(diesel_pools.clone()),
            ),
            overtime_request_repo: overtime_request_repo.clone(),
            overtime_request_service: overtime_request_service.clone(),
        },
        reports: apiserver::ReportState {
            service: report_service,
            coordinator: shared_redis_pool.clone().map(|redis| {
                let queue_repo: Arc<
                    dyn apiserver::repositories::queue_repository::QueueRepository,
                > = Arc::new(
                    apiserver::repositories::queue_repository::RedisQueueRepository::new(redis),
                );
                Arc::new(apiserver::services::ReportCoordinator::new(
                    queue_repo,
                    employee_repo.clone(),
                    workspace_repo.clone(),
                ))
            }),
            stats_repo: Arc::new(
                apiserver::repositories::DieselWorkspaceStatsRepository::new(diesel_pools.clone()),
            ),
        },
        core: apiserver::CoreState {
            auth_service,
            workspace_service,
            workspace_repo: workspace_repo.clone(),
            audit_repo: audit_repo.clone(),
            user_repo: user_repo.clone(),
            session_repo: session_repo.clone(),
            jwt_secret: "test_secret_tower".to_string(),
            workspace_config_repo: workspace_config_repo.clone(),
            workspace_config_service: workspace_config_service.clone(),
            approval_workflow_repo: approval_workflow_repo.clone(),
            approval_workflow_service: approval_workflow_service.clone(),
        },
        agents: apiserver::AgentState {
            manager: agent_manager,
            service: agent_service,
            repo: agent_repo.clone(),
        },
        infra: apiserver::InfraState {
            diesel: diesel_pools.clone(),
            redis: shared_redis_pool.clone(),
            cache: shared_redis_pool.clone().map(|r| {
                Arc::new(apiserver::cache::RedisCache::new(r))
                    as Arc<dyn apiserver::repositories::CacheRepository>
            }),
            roster_cache: None,
            queue: shared_redis_pool.map(|r| {
                Arc::new(apiserver::repositories::queue_repository::RedisQueueRepository::new(r))
                    as Arc<dyn apiserver::repositories::queue_repository::QueueRepository>
            }),
            tenant_manager: Arc::new(apiserver::supavisor::NoopTenantManager),
            tenant_provisioner: std::sync::Arc::new(DummyTenantProvisioner),
            crypto_service: std::env::var("ENCRYPTION_KEYS_JSON")
                .ok()
                .and_then(|keys_json| {
                    apiserver::crypto::CryptoService::from_json(&keys_json)
                        .ok()
                        .map(std::sync::Arc::new)
                }),
            system_db_cluster_repo: Arc::new(apiserver::repositories::diesel_system_db_cluster_repository::DieselSystemDbClusterRepository::new(diesel_pools.clone())),
        },
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://127.0.0.1:{port}"), test_db)
}

/// App spawn with factory - returns address, `workspace_id`, `user_id`, factory, and `TestDatabase`
#[allow(dead_code)]
pub async fn spawn_app_with_factory() -> (String, Uuid, Uuid, TestFactory, TestDatabase) {
    spawn_app_with_factory_extended(Arc::new(apiserver::supavisor::NoopTenantManager)).await
}

#[allow(dead_code)]
pub async fn spawn_app_with_factory_extended(
    tenant_manager: Arc<dyn apiserver::supavisor::TenantManager>,
) -> (String, Uuid, Uuid, TestFactory, TestDatabase) {
    init_tracing();
    tracing::info!("spawn_app_with_factory_extended: creating TestDatabase");
    let test_db = TestDatabase::new().await;
    tracing::info!("spawn_app_with_factory_extended: TestDatabase created");

    let pool = test_db.diesel_pools.clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    tracing::info!("spawn_app_with_factory_extended: creating Redis pool");
    let shared_redis_pool = create_test_redis_pool().await;
    tracing::info!("spawn_app_with_factory_extended: Redis pool created");

    let factory = TestFactory::new(pool.clone()).with_redis(shared_redis_pool.clone());

    tracing::info!("spawn_app_with_factory_extended: creating factory user");
    let user_id = factory.user().with_email("test@example.com").create().await;
    tracing::info!("spawn_app_with_factory_extended: creating factory workspace");
    let workspace_id = factory
        .workspace()
        .with_name("Test Workspace")
        .create_with_owner(user_id)
        .await;
    tracing::info!("spawn_app_with_factory_extended: workspace created");

    let agent_manager = AgentManager::new(None);
    let staging_repo: Arc<
        dyn apiserver::repositories::staging_log_repository::StagingLogRepository,
    > = Arc::new(apiserver::repositories::DieselStagingLogRepository::new(
        diesel_pools.clone(),
    ));

    let workspace_repo: Arc<dyn apiserver::repositories::WorkspaceRepository> = Arc::new(
        apiserver::repositories::DieselWorkspaceRepository::new(diesel_pools.clone()),
    );
    let user_repo: Arc<dyn apiserver::repositories::UserRepository> = Arc::new(
        apiserver::repositories::DieselUserRepository::new(diesel_pools.clone()),
    );
    let session_repo: Arc<dyn apiserver::repositories::SessionRepository> = Arc::new(
        apiserver::repositories::DieselSessionRepository::new(diesel_pools.clone()),
    );
    let roster_repo: Arc<dyn apiserver::repositories::roster::RosterRepository> = Arc::new(
        apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone()),
    );
    let shift_repo: Arc<dyn apiserver::repositories::shift_repository::ShiftRepository> = Arc::new(
        apiserver::repositories::DieselShiftRepository::new(diesel_pools.clone()),
    );
    let holiday_repo: Arc<dyn apiserver::repositories::holiday_repository::HolidayRepository> =
        Arc::new(apiserver::repositories::DieselHolidayRepository::new(
            diesel_pools.clone(),
        ));

    let audit_repo: Arc<dyn apiserver::repositories::AuditLogRepository> = Arc::new(
        apiserver::repositories::DieselAuditLogRepository::new(diesel_pools.clone()),
    );

    let agent_repo = Arc::new(apiserver::repositories::DieselAgentRepository::new(
        diesel_pools.clone(),
    ));
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let employee_repo = Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
        diesel_pools.clone(),
    ));
    let employee_service = Arc::new(apiserver::services::EmployeeService::new(
        employee_repo.clone(),
        Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            diesel_pools.clone(),
        )),
        user_repo.clone(),
        staging_repo.clone(),
    ));
    let time_entry_repo: Arc<
        dyn apiserver::repositories::time_entry_repository::TimeEntryRepository,
    > = Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
        diesel_pools.clone(),
    ));
    let policy_repo: Arc<
        dyn apiserver::repositories::attendance_policy_repository::AttendancePolicyRepository,
    > = Arc::new(
        apiserver::repositories::DieselAttendancePolicyRepository::new(diesel_pools.clone()),
    );
    let attendance_summary_repo: Arc<
        dyn apiserver::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository,
    > = Arc::new(
        apiserver::repositories::DieselAttendanceDailySummaryRepository::new(diesel_pools.clone()),
    );
    let leave_request_repo: Arc<
        dyn apiserver::repositories::leave_request_repository::LeaveRequestRepository,
    > = Arc::new(apiserver::repositories::DieselLeaveRequestRepository::new(
        diesel_pools.clone(),
    ));
    let time_tracking_service = Arc::new(apiserver::services::TimeTrackingService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        policy_repo.clone(),
        employee_repo.clone(),
        workspace_repo.clone(),
        shift_repo.clone(),
        holiday_repo.clone(),
        leave_request_repo.clone(),
        attendance_summary_repo.clone(),
    ));
    let attendance_summary_service = Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
            roster_repo: roster_repo.clone(),
            policy_repo: policy_repo.clone(),
            workspace_repo: Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
                diesel_pools.clone(),
            )),
            employee_repo: Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            holiday_repo: Arc::new(apiserver::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            )),
            leave_request_repo: Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            daily_summary_repo: Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
        },
    ));
    let report_service = Arc::new(apiserver::services::ReportService::new(
        apiserver::services::report_service::ReportServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
            roster_repo: roster_repo.clone(),
            policy_repo: policy_repo.clone(),
            employee_repo: employee_repo.clone(),
            attendance_daily_summary_repo: attendance_summary_repo.clone(),
            leave_repo: leave_request_repo.clone(),
            holiday_repo: Arc::new(apiserver::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            )),
            workspace_repo: Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
                diesel_pools.clone(),
            )),
        },
    ));
    let pairing_service = Arc::new(apiserver::services::pairing_service::PairingService::new(
        apiserver::services::pairing_service::PairingServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
            roster_repo: roster_repo.clone(),
            policy_repo: policy_repo.clone(),
            workspace_repo: Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
                diesel_pools.clone(),
            )),
            employee_repo: Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            holiday_repo: Arc::new(apiserver::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            )),
            leave_request_repo: Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            daily_summary_repo: Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            raw_log_repo: Arc::new(
                apiserver::repositories::DieselRawAttendanceLogRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            shift_repo: Arc::new(apiserver::repositories::DieselShiftRepository::new(
                diesel_pools.clone(),
            )),
        },
    ));
    let adj_repo: Arc<dyn apiserver::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository> = Arc::new(apiserver::repositories::DieselAttendanceAdjustmentRepository::new(diesel_pools.clone()));
    let import_repo: Arc<dyn apiserver::repositories::import_job_repository::ImportJobRepository> =
        Arc::new(apiserver::repositories::DieselImportJobRepository::new(
            diesel_pools.clone(),
        ));
    let leave_balance_repo: Arc<
        dyn apiserver::repositories::leave_balance_repository::LeaveBalanceRepository,
    > = Arc::new(apiserver::repositories::DieselLeaveBalanceRepository::new(
        diesel_pools.clone(),
    ));
    let shift_swap_repo: Arc<
        dyn apiserver::repositories::shift_swap_repository::ShiftSwapRepository,
    > = Arc::new(apiserver::repositories::DieselShiftSwapRepository::new(
        diesel_pools.clone(),
    ));
    let adjustment_service = Arc::new(
        apiserver::services::attendance_adjustment_service::AttendanceAdjustmentService::new(
            adj_repo.clone(),
            time_entry_repo.clone(),
        ),
    );
    let import_service = Arc::new(
        apiserver::services::attendance_import_service::AttendanceImportService::new(
            import_repo.clone(),
            adj_repo.clone(),
            employee_repo.clone(),
            time_entry_repo.clone(),
            workspace_repo.clone(),
            Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
                diesel_pools.clone(),
            )),
            staging_repo.clone(),
            apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone()),
            None, // log_svc
        ),
    );
    let payroll_repo: Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> =
        Arc::new(apiserver::repositories::DieselPayrollRepository::new(
            diesel_pools.clone(),
        ));
    let payroll_rule_repo: Arc<
        dyn apiserver::repositories::payroll_rule_repository::PayrollRuleRepository,
    > = Arc::new(apiserver::repositories::DieselPayrollRuleRepository::new(
        diesel_pools.clone(),
    ));
    let department_repo: Arc<
        dyn apiserver::repositories::department_repository::DepartmentRepository,
    > = Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
        diesel_pools.clone(),
    ));

    let workspace_config_repo: Arc<dyn apiserver::repositories::workspace_config_repository::WorkspaceConfigRepository> =
        Arc::new(apiserver::repositories::diesel_workspace_config_repository::DieselWorkspaceConfigRepository::new(diesel_pools.clone()));
    let approval_workflow_repo: Arc<dyn apiserver::repositories::approval_workflow_repository::ApprovalWorkflowRepository> =
        Arc::new(apiserver::repositories::diesel_approval_workflow_repository::DieselApprovalWorkflowRepository::new(diesel_pools.clone()));
    let overtime_request_repo: Arc<dyn apiserver::repositories::overtime_request_repository::OvertimeRequestRepository> =
        Arc::new(apiserver::repositories::diesel_overtime_request_repository::DieselOvertimeRequestRepository::new(diesel_pools.clone()));
    let workspace_config_service = Arc::new(apiserver::services::WorkspaceConfigService::new(
        workspace_config_repo.clone(),
    ));
    let approval_workflow_service = Arc::new(apiserver::services::ApprovalWorkflowService::new(
        approval_workflow_repo.clone(),
    ));
    let overtime_request_service = Arc::new(apiserver::services::OvertimeRequestService::new(
        overtime_request_repo.clone(),
    ));

    let payroll_service = Arc::new(apiserver::services::payroll_service::PayrollService::new(
        apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: payroll_repo.clone(),
            department_repo: department_repo.clone(),
            time_entry_repo: time_entry_repo.clone(),
            workspace_repo: workspace_repo.clone(),
            holiday_repo: holiday_repo.clone(),
            roster_repo: roster_repo.clone(),
            leave_repo: leave_request_repo.clone(),
            employee_repo: employee_repo.clone(),
            policy_repo: policy_repo.clone(),
            overtime_policy_repo: Arc::new(
                apiserver::repositories::DieselOvertimePolicyRepository::new(diesel_pools.clone()),
            ),
            salary_base_repo: std::sync::Arc::new(
                apiserver::repositories::DieselSalaryBaseRepository::new(diesel_pools.clone()),
            ),

            roster_provider: std::sync::Arc::new(
                apiserver::services::payroll_service::DirectRosterDataProvider(roster_repo.clone()),
            ),
        },
    ));
    let auth_service = Arc::new(apiserver::services::auth_service::AuthService::new(
        user_repo.clone(),
        session_repo.clone(),
    ));
    let shift_service = Arc::new(apiserver::services::shift_service::ShiftService::new(
        shift_repo.clone(),
    ));
    let workspace_service = Arc::new(
        apiserver::services::workspace_service::WorkspaceService::new(
            workspace_repo.clone(),
            user_repo.clone(),
            std::sync::Arc::new(DummyTenantProvisioner),
            std::sync::Arc::new(DummyTenantManager),
            std::sync::Arc::new(DummySystemDbClusterRepository),
        ),
    );
    let leave_accrual_log_repo: Arc<
        dyn apiserver::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository,
    > = Arc::new(
        apiserver::repositories::DieselLeaveAccrualLogRepository::new(diesel_pools.clone()),
    );
    let leave_service = Arc::new(apiserver::services::leave_service::LeaveService::new(
        leave_balance_repo.clone(),
        leave_request_repo.clone(),
        employee_repo.clone(),
        roster_repo.clone(),
        leave_accrual_log_repo.clone(),
    ));
    let shift_swap_service = Arc::new(
        apiserver::services::shift_swap_service::ShiftSwapService::new(
            shift_swap_repo.clone(),
            roster_repo.clone(),
            employee_repo.clone(),
            shift_repo.clone(),
            payroll_repo.clone(),
        ),
    );
    let feature_flag_repo: Arc<
        dyn apiserver::repositories::feature_flag_repository::FeatureFlagRepository,
    > = Arc::new(apiserver::repositories::DieselFeatureFlagRepository::new(
        diesel_pools.clone(),
    ));
    let assignment_rule_repo: Arc<
        dyn apiserver::repositories::assignment_rule_repository::AssignmentRuleRepository,
    > = Arc::new(
        apiserver::repositories::DieselAssignmentRuleRepository::new(diesel_pools.clone()),
    );
    let leave_policy_repo: Arc<
        dyn apiserver::repositories::leave_policy_repository::LeavePolicyRepository,
    > = Arc::new(apiserver::repositories::DieselLeavePolicyRepository::new(
        diesel_pools.clone(),
    ));
    let working_pattern_repo: Arc<
        dyn apiserver::repositories::working_pattern_repository::WorkingPatternRepositoryTrait,
    > = Arc::new(
        apiserver::repositories::DieselWorkingPatternRepository::new(diesel_pools.clone()),
    );
    let holiday_leave_service = Arc::new(
        apiserver::services::holiday_leave_service::HolidayLeaveService::new(
            employee_repo.clone(),
            leave_balance_repo.clone(),
            leave_request_repo.clone(),
            roster_repo.clone(),
            holiday_repo.clone(),
            leave_policy_repo.clone(),
        ),
    );
    let leave_accrual_service = Arc::new(
        apiserver::services::leave_accrual_service::LeaveAccrualService::new(
            employee_repo.clone(),
            department_repo.clone(),
            leave_policy_repo.clone(),
            Arc::new(
                apiserver::repositories::DieselLeaveAccrualLogRepository::new(diesel_pools.clone()),
            ),
            time_entry_repo.clone(),
            leave_request_repo.clone(),
            Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            Arc::new(
                apiserver::repositories::DieselEmployeeContractRepository::new(
                    diesel_pools.clone(),
                ),
            ),
        ),
    );

    let attendance_summary_service = Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
            roster_repo: roster_repo.clone(),
            policy_repo: policy_repo.clone(),
            workspace_repo: Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
                diesel_pools.clone(),
            )),
            employee_repo: Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            holiday_repo: Arc::new(apiserver::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            )),
            leave_request_repo: Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            daily_summary_repo: Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
        },
    ));

    let raw_log_repo: Arc<
        dyn apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository,
    > = Arc::new(
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone()),
    );
    let error_repo: Arc<
        dyn apiserver::repositories::attendance_log_error_repository::AttendanceLogErrorRepository,
    > = Arc::new(
        apiserver::repositories::DieselAttendanceLogErrorRepository::new(diesel_pools.clone()),
    );

    let recruitment_repo: Arc<dyn apiserver::repositories::employee::EmployeeRecruitmentRepository> = employee_repo.clone();
    let recruitment_service = Arc::new(apiserver::services::RecruitmentService::new(
        recruitment_repo,
    ));

    let state = Arc::new(AppState {
        employees: apiserver::EmployeeState {
            service: employee_service,
            recruitment_service: recruitment_service.clone(),
            repo: employee_repo.clone(),
            contract_repo: Arc::new(
                apiserver::repositories::DieselEmployeeContractRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            employment_phase_repo: Arc::new(
                apiserver::repositories::DieselEmploymentPhaseRepository::new(diesel_pools.clone()),
            ),
        },
        attendance: apiserver::AttendanceState {
            time_tracking_service,
            summary_service: attendance_summary_service,
            log_service: Arc::new(
                AttendanceLogService::new(
                    pairing_service.clone(),
                    apiserver::services::attendance_log_service::AttendanceLogServiceConfig {
                        buffer_capacity: 100,
                        batch_size: 10,
                        flush_interval_ms: 1000,
                    },
                    apiserver::services::attendance_log_service::AttendanceLogServiceDeps {
                        employee_repo: employee_repo.clone(),
                        raw_log_repo: raw_log_repo.clone(),
                        staging_repo: staging_repo.clone(),
                        error_repo: error_repo.clone(),
                    },
                )
                .0,
            ),
            pairing_service,
            adjustment_service,
            import_service,
            time_entry_repo: time_entry_repo.clone(),
            staging_repo: staging_repo.clone(),
            raw_log_repo: raw_log_repo.clone(),
            import_repo: import_repo.clone(),
            error_repo: error_repo.clone(),
            policy_repo: policy_repo.clone(),
            attendance_daily_summary_repo: attendance_summary_repo.clone(),
        },
        shifts: apiserver::ShiftState {
            service: shift_service,
            swap_service: shift_swap_service,
            roster_repo: roster_repo.clone(),
            shift_repo: shift_repo.clone(),
            swap_repo: shift_swap_repo.clone(),
            assignment_rule_repo: assignment_rule_repo.clone(),
            working_pattern_repo: working_pattern_repo.clone(),
        },
        leaves: apiserver::LeaveState {
            service: leave_service,
            holiday_leave_service,
            accrual_service: leave_accrual_service,
            request_repo: leave_request_repo.clone(),
            policy_repo: leave_policy_repo.clone(),
            holiday_repo: holiday_repo.clone(),
            balance_repo: leave_balance_repo.clone(),
        },
        payroll: apiserver::PayrollState {
            service: payroll_service,
            repo: payroll_repo.clone(),
            rule_repo: payroll_rule_repo.clone(),
            department_repo: department_repo.clone(),
            feature_flag_repo: feature_flag_repo.clone(),
            overtime_policy_repo: Arc::new(
                apiserver::repositories::DieselOvertimePolicyRepository::new(diesel_pools.clone()),
            ),
            salary_base_repo: std::sync::Arc::new(
                apiserver::repositories::DieselSalaryBaseRepository::new(diesel_pools.clone()),
            ),
            overtime_request_repo: overtime_request_repo.clone(),
            overtime_request_service: overtime_request_service.clone(),
        },
        reports: apiserver::ReportState {
            service: report_service,
            coordinator: shared_redis_pool.clone().map(|redis| {
                let queue_repo: Arc<
                    dyn apiserver::repositories::queue_repository::QueueRepository,
                > = Arc::new(
                    apiserver::repositories::queue_repository::RedisQueueRepository::new(redis),
                );
                Arc::new(apiserver::services::ReportCoordinator::new(
                    queue_repo,
                    employee_repo.clone(),
                    workspace_repo.clone(),
                ))
            }),
            stats_repo: Arc::new(
                apiserver::repositories::DieselWorkspaceStatsRepository::new(diesel_pools.clone()),
            ),
        },
        core: apiserver::CoreState {
            auth_service,
            workspace_service,
            workspace_repo: workspace_repo.clone(),
            audit_repo: audit_repo.clone(),
            user_repo: user_repo.clone(),
            session_repo: session_repo.clone(),
            jwt_secret: "test_secret_tower".to_string(),
            workspace_config_repo: workspace_config_repo.clone(),
            workspace_config_service: workspace_config_service.clone(),
            approval_workflow_repo: approval_workflow_repo.clone(),
            approval_workflow_service: approval_workflow_service.clone(),
        },
        agents: apiserver::AgentState {
            manager: agent_manager,
            service: agent_service,
            repo: agent_repo.clone(),
        },
        infra: apiserver::InfraState {
            diesel: diesel_pools.clone(),
            redis: shared_redis_pool.clone(),
            cache: shared_redis_pool.clone().map(|r| {
                Arc::new(apiserver::cache::RedisCache::new(r))
                    as Arc<dyn apiserver::repositories::CacheRepository>
            }),
            roster_cache: None,
            queue: shared_redis_pool.clone().map(|r| {
                Arc::new(apiserver::repositories::queue_repository::RedisQueueRepository::new(r))
                    as Arc<dyn apiserver::repositories::queue_repository::QueueRepository>
            }),
            tenant_manager,
            tenant_provisioner: std::sync::Arc::new(DummyTenantProvisioner),
            crypto_service: std::env::var("ENCRYPTION_KEYS_JSON")
                .ok()
                .and_then(|keys_json| {
                    apiserver::crypto::CryptoService::from_json(&keys_json)
                        .ok()
                        .map(std::sync::Arc::new)
                }),
            system_db_cluster_repo: Arc::new(apiserver::repositories::diesel_system_db_cluster_repository::DieselSystemDbClusterRepository::new(diesel_pools.clone())),
        },
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (
        format!("http://127.0.0.1:{port}"),
        workspace_id,
        user_id,
        factory,
        test_db,
    )
}

static TRACING_INITIALIZED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
fn init_tracing() {
    TRACING_INITIALIZED.get_or_init(|| {
        tracing_subscriber::fmt()
            .with_env_filter("trace")
            .with_writer(std::io::stdout)
            .init();
    });
}

/// App spawn with member - creates workspace with owner and a member user
#[allow(dead_code)]
pub async fn spawn_app_with_member() -> (String, Uuid, Uuid, Uuid, TestFactory, TestDatabase) {
    let (address, workspace_id, owner_id, factory, test_db) = spawn_app_with_factory().await;

    let member_id = factory
        .user()
        .with_email("member@example.com")
        .create()
        .await;
    factory
        .add_member(
            workspace_id,
            member_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    (address, workspace_id, owner_id, member_id, factory, test_db)
}

/// Setup app for Tower-based testing (no HTTP server, direct router calls)
#[allow(dead_code)]
pub async fn setup_app_tower() -> (TestDatabase, TestFactory, Arc<AppState>, axum::Router) {
    setup_app_tower_internal(true).await
}

#[allow(dead_code)]
pub async fn setup_app_tower_without_redis(
) -> (TestDatabase, TestFactory, Arc<AppState>, axum::Router) {
    setup_app_tower_internal(false).await
}

pub async fn setup_app_tower_internal(
    use_redis: bool,
) -> (TestDatabase, TestFactory, Arc<AppState>, axum::Router) {
    setup_app_tower_internal_extended(use_redis, Arc::new(apiserver::supavisor::NoopTenantManager))
        .await
}

pub async fn setup_app_tower_internal_extended(
    use_redis: bool,
    tenant_manager: Arc<dyn apiserver::supavisor::TenantManager>,
) -> (TestDatabase, TestFactory, Arc<AppState>, axum::Router) {
    init_tracing();
    let test_db = TestDatabase::new().await;
    let shared_redis_pool = if use_redis {
        create_test_redis_pool().await
    } else {
        None
    };
    let pool = test_db.diesel_pools.clone().clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let factory = TestFactory::new(pool.clone()).with_redis(shared_redis_pool.clone());

    let agent_repo = Arc::new(apiserver::repositories::DieselAgentRepository::new(
        diesel_pools.clone(),
    ));
    let workspace_repo: Arc<dyn apiserver::repositories::WorkspaceRepository> = Arc::new(
        apiserver::repositories::DieselWorkspaceRepository::new(diesel_pools.clone()),
    );
    let user_repo: Arc<dyn apiserver::repositories::UserRepository> = Arc::new(
        apiserver::repositories::DieselUserRepository::new(diesel_pools.clone()),
    );
    let session_repo: Arc<dyn apiserver::repositories::SessionRepository> = Arc::new(
        apiserver::repositories::DieselSessionRepository::new(diesel_pools.clone()),
    );
    let roster_repo: Arc<dyn apiserver::repositories::roster::RosterRepository> = Arc::new(
        apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone()),
    );
    let shift_repo: Arc<dyn apiserver::repositories::shift_repository::ShiftRepository> = Arc::new(
        apiserver::repositories::DieselShiftRepository::new(diesel_pools.clone()),
    );
    let holiday_repo: Arc<dyn apiserver::repositories::holiday_repository::HolidayRepository> =
        Arc::new(apiserver::repositories::DieselHolidayRepository::new(
            diesel_pools.clone(),
        ));

    let audit_repo: Arc<dyn apiserver::repositories::AuditLogRepository> = Arc::new(
        apiserver::repositories::DieselAuditLogRepository::new(diesel_pools.clone()),
    );

    let agent_manager = AgentManager::new(None);
    let staging_repo: Arc<
        dyn apiserver::repositories::staging_log_repository::StagingLogRepository,
    > = Arc::new(apiserver::repositories::DieselStagingLogRepository::new(
        diesel_pools.clone(),
    ));

    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let employee_repo = Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
        diesel_pools.clone(),
    ));
    let employee_service = Arc::new(apiserver::services::EmployeeService::new(
        employee_repo.clone(),
        Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            diesel_pools.clone(),
        )),
        user_repo.clone(),
        staging_repo.clone(),
    ));
    let time_entry_repo: Arc<
        dyn apiserver::repositories::time_entry_repository::TimeEntryRepository,
    > = Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
        diesel_pools.clone(),
    ));

    let policy_repo: Arc<
        dyn apiserver::repositories::attendance_policy_repository::AttendancePolicyRepository,
    > = Arc::new(
        apiserver::repositories::DieselAttendancePolicyRepository::new(diesel_pools.clone()),
    );
    let leave_request_repo: Arc<
        dyn apiserver::repositories::leave_request_repository::LeaveRequestRepository,
    > = Arc::new(apiserver::repositories::DieselLeaveRequestRepository::new(
        diesel_pools.clone(),
    ));
    let attendance_daily_summary_repo: Arc<
        dyn apiserver::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository,
    > = Arc::new(
        apiserver::repositories::DieselAttendanceDailySummaryRepository::new(diesel_pools.clone()),
    );
    let time_tracking_service = Arc::new(apiserver::services::TimeTrackingService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        policy_repo.clone(),
        employee_repo.clone(),
        workspace_repo.clone(),
        shift_repo.clone(),
        holiday_repo.clone(),
        leave_request_repo.clone(),
        attendance_daily_summary_repo.clone(),
    ));
    let attendance_summary_service = Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
            roster_repo: roster_repo.clone(),
            policy_repo: policy_repo.clone(),
            workspace_repo: Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
                diesel_pools.clone(),
            )),
            employee_repo: Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            holiday_repo: Arc::new(apiserver::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            )),
            leave_request_repo: Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            daily_summary_repo: Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
        },
    ));
    let report_service = Arc::new(apiserver::services::ReportService::new(
        apiserver::services::report_service::ReportServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
            roster_repo: roster_repo.clone(),
            policy_repo: policy_repo.clone(),
            employee_repo: employee_repo.clone(),
            attendance_daily_summary_repo: Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            leave_repo: Arc::new(apiserver::repositories::DieselLeaveRequestRepository::new(
                diesel_pools.clone(),
            )),
            holiday_repo: Arc::new(apiserver::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            )),
            workspace_repo: Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
                diesel_pools.clone(),
            )),
        },
    ));
    let pairing_service = Arc::new(apiserver::services::pairing_service::PairingService::new(
        apiserver::services::pairing_service::PairingServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
            roster_repo: roster_repo.clone(),
            policy_repo: policy_repo.clone(),
            workspace_repo: Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
                diesel_pools.clone(),
            )),
            employee_repo: Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            holiday_repo: Arc::new(apiserver::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            )),
            leave_request_repo: Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            daily_summary_repo: Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            raw_log_repo: Arc::new(
                apiserver::repositories::DieselRawAttendanceLogRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            shift_repo: Arc::new(apiserver::repositories::DieselShiftRepository::new(
                diesel_pools.clone(),
            )),
        },
    ));
    let adj_repo: Arc<dyn apiserver::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository> = Arc::new(apiserver::repositories::DieselAttendanceAdjustmentRepository::new(diesel_pools.clone()));
    let import_repo: Arc<dyn apiserver::repositories::import_job_repository::ImportJobRepository> =
        Arc::new(apiserver::repositories::DieselImportJobRepository::new(
            diesel_pools.clone(),
        ));
    let leave_balance_repo: Arc<
        dyn apiserver::repositories::leave_balance_repository::LeaveBalanceRepository,
    > = Arc::new(apiserver::repositories::DieselLeaveBalanceRepository::new(
        diesel_pools.clone(),
    ));
    let leave_request_repo: Arc<
        dyn apiserver::repositories::leave_request_repository::LeaveRequestRepository,
    > = Arc::new(apiserver::repositories::DieselLeaveRequestRepository::new(
        diesel_pools.clone(),
    ));
    let shift_swap_repo: Arc<
        dyn apiserver::repositories::shift_swap_repository::ShiftSwapRepository,
    > = Arc::new(apiserver::repositories::DieselShiftSwapRepository::new(
        diesel_pools.clone(),
    ));
    let adjustment_service = Arc::new(
        apiserver::services::attendance_adjustment_service::AttendanceAdjustmentService::new(
            adj_repo.clone(),
            time_entry_repo.clone(),
        ),
    );
    let import_service = Arc::new(
        apiserver::services::attendance_import_service::AttendanceImportService::new(
            import_repo.clone(),
            adj_repo.clone(),
            employee_repo.clone(),
            time_entry_repo.clone(),
            workspace_repo.clone(),
            Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
                diesel_pools.clone(),
            )),
            staging_repo.clone(),
            apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone()),
            None, // log_svc
        ),
    );
    let payroll_repo: Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> =
        Arc::new(apiserver::repositories::DieselPayrollRepository::new(
            diesel_pools.clone(),
        ));
    let payroll_rule_repo: Arc<
        dyn apiserver::repositories::payroll_rule_repository::PayrollRuleRepository,
    > = Arc::new(apiserver::repositories::DieselPayrollRuleRepository::new(
        diesel_pools.clone(),
    ));
    let department_repo: Arc<
        dyn apiserver::repositories::department_repository::DepartmentRepository,
    > = Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
        diesel_pools.clone(),
    ));

    let workspace_config_repo: Arc<dyn apiserver::repositories::workspace_config_repository::WorkspaceConfigRepository> =
        Arc::new(apiserver::repositories::diesel_workspace_config_repository::DieselWorkspaceConfigRepository::new(diesel_pools.clone()));
    let approval_workflow_repo: Arc<dyn apiserver::repositories::approval_workflow_repository::ApprovalWorkflowRepository> =
        Arc::new(apiserver::repositories::diesel_approval_workflow_repository::DieselApprovalWorkflowRepository::new(diesel_pools.clone()));
    let overtime_request_repo: Arc<dyn apiserver::repositories::overtime_request_repository::OvertimeRequestRepository> =
        Arc::new(apiserver::repositories::diesel_overtime_request_repository::DieselOvertimeRequestRepository::new(diesel_pools.clone()));
    let workspace_config_service = Arc::new(apiserver::services::WorkspaceConfigService::new(
        workspace_config_repo.clone(),
    ));
    let approval_workflow_service = Arc::new(apiserver::services::ApprovalWorkflowService::new(
        approval_workflow_repo.clone(),
    ));
    let overtime_request_service = Arc::new(apiserver::services::OvertimeRequestService::new(
        overtime_request_repo.clone(),
    ));

    let payroll_service = Arc::new(apiserver::services::payroll_service::PayrollService::new(
        apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: payroll_repo.clone(),
            department_repo: department_repo.clone(),
            time_entry_repo: time_entry_repo.clone(),
            workspace_repo: workspace_repo.clone(),
            holiday_repo: holiday_repo.clone(),
            roster_repo: roster_repo.clone(),
            leave_repo: leave_request_repo.clone(),
            employee_repo: employee_repo.clone(),
            policy_repo: policy_repo.clone(),
            overtime_policy_repo: Arc::new(
                apiserver::repositories::DieselOvertimePolicyRepository::new(diesel_pools.clone()),
            ),
            salary_base_repo: std::sync::Arc::new(
                apiserver::repositories::DieselSalaryBaseRepository::new(diesel_pools.clone()),
            ),

            roster_provider: std::sync::Arc::new(
                apiserver::services::payroll_service::DirectRosterDataProvider(roster_repo.clone()),
            ),
        },
    ));
    let auth_service = Arc::new(apiserver::services::auth_service::AuthService::new(
        user_repo.clone(),
        session_repo.clone(),
    ));
    let shift_service = Arc::new(apiserver::services::shift_service::ShiftService::new(
        shift_repo.clone(),
    ));
    let workspace_service = Arc::new(
        apiserver::services::workspace_service::WorkspaceService::new(
            workspace_repo.clone(),
            user_repo.clone(),
            std::sync::Arc::new(DummyTenantProvisioner),
            std::sync::Arc::new(DummyTenantManager),
            std::sync::Arc::new(DummySystemDbClusterRepository),
        ),
    );
    let leave_accrual_log_repo: Arc<
        dyn apiserver::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository,
    > = Arc::new(
        apiserver::repositories::DieselLeaveAccrualLogRepository::new(diesel_pools.clone()),
    );
    let leave_service = Arc::new(apiserver::services::leave_service::LeaveService::new(
        leave_balance_repo.clone(),
        leave_request_repo.clone(),
        employee_repo.clone(),
        roster_repo.clone(),
        leave_accrual_log_repo.clone(),
    ));
    let shift_swap_service = Arc::new(
        apiserver::services::shift_swap_service::ShiftSwapService::new(
            shift_swap_repo.clone(),
            roster_repo.clone(),
            employee_repo.clone(),
            shift_repo.clone(),
            payroll_repo.clone(),
        ),
    );
    let feature_flag_repo: Arc<
        dyn apiserver::repositories::feature_flag_repository::FeatureFlagRepository,
    > = Arc::new(apiserver::repositories::DieselFeatureFlagRepository::new(
        diesel_pools.clone(),
    ));
    let assignment_rule_repo: Arc<
        dyn apiserver::repositories::assignment_rule_repository::AssignmentRuleRepository,
    > = Arc::new(
        apiserver::repositories::DieselAssignmentRuleRepository::new(diesel_pools.clone()),
    );
    let leave_policy_repo: Arc<
        dyn apiserver::repositories::leave_policy_repository::LeavePolicyRepository,
    > = Arc::new(apiserver::repositories::DieselLeavePolicyRepository::new(
        diesel_pools.clone(),
    ));
    let working_pattern_repo: Arc<
        dyn apiserver::repositories::working_pattern_repository::WorkingPatternRepositoryTrait,
    > = Arc::new(
        apiserver::repositories::DieselWorkingPatternRepository::new(diesel_pools.clone()),
    );
    let holiday_leave_service = Arc::new(
        apiserver::services::holiday_leave_service::HolidayLeaveService::new(
            employee_repo.clone(),
            leave_balance_repo.clone(),
            leave_request_repo.clone(),
            roster_repo.clone(),
            holiday_repo.clone(),
            leave_policy_repo.clone(),
        ),
    );
    let leave_accrual_service = Arc::new(
        apiserver::services::leave_accrual_service::LeaveAccrualService::new(
            employee_repo.clone(),
            department_repo.clone(),
            leave_policy_repo.clone(),
            Arc::new(
                apiserver::repositories::DieselLeaveAccrualLogRepository::new(diesel_pools.clone()),
            ),
            time_entry_repo.clone(),
            leave_request_repo.clone(),
            Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            Arc::new(
                apiserver::repositories::DieselEmployeeContractRepository::new(
                    diesel_pools.clone(),
                ),
            ),
        ),
    );

    let attendance_summary_service = Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
            roster_repo: roster_repo.clone(),
            policy_repo: policy_repo.clone(),
            workspace_repo: Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
                diesel_pools.clone(),
            )),
            employee_repo: Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            holiday_repo: Arc::new(apiserver::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            )),
            leave_request_repo: Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            daily_summary_repo: Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
        },
    ));

    let raw_log_repo: Arc<
        dyn apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository,
    > = Arc::new(
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone()),
    );
    let error_repo: Arc<
        dyn apiserver::repositories::attendance_log_error_repository::AttendanceLogErrorRepository,
    > = Arc::new(
        apiserver::repositories::DieselAttendanceLogErrorRepository::new(diesel_pools.clone()),
    );

    let recruitment_repo: Arc<dyn apiserver::repositories::employee::EmployeeRecruitmentRepository> = employee_repo.clone();
    let recruitment_service = Arc::new(apiserver::services::RecruitmentService::new(
        recruitment_repo,
    ));

    let app_state = Arc::new(AppState {
        employees: apiserver::EmployeeState {
            service: employee_service,
            recruitment_service: recruitment_service.clone(),
            repo: employee_repo.clone(),
            contract_repo: Arc::new(
                apiserver::repositories::DieselEmployeeContractRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            employment_phase_repo: Arc::new(
                apiserver::repositories::DieselEmploymentPhaseRepository::new(diesel_pools.clone()),
            ),
        },
        attendance: apiserver::AttendanceState {
            time_tracking_service,
            summary_service: attendance_summary_service,
            log_service: Arc::new(
                AttendanceLogService::new(
                    pairing_service.clone(),
                    apiserver::services::attendance_log_service::AttendanceLogServiceConfig {
                        buffer_capacity: 100,
                        batch_size: 10,
                        flush_interval_ms: 1000,
                    },
                    apiserver::services::attendance_log_service::AttendanceLogServiceDeps {
                        employee_repo: employee_repo.clone(),
                        raw_log_repo: raw_log_repo.clone(),
                        staging_repo: staging_repo.clone(),
                        error_repo: error_repo.clone(),
                    },
                )
                .0,
            ),
            pairing_service,
            adjustment_service,
            import_service,
            time_entry_repo: time_entry_repo.clone(),
            staging_repo: staging_repo.clone(),
            raw_log_repo: raw_log_repo.clone(),
            import_repo: import_repo.clone(),
            error_repo: error_repo.clone(),
            policy_repo: policy_repo.clone(),
            attendance_daily_summary_repo: Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
        },
        shifts: apiserver::ShiftState {
            service: shift_service,
            swap_service: shift_swap_service,
            roster_repo: roster_repo.clone(),
            shift_repo: shift_repo.clone(),
            swap_repo: shift_swap_repo.clone(),
            assignment_rule_repo: assignment_rule_repo.clone(),
            working_pattern_repo: working_pattern_repo.clone(),
        },
        leaves: apiserver::LeaveState {
            service: leave_service,
            holiday_leave_service,
            accrual_service: leave_accrual_service,
            request_repo: leave_request_repo.clone(),
            policy_repo: leave_policy_repo.clone(),
            holiday_repo: holiday_repo.clone(),
            balance_repo: leave_balance_repo.clone(),
        },
        payroll: apiserver::PayrollState {
            service: payroll_service,
            repo: payroll_repo.clone(),
            rule_repo: payroll_rule_repo.clone(),
            department_repo: department_repo.clone(),
            feature_flag_repo: feature_flag_repo.clone(),
            overtime_policy_repo: Arc::new(
                apiserver::repositories::DieselOvertimePolicyRepository::new(diesel_pools.clone()),
            ),
            salary_base_repo: std::sync::Arc::new(
                apiserver::repositories::DieselSalaryBaseRepository::new(diesel_pools.clone()),
            ),
            overtime_request_repo: overtime_request_repo.clone(),
            overtime_request_service: overtime_request_service.clone(),
        },
        reports: apiserver::ReportState {
            service: report_service,
            coordinator: shared_redis_pool.clone().map(|redis| {
                let queue_repo: Arc<
                    dyn apiserver::repositories::queue_repository::QueueRepository,
                > = Arc::new(
                    apiserver::repositories::queue_repository::RedisQueueRepository::new(redis),
                );
                Arc::new(apiserver::services::ReportCoordinator::new(
                    queue_repo,
                    employee_repo.clone(),
                    workspace_repo.clone(),
                ))
            }),
            stats_repo: Arc::new(
                apiserver::repositories::DieselWorkspaceStatsRepository::new(diesel_pools.clone()),
            ),
        },
        core: apiserver::CoreState {
            auth_service,
            workspace_service,
            workspace_repo: workspace_repo.clone(),
            audit_repo: audit_repo.clone(),
            user_repo,
            session_repo,
            jwt_secret: "test_secret_tower".to_string(),
            workspace_config_repo: workspace_config_repo.clone(),
            workspace_config_service: workspace_config_service.clone(),
            approval_workflow_repo: approval_workflow_repo.clone(),
            approval_workflow_service: approval_workflow_service.clone(),
        },
        agents: apiserver::AgentState {
            manager: agent_manager,
            service: agent_service.clone(),
            repo: agent_repo.clone(),
        },
        infra: apiserver::InfraState {
            diesel: diesel_pools.clone(),
            redis: shared_redis_pool.clone(),
            cache: shared_redis_pool.clone().map(|r| {
                Arc::new(apiserver::cache::RedisCache::new(r))
                    as Arc<dyn apiserver::repositories::CacheRepository>
            }),
            roster_cache: None,
            queue: shared_redis_pool.clone().map(|r| {
                Arc::new(apiserver::repositories::queue_repository::RedisQueueRepository::new(r))
                    as Arc<dyn apiserver::repositories::queue_repository::QueueRepository>
            }),
            tenant_manager,
            tenant_provisioner: std::sync::Arc::new(DummyTenantProvisioner),
            crypto_service: std::env::var("ENCRYPTION_KEYS_JSON")
                .ok()
                .and_then(|keys_json| {
                    apiserver::crypto::CryptoService::from_json(&keys_json)
                        .ok()
                        .map(std::sync::Arc::new)
                }),
            system_db_cluster_repo: Arc::new(apiserver::repositories::diesel_system_db_cluster_repository::DieselSystemDbClusterRepository::new(diesel_pools.clone())),
        },
    });

    let app = apiserver::create_app(app_state.clone());
    (test_db, factory, app_state, app)
}
