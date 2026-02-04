mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::models::Employee;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::StatusCode;
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_app() -> (TestDatabase, TestFactory, axum::Router) {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("apiserver=debug,sqlx=info")
        .with_test_writer()
        .try_init();

    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository::new(
        pool.clone(),
    ));
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_manager = AgentManager::new();
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    // Setup test storage

    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
        roster_cache: None,
    });

    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_employee_crud_lifecycle() {
    let (_test_db, factory, app) = setup_app().await;

    // 1. Setup: User, Workspace
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // 2. Create Employee
    let employee_req = serde_json::json!({
        "full_name": "John Doe",
        "employee_id_number": "EMP001",
        "hire_date": "2026-01-01",
        "is_active": true
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/employees", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&employee_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let employee: Employee = serde_json::from_slice(&body).unwrap();
    assert_eq!(employee.full_name, "John Doe");

    // 3. Get Employee
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{}/employees/{}",
                    ws_id, employee.id
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let fetched: Employee = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched.id, employee.id);

    // 4. Update Employee
    let update_req = serde_json::json!({
        "full_name": "Johnny Doe"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!(
                    "/api/workspaces/{}/employees/{}",
                    ws_id, employee.id
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&update_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: Employee = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated.full_name, "Johnny Doe");

    // 5. Delete Employee
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/workspaces/{}/employees/{}",
                    ws_id, employee.id
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 6. Verify Deletion
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{}/employees/{}",
                    ws_id, employee.id
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_employee_invalid_user() {
    let (_test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let employee_req = serde_json::json!({
        "user_id": uuid::Uuid::new_v4(), // Non-existent user
        "full_name": "John Doe",
        "employee_id_number": "EMP002",
        "hire_date": "2026-01-01"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/employees", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&employee_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_employees_list() {
    let (_test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    factory.employee().with_workspace(ws_id).create().await;

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/employees", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_employees_search() {
    let (_test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create 2 employees
    factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Alice Wonder")
        .with_employee_id_number("ALICE001")
        .create()
        .await;

    factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Bob Builder")
        .with_employee_id_number("BOB999")
        .create()
        .await;

    // Search by name
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/employees?q=Alice", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let res: apiserver::models::PaginatedResponse<Employee> = serde_json::from_slice(&body).unwrap();
    let employees = res.data;
    assert_eq!(employees.len(), 1);
    assert_eq!(employees[0].full_name, "Alice Wonder");

    // Search by ID
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/employees?q=999", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let res: apiserver::models::PaginatedResponse<Employee> = serde_json::from_slice(&body).unwrap();
    let employees = res.data;
    assert_eq!(employees.len(), 1);
    assert_eq!(employees[0].full_name, "Bob Builder");
}
#[tokio::test]
async fn test_update_employee_complex() {
    let (_test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let update_req = serde_json::json!({
        "termination_date": "2026-12-31",
        "is_active": false
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!("/api/workspaces/{}/employees/{}", ws_id, emp_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&update_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_delete_non_existent_employee() {
    let (_test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/workspaces/{}/employees/{}",
                    ws_id,
                    uuid::Uuid::new_v4()
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_employee_dept_history_not_found() {
    let (_test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{}/employees/{}/departments",
                    ws_id,
                    uuid::Uuid::new_v4()
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_transfer_non_existent_employee() {
    let (_test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let transfer_req = serde_json::json!({
        "department_id": uuid::Uuid::new_v4(),
        "effective_date": "2026-01-01"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{}/employees/{}/departments",
                    ws_id,
                    uuid::Uuid::new_v4()
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(transfer_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}