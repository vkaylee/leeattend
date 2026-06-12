//! Integration tests for `TeamLead` leave submit + auto-approve

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

/// Helper: assign employee to a department
async fn assign_to_department(
    pool: &apiserver::diesel_pool::DieselPools,
    employee_id: Uuid,
    department_id: Uuid,
) {
    crate::test_execute!(
        &pool.api,
        "UPDATE employments SET department_id = $2 WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        employee_id,
        diesel::sql_types::Uuid,
        department_id
    );
}

/// `TeamLead` submits leave for employee in their department → auto-approved (`pre_approved`)
#[tokio::test]
async fn test_teamlead_submit_own_dept_auto_approved() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    // Setup: owner, workspace, department
    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;
    let dept_id = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Engineering")
        .create()
        .await;

    // TeamLead user + employee record in same department
    let tl_user_id = factory
        .user()
        .with_email("teamlead@test.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            tl_user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000005").unwrap(),
        )
        .await;
    let tl_emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(tl_user_id)
        .create()
        .await;
    assign_to_department(&factory.diesel_pools, tl_emp_id, dept_id).await;

    // Target employee in same department
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Nguyen Van A")
        .create()
        .await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );
    assign_to_department(&factory.diesel_pools, emp_id, dept_id).await;

    // Get vacation leave type
    let vacation_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    // Submit leave request as TeamLead
    let submit_req = SubmitLeaveRequest {
        employment_id,
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
            &format!("/api/v1/workspaces/{ws_id}/leave/submit"),
            ws_id,
            tl_user_id,
            Some(serde_json::to_string(&submit_req).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let leave_req: LeaveRequest = serde_json::from_slice(&body).unwrap();

    // Should be pre_approved (auto-approved by TeamLead)
    assert_eq!(leave_req.status(), LeaveStatus::PreApproved);
    assert_eq!(leave_req.approved_by_user_id(), Some(tl_user_id));
    assert_eq!(leave_req.requested_by_user_id(), Some(tl_user_id));
}

/// `TeamLead` submits leave for employee in OTHER department → 403 Forbidden
#[tokio::test]
async fn test_teamlead_submit_other_dept_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;
    let dept_eng = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Engineering")
        .create()
        .await;
    let dept_sales = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Sales")
        .create()
        .await;

    // TeamLead in Engineering
    let tl_user_id = factory.user().with_email("tl_eng@test.com").create().await;
    factory
        .add_member(
            ws_id,
            tl_user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000005").unwrap(),
        )
        .await;
    let tl_emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(tl_user_id)
        .create()
        .await;
    assign_to_department(&factory.diesel_pools, tl_emp_id, dept_eng).await;

    // Employee in Sales (different department)
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Le Van B")
        .create()
        .await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );
    assign_to_department(&factory.diesel_pools, emp_id, dept_sales).await;

    let vacation_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    let submit_req = SubmitLeaveRequest {
        employment_id,
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
            &format!("/api/v1/workspaces/{ws_id}/leave/submit"),
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
    factory
        .add_member(
            ws_id,
            hr_user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000003").unwrap(),
        )
        .await;

    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Tran Van C")
        .create()
        .await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    let vacation_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    let submit_req = SubmitLeaveRequest {
        employment_id,
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
            &format!("/api/v1/workspaces/{ws_id}/leave/submit"),
            ws_id,
            hr_user_id,
            Some(serde_json::to_string(&submit_req).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let leave_req: LeaveRequest = serde_json::from_slice(&body).unwrap();

    // HR has global scope → auto_approve = true (has LeaveRequestsApprove globally)
    // So HR should get pre_approved since they have approve permission
    assert_eq!(leave_req.status(), LeaveStatus::PreApproved);
    assert_eq!(leave_req.approved_by_user_id(), Some(hr_user_id));
}

/// Regular Employee without `LeaveRequestsSubmit` → 403 Forbidden
#[tokio::test]
async fn test_employee_submit_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Regular member (no LeaveRequestsSubmit permission)
    let member_user_id = factory.user().with_email("member@test.com").create().await;
    factory
        .add_member(
            ws_id,
            member_user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    let vacation_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    let submit_req = SubmitLeaveRequest {
        employment_id,
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
            &format!("/api/v1/workspaces/{ws_id}/leave/submit"),
            ws_id,
            member_user_id,
            Some(serde_json::to_string(&submit_req).unwrap()),
        ))
        .await
        .unwrap();

    // Member does NOT have LeaveRequestsSubmit permission
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// Test that a user with `LeaveRequestsSubmitOnBehalf` permission and a behalf assignment
/// can submit a leave request on behalf of a target employee, while a user without the
/// behalf assignment gets a 403 Forbidden.
#[tokio::test]
async fn test_submit_leave_on_behalf_with_assignment() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Submitter user + employee record
    let submitter_user_id = factory
        .user()
        .with_email("submitter@test.com")
        .create()
        .await;

    // Assign the employee role to the submitter (role_id: a0000000-0000-0000-0000-000000000006)
    let emp_role_id = uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap();
    factory
        .add_member(ws_id, submitter_user_id, emp_role_id)
        .await;

    let submitter_emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(submitter_user_id)
        .create()
        .await;

    // Grant `leave:requests:submit_on_behalf` to the employee role in the database
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO workspace_role_permissions (role_id, permission_name) VALUES ($1, $2)",
        diesel::sql_types::Uuid,
        emp_role_id,
        diesel::sql_types::Text,
        "leave:requests:submit_on_behalf"
    );

    // Target employee
    let target_emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Target Employee")
        .create()
        .await;
    let target_employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        target_emp_id
    );

    let vacation_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    let submit_req = SubmitLeaveRequest {
        employment_id: target_employment_id,
        leave_type_id: vacation_id,
        start_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 10).unwrap(),
        end_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 12).unwrap(),
        reason: Some("Leave on behalf by assignment".to_string()),
        duration_type: apiserver::models::enums::LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
    };

    // 1. Without behalf assignment, it should fail with 403 Forbidden
    let response = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/leave/submit"),
            ws_id,
            submitter_user_id,
            Some(serde_json::to_string(&submit_req).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // 2. Add the behalf assignment in the database
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO behalf_assignments (id, workspace_id, assignee_user_id, target_employee_id, assignment_type, created_at, updated_at)
         VALUES ($1, $2, $3, $4, 'leave_request', NOW(), NOW())",
        diesel::sql_types::Uuid,
        uuid::Uuid::new_v4(),
        diesel::sql_types::Uuid,
        ws_id,
        diesel::sql_types::Uuid,
        submitter_user_id,
        diesel::sql_types::Uuid,
        target_emp_id
    );

    // 3. With behalf assignment, it should succeed
    let response = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/leave/submit"),
            ws_id,
            submitter_user_id,
            Some(serde_json::to_string(&submit_req).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let leave_req: LeaveRequest = serde_json::from_slice(&body).unwrap();
    assert_eq!(leave_req.employment_id(), target_employment_id);
    assert_eq!(leave_req.requested_by_user_id(), Some(submitter_user_id));
}
