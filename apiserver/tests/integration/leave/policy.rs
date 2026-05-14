//! Leave policy API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_leave_policy_api_lifecycle() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(admin_id).await;

    // 1. Create Policy
    let vacation_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
    )
    .bind(ws_id)
    .fetch_one(&test_db.pool())
    .await
    .unwrap();

    let policy_req = serde_json::json!({
        "name": "Standard Annual Leave",
        "description": "Standard policy for all employees",
        "leave_type_id": vacation_id,
        "rule_config": {
            "frequency": "monthly",
            "accrual_amount": 1.25,
            "accrual_timing": "start_of_period"
        }
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{ws_id}/leave/policies"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::from(policy_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let policy: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 10000)
            .await
            .unwrap(),
    )
    .unwrap();
    let policy_id = policy["id"].as_str().unwrap();

    // 2. Assign Policy
    let assign_req = serde_json::json!({
        "policy_id": policy_id,
        "assignee_type": "workspace",
        "assignee_id": null,
        "effective_from": "2026-01-01"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{ws_id}/leave/policies/assign"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::from(assign_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // 3. Delete Policy
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/workspaces/{ws_id}/leave/policies/{policy_id}"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
