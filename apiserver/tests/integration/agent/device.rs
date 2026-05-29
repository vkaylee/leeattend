//! Device API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;
fn req(
    method: &str,
    uri: &str,
    ws_id: Uuid,
    user_id: Uuid,
    body: Option<serde_json::Value>,
) -> axum::http::Request<axum::body::Body> {
    let mut builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if let Some(b) = body {
        builder = builder.header("Content-Type", "application/json");
        builder.body(axum::body::Body::from(b.to_string())).unwrap()
    } else {
        builder.body(axum::body::Body::empty()).unwrap()
    }
}

async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn test_device_crud_endpoints() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    
    // Create an agent first
    let agent_id: String = factory
        .agent()
        .with_workspace(ws_id)
        .with_trust(true)
        .create()
        .await;

    // 1. Create Device
    let create_req = serde_json::json!({
        "agent_id": agent_id,
        "name": "Main Entrance",
        "ip": "192.168.1.50",
        "port": 4370,
        "brand": "zkteco",
        "timezone": "UTC"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/devices",
            ws_id,
            user_id,
            Some(create_req),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = body_json(resp).await;
    let device_id = created["id"].as_str().unwrap().to_string();

    // 2. Get Devices
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            "/api/v1/devices",
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let list = body_json(resp).await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["name"], "Main Entrance");

    // 3. Update Device
    let update_req = serde_json::json!({
        "name": "Back Entrance",
        "is_enabled": false
    });

    let resp = app
        .clone()
        .oneshot(req(
            "PUT",
            &format!("/api/v1/devices/{}", device_id),
            ws_id,
            user_id,
            Some(update_req),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // Verify Update
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            "/api/v1/devices",
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    let list = body_json(resp).await;
    assert_eq!(list[0]["name"], "Back Entrance");
    assert_eq!(list[0]["is_enabled"], false);

    // 4. Delete Device
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/v1/devices/{}", device_id),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify Delete
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            "/api/v1/devices",
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    let list = body_json(resp).await;
    assert!(list.as_array().unwrap().is_empty());
}
