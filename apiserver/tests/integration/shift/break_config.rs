//! Shift break configuration tests

use crate::integration::common::app::setup_app_tower;
use crate::integration::common::db::TestDatabase;
use crate::integration::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_create_shift_with_fixed_break() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) = setup_app_tower().await;

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    let shift_req = serde_json::json!({
        "name": "Fixed Break Shift",
        "start_time": "08:00:00",
        "end_time": "17:00:00",
        "break_duration_minutes": 60,
        "break_start": "12:00:00",
        "break_end": "13:00:00",
        "is_break_auto_deduct": true,
        "break_is_overtime": false,
        "early_punch_in_grace_period_minutes": 15
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/shifts", workspace_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(shift_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
    let created_shift: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(created_shift["name"], "Fixed Break Shift");
    assert_eq!(created_shift["break_start"], "12:00:00"); 
}

#[tokio::test]
async fn test_create_shift_with_flexible_break() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) = setup_app_tower().await;

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    let shift_req = serde_json::json!({
        "name": "Flexible Break Shift",
        "start_time": "09:00:00",
        "end_time": "18:00:00",
        "break_duration_minutes": 45,
        "break_is_overtime": false,
        "early_punch_in_grace_period_minutes": 10
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/shifts", workspace_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(shift_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}
