//! Authentication session lifecycle integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::services::auth_service::AuthService;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_auth_session_lifecycle_smoke() {
    let (_test_db, factory, state, _app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;
    let user_id: Uuid = factory.user().create().await;

    // 1. Create session
    let refresh_token = state
        .core()
        .auth_service()
        .create_session(user_id)
        .await
        .unwrap();
    assert!(!refresh_token.is_empty());

    // 2. Refresh session
    let auth_res = state
        .core()
        .auth_service()
        .refresh_session(&refresh_token, &state.core().jwt_secret())
        .await
        .unwrap();
    assert_eq!(auth_res.user.id, user_id);
    assert!(!auth_res.access_token.is_empty());
    assert!(!auth_res.refresh_token.is_empty());
    assert_ne!(auth_res.refresh_token, refresh_token); // Token rotation

    let new_refresh_token = auth_res.refresh_token;

    // 3. Verify access token
    let verified_id =
        AuthService::verify_token(&auth_res.access_token, &state.core().jwt_secret()).unwrap();
    assert_eq!(verified_id, user_id);

    // 4. Revoke session
    state
        .core()
        .auth_service()
        .revoke_session(&new_refresh_token)
        .await
        .unwrap();

    // 5. Try to refresh again (should fail)
    let result = state
        .core()
        .auth_service()
        .refresh_session(&new_refresh_token, &state.core().jwt_secret())
        .await;
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid or expired refresh token"));
}

#[tokio::test]
async fn test_revoke_all_sessions() {
    let (_test_db, factory, state, _app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;
    let user_id: Uuid = factory.user().create().await;

    let token1 = state
        .core()
        .auth_service()
        .create_session(user_id)
        .await
        .unwrap();
    let token2 = state
        .core()
        .auth_service()
        .create_session(user_id)
        .await
        .unwrap();

    // Revoke all
    state
        .core()
        .auth_service()
        .revoke_all_sessions_for_user(user_id)
        .await
        .unwrap();

    // Verify both are invalid
    assert!(state
        .core()
        .auth_service()
        .refresh_session(&token1, &state.core().jwt_secret())
        .await
        .is_err());
    assert!(state
        .core()
        .auth_service()
        .refresh_session(&token2, &state.core().jwt_secret())
        .await
        .is_err());
}

#[tokio::test]
async fn test_get_me_and_logout_smoke() {
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let refresh_token = state
        .core()
        .auth_service()
        .create_session(user_id)
        .await
        .unwrap();

    // 1. Get Me
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

    // 2. Logout
    let logout_req = serde_json::json!({
        "refresh_token": refresh_token
    });
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(logout_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. Verify session revoked
    let result = state
        .core()
        .auth_service()
        .refresh_session(&refresh_token, &state.core().jwt_secret())
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_me_not_found() {
    let (_test_db, _factory, _state, app): (
        TestDatabase,
        TestFactory,
        Arc<AppState>,
        axum::Router,
    ) = setup_app_tower().await;

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/v1/me")
                .header("X-User-Id", Uuid::new_v4().to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_refresh_token_api() {
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let refresh_token = state
        .core()
        .auth_service()
        .create_session(user_id)
        .await
        .unwrap();

    let req = serde_json::json!({
        "refresh_token": refresh_token
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh")
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_refresh_token() {
    let (_test_db, _factory, state, _app): (
        TestDatabase,
        TestFactory,
        Arc<AppState>,
        axum::Router,
    ) = setup_app_tower().await;
    let result = state
        .core()
        .auth_service()
        .refresh_session("non-existent-token", &state.core().jwt_secret())
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_refresh_token_reuse_detection() {
    let (_test_db, factory, state, _app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;
    let user_id: Uuid = factory.user().create().await;

    // Skip if Redis is not configured in this test environment
    if state.infra().redis().is_none() {
        return;
    }

    // 1. Create session
    let refresh_token = state
        .core()
        .auth_service()
        .create_session(user_id)
        .await
        .unwrap();

    // 2. Refresh session (first time - rotates refresh token)
    let auth_res = state
        .core()
        .auth_service()
        .refresh_session(&refresh_token, &state.core().jwt_secret())
        .await
        .unwrap();

    let rotated_token = auth_res.refresh_token;

    // 3. Try to refresh again using the OLD (already rotated) refresh token
    let result = state
        .core()
        .auth_service()
        .refresh_session(&refresh_token, &state.core().jwt_secret())
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Token theft detected"));

    // 4. Verify that the new (rotated) refresh token is now ALSO invalid because all sessions were revoked
    let result2 = state
        .core()
        .auth_service()
        .refresh_session(&rotated_token, &state.core().jwt_secret())
        .await;
    assert!(result2.is_err());
}
