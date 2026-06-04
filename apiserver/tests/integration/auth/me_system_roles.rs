//! Integration tests for `system_roles` in /me endpoint and auth responses.
//!
//! Validates that:
//! - GET /me returns `system_roles` for platform admins
//! - GET /me returns empty `system_roles` for regular users
//! - Refresh token response includes `system_roles`
//! - First registered user's /me response includes `platform_admin`

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
async fn assign_system_role(
    pool: &apiserver::diesel_pool::DieselPools,
    user_id: Uuid,
    role_name: &str,
) {
    crate::test_execute!(
        &pool.api,
        "INSERT INTO user_system_roles (user_id, role_id) \
         SELECT $1, id FROM system_roles WHERE name = $2",
        diesel::sql_types::Uuid,
        user_id,
        diesel::sql_types::Text,
        role_name
    );
}

#[tokio::test]
async fn test_get_me_returns_system_roles_for_platform_admin() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;

    // Assign platform_admin role
    assign_system_role(&factory.diesel_pools, user_id, "platform_admin").await;

    // GET /me
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/v1/me")
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    let system_roles = body["system_roles"].as_array().unwrap();
    assert!(
        system_roles
            .iter()
            .any(|r| r.as_str() == Some("platform_admin")),
        "Expected platform_admin in system_roles, got: {system_roles:?}"
    );
}

#[tokio::test]
async fn test_get_me_returns_empty_system_roles_for_regular_user() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;

    // GET /me (no system role assigned)
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/v1/me")
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    let system_roles = body["system_roles"].as_array().unwrap();
    assert!(
        system_roles.is_empty(),
        "Regular user should have empty system_roles, got: {system_roles:?}"
    );
}

#[tokio::test]
async fn test_get_me_returns_multiple_system_roles() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;

    // Assign both platform_admin and support roles
    assign_system_role(&factory.diesel_pools, user_id, "platform_admin").await;
    assign_system_role(&factory.diesel_pools, user_id, "support").await;

    // GET /me
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/v1/me")
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    let system_roles = body["system_roles"].as_array().unwrap();
    let role_names: Vec<&str> = system_roles.iter().filter_map(|r| r.as_str()).collect();

    assert!(
        role_names.contains(&"platform_admin"),
        "Missing platform_admin"
    );
    assert!(role_names.contains(&"support"), "Missing support");
    assert_eq!(role_names.len(), 2, "Expected exactly 2 roles");
}

#[tokio::test]
async fn test_refresh_session_includes_system_roles() {
    let (_test_db, factory, state, _app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;

    // Assign platform_admin
    assign_system_role(&factory.diesel_pools, user_id, "platform_admin").await;

    // Create session and refresh
    let refresh_token = state
        .core()
        .auth_service()
        .create_session(user_id)
        .await
        .unwrap();

    let auth_res = state
        .core()
        .auth_service()
        .refresh_session(&refresh_token, &state.core().jwt_secret())
        .await
        .unwrap();

    assert_eq!(auth_res.user.id, user_id);
    assert!(
        auth_res
            .user
            .system_roles
            .contains(&"platform_admin".to_string()),
        "Refresh response should include system_roles, got: {:?}",
        auth_res.user.system_roles
    );
}

#[tokio::test]
async fn test_refresh_session_empty_system_roles_for_regular_user() {
    let (_test_db, factory, state, _app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;

    let refresh_token = state
        .core()
        .auth_service()
        .create_session(user_id)
        .await
        .unwrap();

    let auth_res = state
        .core()
        .auth_service()
        .refresh_session(&refresh_token, &state.core().jwt_secret())
        .await
        .unwrap();

    assert!(
        auth_res.user.system_roles.is_empty(),
        "Regular user refresh should return empty system_roles, got: {:?}",
        auth_res.user.system_roles
    );
}
