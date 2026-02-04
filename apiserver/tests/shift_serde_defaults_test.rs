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

    // Setup test storage

    let state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
        roster_cache: None,
    });

    create_app(state)
}

#[tokio::test]
async fn test_create_shift_with_missing_fields_should_use_defaults() {
    let pool = setup_test_db().await;
    let factory = TestFactory::new(pool.clone());
    let app = setup_test_app(pool.clone()).await;

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // missing break_is_overtime and early_punch_in_grace_period_minutes
    let shift_req = json!({
        "name": "Minimal Shift",
        "start_time": "09:00:00",
        "end_time": "18:00:00",
        "break_duration_minutes": 60
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/shifts", workspace_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(Body::from(serde_json::to_vec(&shift_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should NOT be 422
    assert_eq!(response.status(), StatusCode::CREATED, "Should succeed despite missing fields because of Serde defaults");

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let shift: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(shift["name"], "Minimal Shift");
    assert_eq!(shift["break_is_overtime"], false); // Default value
    assert_eq!(shift["early_punch_in_grace_period_minutes"], 0); // Default value
    assert_eq!(shift["is_break_auto_deduct"], true); // Existing default

    factory.cleanup().await;
}

#[tokio::test]
async fn test_update_shift_with_missing_fields_should_use_defaults() {
    let pool = setup_test_db().await;
    let factory = TestFactory::new(pool.clone());
    let app = setup_test_app(pool.clone()).await;

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    
    // Create initial shift
    let shift_id = factory.shift().with_workspace(workspace_id).create().await;

    // Update with minimal fields
    let update_req = json!({
        "name": "Updated Minimal Shift",
        "start_time": "10:00:00",
        "end_time": "19:00:00",
        "break_duration_minutes": 30,
        "is_correction": true
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/workspaces/{}/shifts/{}", workspace_id, shift_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(Body::from(serde_json::to_vec(&update_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK, "Update should succeed despite missing fields");

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let shift: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(shift["name"], "Updated Minimal Shift");
    assert_eq!(shift["break_is_overtime"], false);
    assert_eq!(shift["early_punch_in_grace_period_minutes"], 0);

    factory.cleanup().await;
}
