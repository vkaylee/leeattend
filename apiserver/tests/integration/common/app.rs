//! Shared application spawning utilities for integration tests.
//!
//! This module provides various `spawn_app` variants to avoid duplication
//! across test files.

use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::models::MemberRole;
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
    
    let employee_repo = Arc::new(apiserver::repositories::SqlxEmployeeRepository::new(pool.clone()));
    let employee_service = Arc::new(apiserver::services::EmployeeService::new(employee_repo.clone(), pool.clone()));
    let time_entry_repo = Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(pool.clone()));
    let time_tracking_service = Arc::new(apiserver::services::TimeTrackingService::new(time_entry_repo.clone(), pool.clone()));
    let attendance_summary_service = Arc::new(apiserver::services::AttendanceSummaryService::new(time_entry_repo.clone(), pool.clone()));
    let report_service = Arc::new(apiserver::services::ReportService::new(time_entry_repo.clone(), pool.clone()));
    let pairing_service = Arc::new(apiserver::services::pairing_service::PairingService::new(time_entry_repo.clone(), pool.clone()));
    let adjustment_service = Arc::new(apiserver::services::attendance_adjustment_service::AttendanceAdjustmentService::new(time_entry_repo.clone(), pool.clone()));
    let import_service = Arc::new(apiserver::services::attendance_import_service::AttendanceImportService::new(time_entry_repo.clone(), pool.clone()));
    let payroll_service = Arc::new(apiserver::services::payroll_service::PayrollService::new(pool.clone()));
    let auth_service = Arc::new(apiserver::services::auth_service::AuthService::new(pool.clone()));
    let shift_service = Arc::new(apiserver::services::shift_service::ShiftService::new(pool.clone()));
    let workspace_service = Arc::new(apiserver::services::workspace_service::WorkspaceService::new(pool.clone()));
    let leave_service = Arc::new(apiserver::services::leave_service::LeaveService::new(pool.clone()));
    let shift_swap_service = Arc::new(apiserver::services::shift_swap_service::ShiftSwapService::new(pool.clone()));
    let holiday_leave_service = Arc::new(apiserver::services::holiday_leave_service::HolidayLeaveService::new(pool.clone()));
    let leave_accrual_service = Arc::new(apiserver::services::leave_accrual_service::LeaveAccrualService::new(pool.clone()));

    let state = Arc::new(AppState {
        agent_manager,
        db: apiserver::db_pool::DbPools::for_test(pool.clone()),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret_tower".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), pairing_service.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
        roster_cache: None,
        audit_repo: Arc::new(apiserver::repositories::SqlxAuditLogRepository { pool: pool.clone() }),
        employee_repo,
        employee_service,
        time_tracking_service,
        attendance_summary_service,
        report_service,
        pairing_service,
        adjustment_service,
        import_service,
        payroll_service,
        auth_service,
        shift_service,
        workspace_service,
        leave_service,
        shift_swap_service,
        holiday_leave_service,
        leave_accrual_service,
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
    let employee_repo = Arc::new(apiserver::repositories::SqlxEmployeeRepository::new(pool.clone()));
    let employee_service = Arc::new(apiserver::services::EmployeeService::new(employee_repo.clone(), pool.clone()));
    let time_entry_repo = Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(pool.clone()));
    let time_tracking_service = Arc::new(apiserver::services::TimeTrackingService::new(time_entry_repo.clone(), pool.clone()));
    let attendance_summary_service = Arc::new(apiserver::services::AttendanceSummaryService::new(time_entry_repo.clone(), pool.clone()));
    let report_service = Arc::new(apiserver::services::ReportService::new(time_entry_repo.clone(), pool.clone()));
    let pairing_service = Arc::new(apiserver::services::pairing_service::PairingService::new(time_entry_repo.clone(), pool.clone()));
    let adjustment_service = Arc::new(apiserver::services::attendance_adjustment_service::AttendanceAdjustmentService::new(time_entry_repo.clone(), pool.clone()));
    let import_service = Arc::new(apiserver::services::attendance_import_service::AttendanceImportService::new(time_entry_repo.clone(), pool.clone()));
    let payroll_service = Arc::new(apiserver::services::payroll_service::PayrollService::new(pool.clone()));
    let auth_service = Arc::new(apiserver::services::auth_service::AuthService::new(pool.clone()));
    let shift_service = Arc::new(apiserver::services::shift_service::ShiftService::new(pool.clone()));
    let workspace_service = Arc::new(apiserver::services::workspace_service::WorkspaceService::new(pool.clone()));
    let leave_service = Arc::new(apiserver::services::leave_service::LeaveService::new(pool.clone()));
    let shift_swap_service = Arc::new(apiserver::services::shift_swap_service::ShiftSwapService::new(pool.clone()));
    let holiday_leave_service = Arc::new(apiserver::services::holiday_leave_service::HolidayLeaveService::new(pool.clone()));
    let leave_accrual_service = Arc::new(apiserver::services::leave_accrual_service::LeaveAccrualService::new(pool.clone()));

    let state = Arc::new(AppState {
        agent_manager,
        db: apiserver::db_pool::DbPools::for_test(db_pool.clone()),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret_tower".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(db_pool.clone(), pairing_service.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
        roster_cache: None,
        audit_repo: Arc::new(apiserver::repositories::SqlxAuditLogRepository { pool: db_pool.clone() }),
        employee_repo,
        employee_service,
        time_tracking_service,
        attendance_summary_service,
        report_service,
        pairing_service,
        adjustment_service,
        import_service,
        payroll_service,
        auth_service,
        shift_service,
        workspace_service,
        leave_service,
        shift_swap_service,
        holiday_leave_service,
        leave_accrual_service,
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
        .add_member(workspace_id, member_id, MemberRole::Member)
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
    
    let employee_repo = Arc::new(apiserver::repositories::SqlxEmployeeRepository::new(pool.clone()));
    let employee_service = Arc::new(apiserver::services::EmployeeService::new(employee_repo.clone(), pool.clone()));
    let time_entry_repo = Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(pool.clone()));
    let time_tracking_service = Arc::new(apiserver::services::TimeTrackingService::new(time_entry_repo.clone(), pool.clone()));
    let attendance_summary_service = Arc::new(apiserver::services::AttendanceSummaryService::new(time_entry_repo.clone(), pool.clone()));
    let report_service = Arc::new(apiserver::services::ReportService::new(time_entry_repo.clone(), pool.clone()));
    let pairing_service = Arc::new(apiserver::services::pairing_service::PairingService::new(time_entry_repo.clone(), pool.clone()));
    let adjustment_service = Arc::new(apiserver::services::attendance_adjustment_service::AttendanceAdjustmentService::new(time_entry_repo.clone(), pool.clone()));
    let import_service = Arc::new(apiserver::services::attendance_import_service::AttendanceImportService::new(time_entry_repo.clone(), pool.clone()));
    let payroll_service = Arc::new(apiserver::services::payroll_service::PayrollService::new(pool.clone()));
    let auth_service = Arc::new(apiserver::services::auth_service::AuthService::new(pool.clone()));
    let shift_service = Arc::new(apiserver::services::shift_service::ShiftService::new(pool.clone()));
    let workspace_service = Arc::new(apiserver::services::workspace_service::WorkspaceService::new(pool.clone()));
    let leave_service = Arc::new(apiserver::services::leave_service::LeaveService::new(pool.clone()));
    let shift_swap_service = Arc::new(apiserver::services::shift_swap_service::ShiftSwapService::new(pool.clone()));
    let holiday_leave_service = Arc::new(apiserver::services::holiday_leave_service::HolidayLeaveService::new(pool.clone()));
    let leave_accrual_service = Arc::new(apiserver::services::leave_accrual_service::LeaveAccrualService::new(pool.clone()));

    let app_state = Arc::new(AppState {
        agent_manager,
        db: apiserver::db_pool::DbPools::for_test(pool.clone()),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret_tower".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), pairing_service.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
        roster_cache: None,
        audit_repo: Arc::new(apiserver::repositories::SqlxAuditLogRepository { pool: pool.clone() }),
        employee_repo,
        employee_service,
        time_tracking_service,
        attendance_summary_service,
        report_service,
        pairing_service,
        adjustment_service,
        import_service,
        payroll_service,
        auth_service,
        shift_service,
        workspace_service,
        leave_service,
        shift_swap_service,
        holiday_leave_service,
        leave_accrual_service,
    });

    let app = apiserver::create_app(app_state.clone());
    (test_db, factory, app_state, app)
}
