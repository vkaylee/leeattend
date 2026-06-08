//! Integration tests for leave requests CRUD

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{LeaveRequest, LeaveStatus, SubmitLeaveRequest};
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

#[tokio::test]
async fn test_leave_requests_crud() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    // Setup: owner, workspace
    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Create an employee (has user_id linked implicitly or explicitly)
    let emp_user_id = factory.user().with_email("emp_req@test.com").create().await;
    factory
        .add_member(
            ws_id,
            emp_user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(), // member role
        )
        .await;

    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(emp_user_id)
        .create()
        .await;

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    // Get Vacation leave type id
    let vacation_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    // 1. Submit Request (as Owner for employee)
    let submit_req = SubmitLeaveRequest {
        employment_id,
        leave_type_id: vacation_id,
        start_date: chrono::NaiveDate::from_ymd_opt(2027, 8, 1).unwrap(),
        end_date: chrono::NaiveDate::from_ymd_opt(2027, 8, 2).unwrap(),
        reason: Some("Family vacation".to_string()),
        duration_type: apiserver::models::enums::LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
    };

    let response = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/leave/submit"),
            ws_id,
            owner_id, // Owner submitting
            Some(serde_json::to_string(&submit_req).unwrap()),
        ))
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    if status != StatusCode::CREATED {
        panic!("Failed to submit request: {}, {}", status, body_str);
    }

    let created: LeaveRequest = serde_json::from_str(&body_str).unwrap();
    let request_id = created.id();
    // Auto approved since owner created it
    assert_eq!(created.status(), LeaveStatus::PreApproved);

    // 2. Get my leave requests (as employee)
    let response = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/employees/{emp_id}/leave/requests"),
            ws_id,
            emp_user_id,
            None,
        ))
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    if status != StatusCode::OK {
        panic!("Failed to get my requests: {}, {}", status, body_str);
    }
    let my_requests: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    let items = my_requests.as_array().unwrap();
    assert!(!items.is_empty(), "Expected my requests to not be empty");

    // 3. Get workspace leave requests (as owner)
    let response = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/leave/requests"),
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
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    let ws_requests: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    let items = ws_requests.as_array().unwrap();
    assert!(
        !items.is_empty(),
        "Expected workspace requests to not be empty"
    );

    // 4. Approve request (as owner)
    let approve_req = serde_json::json!({
        "status": "approved",
        "manager_comment": "Have fun!"
    });

    let response = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/leave/requests/{request_id}/approve"),
            ws_id,
            owner_id,
            Some(approve_req.to_string()),
        ))
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    if status != StatusCode::OK {
        panic!("Failed to approve request: {}, {}", status, body_str);
    }
}
