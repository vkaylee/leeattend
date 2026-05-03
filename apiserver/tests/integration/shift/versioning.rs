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
        "start_time": "09:00:00",
        "end_time": "17:00:00",
        "break_duration_minutes": 60,
        "is_break_auto_deduct": true,
        "break_is_overtime": false,
        "early_punch_in_grace_period_minutes": 0
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{workspace_id}/shifts"))
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
        "start_time": "08:00:00", // Changed
        "end_time": "16:00:00",
        "break_duration_minutes": 60,
        "is_break_auto_deduct": true,
        "break_is_overtime": false,
        "early_punch_in_grace_period_minutes": 0,
        "is_correction": false, // New Version
        "effective_date": "2026-02-01"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!("/api/workspaces/{workspace_id}/shifts/{shift_id}"))
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
