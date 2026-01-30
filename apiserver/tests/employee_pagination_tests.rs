mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::models::{Employee, PaginatedResponse};
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::StatusCode;
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn setup_app() -> (TestDatabase, TestFactory, axum::Router) {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository::new(pool.clone()));
    let workspace_repo = Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_manager = AgentManager::new();
    let agent_service = Arc::new(apiserver::services::AgentService::new(agent_repo.clone(), agent_manager.clone()));

    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
    });

    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_employee_pagination_logic() {
    let (test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let pool = test_db.pool();

    // Create 25 employees (sorted by name: User 00 to User 24)
    for i in 0..25 {
        let employee_id = Uuid::new_v4();
        let name = format!("User {:02}", i);
        let id_num = format!("EMP{:02}", i);
        
        sqlx::query(
            r#"INSERT INTO employees (id, workspace_id, full_name, employee_id_number, is_active, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#
        )
        .bind(employee_id)
        .bind(ws_id)
        .bind(&name)
        .bind(&id_num)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"INSERT INTO employment_periods (employee_id, hire_date) VALUES ($1, CURRENT_DATE)"#
        )
        .bind(employee_id)
        .execute(&pool)
        .await
        .unwrap();
    }

    // Request Page 1, Limit 10
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/employees?page=1&limit=10", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let res: PaginatedResponse<Employee> = serde_json::from_slice(&body).unwrap();
    assert_eq!(res.data.len(), 10);
    assert_eq!(res.total, 25);
    assert_eq!(res.data[0].full_name, "User 00");
    assert_eq!(res.data[9].full_name, "User 09");

    // Request Page 2, Limit 10
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/employees?page=2&limit=10", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let res: PaginatedResponse<Employee> = serde_json::from_slice(&body).unwrap();
    assert_eq!(res.data.len(), 10);
    assert_eq!(res.data[0].full_name, "User 10");
    assert_eq!(res.data[9].full_name, "User 19");

    // Request Page 3, Limit 10 (Last page with 5 records)
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/employees?page=3&limit=10", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let res: PaginatedResponse<Employee> = serde_json::from_slice(&body).unwrap();
    assert_eq!(res.data.len(), 5);
    assert_eq!(res.data[0].full_name, "User 20");
    assert_eq!(res.data[4].full_name, "User 24");
}