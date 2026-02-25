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
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

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
                .header("X-User-Id", user_id.to_string())
                .header("X-Workspace-Id", ws_id.to_string())
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
                .header("X-User-Id", user_id.to_string())
                .header("X-Workspace-Id", ws_id.to_string())
                .body(axum::body::Body::from(command_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify response is JSON and contains message
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response must be valid JSON");
    assert_eq!(json["message"], "Command sent");
}

#[tokio::test]
async fn test_get_agents_device_status() {
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    
    // Create an agent
    let agent_id: String = factory
        .agent()
        .with_workspace(ws_id)
        .with_trust(true)
        .create()
        .await;

    let dev_enabled_id = Uuid::new_v4();
    let dev_disabled_id = Uuid::new_v4();

    // Insert enabled device
    sqlx::query(
        "INSERT INTO devices (id, agent_id, name, ip, port, workspace_id, brand, is_enabled) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(dev_enabled_id)
    .bind(&agent_id)
    .bind("Enabled Device")
    .bind("192.168.1.100")
    .bind(4370)
    .bind(ws_id)
    .bind("zkteco")
    .bind(true)
    .execute(&factory.pool)
    .await
    .unwrap();

    // Insert disabled device
    sqlx::query(
        "INSERT INTO devices (id, agent_id, name, ip, port, workspace_id, brand, is_enabled) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(dev_disabled_id)
    .bind(&agent_id)
    .bind("Disabled Device")
    .bind("192.168.1.101")
    .bind(4370)
    .bind(ws_id)
    .bind("zkteco")
    .bind(false)
    .execute(&factory.pool)
    .await
    .unwrap();

    // 1. Agent is offline. Result should be both devices show offline.
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
    let body = axum::body::to_bytes(response.into_body(), 1024*1024).await.unwrap();
    let agents: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    
    // Find our agent
    let agent = agents.iter().find(|a| a["agent_id"] == agent_id).expect("Agent must be returned");
    assert_eq!(agent["is_online"], false);
    
    // Check devices
    let devices = agent["devices"].as_array().expect("Devices should be an array");
    assert_eq!(devices.len(), 2);
    for dev in devices {
        assert_eq!(dev["status"], "offline", "All devices should be offline when agent is offline");
    }

    // 2. Mock agent being online
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    state.agent_manager.add_agent(agent_id.clone(), tx).await;

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
    let body = axum::body::to_bytes(response.into_body(), 1024*1024).await.unwrap();
    let agents: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    
    let agent = agents.iter().find(|a| a["agent_id"] == agent_id).expect("Agent must be returned");
    assert_eq!(agent["is_online"], true);
    
    let devices = agent["devices"].as_array().expect("Devices should be an array");
    assert_eq!(devices.len(), 2);
    
    // Now the enabled device should be "online", disabled should be "offline"
    for dev in devices {
        let dev_id = dev["device_id"].as_str().unwrap();
        if dev_id == dev_enabled_id.to_string() {
            assert_eq!(dev["status"], "online", "Enabled device should be online when agent is online");
        } else if dev_id == dev_disabled_id.to_string() {
            assert_eq!(dev["status"], "offline", "Disabled device should be offline even when agent is online");
        }
    }
}
