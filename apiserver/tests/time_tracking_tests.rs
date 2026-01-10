use apiserver::{
    create_app,
    models::{TimeEntry},
    services::auth_service::AuthService,
    repositories::{SqlxAgentRepository, SqlxWorkspaceRepository},
    AppState, agent_manager::AgentManager,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::Local;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;

mod common;
use common::factory::TestFactory;
use common::db::TestDatabase;

async fn setup_test_db() -> (TestDatabase, TestFactory) {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.pool());
    (test_db, factory)
}

async fn setup_test_app(pool: PgPool) -> axum::Router {
    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo = Arc::new(SqlxWorkspaceRepository { pool: pool.clone() });
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
async fn test_punch_in_grace_period_enforcement() {
    let (test_db, factory) = setup_test_db().await;
    let pool = test_db.pool();
    let app = setup_test_app(pool.clone()).await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory.employee().with_workspace(ws_id).with_user(user_id).create().await;
    
    // Create a shift that starts in 30 minutes
    let now_local = Local::now();
    let now_time = now_local.time();
    let start_offset = chrono::Duration::minutes(30);
    let future_start = now_time + start_offset;
    
    let shift_id = factory.shift()
        .with_workspace(ws_id)
        .with_times(future_start, future_start + chrono::Duration::hours(8))
        .with_grace_period(5) // 5 mins grace
        .assign_to_employee(employee_id, now_local.date_naive())
        .await;

    let token = AuthService::generate_token(user_id, "test_secret", 3600).unwrap();

    // Try to punch in NOW (which is 30 mins before start) -> Should fail (400) because 30 > 5
    let punch_payload = json!({
        "employee_id": employee_id,
        "entry_type": "work"
    });

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/workspaces/{}/time/punch-in", ws_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&punch_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Now update shift to have a large grace period (60 mins) that covers NOW (30 mins before)
    sqlx::query("UPDATE shifts SET early_punch_in_grace_period_minutes = 60 WHERE id = $1")
        .bind(shift_id)
        .execute(&pool)
        .await
        .unwrap();

    let req2 = Request::builder()
        .method("POST")
        .uri(format!("/api/workspaces/{}/time/punch-in", ws_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&punch_payload).unwrap()))
        .unwrap();

    let response2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response2.into_body(), usize::MAX).await.unwrap();
    let entry: TimeEntry = serde_json::from_slice(&body).unwrap();
    assert_eq!(entry.shift_id, Some(shift_id));
}
