//! Leave Policy API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

fn req(
    method: &str,
    uri: &str,
    ws_id: Uuid,
    user_id: Uuid,
    body: Option<serde_json::Value>,
) -> axum::http::Request<axum::body::Body> {
    let mut builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if let Some(b) = body {
        builder = builder.header("Content-Type", "application/json");
        builder.body(axum::body::Body::from(b.to_string())).unwrap()
    } else {
        builder.body(axum::body::Body::empty()).unwrap()
    }
}

async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn test_leave_policy_crud() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Get a leave type
    let vacation_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    // 1. Create Policy
    let create_req = serde_json::json!({
        "name": "Standard Vacation Policy",
        "description": "Standard 15 days",
        "leave_type_id": vacation_id,
        "rule_config": {
            "frequency": "monthly",
            "accrual_amount": 1.25,
            "accrual_timing": "start_of_period"
        }
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/leave/policies"),
            ws_id,
            admin_id,
            Some(create_req),
        ))
        .await
        .unwrap();

    let status = resp.status();
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    if status != StatusCode::CREATED {
        panic!(
            "Failed to create policy. Status: {}, body: {}",
            status, body_str
        );
    }
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let policy_id = created["id"].as_str().unwrap().to_string();

    // 2. Update Policy
    let update_req = serde_json::json!({
        "name": "Updated Vacation Policy"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/workspaces/{ws_id}/leave/policies/{policy_id}"),
            ws_id,
            admin_id,
            Some(update_req),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let updated = body_json(resp).await;
    assert_eq!(updated["name"], "Updated Vacation Policy");

    // 2.5 Get Policies
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/leave/policies"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let policies = body_json(resp).await;
    let items = policies.as_array().unwrap();
    assert!(!items.is_empty());

    // 3. Assign Leave Policy
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let assign_req = serde_json::json!({
        "policy_id": policy_id,
        "assignee_id": emp_id,
        "assignee_type": "employee",
        "effective_from": "2024-01-01"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/leave/policies/assign"),
            ws_id,
            admin_id,
            Some(assign_req),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);

    // 4. Get Assignments
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/leave/policies/assignments?policy_id={policy_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let assignments = body_json(resp).await;
    assert!(!assignments.as_array().unwrap().is_empty());

    // 3. Delete Policy
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/leave/policies/{policy_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_leave_policy_assignment_deletion_and_healing() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Get a leave type
    let vacation_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    // 1. Create Policy definition
    let create_req = serde_json::json!({
        "name": "Standard Vacation Policy",
        "description": "Standard 15 days",
        "leave_type_id": vacation_id,
        "rule_config": {
            "frequency": "monthly",
            "accrual_amount": 1.25,
            "accrual_timing": "start_of_period"
        }
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/leave/policies"),
            ws_id,
            admin_id,
            Some(create_req),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = body_json(resp).await;
    let policy_id = created["id"].as_str().unwrap().to_string();

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // 2. Assign Policy 1 starting Jan 1st (forever)
    let assign_req1 = serde_json::json!({
        "policy_id": policy_id,
        "assignee_id": emp_id,
        "assignee_type": "employee",
        "effective_from": "2024-01-01"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/leave/policies/assign"),
            ws_id,
            admin_id,
            Some(assign_req1),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let assignment1 = body_json(resp).await;
    let assignment1_id = assignment1["id"].as_str().unwrap().to_string();

    // 3. Assign same policy starting June 1st (forever) - this should auto-close Policy 1 to end on May 31st
    let assign_req2 = serde_json::json!({
        "policy_id": policy_id,
        "assignee_id": emp_id,
        "assignee_type": "employee",
        "effective_from": "2024-06-01"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/leave/policies/assign"),
            ws_id,
            admin_id,
            Some(assign_req2),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let assignment2 = body_json(resp).await;
    let assignment2_id = assignment2["id"].as_str().unwrap().to_string();

    // 4. Verify assignment 1 is closed on May 31st
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/leave/policies/assignments"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let assignments = body_json(resp).await;
    let items = assignments.as_array().unwrap();

    let a1 = items
        .iter()
        .find(|i| i["id"].as_str().unwrap() == assignment1_id)
        .unwrap();
    assert_eq!(a1["effective_to"].as_str(), Some("2024-05-31"));

    // 5. Delete Assignment 2
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/leave/policies/assignments/{assignment2_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 6. Verify Assignment 2 is deleted and Assignment 1 is healed (effective_to is back to NULL/forever)
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/leave/policies/assignments"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let assignments = body_json(resp).await;
    let items = assignments.as_array().unwrap();

    // Assignment 2 should be gone (soft-deleted)
    assert!(items
        .iter()
        .all(|i| i["id"].as_str().unwrap() != assignment2_id));

    // Assignment 1 should have effective_to as null (forever)
    let a1_updated = items
        .iter()
        .find(|i| i["id"].as_str().unwrap() == assignment1_id)
        .unwrap();
    assert!(a1_updated["effective_to"].is_null());
}
