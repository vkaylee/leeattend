//! Integration tests for TeamLead leave submit + auto-approve

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{LeaveRequest, LeaveStatus, MemberRole, SubmitLeaveRequest};
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

/// Helper: assign employee to a department
async fn assign_to_department(pool: &sqlx::PgPool, employee_id: Uuid, department_id: Uuid) {
    sqlx::query(
        "INSERT INTO employee_department_history (employee_id, department_id, effective_date)
         VALUES ($1, $2, CURRENT_DATE)",
    )
    .bind(employee_id)
    .bind(department_id)
    .execute(pool)
    .await
    .unwrap();
}

/// TeamLead submits leave for employee in their department → auto-approved (pre_approved)
#[tokio::test]
async fn test_teamlead_submit_own_dept_auto_approved() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    // Setup: owner, workspace, department
    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;
    let dept_id = factory.department().with_workspace(ws_id).with_name("Engineering").create().await;

    // TeamLead user + employee record in same department
    let tl_user_id = factory.user().with_email("teamlead@test.com").create().await;
    factory.add_member(ws_id, tl_user_id, MemberRole::TeamLead).await;
    let tl_emp_id = factory.employee().with_workspace(ws_id).with_user(tl_user_id).create().await;
    assign_to_department(&factory.pool, tl_emp_id, dept_id).await;

    // Target employee in same department
    let emp_id = factory.employee().with_workspace(ws_id).with_full_name("Nguyen Van A").create().await;
    assign_to_department(&factory.pool, emp_id, dept_id).await;

    // Get vacation leave type
    let vacation_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        ws_id
    )
    .fetch_one(&factory.pool)
    .await
    .unwrap();

    // Submit leave request as TeamLead
    let submit_req = SubmitLeaveRequest {
        employee_id: emp_id,
        leave_type_id: vacation_id,
        start_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        end_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 3).unwrap(),
        reason: Some("Pre-approved by TeamLead".to_string()),
        duration_type: apiserver::models::enums::LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
    };

    let response = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/leave/submit"),
            ws_id,
            tl_user_id,
            Some(serde_json::to_string(&submit_req).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let leave_req: LeaveRequest = serde_json::from_slice(&body).unwrap();

    // Should be pre_approved (auto-approved by TeamLead)
    assert_eq!(leave_req.status, LeaveStatus::PreApproved);
    assert_eq!(leave_req.approved_by_user_id, Some(tl_user_id));
    assert_eq!(leave_req.requested_by_user_id, Some(tl_user_id));
}

/// TeamLead submits leave for employee in OTHER department → 403 Forbidden
#[tokio::test]
async fn test_teamlead_submit_other_dept_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;
    let dept_eng = factory.department().with_workspace(ws_id).with_name("Engineering").create().await;
    let dept_sales = factory.department().with_workspace(ws_id).with_name("Sales").create().await;

    // TeamLead in Engineering
    let tl_user_id = factory.user().with_email("tl_eng@test.com").create().await;
    factory.add_member(ws_id, tl_user_id, MemberRole::TeamLead).await;
    let tl_emp_id = factory.employee().with_workspace(ws_id).with_user(tl_user_id).create().await;
    assign_to_department(&factory.pool, tl_emp_id, dept_eng).await;

    // Employee in Sales (different department)
    let emp_id = factory.employee().with_workspace(ws_id).with_full_name("Le Van B").create().await;
    assign_to_department(&factory.pool, emp_id, dept_sales).await;

    let vacation_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        ws_id
    )
    .fetch_one(&factory.pool)
    .await
    .unwrap();

    let submit_req = SubmitLeaveRequest {
        employee_id: emp_id,
        leave_type_id: vacation_id,
        start_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        end_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 3).unwrap(),
        reason: Some("Should fail".to_string()),
        duration_type: apiserver::models::enums::LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
    };

    let response = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/leave/submit"),
            ws_id,
            tl_user_id,
            Some(serde_json::to_string(&submit_req).unwrap()),
        ))
        .await
        .unwrap();

    // TeamLead cannot submit for employee outside their department
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// HR Manager submits leave → normal pending status (not auto-approved)
#[tokio::test]
async fn test_hr_submit_stays_pending() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // HR Manager (global scope — managed_department_ids = None)
    let hr_user_id = factory.user().with_email("hr@test.com").create().await;
    factory.add_member(ws_id, hr_user_id, MemberRole::HrManager).await;

    let emp_id = factory.employee().with_workspace(ws_id).with_full_name("Tran Van C").create().await;

    let vacation_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        ws_id
    )
    .fetch_one(&factory.pool)
    .await
    .unwrap();

    let submit_req = SubmitLeaveRequest {
        employee_id: emp_id,
        leave_type_id: vacation_id,
        start_date: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
        end_date: chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap(),
        reason: Some("HR submit on behalf".to_string()),
        duration_type: apiserver::models::enums::LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
    };

    let response = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/leave/submit"),
            ws_id,
            hr_user_id,
            Some(serde_json::to_string(&submit_req).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let leave_req: LeaveRequest = serde_json::from_slice(&body).unwrap();

    // HR has global scope → auto_approve = true (has LeaveRequestsApprove globally)
    // So HR should get pre_approved since they have approve permission
    assert_eq!(leave_req.status, LeaveStatus::PreApproved);
    assert_eq!(leave_req.approved_by_user_id, Some(hr_user_id));
}

/// Regular Employee without LeaveRequestsSubmit → 403 Forbidden
#[tokio::test]
async fn test_employee_submit_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Regular member (no LeaveRequestsSubmit permission)
    let member_user_id = factory.user().with_email("member@test.com").create().await;
    factory.add_member(ws_id, member_user_id, MemberRole::Member).await;

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let vacation_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        ws_id
    )
    .fetch_one(&factory.pool)
    .await
    .unwrap();

    let submit_req = SubmitLeaveRequest {
        employee_id: emp_id,
        leave_type_id: vacation_id,
        start_date: chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        end_date: chrono::NaiveDate::from_ymd_opt(2026, 8, 2).unwrap(),
        reason: Some("Should fail".to_string()),
        duration_type: apiserver::models::enums::LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
    };

    let response = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/leave/submit"),
            ws_id,
            member_user_id,
            Some(serde_json::to_string(&submit_req).unwrap()),
        ))
        .await
        .unwrap();

    // Member does NOT have LeaveRequestsSubmit permission
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
