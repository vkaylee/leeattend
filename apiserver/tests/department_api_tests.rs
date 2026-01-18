mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::models::{Department, EmployeeDepartmentHistory};
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

    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret".to_string(),
    });

    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_department_management() {
    let (_test_db, factory, app) = setup_app().await;

    // 1. Setup: User, Workspace, Employee
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // 2. Create Department
    let dept_req = serde_json::json!({
        "name": "Engineering"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/departments", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&dept_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let dept: Department = serde_json::from_slice(&body).unwrap();
    assert_eq!(dept.name, "Engineering");

    // 3. List Departments
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/departments", ws_id))
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
    let depts: Vec<Department> = serde_json::from_slice(&body).unwrap();
    assert!(!depts.is_empty());

    // 4. Transfer Employee to Department
    let transfer_req = serde_json::json!({
        "department_id": dept.id,
        "effective_date": "2026-01-10"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{}/employees/{}/departments",
                    ws_id, emp_id
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&transfer_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 5. Get Employee Department History
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{}/employees/{}/departments",
                    ws_id, emp_id
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
    let history: Vec<EmployeeDepartmentHistory> = serde_json::from_slice(&body).unwrap();
    assert!(!history.is_empty());
    assert_eq!(history[0].department_id, dept.id);
}

#[tokio::test]
async fn test_department_working_pattern_api() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create a pattern first
    let pattern = apiserver::repositories::working_pattern_repository::create_pattern(
        &factory.pool,
        ws_id,
        "Dept Pattern",
        None,
    )
    .await
    .unwrap();

    // 1. Create Department with Pattern
    let dept_req = serde_json::json!({
        "name": "HR",
        "default_working_pattern_id": pattern.id
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/departments", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&dept_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let dept: Department = serde_json::from_slice(&body).unwrap();
    assert_eq!(dept.default_working_pattern_id, Some(pattern.id));

    // 2. Update Department Pattern
    let update_req = serde_json::json!({
        "default_working_pattern_id": null
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri(format!("/api/workspaces/{}/departments/{}", ws_id, dept.id))
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
    let updated_dept: Department = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated_dept.default_working_pattern_id, None);
}
