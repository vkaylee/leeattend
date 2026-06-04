//! Integration tests for Employee Working Conditions API

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// Role IDs from seeded system roles
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
async fn test_working_conditions_crud_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let create_payload = json!({
        "type_": "maternity_leave",
        "minute_allowance": 480,
        "is_paid": true,
        "start_date": "2026-03-01",
        "end_date": "2026-06-01"
    });

    // 1. Create working condition
    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/conditions"),
            ws_id,
            user_id,
            Some(create_payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Should create working condition"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let cond_id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["type_"], "maternity_leave");
    assert_eq!(created["minute_allowance"], 480);
    assert_eq!(created["is_paid"], true);

    // 2. List working conditions
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/conditions"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let paginated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let conditions = paginated["data"].as_array().unwrap().clone();
    assert_eq!(conditions.len(), 1, "Should have exactly one condition");
    assert_eq!(conditions[0]["id"], cond_id);

    // 3. Delete working condition
    let response = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/conditions/{cond_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NO_CONTENT,
        "Should delete working condition"
    );

    // 4. Verify empty after deletion
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/conditions"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let paginated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let conditions = paginated["data"].as_array().unwrap().clone();
    assert_eq!(
        conditions.len(),
        0,
        "Should have no conditions after deletion"
    );
}

#[tokio::test]
async fn test_working_conditions_permission_enforcement() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().with_email("owner@wc.test").create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Unprivileged user (Employee role — no EmployeesUpdate)
    let regular_user_id = factory.user().with_email("regular@wc.test").create().await;
    factory
        .add_member(
            ws_id,
            regular_user_id,
            Uuid::parse_str(EMPLOYEE_ROLE_ID).unwrap(),
        )
        .await;

    let create_payload = json!({
        "type_": "paternity_leave",
        "minute_allowance": 240,
        "is_paid": false,
        "start_date": "2026-04-01"
    });

    // Regular employee should NOT be able to create a working condition
    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/conditions"),
            ws_id,
            regular_user_id,
            Some(create_payload.clone()),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Employee role must not create working conditions"
    );

    // Owner CAN create it
    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/conditions"),
            ws_id,
            owner_id,
            Some(create_payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Owner must be able to create working conditions"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let cond_id = created["id"].as_str().unwrap().to_string();

    // Regular employee cannot DELETE either
    let response = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/conditions/{cond_id}"),
            ws_id,
            regular_user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Employee role must not delete working conditions"
    );
}

#[tokio::test]
async fn test_working_condition_for_nonexistent_employee_returns_not_found() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().with_email("owner@wc2.test").create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;
    let fake_emp_id = Uuid::new_v4();

    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{fake_emp_id}/conditions"),
            ws_id,
            owner_id,
            Some(json!({
                "type_": "parental_leave",
                "minute_allowance": 120,
                "is_paid": true,
                "start_date": "2026-05-01"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Non-existent employee should return 404"
    );
}
