//! Feature Flags API integration tests (Tower-based)

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::feature_flag::FeatureFlag;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// POST a new feature flag and return the parsed `FeatureFlag`.
async fn create_flag(
    app: &axum::Router,
    ws_id: Uuid,
    user_id: Uuid,
    body: &serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{ws_id}/feature-flags"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    (status, value)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Full CRUD lifecycle: create → get → list → update → check → delete
#[tokio::test]
async fn test_feature_flag_crud_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // 1. Create a feature flag (disabled by default)
    let (status, flag) = create_flag(
        &app,
        ws_id,
        user_id,
        &serde_json::json!({
            "flag_key": "dark_mode",
            "description": "Enable dark mode UI"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(flag["flag_key"], "dark_mode");
    assert_eq!(flag["enabled"], false);
    assert_eq!(flag["description"], "Enable dark mode UI");
    let flag_id = flag["id"].as_str().unwrap();

    // 2. Get single feature flag
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{ws_id}/feature-flags/{flag_id}"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let fetched: FeatureFlag = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched.flag_key, "dark_mode");
    assert!(!fetched.enabled);

    // 3. List feature flags — should contain at least our flag
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{ws_id}/feature-flags"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let flags: Vec<FeatureFlag> = serde_json::from_slice(&body).unwrap();
    assert!(!flags.is_empty());
    assert!(flags.iter().any(|f| f.flag_key == "dark_mode"));

    // 4. Update — enable the flag + set metadata
    let update_req = serde_json::json!({
        "enabled": true,
        "metadata": { "rollout_percent": 100 }
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri(format!("/api/workspaces/{ws_id}/feature-flags/{flag_id}"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&update_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: FeatureFlag = serde_json::from_slice(&body).unwrap();
    assert!(updated.enabled);
    assert_eq!(updated.metadata["rollout_percent"], 100);

    // 5. Check by key — should be enabled
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{ws_id}/feature-flags/check/dark_mode"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let check: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(check["key"], "dark_mode");
    assert_eq!(check["enabled"], true);

    // 6. Delete
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!("/api/workspaces/{ws_id}/feature-flags/{flag_id}"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 7. Verify deletion — GET should 404
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{ws_id}/feature-flags/{flag_id}"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test upsert: first call creates, second call updates the same key.
#[tokio::test]
async fn test_feature_flag_upsert() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    let upsert_body = serde_json::json!({
        "flag_key": "beta_payroll",
        "enabled": false,
        "description": "Beta payroll module"
    });

    // First upsert — should create
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!("/api/workspaces/{ws_id}/feature-flags"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&upsert_body).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let first: FeatureFlag = serde_json::from_slice(&body).unwrap();
    assert_eq!(first.flag_key, "beta_payroll");
    assert!(!first.enabled);

    // Second upsert — same key, now enabled
    let upsert_update = serde_json::json!({
        "flag_key": "beta_payroll",
        "enabled": true,
        "description": "Beta payroll — GA"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!("/api/workspaces/{ws_id}/feature-flags"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&upsert_update).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let second: FeatureFlag = serde_json::from_slice(&body).unwrap();
    assert_eq!(second.flag_key, "beta_payroll");
    assert!(second.enabled);
    assert_eq!(second.description.as_deref(), Some("Beta payroll — GA"));
    // Same row — id should be stable
    assert_eq!(first.id, second.id);
}

/// Check a non-existent flag key returns false.
#[tokio::test]
async fn test_check_nonexistent_flag_returns_false() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{ws_id}/feature-flags/check/nonexistent_flag"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let check: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(check["enabled"], false);
}

/// GET and DELETE a non-existent flag ID should return 404.
#[tokio::test]
async fn test_get_and_delete_nonexistent_flag_returns_404() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let fake_id = Uuid::new_v4();

    // GET should 404
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{ws_id}/feature-flags/{fake_id}"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // DELETE should 404
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!("/api/workspaces/{ws_id}/feature-flags/{fake_id}"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Create with all fields including metadata.
#[tokio::test]
async fn test_create_flag_with_metadata() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    let (status, flag) = create_flag(
        &app,
        ws_id,
        user_id,
        &serde_json::json!({
            "flag_key": "gradual_rollout",
            "enabled": true,
            "description": "Gradual feature rollout",
            "metadata": {
                "rollout_percent": 25,
                "allowed_groups": ["beta_testers", "staff"]
            }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(flag["enabled"].as_bool().unwrap());
    assert_eq!(flag["metadata"]["rollout_percent"], 25);
    let groups = flag["metadata"]["allowed_groups"].as_array().unwrap();
    assert_eq!(groups.len(), 2);
}
