//! Leave API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{
    ApproveLeaveRequest, EmployeeLeaveBalance, LeaveRequest, LeaveStatus, PaginatedResponse,
    SubmitLeaveRequest, WorkspaceLeaveBalance,
};
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_leave_request_lifecycle_smoke() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let emp_id: Uuid = factory.employee().with_workspace(ws_id).create().await;

    // 1. Get default leave type (Vacation)
    let vacation_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        ws_id
    )
    .fetch_one(&factory.pool)
    .await
    .unwrap();

    // 2. Submit Leave Request
    let submit_req = SubmitLeaveRequest {
        employee_id: emp_id,
        leave_type_id: vacation_id,
        start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(), // 3 days
        reason: Some("Vacation".to_string()),
        duration_type: apiserver::models::enums::LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
    };

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{ws_id}/leave/submit"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&submit_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let leave_req: LeaveRequest = serde_json::from_slice(&body).unwrap();
    assert_eq!(leave_req.status, LeaveStatus::PreApproved);

    // 3. Approve Request
    let approve_req = ApproveLeaveRequest {
        status: LeaveStatus::Approved,
    };

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{}/leave/requests/{}/approve",
                    ws_id, leave_req.id
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&approve_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 4. Verify Balance Deducted (Assuming initial balance logic or unlimited for test)
    // In this test environment, we might need to manually check or ensure balance exists.
    // Testing deduction requires existing balance usually.
}

#[tokio::test]
async fn test_leave_balances_smoke() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let emp_id: Uuid = factory.employee().with_workspace(ws_id).create().await;

    // Create a balance manually
    let vacation_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        ws_id
    )
    .fetch_one(&factory.pool)
    .await
    .unwrap();

    factory
        .leave_balance()
        .with_employee(emp_id)
        .with_leave_type(vacation_id)
        .with_entitlement(12)
        .create()
        .await;

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{ws_id}/employees/{emp_id}/leave/balances"
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
    let balances: Vec<EmployeeLeaveBalance> = serde_json::from_slice(&body).unwrap();
    assert!(!balances.is_empty());
}

#[tokio::test]
async fn test_workspace_leave_balances_filtering() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // emp_1 will have a balance
    let emp_1: Uuid = factory.employee().with_workspace(ws_id).create().await;
    // emp_2 will NOT have a balance and NO policy
    let emp_2: Uuid = factory.employee().with_workspace(ws_id).create().await;

    let vacation_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        ws_id
    )
    .fetch_one(&factory.pool)
    .await
    .unwrap();

    // Give emp_1 a balance
    factory
        .leave_balance()
        .with_employee(emp_1)
        .with_leave_type(vacation_id)
        .with_entitlement(12)
        .create()
        .await;

    // Request the workspace balances
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{ws_id}/leave/balances"))
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

    let res: PaginatedResponse<WorkspaceLeaveBalance> = serde_json::from_slice(&body).unwrap();

    // Verify emp_1 is included
    assert!(res.data.iter().any(|b| b.employee_id == emp_1));
    // Verify emp_2 is filtered out (has no balance or policy)
    assert!(!res.data.iter().any(|b| b.employee_id == emp_2));
}
