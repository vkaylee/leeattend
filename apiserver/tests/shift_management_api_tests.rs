use apiserver::services::AttendanceLogService;
use crate::common::factory::TestFactory;
use apiserver::{agent_manager::AgentManager, create_app, AppState};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;

mod common;

async fn setup_test_db() -> PgPool {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres")
}

async fn setup_test_app(pool: PgPool) -> axum::Router {
    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
    });

    create_app(state)
}

#[tokio::test]
async fn test_create_shift_api_flow() {
    let pool = setup_test_db().await;
    let factory = TestFactory::new(pool.clone());
    let app = setup_test_app(pool.clone()).await;

    // 1. Setup: Tạo user và workspace
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // 2. Test: Tạo ca mới (Trường hợp thành công)
    let shift_req = json!({
        "name": "Test Shift API",
        "start_time": "08:00:00",
        "end_time": "17:00:00",
        "break_duration_minutes": 60,
        "break_start": "12:00:00",
        "break_end": "13:00:00",
        "is_break_auto_deduct": true,
        "break_is_overtime": false,
        "early_punch_in_grace_period_minutes": 15
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/shifts", workspace_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string()) // Dev mode auth
                .body(Body::from(serde_json::to_vec(&shift_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 3. Test: Truy vấn danh sách ca
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/shifts", workspace_id))
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let shifts: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(!shifts.as_array().unwrap().is_empty());

    // Tìm shift vừa tạo trong danh sách
    let created_shift = shifts
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == "Test Shift API")
        .unwrap();
    assert_eq!(created_shift["break_start"], "12:00:00");
    assert_eq!(created_shift["break_end"], "13:00:00");
    assert_eq!(created_shift["is_break_auto_deduct"], true);

    factory.cleanup().await;
}
