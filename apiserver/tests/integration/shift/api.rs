//! Shift API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::Utc;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_shift_api_lifecycle_smoke() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // 1. Create Shift
    let shift_req = serde_json::json!({
        "name": "Night Shift",
        "intervals": [{
            "start_time": "22:00:00",
            "end_time": "06:00:00",
            "break_duration_minutes": 30,
            "is_break_auto_deduct": true,
            "break_is_overtime": false,
            "early_punch_in_grace_period_minutes": 15
        }]
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{ws_id}/shifts"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::from(shift_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let shift: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let shift_id = shift["id"].as_str().unwrap();

    // 2. List Shifts
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{ws_id}/shifts"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. Assign Shift
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        emp_id
    );
    let assign_req = serde_json::json!({
        "employment_id": employment_id,
        "shift_id": shift_id,
        "start_date": Utc::now().date_naive().to_string()
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{ws_id}/shifts/assign"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::from(assign_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
