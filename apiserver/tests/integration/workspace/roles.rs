//! Workspace Roles API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::MemberRole;
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
async fn test_extra_role_assign_and_revoke() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Create member
    let member_id = factory.user().with_email("staff@example.com").create().await;
    factory.add_member(ws_id, member_id, MemberRole::Member).await;

    // 1. Get member roles (should have primary only)
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/members/{member_id}/roles"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let roles = body_json(resp).await;
    assert!(roles["primary"].as_str().is_some());

    // 2. Assign extra role (TeamLead, expires in 1 hour)
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
    let assign_body = serde_json::json!({
        "role": "team_lead",
        "expires_at": expires_at.to_rfc3339()
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/members/{member_id}/roles"),
            ws_id,
            admin_id,
            Some(assign_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let extra_role = body_json(resp).await;
    let role_id = extra_role["id"].as_str().unwrap();

    // 3. Verify roles now include extra
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/members/{member_id}/roles"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let roles = body_json(resp).await;
    assert!(!roles["extra"].as_array().unwrap().is_empty());

    // 4. Revoke extra role
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/workspaces/{ws_id}/members/{member_id}/roles/{role_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_assign_role_expired_returns_validation_error() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let member_id = factory.user().with_email("staff2@example.com").create().await;
    factory.add_member(ws_id, member_id, MemberRole::Member).await;

    // Try to assign with expired time
    let expired = chrono::Utc::now() - chrono::Duration::hours(1);
    let body = serde_json::json!({
        "role": "hr_manager",
        "expires_at": expired.to_rfc3339()
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
    // Should return validation error (400 or 422)
    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn test_assign_role_rbac_member_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let member_id = factory.user().with_email("member@example.com").create().await;
    factory.add_member(ws_id, member_id, MemberRole::Member).await;

    let target_id = factory.user().with_email("target@example.com").create().await;
    factory.add_member(ws_id, target_id, MemberRole::Member).await;

    let body = serde_json::json!({
        "role": "hr_manager",
        "expires_at": (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339()
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/members/{target_id}/roles"),
            ws_id,
            member_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
