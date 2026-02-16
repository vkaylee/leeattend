//! Attendance Adjustment API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use apiserver::models::{AttendanceAdjustmentRequest, AdjustmentStatus};
use axum::http::StatusCode;
use chrono::Utc;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_attendance_adjustment_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) = setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // 1. Create Adjustment
    let req_body = serde_json::json!({
        "employee_id": emp_id,
        "requested_punch_in_time": Utc::now().to_rfc3339(),
        "is_break_overtime": false,
        "reason": "Forgot punch"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{ws_id}/attendance/adjustments"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(req_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Verify creation
    let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
    let adjustment: AttendanceAdjustmentRequest = serde_json::from_slice(&body).unwrap();
    assert_eq!(adjustment.status, AdjustmentStatus::Pending);
}
