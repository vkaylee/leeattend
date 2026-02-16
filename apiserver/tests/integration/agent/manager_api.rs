//! Agent manager API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_agent_commands_api() {
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) = setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let agent_id: String = factory
        .agent()
        .with_workspace(ws_id)
        .with_trust(true)
        .create()
        .await;

    // 1. Get agents (none online yet)
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/agents")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 2. Send command to offline agent (should fail with 404)
    let command_req = serde_json::json!({
        "method": "reboot",
        "data": null
    });
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/agent/{agent_id}/command"))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(command_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // 3. Mock agent being online
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    state.agent_manager.add_agent(agent_id.clone(), tx).await;

    // 4. Send command to online agent
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/agent/{agent_id}/command"))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(command_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
