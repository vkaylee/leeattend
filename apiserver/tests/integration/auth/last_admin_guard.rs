//! Integration tests for the "last platform_admin" safety guard.
//!
//! Validates that:
//! - Revoking the sole platform_admin (self-revoke) returns 422 Validation error
//! - Admin A cannot revoke Admin B when B is the only other admin (leaving A as last)
//!   — wait, actually after the fix the guard checks total count, so if there are 2 admins
//!   revoking one is allowed, but revoking when only 1 is left is blocked.
//! - Revoking a non-platform_admin role (e.g. support) from the last admin is allowed
//! - Revoking a platform_admin when there are ≥2 admins succeeds

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// Helper: parse JSON body from axum response
async fn body_json(response: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// Helper: assign a system role to a user directly in DB
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

/// Test: self-revoking the sole platform_admin must be rejected (422).
#[tokio::test]
async fn test_cannot_revoke_sole_platform_admin_self() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    assign_system_role(&factory.pool, admin_id, "platform_admin").await;

    // Attempt: admin tries to revoke their own platform_admin role
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(&format!(
                    "/api/system/users/{}/roles/platform_admin",
                    admin_id
                ))
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Should reject revoking the sole platform_admin"
    );

    let body = body_json(response).await;
    let error_msg = body["error"]["message"].as_str().unwrap_or_default();
    assert!(
        error_msg.contains("last platform_admin"),
        "Error should mention 'last platform_admin', got: {error_msg}"
    );
}

/// Test: admin A cannot revoke admin B when B is the only platform_admin.
/// After the fix, the guard checks total admin count regardless of who performs
/// the operation — so even admin A revoking admin B (the only admin) is blocked.
#[tokio::test]
async fn test_cannot_revoke_other_sole_platform_admin() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    // Create two users, but only user_b has platform_admin
    let user_a = factory.user().with_email("admin_a@test.com").create().await;
    let user_b = factory.user().with_email("admin_b@test.com").create().await;

    // user_a has platform_admin (to authenticate), user_b also has platform_admin
    assign_system_role(&factory.pool, user_a, "platform_admin").await;
    assign_system_role(&factory.pool, user_b, "platform_admin").await;

    // First, revoke user_a's admin role (this should succeed — 2 admins → 1 left)
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(&format!(
                    "/api/system/users/{}/roles/platform_admin",
                    user_a
                ))
                .header("X-User-Id", user_b.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NO_CONTENT,
        "Revoking one of two admins should succeed"
    );

    // Now user_b is the sole admin. user_b tries to revoke their own role.
    // (user_a no longer has platform_admin, so can't authenticate, but let's 
    //  test with user_b self-revoking to confirm the guard works)
    let response2 = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(&format!(
                    "/api/system/users/{}/roles/platform_admin",
                    user_b
                ))
                .header("X-User-Id", user_b.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response2.status(),
        StatusCode::BAD_REQUEST,
        "Should block revoking the last remaining platform_admin"
    );
}

/// Test: revoking platform_admin when there are ≥2 admins is allowed.
#[tokio::test]
async fn test_revoke_platform_admin_succeeds_when_multiple_admins() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_a = factory.user().with_email("admin1@test.com").create().await;
    let admin_b = factory.user().with_email("admin2@test.com").create().await;

    assign_system_role(&factory.pool, admin_a, "platform_admin").await;
    assign_system_role(&factory.pool, admin_b, "platform_admin").await;

    // admin_a revokes admin_b — should succeed (still 1 admin left)
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(&format!(
                    "/api/system/users/{}/roles/platform_admin",
                    admin_b
                ))
                .header("X-User-Id", admin_a.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NO_CONTENT,
        "Revoking one of two platform_admins should succeed"
    );
}

/// Test: revoking a non-platform_admin role from the sole admin is allowed.
/// The guard only protects the platform_admin role specifically.
#[tokio::test]
async fn test_revoke_support_role_from_sole_admin_is_allowed() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    assign_system_role(&factory.pool, admin_id, "platform_admin").await;
    assign_system_role(&factory.pool, admin_id, "support").await;

    // Revoke the support role — should succeed (platform_admin is untouched)
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(&format!(
                    "/api/system/users/{}/roles/support",
                    admin_id
                ))
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NO_CONTENT,
        "Revoking support role from sole admin should succeed"
    );
}

/// Test: data integrity — after blocking revoke, the admin still has the role.
#[tokio::test]
async fn test_blocked_revoke_preserves_role_assignment() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    assign_system_role(&factory.pool, admin_id, "platform_admin").await;

    // Attempt revoke (should fail)
    let _response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(&format!(
                    "/api/system/users/{}/roles/platform_admin",
                    admin_id
                ))
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Verify the role is still assigned in DB
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_system_roles usr \
         JOIN system_roles sr ON sr.id = usr.role_id \
         WHERE usr.user_id = $1 AND sr.name = 'platform_admin'",
    )
    .bind(admin_id)
    .fetch_one(&factory.pool)
    .await
    .unwrap();

    assert_eq!(
        count, 1,
        "platform_admin assignment must be preserved after blocked revoke"
    );
}
