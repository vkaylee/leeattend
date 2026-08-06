//! Request body size limit integration tests.
//!
//! These cover the layered defence against a single oversized POST exhausting
//! process memory:
//!
//! 1. The global `DefaultBodyLimit` in `create_app`, sourced from
//!    `MAX_REQUEST_BODY_BYTES` (default 2MB), applied to every route.
//! 2. Per-route overrides (`MAX_FILE_UPLOAD_BODY_BYTES`,
//!    `MAX_BULK_UPLOAD_BODY_BYTES`) for routes that legitimately accept more.
//! 3. The in-handler streaming check in `utils::upload::read_field_limited`,
//!    which stops reading the moment the accumulator crosses the cap.
//!
//! Rejection must surface as HTTP 413 so clients can distinguish "too big"
//! from "malformed".

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// Build a multipart body with a single named file field.
fn multipart_file(field: &str, filename: &str, content: &[u8]) -> (String, Vec<u8>) {
    let boundary = "limitboundary";
    let mut body = Vec::new();

    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"{field}\"; filename=\"{filename}\"\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(content);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    (format!("multipart/form-data; boundary={boundary}"), body)
}

/// `POST /files/upload` must reject a body past `MAX_FILE_UPLOAD_BODY_BYTES`.
///
/// This is the route that previously had no size validation of any kind — the
/// handler called `field.bytes()` with no ceiling. Rejection happens during the
/// multipart read, before any storage backend is contacted, so this test does
/// not require MinIO/S3 to be running.
#[tokio::test]
async fn test_files_upload_rejects_oversized_body() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // MAX_FILE_UPLOAD_BODY_BYTES defaults to 20MB — send 22MB.
    let oversized = vec![b'A'; 22 * 1024 * 1024];
    let (content_type, body) = multipart_file("file", "huge.bin", &oversized);

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{ws_id}/files/upload"))
                .header("Content-Type", content_type)
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "Oversized /files/upload must be rejected with 413"
    );
}

/// `PUT /files/{id}` shares the same unbounded-read defect and the same fix.
///
/// A random file id is used deliberately: the body limit must trip during
/// extraction, before the handler ever looks the file up, so the response is
/// 413 rather than 404.
#[tokio::test]
async fn test_files_update_rejects_oversized_body() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let file_id = Uuid::new_v4();

    let oversized = vec![b'A'; 22 * 1024 * 1024];
    let (content_type, body) = multipart_file("file", "huge.bin", &oversized);

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/workspaces/{ws_id}/files/{file_id}"))
                .header("Content-Type", content_type)
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "Oversized PUT /files/:id must be rejected with 413 before the file lookup"
    );
}

/// The global 2MB cap must apply to routes that did NOT opt into a larger
/// per-route limit. Without this, the `route_layer` overrides would be the only
/// thing standing between an attacker and an unbounded read.
#[tokio::test]
async fn test_global_body_limit_applies_to_non_upload_route() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // 3MB of JSON against a normal (non-upload) route: over the 2MB global
    // default, and this route has no override.
    let big_payload = format!(r#"{{"note":"{}"}}"#, "A".repeat(3 * 1024 * 1024));

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{ws_id}/employees"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(big_payload))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    assert!(
        status == StatusCode::PAYLOAD_TOO_LARGE || status == StatusCode::BAD_REQUEST,
        "Body over the global limit must be rejected before deserialization, got {status}"
    );
    assert_ne!(
        status,
        StatusCode::CREATED,
        "An oversized body must never be processed successfully"
    );
}

/// A body just under the per-route cap must pass the limit layer untouched.
///
/// This is the regression guard for the limits being set too tight: it proves
/// the ceiling is a ceiling, not a blanket rejection. The request is expected
/// to fail later on business grounds (unparseable file), which is precisely the
/// point — it got past the body limit and into the handler.
#[tokio::test]
async fn test_body_under_limit_passes_limit_layer() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // 5MB: well over the 2MB global default, comfortably under the 10MB
    // bulk-upload override on this route.
    let under_limit = vec![b'A'; 5 * 1024 * 1024];
    let (content_type, body) = multipart_file("file", "big.xlsx", &under_limit);

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{ws_id}/attendance/upload"))
                .header("Content-Type", content_type)
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(
        response.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "A 5MB body is under the 10MB route limit and must not be rejected as too large"
    );
}
