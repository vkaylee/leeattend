use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use serde_json::json;
use std::sync::Arc;
use apiserver::{create_app, AppState, agent_manager::AgentManager};
use sqlx::PgPool;
use crate::common::factory::TestFactory;
use crate::common::db::TestDatabase;

mod common;

async fn setup_test_db() -> (TestDatabase, TestFactory) {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.pool());
    (test_db, factory)
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
        db: pool,
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret".to_string(),
    });

    create_app(state)
}

#[tokio::test]
async fn test_create_shift_with_fixed_break() {
    let (test_db, factory) = setup_test_db().await;
    let pool = test_db.pool();
    let app = setup_test_app(pool.clone()).await;

    // 1. Setup
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // 2. Act: Create Shift with Fixed Break
    let shift_req = json!({
        "name": "Fixed Break Shift",
        "start_time": "08:00:00",
        "end_time": "17:00:00",
        "break_duration_minutes": 60,
        "break_start": "12:00:00",
        "break_end": "13:00:00",
        "is_break_auto_deduct": true,
        "break_is_overtime": false,
        "early_punch_in_grace_period_minutes": 15
    });

    let response = app.clone()
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

    assert_eq!(response.status(), StatusCode::CREATED);

    // 3. Assert: Verify response
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_shift: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_shift["name"], "Fixed Break Shift");
    assert_eq!(created_shift["break_start"], "12:00:00");
    assert_eq!(created_shift["break_end"], "13:00:00");
    assert_eq!(created_shift["is_break_auto_deduct"], true);
}

#[tokio::test]
async fn test_create_shift_with_flexible_break() {
    let (test_db, factory) = setup_test_db().await;
    let pool = test_db.pool();
    let app = setup_test_app(pool.clone()).await;

    // 1. Setup
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // 2. Act: Create Shift with Flexible Break (no start/end)
    let shift_req = json!({
        "name": "Flexible Break Shift",
        "start_time": "09:00:00",
        "end_time": "18:00:00",
        "break_duration_minutes": 45,
        "break_is_overtime": false,
        "early_punch_in_grace_period_minutes": 10
        // break_start, break_end omitted -> should be null
        // is_break_auto_deduct omitted -> should default to true (as per serde default)
    });

    let response = app.clone()
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

    assert_eq!(response.status(), StatusCode::CREATED);

    // 3. Assert: Verify response
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_shift: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(created_shift["name"], "Flexible Break Shift");
    assert!(created_shift["break_start"].is_null());
    assert!(created_shift["break_end"].is_null());
    assert_eq!(created_shift["is_break_auto_deduct"], true); // Default value
}