//! Integration tests for system administration endpoints.
//!
//! Validates that:
//! - GET /system/roles works for PlatformAdmin
//! - POST /system/users/:id/roles can assign roles
//! - GET /system/users/:id/roles shows assigned roles
//! - DELETE /system/users/:id/roles/:name revokes roles
//! - GET /system/health is accessible by PlatformAdmin and Support
//! - Non-PlatformAdmin gets 403 on restricted routes
//! - Cannot revoke the last PlatformAdmin

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn body_json(response: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn assign_system_role(pool: &sqlx::PgPool, user_id: Uuid, role_name: &str) {
    sqlx::query(
        "INSERT INTO user_system_roles (user_id, role_id) \
         SELECT $1, id FROM system_roles WHERE name = $2",
    )
    .bind(user_id)
    .bind(role_name)
    .execute(pool)
    .await
    .unwrap();
}

fn make_req(
    method: &str,
    uri: &str,
    user_id: Uuid,
    body: Option<serde_json::Value>,
) -> axum::http::Request<axum::body::Body> {
    let req = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-User-Id", user_id.to_string())
        .header("Content-Type", "application/json");

    if let Some(b) = body {
        req.body(axum::body::Body::from(serde_json::to_string(&b).unwrap()))
            .unwrap()
    } else {
        req.body(axum::body::Body::empty()).unwrap()
    }
}

#[tokio::test]
async fn test_system_health_access() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let support_id = factory.user().create().await;
    let regular_id = factory.user().create().await;

    assign_system_role(&factory.pool, admin_id, "platform_admin").await;
    assign_system_role(&factory.pool, support_id, "support").await;

    // PlatformAdmin can access
    let res = app
        .clone()
        .oneshot(make_req("GET", "/api/system/health", admin_id, None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Support can access
    let res = app
        .clone()
        .oneshot(make_req("GET", "/api/system/health", support_id, None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Regular user cannot access
    let res = app
        .clone()
        .oneshot(make_req("GET", "/api/system/health", regular_id, None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_system_roles_crud_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let target_id = factory.user().create().await;

    assign_system_role(&factory.pool, admin_id, "platform_admin").await;

    // 1. List system roles
    let res = app
        .clone()
        .oneshot(make_req("GET", "/api/system/roles", admin_id, None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let roles = body_json(res).await;
    assert!(roles.as_array().unwrap().len() >= 3); // platform_admin, support, billing_admin

    // 2. Target user has no roles initially
    let res = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/system/users/{target_id}/roles"),
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let target_roles = body_json(res).await;
    assert!(target_roles.as_array().unwrap().is_empty());

    // 3. Admin assigns 'support' role to target
    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/system/users/{target_id}/roles"),
            admin_id,
            Some(json!({ "role_name": "support" })),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // 4. Verify target now has 'support' role
    let res = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/system/users/{target_id}/roles"),
            admin_id,
            None,
        ))
        .await
        .unwrap();
    let target_roles = body_json(res).await;
    let role_names: Vec<&str> = target_roles
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r.as_str())
        .collect();
    assert!(role_names.contains(&"support"));

    // 5. Admin revokes 'support' role
    let res = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/system/users/{target_id}/roles/support"),
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // 6. Verify role was revoked
    let res = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/system/users/{target_id}/roles"),
            admin_id,
            None,
        ))
        .await
        .unwrap();
    let target_roles = body_json(res).await;
    assert!(target_roles.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_cannot_revoke_last_platform_admin() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    // Delete any existing platform_admins to ensure we have exactly one
    sqlx::query(
        "DELETE FROM user_system_roles WHERE role_id = (SELECT id FROM system_roles WHERE name = 'platform_admin')"
    )
    .execute(&factory.pool)
    .await
    .unwrap();

    let admin_id = factory.user().create().await;
    assign_system_role(&factory.pool, admin_id, "platform_admin").await;

    // Admin tries to revoke their own role
    let res = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/system/users/{admin_id}/roles/platform_admin"),
            admin_id,
            None,
        ))
        .await
        .unwrap();

    // Should be blocked
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // Now create a second admin
    let second_admin_id = factory.user().create().await;
    assign_system_role(&factory.pool, second_admin_id, "platform_admin").await;

    // First admin should now be able to revoke the second admin
    let res = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/system/users/{second_admin_id}/roles/platform_admin"),
            admin_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_regular_user_cannot_manage_system_roles() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let regular_id = factory.user().create().await;

    // Try to list roles
    let res = app
        .clone()
        .oneshot(make_req("GET", "/api/system/roles", regular_id, None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // Try to assign a role
    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/system/users/{regular_id}/roles"),
            regular_id,
            Some(json!({ "role_name": "support" })),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
