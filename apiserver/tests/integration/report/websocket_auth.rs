//! WebSocket report authentication integration tests
//!
//! Validates: JWT token required, invalid token rejected, valid token passes auth

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_ws_report_missing_token_returns_401() {
    let (_test_db, _factory, _state, app): (
        TestDatabase,
        TestFactory,
        Arc<AppState>,
        axum::Router,
    ) = setup_app_tower().await;

    let fake_job_id = uuid::Uuid::new_v4();
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/ws/reports/{fake_job_id}"))
                .header("Connection", "Upgrade")
                .header("Upgrade", "websocket")
                .header("Sec-WebSocket-Version", "13")
                .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_ws_report_invalid_token_returns_401() {
    let (_test_db, _factory, _state, app): (
        TestDatabase,
        TestFactory,
        Arc<AppState>,
        axum::Router,
    ) = setup_app_tower().await;

    let fake_job_id = uuid::Uuid::new_v4();
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/ws/reports/{fake_job_id}?token=invalid_token_abc123"
                ))
                .header("Connection", "Upgrade")
                .header("Upgrade", "websocket")
                .header("Sec-WebSocket-Version", "13")
                .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_ws_report_valid_token_passes_auth() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let valid_token = apiserver::services::auth_service::AuthService::generate_token(
        user_id,
        "test_secret_tower",
        60,
    )
    .unwrap();

    let fake_job_id = uuid::Uuid::new_v4();
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/ws/reports/{fake_job_id}?token={valid_token}"))
                .header("Connection", "Upgrade")
                .header("Upgrade", "websocket")
                .header("Sec-WebSocket-Version", "13")
                .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Valid token passes auth; should NOT be 401
    // Will get 503 (Redis unavailable) or 101 (WS upgrade)
    let status = res.status();
    assert_ne!(
        status,
        StatusCode::UNAUTHORIZED,
        "Valid token should pass auth check"
    );
}
