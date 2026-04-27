//! Workspace Audit Logs API integration tests

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
    body: Option<String>,
) -> axum::http::Request<axum::body::Body> {
    let mut builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }

    builder
        .body(match body {
            Some(b) => axum::body::Body::from(b),
            None => axum::body::Body::empty(),
        })
        .unwrap()
}

async fn body_json(resp: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 10000).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_get_audit_logs_empty() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/audit-logs"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let result = body_json(resp).await;
    assert_eq!(result["total"].as_i64().unwrap(), 0);
    assert!(result["data"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_get_audit_logs_with_role_action() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Create a member and assign an extra role (this creates audit log entries)
    let member_id = factory.user().with_email("audited@example.com").create().await;
    factory.add_member(ws_id, member_id, uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap()).await;

    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
    let body = serde_json::json!({
        "role_id": "a0000000-0000-0000-0000-000000000005",
        "expires_at": expires_at.to_rfc3339()
    });
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/members/{member_id}/roles"),
            ws_id,
            admin_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Now get audit logs
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/audit-logs"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let result = body_json(resp).await;
    assert!(result["total"].as_i64().unwrap() >= 1);
    let logs = result["data"].as_array().unwrap();
    assert!(logs.iter().any(|l| l["action"].as_str().unwrap() == "assign_extra_role"));
}

#[tokio::test]
async fn test_get_audit_logs_rbac_member_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let member_id = factory.user().with_email("regular@example.com").create().await;
    factory.add_member(ws_id, member_id, uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap()).await;

    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/audit-logs"),
            ws_id,
            member_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
