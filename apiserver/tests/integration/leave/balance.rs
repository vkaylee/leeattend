//! Integration tests for leave balances CRUD

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::Datelike;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

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
async fn test_leave_balances_endpoints() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    // Setup: owner, workspace
    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Create an employee
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Get Vacation leave type id
    let vacation_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    // Give employee some initial balance so it shows up in queries
    let current_year = chrono::Utc::now().year();
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO leave_balances (id, employee_id, leave_type_id, year, entitlement, carried_over, accrued_amount, updated_at) 
         VALUES ($1, $2, $3, $4, 10, 0, 0, NOW())",
        diesel::sql_types::Uuid,
        uuid::Uuid::now_v7(),
        diesel::sql_types::Uuid,
        emp_id,
        diesel::sql_types::Uuid,
        vacation_id,
        diesel::sql_types::Integer,
        current_year
    );

    // 1. Get workspace leave balances
    let response = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/leave/balances"),
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
    let ws_balances: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    let items = ws_balances["data"].as_array().unwrap();
    assert!(
        !items.is_empty(),
        "Expected workspace balances to not be empty"
    );

    // 2. Get employee leave balances
    let response = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/leave/balances"),
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
    let emp_balances: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    // this endpoint returns `Json(balances)` which is `Vec<EmployeeLeaveBalanceResponse>`
    let items = emp_balances.as_array().unwrap();
    assert!(
        !items.is_empty(),
        "Expected employee balances to not be empty"
    );

    // 3. Adjust leave balance
    let adjust_req = serde_json::json!({
        "leave_type_id": vacation_id,
        "amount": 5.0,
        "reason": "Bonus",
        "transaction_date": chrono::Utc::now().naive_utc().date().to_string()
    });

    let response = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/leave/adjust"),
            ws_id,
            owner_id,
            Some(adjust_req.to_string()),
        ))
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    if status != StatusCode::OK {
        panic!("Failed to adjust balance: {}, {}", status, body_str);
    }

    // Check that adjustment updated the balance
    let response = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/leave/balances"),
            ws_id,
            owner_id,
            None,
        ))
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    let emp_balances: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    let items = emp_balances.as_array().unwrap();
    let vacation_balance = items
        .iter()
        .find(|i| i["leave_type_id"].as_str().unwrap() == vacation_id.to_string())
        .unwrap();
    // Should be 10.0 + 5.0 = 15.0
    // Actually `remaining` field in `EmployeeLeaveBalanceResponse` could be parsed as f64
    assert_eq!(vacation_balance["remaining"].as_f64().unwrap(), 15.0);
}
