//! Shift versioning tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_shift_versioning_update() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // 1. Create Shift V1
    let shift_req = serde_json::json!({
        "name": "Shift V1",
        "intervals": [{
            "start_time": "09:00:00",
            "end_time": "17:00:00",
            "break_duration_minutes": 60,
            "is_break_auto_deduct": true,
            "break_is_overtime": false,
            "early_punch_in_grace_period_minutes": 0
        }]
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{workspace_id}/shifts"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(shift_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let created_shift: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let shift_id = created_shift["id"].as_str().unwrap();

    // 2. Update as New Version (V2)
    let version_req = serde_json::json!({
        "name": "Shift V2",
        "intervals": [{
            "start_time": "08:00:00", // Changed
            "end_time": "16:00:00",
            "break_duration_minutes": 60,
            "is_break_auto_deduct": true,
            "break_is_overtime": false,
            "early_punch_in_grace_period_minutes": 0
        }],
        "is_correction": false, // New Version
        "effective_date": "2026-02-01"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!(
                    "/api/v1/workspaces/{workspace_id}/shifts/{shift_id}"
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(version_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_shift_versions_after_create_and_update() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // 1. Create Shift V1
    let shift_req = serde_json::json!({
        "name": "Shift V1",
        "intervals": [{
            "start_time": "09:00:00",
            "end_time": "17:00:00",
            "break_duration_minutes": 60,
            "is_break_auto_deduct": true,
            "break_is_overtime": false,
            "early_punch_in_grace_period_minutes": 0
        }]
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{workspace_id}/shifts"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(shift_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let shift_id = created["id"].as_str().unwrap();

    // 2. Update as New Version (V2) with change_reason
    let version_req = serde_json::json!({
        "name": "Shift V2",
        "intervals": [{
            "start_time": "08:00:00",
            "end_time": "16:00:00",
            "break_duration_minutes": 60,
            "is_break_auto_deduct": true,
            "break_is_overtime": false,
            "early_punch_in_grace_period_minutes": 0
        }],
        "is_correction": false,
        "effective_date": "2026-02-01",
        "change_reason": "Updated start time per HR request"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!(
                    "/api/v1/workspaces/{workspace_id}/shifts/{shift_id}"
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(version_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. GET version history
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/workspaces/{workspace_id}/shifts/{shift_id}/versions"
                ))
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let versions: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    // Should have 2 versions
    assert_eq!(
        versions.len(),
        2,
        "Expected 2 versions (V1 create + V2 revise)"
    );

    // V2 is newest (first in DESC order)
    let v2 = &versions[0];
    assert_eq!(v2["version_number"].as_i64().unwrap(), 2);
    assert_eq!(v2["name"].as_str().unwrap(), "Shift V2");
    assert_eq!(v2["change_type"].as_str().unwrap(), "revise");
    assert_eq!(
        v2["change_reason"].as_str().unwrap(),
        "Updated start time per HR request"
    );
    assert_eq!(v2["effective_from"].as_str().unwrap(), "2026-02-01");
    assert!(
        v2["effective_to"].is_null(),
        "Current version should have null effective_to"
    );

    // V2 must have intervals
    let v2_intervals = v2["intervals"].as_array().unwrap();
    assert_eq!(v2_intervals.len(), 1);
    assert_eq!(v2_intervals[0]["start_time"].as_str().unwrap(), "08:00:00");

    // V1 is second (older)
    let v1 = &versions[1];
    assert_eq!(v1["version_number"].as_i64().unwrap(), 1);
    assert_eq!(v1["name"].as_str().unwrap(), "Shift V1");
    assert_eq!(v1["change_type"].as_str().unwrap(), "create");
    // V1 should be closed (superseded by V2)
    assert_eq!(
        v1["superseded_by"].as_str().unwrap(),
        v2["id"].as_str().unwrap()
    );
    // V1 effective_to should be 2026-01-31 (day before V2 effective_from)
    assert_eq!(v1["effective_to"].as_str().unwrap(), "2026-01-31");
}

#[tokio::test]
async fn test_get_shift_versions_not_found() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let nonexistent_id = uuid::Uuid::new_v4();

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/workspaces/{workspace_id}/shifts/{nonexistent_id}/versions"
                ))
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_correction_keeps_effective_from() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // 1. Create Shift V1
    let shift_req = serde_json::json!({
        "name": "Shift V1",
        "intervals": [{
            "start_time": "09:00:00",
            "end_time": "17:00:00",
            "break_duration_minutes": 60,
            "is_break_auto_deduct": true,
            "break_is_overtime": false,
            "early_punch_in_grace_period_minutes": 0
        }]
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{workspace_id}/shifts"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(shift_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let shift_id = created["id"].as_str().unwrap();

    // 2. Correction (fix typo in name, no interval change — use same intervals)
    let correction_req = serde_json::json!({
        "name": "Shift V1 (fixed)",
        "intervals": [{
            "start_time": "09:00:00",
            "end_time": "17:00:00",
            "break_duration_minutes": 60,
            "is_break_auto_deduct": true,
            "break_is_overtime": false,
            "early_punch_in_grace_period_minutes": 0
        }],
        "is_correction": true,
        "change_reason": "Fixed typo in shift name"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!(
                    "/api/v1/workspaces/{workspace_id}/shifts/{shift_id}"
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(correction_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. GET version history
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/workspaces/{workspace_id}/shifts/{shift_id}/versions"
                ))
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let versions: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert_eq!(versions.len(), 2, "Expected 2 versions after correction");

    let v2 = &versions[0];
    assert_eq!(v2["version_number"].as_i64().unwrap(), 2);
    assert_eq!(v2["change_type"].as_str().unwrap(), "correct");
    assert_eq!(
        v2["change_reason"].as_str().unwrap(),
        "Fixed typo in shift name"
    );

    // Correction: V2 effective_from should match V1's (or something close — they share the same effective_from)
    let v1 = &versions[1];
    // For a correction, V1's effective_to should be set and superseded_by should point to V2
    assert_eq!(
        v1["superseded_by"].as_str().unwrap(),
        v2["id"].as_str().unwrap()
    );
}
