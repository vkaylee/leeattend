//! Integration tests for Leave Types API

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

static EMPLOYEE_ROLE_ID: &str = "a0000000-0000-0000-0000-000000000006";

fn make_req(
    method: &str,
    uri: &str,
    ws_id: Uuid,
    user_id: Uuid,
    body: Option<serde_json::Value>,
) -> axum::http::Request<axum::body::Body> {
    let builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if let Some(b) = body {
        builder
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(serde_json::to_string(&b).unwrap()))
            .unwrap()
    } else {
        builder.body(axum::body::Body::empty()).unwrap()
    }
}

#[tokio::test]
async fn test_leave_types_crud_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory
        .user()
        .with_email("owner@leavetypes.test")
        .create()
        .await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // 1. List leave types — workspace factory seeds 4 default ones
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/workspaces/{ws_id}/leave/types"),
            ws_id,
            owner_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let types: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(
        types.len() >= 4,
        "Should have at least 4 seeded leave types"
    );

    // 2. Create a new leave type
    let create_payload = json!({
        "code": "study",
        "name": "Study Leave",
        "is_paid": true,
        "policy_config": {}
    });

    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/workspaces/{ws_id}/leave/types"),
            ws_id,
            owner_id,
            Some(create_payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Should create new leave type"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let lt_id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["code"], "study");
    assert_eq!(created["name"], "Study Leave");
    assert_eq!(created["is_paid"], true);

    // 3. Update the leave type config
    let update_payload = json!({
        "policy_config": {
            "max_carry_over_days": 5,
            "requires_approval": true
        }
    });

    let response = app
        .clone()
        .oneshot(make_req(
            "PUT",
            &format!("/api/workspaces/{ws_id}/leave/types/{lt_id}/config"),
            ws_id,
            owner_id,
            Some(update_payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should update leave type config"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["id"], lt_id);

    // 4. Delete the new leave type (no balances associated)
    let response = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/workspaces/{ws_id}/leave/types/{lt_id}"),
            ws_id,
            owner_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NO_CONTENT,
        "Should delete leave type with no balances"
    );

    // 5. Verify list is back to original count
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/workspaces/{ws_id}/leave/types"),
            ws_id,
            owner_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let types_after: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        types_after.len(),
        types.len(),
        "Should be back to original count after deletion"
    );
}

#[tokio::test]
async fn test_leave_type_delete_blocked_by_balance() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory
        .user()
        .with_email("owner@leavetypes2.test")
        .create()
        .await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Get vacation leave type id (seeded)
    let vacation_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
    )
    .bind(ws_id)
    .fetch_one(&factory.pool)
    .await
    .unwrap();

    // Create a balance for this leave type
    factory
        .leave_balance()
        .with_employee(emp_id)
        .with_leave_type(vacation_id)
        .create()
        .await;

    // Attempt to delete the vacation leave type (should fail — has balances)
    let response = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/workspaces/{ws_id}/leave/types/{vacation_id}"),
            ws_id,
            owner_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::CONFLICT,
        "Should not delete leave type that has employee balances"
    );
}

#[tokio::test]
async fn test_leave_types_permission_enforcement() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory
        .user()
        .with_email("owner@leavetypes3.test")
        .create()
        .await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Unprivileged user
    let regular_user_id = factory
        .user()
        .with_email("regular@leavetypes3.test")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            regular_user_id,
            Uuid::parse_str(EMPLOYEE_ROLE_ID).unwrap(),
        )
        .await;

    let create_payload = json!({
        "code": "unauthorized",
        "name": "Unauthorized Type",
        "is_paid": false
    });

    // Employee role cannot create leave types
    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/workspaces/{ws_id}/leave/types"),
            ws_id,
            regular_user_id,
            Some(create_payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Employee role must not create leave types"
    );
}
