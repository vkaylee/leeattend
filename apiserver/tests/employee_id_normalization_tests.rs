mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::models::{CreateEmployeeRequest, Employee, UpdateEmployeeRequest};
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::StatusCode;
use axum::body::Body;
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_app() -> (TestDatabase, TestFactory, axum::Router) {
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

    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret_123".to_string(), // Matches AuthHelper default secret
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
    });

    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_create_employee_id_normalization() {
    let (_test_db, factory, app) = setup_app().await;

    // 1. Setup workspace & owner
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let token = factory.auth().create_token(user_id);

    // 2. Create employee with leading zeros "00123"
    let req_body = CreateEmployeeRequest {
        user_id: None,
        full_name: "Test Employee".to_string(),
        employee_id_number: "00123".to_string(),
        hire_date: chrono::Utc::now().date_naive(),
        tags: None,
    };

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/employees", ws_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let created_emp: Employee = serde_json::from_slice(&body).unwrap();

    // 3. Verify response has normalized ID
    assert_eq!(created_emp.employee_id_number, "123");

    // 4. Verify in DB
    let pool = &factory.pool;
    let db_emp = sqlx::query!("SELECT employee_id_number FROM employees WHERE id = $1", created_emp.id)
        .fetch_one(pool)
        .await
        .unwrap();
    
    assert_eq!(db_emp.employee_id_number, "123");
}

#[tokio::test]
async fn test_update_employee_id_normalization() {
    let (_test_db, factory, app) = setup_app().await;

    // 1. Setup workspace & owner
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let token = factory.auth().create_token(user_id);

    // 2. Create initial employee "100"
    let emp_id = factory.employee()
        .with_workspace(ws_id)
        .with_employee_id_number("100")
        .with_full_name("Original Name")
        .create()
        .await;

    // 3. Update employee ID to "00200"
    let req_body = UpdateEmployeeRequest {
        user_id: None,
        full_name: Some("Updated Name".to_string()),
        employee_id_number: Some("00200".to_string()),
        hire_date: None,
        termination_date: None,
        tags: None,
        is_active: None,
        max_concurrent_workers: None,
    };

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!("/api/workspaces/{}/employees/{}", ws_id, emp_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let updated_emp: Employee = serde_json::from_slice(&body).unwrap();

    // 4. Verify response has normalized ID
    assert_eq!(updated_emp.employee_id_number, "200");

    // 5. Verify in DB
    let pool = &factory.pool;
    let db_emp = sqlx::query!("SELECT employee_id_number FROM employees WHERE id = $1", emp_id)
        .fetch_one(pool)
        .await
        .unwrap();
    
    assert_eq!(db_emp.employee_id_number, "200");
}

#[tokio::test]
async fn test_employee_id_uniqueness_on_normalized_values() {
    let (_test_db, factory, app) = setup_app().await;

    // 1. Setup
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let token = factory.auth().create_token(user_id);

    // 2. Create first employee "555"
    let _ = factory.employee()
        .with_workspace(ws_id)
        .with_employee_id_number("555")
        .with_full_name("First Employee")
        .create()
        .await;

    // 3. Try to create second employee with "00555"
    let req_body = CreateEmployeeRequest {
        user_id: None,
        full_name: "Duplicate Employee".to_string(),
        employee_id_number: "00555".to_string(),
        hire_date: chrono::Utc::now().date_naive(),
        tags: None,
    };

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/employees", ws_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // 4. Expect failure due to uniqueness constraint (500 Internal Server Error in current impl)
    assert_ne!(response.status(), StatusCode::CREATED);
}
