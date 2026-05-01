//! Shared application spawning utilities for integration tests.
//!
//! This module provides various `spawn_app` variants to avoid duplication
//! across test files.

use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use std::sync::Arc;
use uuid::Uuid;

use super::db::TestDatabase;
use super::factory::TestFactory;

/// Basic app spawn - returns address (URL) and TestDatabase
#[allow(dead_code)]
pub async fn spawn_app() -> (String, TestDatabase) {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();

    let agent_manager = AgentManager::new(None);
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let employee_repo = Arc::new(apiserver::repositories::SqlxEmployeeRepository::new(
        pool.clone(),
    ));
    let employee_service = Arc::new(apiserver::services::EmployeeService::new(
        employee_repo.clone(),
        pool.clone(),
        None,
    ));
    let time_entry_repo = Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(
        pool.clone(),
    ));
    let roster_repo: Arc<dyn apiserver::repositories::roster::RosterRepository> = Arc::new(
        apiserver::repositories::roster::SqlxRosterRepository::new(pool.clone()),
    );
    let time_tracking_service = Arc::new(apiserver::services::TimeTrackingService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let attendance_summary_service = Arc::new(apiserver::services::AttendanceSummaryService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let report_service = Arc::new(apiserver::services::ReportService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let pairing_service = Arc::new(apiserver::services::pairing_service::PairingService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let adj_repo: Arc<dyn apiserver::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository> = Arc::new(apiserver::repositories::attendance_adjustment_repository::SqlxAttendanceAdjustmentRepository::new(pool.clone()));
    let import_repo: Arc<dyn apiserver::repositories::import_job_repository::ImportJobRepository> =
        Arc::new(
            apiserver::repositories::import_job_repository::SqlxImportJobRepository::new(
                pool.clone(),
            ),
        );
    let leave_balance_repo: Arc<
        dyn apiserver::repositories::leave_balance_repository::LeaveBalanceRepository,
    > = Arc::new(
        apiserver::repositories::leave_balance_repository::SqlxLeaveBalanceRepository::new(
            pool.clone(),
        ),
    );
    let leave_request_repo: Arc<
        dyn apiserver::repositories::leave_request_repository::LeaveRequestRepository,
    > = Arc::new(
        apiserver::repositories::leave_request_repository::SqlxLeaveRequestRepository::new(
            pool.clone(),
        ),
    );
    let shift_swap_repo: Arc<
        dyn apiserver::repositories::shift_swap_repository::ShiftSwapRepository,
    > = Arc::new(
        apiserver::repositories::shift_swap_repository::SqlxShiftSwapRepository::new(pool.clone()),
    );
    let adjustment_service = Arc::new(
        apiserver::services::attendance_adjustment_service::AttendanceAdjustmentService::new(
            adj_repo.clone(),
            time_entry_repo.clone(),
            pool.clone(),
        ),
    );
    let import_service = Arc::new(
        apiserver::services::attendance_import_service::AttendanceImportService::new(
            import_repo.clone(),
            adj_repo.clone(),
            employee_repo.clone(),
            time_entry_repo.clone(),
            workspace_repo.clone(),
            pool.clone(),
        ),
    );
    let payroll_repo: Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> =
        Arc::new(
            apiserver::repositories::payroll_repository::SqlxPayrollRepository::new(pool.clone()),
        );
    let holiday_repo: Arc<dyn apiserver::repositories::holiday_repository::HolidayRepositoryTrait> =
        Arc::new(
            apiserver::repositories::holiday_repository::SqlxHolidayRepository::new(pool.clone()),
        );
    let department_repo: Arc<
        dyn apiserver::repositories::department_repository::DepartmentRepository,
    > = Arc::new(
        apiserver::repositories::department_repository::SqlxDepartmentRepository::new(pool.clone()),
    );
    let payroll_service = Arc::new(apiserver::services::payroll_service::PayrollService::new(
        apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: payroll_repo.clone(),
            department_repo: department_repo.clone(),
            time_entry_repo: time_entry_repo.clone(),
            workspace_repo: workspace_repo.clone(),
            holiday_repo: holiday_repo.clone(),
            roster_repo: roster_repo.clone(),
            pool: pool.clone(),
        },
    ));
    let auth_service = Arc::new(apiserver::services::auth_service::AuthService::new(
        pool.clone(),
    ));
    let shift_service = Arc::new(apiserver::services::shift_service::ShiftService::new(
        pool.clone(),
    ));
    let workspace_service =
        Arc::new(apiserver::services::workspace_service::WorkspaceService::new(pool.clone()));
    let leave_service = Arc::new(apiserver::services::leave_service::LeaveService::new(
        leave_balance_repo.clone(),
        leave_request_repo.clone(),
        employee_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let shift_swap_service = Arc::new(
        apiserver::services::shift_swap_service::ShiftSwapService::new(
            shift_swap_repo.clone(),
            roster_repo.clone(),
            pool.clone(),
        ),
    );
    let leave_policy_repo: Arc<
        dyn apiserver::repositories::leave_policy_repository::LeavePolicyRepository,
    > = Arc::new(
        apiserver::repositories::leave_policy_repository::SqlxLeavePolicyRepository::new(
            pool.clone(),
        ),
    );
    let holiday_leave_service = Arc::new(
        apiserver::services::holiday_leave_service::HolidayLeaveService::new(
            employee_repo.clone(),
            leave_balance_repo.clone(),
            leave_request_repo.clone(),
            roster_repo.clone(),
            pool.clone(),
        ),
    );
    let leave_accrual_service = Arc::new(
        apiserver::services::leave_accrual_service::LeaveAccrualService::new(
            employee_repo.clone(),
            department_repo.clone(),
            leave_policy_repo.clone(),
            Arc::new(apiserver::repositories::leave_accrual_log_repository::SqlxLeaveAccrualLogRepository::new(pool.clone())),
            pool.clone(),
        ),
    );

    let raw_log_repo: Arc<
        dyn apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository,
    > = Arc::new(
        apiserver::repositories::raw_attendance_log_repository::SqlxRawAttendanceLogRepository::new(
            pool.clone(),
        ),
    );
    let staging_repo: Arc<
        dyn apiserver::repositories::staging_log_repository::StagingLogRepository,
    > = Arc::new(
        apiserver::repositories::staging_log_repository::SqlxStagingLogRepository::new(
            pool.clone(),
        ),
    );
    let error_repo: Arc<dyn apiserver::repositories::attendance_log_error_repository::AttendanceLogErrorRepository> = Arc::new(apiserver::repositories::attendance_log_error_repository::SqlxAttendanceLogErrorRepository::new(pool.clone()));

    let state = Arc::new(AppState {
        employees: apiserver::EmployeeState {
            service: employee_service,
            repo: employee_repo.clone(),
        },
        attendance: apiserver::AttendanceState {
            time_tracking_service,
            summary_service: attendance_summary_service,
            log_service: Arc::new(
                AttendanceLogService::new(
                    pool.clone(),
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
                        error_repo,
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
        },
        shifts: apiserver::ShiftState {
            service: shift_service,
            swap_service: shift_swap_service,
            roster_repo: roster_repo.clone(),
            swap_repo: shift_swap_repo.clone(),
        },
        leaves: apiserver::LeaveState {
            service: leave_service,
            holiday_leave_service,
            accrual_service: leave_accrual_service,
            request_repo: leave_request_repo.clone(),
            policy_repo: leave_policy_repo.clone(),
            holiday_repo: holiday_repo.clone(),
        },
        payroll: apiserver::PayrollState {
            service: payroll_service,
            repo: payroll_repo.clone(),
            department_repo: department_repo.clone(),
        },
        reports: apiserver::ReportState {
            service: report_service,
            coordinator: if let Ok(url) = std::env::var("REDIS_URL") {
                if let Ok(redis) = apiserver::queue::RedisPool::new(&url).await {
                    Some(Arc::new(apiserver::services::ReportCoordinator::new(redis, pool.clone())))
                } else { None }
            } else { None },
            stats_repo: Arc::new(apiserver::repositories::workspace_stats_repository::SqlxWorkspaceStatsRepository::new(pool.clone())),
        },
        core: apiserver::CoreState {
            auth_service,
            workspace_service,
            workspace_repo: workspace_repo.clone(),
            audit_repo: Arc::new(
                apiserver::repositories::audit_log_repository::SqlxAuditLogRepository::new(
                    pool.clone(),
                ),
            ),
            user_repo: Arc::new(
                apiserver::repositories::user_repository::SqlxUserRepository::new(pool.clone()),
            ),
            jwt_secret: "test_secret_tower".to_string(),
        },
        agents: apiserver::AgentState {
            manager: agent_manager,
            service: agent_service,
            repo: agent_repo.clone(),
        },
        infra: apiserver::InfraState {
            db: apiserver::db_pool::DbPools::for_test(pool.clone()),
            redis: if let Ok(url) = std::env::var("REDIS_URL") {
                Some(
                    apiserver::queue::RedisPool::new(&url)
                        .await
                        .expect("Failed to create Redis pool for test"),
                )
            } else {
                None
            },
            roster_cache: None,
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

/// App spawn with factory - returns address, workspace_id, user_id, factory, and TestDatabase
#[allow(dead_code)]
pub async fn spawn_app_with_factory() -> (String, Uuid, Uuid, TestFactory, TestDatabase) {
    let test_db = TestDatabase::new().await;
    let db_pool = test_db.pool();

    let factory = TestFactory::new(db_pool.clone());

    // Create a test user and workspace using factory
    let user_id = factory.user().with_email("test@example.com").create().await;
    let workspace_id = factory
        .workspace()
        .with_name("Test Workspace")
        .create_with_owner(user_id)
        .await;

    let agent_manager = AgentManager::new(None);
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository {
        pool: db_pool.clone(),
    });
    let workspace_repo = Arc::new(apiserver::repositories::SqlxWorkspaceRepository {
        pool: db_pool.clone(),
    });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let pool = db_pool.clone();
    let employee_repo = Arc::new(apiserver::repositories::SqlxEmployeeRepository::new(
        pool.clone(),
    ));
    let employee_service = Arc::new(apiserver::services::EmployeeService::new(
        employee_repo.clone(),
        pool.clone(),
        None,
    ));
    let time_entry_repo = Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(
        pool.clone(),
    ));
    let roster_repo: Arc<dyn apiserver::repositories::roster::RosterRepository> = Arc::new(
        apiserver::repositories::roster::SqlxRosterRepository::new(pool.clone()),
    );
    let time_tracking_service = Arc::new(apiserver::services::TimeTrackingService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let attendance_summary_service = Arc::new(apiserver::services::AttendanceSummaryService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let report_service = Arc::new(apiserver::services::ReportService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let pairing_service = Arc::new(apiserver::services::pairing_service::PairingService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let adj_repo: Arc<dyn apiserver::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository> = Arc::new(apiserver::repositories::attendance_adjustment_repository::SqlxAttendanceAdjustmentRepository::new(pool.clone()));
    let import_repo: Arc<dyn apiserver::repositories::import_job_repository::ImportJobRepository> =
        Arc::new(
            apiserver::repositories::import_job_repository::SqlxImportJobRepository::new(
                pool.clone(),
            ),
        );
    let leave_balance_repo: Arc<
        dyn apiserver::repositories::leave_balance_repository::LeaveBalanceRepository,
    > = Arc::new(
        apiserver::repositories::leave_balance_repository::SqlxLeaveBalanceRepository::new(
            pool.clone(),
        ),
    );
    let leave_request_repo: Arc<
        dyn apiserver::repositories::leave_request_repository::LeaveRequestRepository,
    > = Arc::new(
        apiserver::repositories::leave_request_repository::SqlxLeaveRequestRepository::new(
            pool.clone(),
        ),
    );
    let shift_swap_repo: Arc<
        dyn apiserver::repositories::shift_swap_repository::ShiftSwapRepository,
    > = Arc::new(
        apiserver::repositories::shift_swap_repository::SqlxShiftSwapRepository::new(pool.clone()),
    );
    let adjustment_service = Arc::new(
        apiserver::services::attendance_adjustment_service::AttendanceAdjustmentService::new(
            adj_repo.clone(),
            time_entry_repo.clone(),
            pool.clone(),
        ),
    );
    let import_service = Arc::new(
        apiserver::services::attendance_import_service::AttendanceImportService::new(
            import_repo.clone(),
            adj_repo.clone(),
            employee_repo.clone(),
            time_entry_repo.clone(),
            workspace_repo.clone(),
            pool.clone(),
        ),
    );
    let payroll_repo: Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> =
        Arc::new(
            apiserver::repositories::payroll_repository::SqlxPayrollRepository::new(pool.clone()),
        );
    let holiday_repo: Arc<dyn apiserver::repositories::holiday_repository::HolidayRepositoryTrait> =
        Arc::new(
            apiserver::repositories::holiday_repository::SqlxHolidayRepository::new(pool.clone()),
        );
    let department_repo: Arc<
        dyn apiserver::repositories::department_repository::DepartmentRepository,
    > = Arc::new(
        apiserver::repositories::department_repository::SqlxDepartmentRepository::new(pool.clone()),
    );
    let payroll_service = Arc::new(apiserver::services::payroll_service::PayrollService::new(
        apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: payroll_repo.clone(),
            department_repo: department_repo.clone(),
            time_entry_repo: time_entry_repo.clone(),
            workspace_repo: workspace_repo.clone(),
            holiday_repo: holiday_repo.clone(),
            roster_repo: roster_repo.clone(),
            pool: pool.clone(),
        },
    ));
    let auth_service = Arc::new(apiserver::services::auth_service::AuthService::new(
        pool.clone(),
    ));
    let shift_service = Arc::new(apiserver::services::shift_service::ShiftService::new(
        pool.clone(),
    ));
    let workspace_service =
        Arc::new(apiserver::services::workspace_service::WorkspaceService::new(pool.clone()));
    let leave_service = Arc::new(apiserver::services::leave_service::LeaveService::new(
        leave_balance_repo.clone(),
        leave_request_repo.clone(),
        employee_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let shift_swap_service = Arc::new(
        apiserver::services::shift_swap_service::ShiftSwapService::new(
            shift_swap_repo.clone(),
            roster_repo.clone(),
            pool.clone(),
        ),
    );
    let leave_policy_repo: Arc<
        dyn apiserver::repositories::leave_policy_repository::LeavePolicyRepository,
    > = Arc::new(
        apiserver::repositories::leave_policy_repository::SqlxLeavePolicyRepository::new(
            pool.clone(),
        ),
    );
    let holiday_leave_service = Arc::new(
        apiserver::services::holiday_leave_service::HolidayLeaveService::new(
            employee_repo.clone(),
            leave_balance_repo.clone(),
            leave_request_repo.clone(),
            roster_repo.clone(),
            pool.clone(),
        ),
    );
    let leave_accrual_service = Arc::new(
        apiserver::services::leave_accrual_service::LeaveAccrualService::new(
            employee_repo.clone(),
            department_repo.clone(),
            leave_policy_repo.clone(),
            Arc::new(apiserver::repositories::leave_accrual_log_repository::SqlxLeaveAccrualLogRepository::new(pool.clone())),
            pool.clone(),
        ),
    );

    let raw_log_repo: Arc<
        dyn apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository,
    > = Arc::new(
        apiserver::repositories::raw_attendance_log_repository::SqlxRawAttendanceLogRepository::new(
            db_pool.clone(),
        ),
    );
    let staging_repo: Arc<
        dyn apiserver::repositories::staging_log_repository::StagingLogRepository,
    > = Arc::new(
        apiserver::repositories::staging_log_repository::SqlxStagingLogRepository::new(
            db_pool.clone(),
        ),
    );
    let error_repo: Arc<dyn apiserver::repositories::attendance_log_error_repository::AttendanceLogErrorRepository> = Arc::new(apiserver::repositories::attendance_log_error_repository::SqlxAttendanceLogErrorRepository::new(db_pool.clone()));

    let state = Arc::new(AppState {
        employees: apiserver::EmployeeState {
            service: employee_service,
            repo: employee_repo.clone(),
        },
        attendance: apiserver::AttendanceState {
            time_tracking_service,
            summary_service: attendance_summary_service,
            log_service: Arc::new(
                AttendanceLogService::new(
                    db_pool.clone(),
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
                        error_repo,
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
        },
        shifts: apiserver::ShiftState {
            service: shift_service,
            swap_service: shift_swap_service,
            roster_repo: roster_repo.clone(),
            swap_repo: shift_swap_repo.clone(),
        },
        leaves: apiserver::LeaveState {
            service: leave_service,
            holiday_leave_service,
            accrual_service: leave_accrual_service,
            request_repo: leave_request_repo.clone(),
            policy_repo: leave_policy_repo.clone(),
            holiday_repo: holiday_repo.clone(),
        },
        payroll: apiserver::PayrollState {
            service: payroll_service,
            repo: payroll_repo.clone(),
            department_repo: department_repo.clone(),
        },
        reports: apiserver::ReportState {
            service: report_service,
            coordinator: if let Ok(url) = std::env::var("REDIS_URL") {
                if let Ok(redis) = apiserver::queue::RedisPool::new(&url).await {
                    Some(Arc::new(apiserver::services::ReportCoordinator::new(redis, db_pool.clone())))
                } else { None }
            } else { None },
            stats_repo: Arc::new(apiserver::repositories::workspace_stats_repository::SqlxWorkspaceStatsRepository::new(db_pool.clone())),
        },
        core: apiserver::CoreState {
            auth_service,
            workspace_service,
            workspace_repo: workspace_repo.clone(),
            audit_repo: Arc::new(
                apiserver::repositories::audit_log_repository::SqlxAuditLogRepository::new(
                    db_pool.clone(),
                ),
            ),
            user_repo: Arc::new(
                apiserver::repositories::user_repository::SqlxUserRepository::new(db_pool.clone()),
            ),
            jwt_secret: "test_secret_tower".to_string(),
        },
        agents: apiserver::AgentState {
            manager: agent_manager,
            service: agent_service,
            repo: agent_repo.clone(),
        },
        infra: apiserver::InfraState {
            db: apiserver::db_pool::DbPools::for_test(db_pool.clone()),
            redis: if let Ok(url) = std::env::var("REDIS_URL") {
                Some(
                    apiserver::queue::RedisPool::new(&url)
                        .await
                        .expect("Failed to create Redis pool for test factory"),
                )
            } else {
                None
            },
            roster_cache: None,
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
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository::new(
        pool.clone(),
    ));
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_manager = AgentManager::new(None);
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let employee_repo = Arc::new(apiserver::repositories::SqlxEmployeeRepository::new(
        pool.clone(),
    ));
    let employee_service = Arc::new(apiserver::services::EmployeeService::new(
        employee_repo.clone(),
        pool.clone(),
        None,
    ));
    let time_entry_repo = Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(
        pool.clone(),
    ));
    let roster_repo: Arc<dyn apiserver::repositories::roster::RosterRepository> = Arc::new(
        apiserver::repositories::roster::SqlxRosterRepository::new(pool.clone()),
    );
    let time_tracking_service = Arc::new(apiserver::services::TimeTrackingService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let attendance_summary_service = Arc::new(apiserver::services::AttendanceSummaryService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let report_service = Arc::new(apiserver::services::ReportService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let pairing_service = Arc::new(apiserver::services::pairing_service::PairingService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let adj_repo: Arc<dyn apiserver::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository> = Arc::new(apiserver::repositories::attendance_adjustment_repository::SqlxAttendanceAdjustmentRepository::new(pool.clone()));
    let import_repo: Arc<dyn apiserver::repositories::import_job_repository::ImportJobRepository> =
        Arc::new(
            apiserver::repositories::import_job_repository::SqlxImportJobRepository::new(
                pool.clone(),
            ),
        );
    let leave_balance_repo: Arc<
        dyn apiserver::repositories::leave_balance_repository::LeaveBalanceRepository,
    > = Arc::new(
        apiserver::repositories::leave_balance_repository::SqlxLeaveBalanceRepository::new(
            pool.clone(),
        ),
    );
    let leave_request_repo: Arc<
        dyn apiserver::repositories::leave_request_repository::LeaveRequestRepository,
    > = Arc::new(
        apiserver::repositories::leave_request_repository::SqlxLeaveRequestRepository::new(
            pool.clone(),
        ),
    );
    let shift_swap_repo: Arc<
        dyn apiserver::repositories::shift_swap_repository::ShiftSwapRepository,
    > = Arc::new(
        apiserver::repositories::shift_swap_repository::SqlxShiftSwapRepository::new(pool.clone()),
    );
    let adjustment_service = Arc::new(
        apiserver::services::attendance_adjustment_service::AttendanceAdjustmentService::new(
            adj_repo.clone(),
            time_entry_repo.clone(),
            pool.clone(),
        ),
    );
    let import_service = Arc::new(
        apiserver::services::attendance_import_service::AttendanceImportService::new(
            import_repo.clone(),
            adj_repo.clone(),
            employee_repo.clone(),
            time_entry_repo.clone(),
            workspace_repo.clone(),
            pool.clone(),
        ),
    );
    let payroll_repo: Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> =
        Arc::new(
            apiserver::repositories::payroll_repository::SqlxPayrollRepository::new(pool.clone()),
        );
    let holiday_repo: Arc<dyn apiserver::repositories::holiday_repository::HolidayRepositoryTrait> =
        Arc::new(
            apiserver::repositories::holiday_repository::SqlxHolidayRepository::new(pool.clone()),
        );
    let department_repo: Arc<
        dyn apiserver::repositories::department_repository::DepartmentRepository,
    > = Arc::new(
        apiserver::repositories::department_repository::SqlxDepartmentRepository::new(pool.clone()),
    );
    let payroll_service = Arc::new(apiserver::services::payroll_service::PayrollService::new(
        apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: payroll_repo.clone(),
            department_repo: department_repo.clone(),
            time_entry_repo: time_entry_repo.clone(),
            workspace_repo: workspace_repo.clone(),
            holiday_repo: holiday_repo.clone(),
            roster_repo: roster_repo.clone(),
            pool: pool.clone(),
        },
    ));
    let auth_service = Arc::new(apiserver::services::auth_service::AuthService::new(
        pool.clone(),
    ));
    let shift_service = Arc::new(apiserver::services::shift_service::ShiftService::new(
        pool.clone(),
    ));
    let workspace_service =
        Arc::new(apiserver::services::workspace_service::WorkspaceService::new(pool.clone()));
    let leave_service = Arc::new(apiserver::services::leave_service::LeaveService::new(
        leave_balance_repo.clone(),
        leave_request_repo.clone(),
        employee_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let shift_swap_service = Arc::new(
        apiserver::services::shift_swap_service::ShiftSwapService::new(
            shift_swap_repo.clone(),
            roster_repo.clone(),
            pool.clone(),
        ),
    );
    let leave_policy_repo: Arc<
        dyn apiserver::repositories::leave_policy_repository::LeavePolicyRepository,
    > = Arc::new(
        apiserver::repositories::leave_policy_repository::SqlxLeavePolicyRepository::new(
            pool.clone(),
        ),
    );
    let holiday_leave_service = Arc::new(
        apiserver::services::holiday_leave_service::HolidayLeaveService::new(
            employee_repo.clone(),
            leave_balance_repo.clone(),
            leave_request_repo.clone(),
            roster_repo.clone(),
            pool.clone(),
        ),
    );
    let leave_accrual_service = Arc::new(
        apiserver::services::leave_accrual_service::LeaveAccrualService::new(
            employee_repo.clone(),
            department_repo.clone(),
            leave_policy_repo.clone(),
            Arc::new(apiserver::repositories::leave_accrual_log_repository::SqlxLeaveAccrualLogRepository::new(pool.clone())),
            pool.clone(),
        ),
    );

    let raw_log_repo: Arc<
        dyn apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository,
    > = Arc::new(
        apiserver::repositories::raw_attendance_log_repository::SqlxRawAttendanceLogRepository::new(
            pool.clone(),
        ),
    );
    let staging_repo: Arc<
        dyn apiserver::repositories::staging_log_repository::StagingLogRepository,
    > = Arc::new(
        apiserver::repositories::staging_log_repository::SqlxStagingLogRepository::new(
            pool.clone(),
        ),
    );
    let error_repo: Arc<dyn apiserver::repositories::attendance_log_error_repository::AttendanceLogErrorRepository> = Arc::new(apiserver::repositories::attendance_log_error_repository::SqlxAttendanceLogErrorRepository::new(pool.clone()));

    let app_state = Arc::new(AppState {
        employees: apiserver::EmployeeState {
            service: employee_service,
            repo: employee_repo.clone(),
        },
        attendance: apiserver::AttendanceState {
            time_tracking_service,
            summary_service: attendance_summary_service,
            log_service: Arc::new(
                AttendanceLogService::new(
                    pool.clone(),
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
                        error_repo,
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
        },
        shifts: apiserver::ShiftState {
            service: shift_service,
            swap_service: shift_swap_service,
            roster_repo: roster_repo.clone(),
            swap_repo: shift_swap_repo.clone(),
        },
        leaves: apiserver::LeaveState {
            service: leave_service,
            holiday_leave_service,
            accrual_service: leave_accrual_service,
            request_repo: leave_request_repo.clone(),
            policy_repo: leave_policy_repo.clone(),
            holiday_repo: holiday_repo.clone(),
        },
        payroll: apiserver::PayrollState {
            service: payroll_service,
            repo: payroll_repo.clone(),
            department_repo: department_repo.clone(),
        },
        reports: apiserver::ReportState {
            service: report_service,
            coordinator: None,
            stats_repo: Arc::new(apiserver::repositories::workspace_stats_repository::SqlxWorkspaceStatsRepository::new(pool.clone())),
        },
        core: apiserver::CoreState {
            auth_service,
            workspace_service,
            workspace_repo: workspace_repo.clone(),
            audit_repo: Arc::new(
                apiserver::repositories::audit_log_repository::SqlxAuditLogRepository::new(
                    pool.clone(),
                ),
            ),
            user_repo: Arc::new(
                apiserver::repositories::user_repository::SqlxUserRepository::new(pool.clone()),
            ),
            jwt_secret: "test_secret_tower".to_string(),
        },
        agents: apiserver::AgentState {
            manager: agent_manager,
            service: agent_service.clone(),
            repo: agent_repo.clone(),
        },
        infra: apiserver::InfraState {
            db: apiserver::db_pool::DbPools::for_test(pool.clone()),
            redis: if let Ok(url) = std::env::var("REDIS_URL") {
                Some(
                    apiserver::queue::RedisPool::new(&url)
                        .await
                        .expect("Failed to create Redis pool for tower test"),
                )
            } else {
                None
            },
            roster_cache: None,
        },
    });

    let app = apiserver::create_app(app_state.clone());
    (test_db, factory, app_state, app)
}
