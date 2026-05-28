//! RBAC integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_rbac_admin_access_smoke() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let admin_id = factory.user().create().await;
    factory
        .add_member(
            ws_id,
            admin_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000002").unwrap(),
        )
        .await;

    let member_id = factory.user().create().await;
    factory
        .add_member(
            ws_id,
            member_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    // Test Admin Access to Restricted Resource (e.g. settings)
    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{ws_id}")) // Workspace details usually require at least Member, but Update requires Admin
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Test Member Update (Should Fail)
    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/workspaces/{ws_id}"))
                .header("X-User-Id", member_id.to_string())
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::json!({"name": "New Name"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
