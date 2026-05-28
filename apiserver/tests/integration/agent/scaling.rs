//! Agent scaling configuration integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_agent_scaling_config() {
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory
        .user()
        .with_email("scaling@example.com")
        .create()
        .await;
    let workspace_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // Generate token for auth
    let token = apiserver::services::auth_service::AuthService::generate_token(
        user_id,
        "test_secret_tower",
        3600,
    )
    .unwrap();

    // 1. Create Agent with default workers (2)
    let agent_id = "AGENT_SCALE_TEST";
    let create_req = json!({
        "id": agent_id,
        "name": "Scaling Agent",
        "token": "secret_token",
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/authorized-agents")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Workspace-Id", workspace_id.to_string())
        .body(Body::from(create_req.to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Verify DB default
    let agent = state
        .agents
        .repo
        .get_agent_by_id_in_workspace(
            agent_id,
            &apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(agent.max_concurrent_workers, 2);

    // 2. Update to 5 workers
    let update_req = json!({
        "max_concurrent_workers": 5
    });

    let req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/v1/authorized-agents/{agent_id}"))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Workspace-Id", workspace_id.to_string())
        .body(Body::from(update_req.to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify DB update
    let agent = state
        .agents
        .repo
        .get_agent_by_id_in_workspace(
            agent_id,
            &apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(agent.max_concurrent_workers, 5);

    // 3. Update to 1 worker
    let update_req = json!({
        "max_concurrent_workers": 1
    });

    let req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/v1/authorized-agents/{agent_id}"))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Workspace-Id", workspace_id.to_string())
        .body(Body::from(update_req.to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify DB update
    let agent = state
        .agents
        .repo
        .get_agent_by_id_in_workspace(
            agent_id,
            &apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(agent.max_concurrent_workers, 1);
}
