//! Leave types API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// Helper to build a request with standard workspace + user headers
fn req(
    method: &str,
    uri: &str,
    ws_id: Uuid,
    user_id: Uuid,
    body: Option<String>,
) -> axum::http::Request<axum::body::Body> {
    let mut builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }

    builder
        .body(match body {
            Some(b) => axum::body::Body::from(b),
            None => axum::body::Body::empty(),
        })
        .unwrap()
}

async fn body_json(resp: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 10000).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_leave_type_crud_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // 1. Create leave type
    let create_body = serde_json::json!({
        "code": "compassionate",
        "name": "Compassionate Leave",
        "is_paid": true,
        "policy_config": {
            "requires_document": true,
            "allow_negative_balance": false,
            "approval_required": true
        }
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/leave/types"),
            ws_id,
            admin_id,
            Some(create_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = body_json(resp).await;
    let type_id = created["id"].as_str().unwrap();
    assert_eq!(created["code"].as_str().unwrap(), "compassionate");
    assert_eq!(created["name"].as_str().unwrap(), "Compassionate Leave");
    assert!(created["is_paid"].as_bool().unwrap());

    // 2. List leave types — should contain our new type
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/leave/types"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list = body_json(resp).await;
    let types = list.as_array().unwrap();
    assert!(types
        .iter()
        .any(|t| t["code"].as_str().unwrap() == "compassionate"));

    // 3. Update config
    let update_body = serde_json::json!({
        "policy_config": {
            "requires_document": false,
            "allow_negative_balance": true,
            "negative_balance_limit": 5.0,
            "approval_required": false
        }
    });

    let resp = app
        .clone()
        .oneshot(req(
            "PUT",
            &format!("/api/workspaces/{ws_id}/leave/types/{type_id}/config"),
            ws_id,
            admin_id,
            Some(update_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 4. Delete leave type
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/workspaces/{ws_id}/leave/types/{type_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_leave_type_delete_conflict_when_in_use() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Create a leave type
    let create_body = serde_json::json!({
        "code": "maternity",
        "name": "Maternity Leave",
        "is_paid": true
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/leave/types"),
            ws_id,
            admin_id,
            Some(create_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = body_json(resp).await;
    let type_id: Uuid = created["id"].as_str().unwrap().parse().unwrap();

    // Create an employee and a leave balance referencing this type
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    factory
        .leave_balance()
        .with_employee(emp_id)
        .with_leave_type(type_id)
        .with_entitlement(12)
        .create()
        .await;

    // Attempt to delete — should fail with 409 Conflict
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/workspaces/{ws_id}/leave/types/{type_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_leave_type_rbac_member_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Create a member without LeaveTypesManage permission
    let member_id = factory
        .user()
        .with_email("member@example.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            member_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    // Member tries to create a leave type — should get 403
    let create_body = serde_json::json!({
        "code": "personal",
        "name": "Personal Leave",
        "is_paid": false
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/leave/types"),
            ws_id,
            member_id,
            Some(create_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
