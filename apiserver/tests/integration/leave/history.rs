//! Leave history and adjustment tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{AdjustBalanceRequest, LeaveHistoryItem, LeaveTransactionType};
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::NaiveDate;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_leave_adjustment() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let emp_id: Uuid = factory.employee().with_workspace(ws_id).create().await;

    let vacation_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    // Perform Adjustment (+5 days)
    let adjust_req = AdjustBalanceRequest {
        leave_type_id: vacation_id,
        amount: 5.0,
        reason: "Initial Balance".to_string(),
        transaction_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
    };

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/employees/{emp_id}/leave/adjust"
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&adjust_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify History
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/employees/{emp_id}/leave/history"
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
    let history: Vec<LeaveHistoryItem> = serde_json::from_slice(&body).unwrap();

    assert_eq!(history.len(), 1);
    assert_eq!(
        history[0].transaction_type,
        LeaveTransactionType::Adjustment
    );
    assert_eq!(history[0].amount, 5.0);
}
