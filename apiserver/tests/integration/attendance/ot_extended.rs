//! Additional tests for attendance overtime and summaries handlers

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::NaiveDate;
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

async fn body_json(resp: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 10000).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_get_attendance_summaries_empty() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/attendance/summaries?start_date=2026-01-01&end_date=2026-01-31"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let summaries = body_json(resp).await;
    assert!(summaries.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_get_attendance_summaries_for_employee() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/attendance/summaries?start_date=2026-01-01&end_date=2026-01-31&employee_id={emp_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let summaries = body_json(resp).await;
    assert!(summaries.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_claim_ot_employee_not_found() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let fake_id = Uuid::new_v4();
    let body = serde_json::json!({
        "date": "2026-01-15",
        "claimed_minutes": 60
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/attendance/claim-ot/{fake_id}"),
            ws_id,
            admin_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_approve_ot_employee_not_found() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let fake_id = Uuid::new_v4();
    let body = serde_json::json!({
        "date": "2026-01-15",
        "approved_minutes": 60,
        "status": "approved"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/attendance/approve-ot/{fake_id}"),
            ws_id,
            admin_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();
    // Not found since no such employee exists
    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn test_approve_ot_invalid_status() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let body = serde_json::json!({
        "date": "2026-01-15",
        "approved_minutes": 60,
        "status": "invalid_status"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/attendance/approve-ot/{emp_id}"),
            ws_id,
            admin_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();
    // Should return validation error
    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn test_approve_ot_rbac_member_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

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

    let body = serde_json::json!({
        "date": "2026-01-15",
        "approved_minutes": 60,
        "status": "approved"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/attendance/approve-ot/{emp_id}"),
            ws_id,
            member_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_claim_ot_delegated_success() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let date = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO attendance_daily_summaries (id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, ot_status, created_at, updated_at) VALUES ($1, $2, $3, $4, 500, 60, $5::ot_status_type, NOW(), NOW())",
        diesel::sql_types::Uuid, uuid::Uuid::new_v4(),
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, emp_id,
        diesel::sql_types::Date, date,
        diesel::sql_types::Text, "pending_employee"
    );

    let clerk_user_id = factory
        .user()
        .with_email("clerk@example.com")
        .create()
        .await;
    let employee_role_id = uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap();
    factory
        .add_member(ws_id, clerk_user_id, employee_role_id)
        .await;

    let clerk_emp_id = factory
        .employee()
        .with_user(clerk_user_id)
        .with_workspace(ws_id)
        .create()
        .await;
    let dept_id = factory.department().with_workspace(ws_id).create().await;

    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO employee_department_history (employee_id, department_id, effective_date) VALUES ($1, $3, CURRENT_DATE), ($2, $3, CURRENT_DATE)",
        diesel::sql_types::Uuid, emp_id,
        diesel::sql_types::Uuid, clerk_emp_id,
        diesel::sql_types::Uuid, dept_id
    );

    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO workspace_role_permissions (role_id, permission_name) VALUES ($1, 'overtime:claim_on_behalf') ON CONFLICT DO NOTHING",
        diesel::sql_types::Uuid, employee_role_id
    );

    let body = serde_json::json!({
        "date": date.to_string(),
        "claimed_minutes": 60,
        "reason": "Delegated OT claim"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/attendance/claim-ot/{emp_id}"),
            ws_id,
            clerk_user_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_claim_ot_delegated_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let normal_user_id = factory
        .user()
        .with_email("normal@example.com")
        .create()
        .await;
    let employee_role_id = uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap();
    factory
        .add_member(ws_id, normal_user_id, employee_role_id)
        .await;

    let normal_emp_id = factory
        .employee()
        .with_user(normal_user_id)
        .with_workspace(ws_id)
        .create()
        .await;
    let dept_id = factory.department().with_workspace(ws_id).create().await;

    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO employee_department_history (employee_id, department_id, effective_date) VALUES ($1, $3, CURRENT_DATE), ($2, $3, CURRENT_DATE)",
        diesel::sql_types::Uuid, emp_id,
        diesel::sql_types::Uuid, normal_emp_id,
        diesel::sql_types::Uuid, dept_id
    );

    crate::test_execute!(
        &factory.diesel_pools.api,
        "DELETE FROM workspace_role_permissions WHERE role_id = $1 AND permission_name = 'overtime:claim_on_behalf'",
        diesel::sql_types::Uuid, employee_role_id
    );

    let body = serde_json::json!({
        "date": "2026-01-20",
        "claimed_minutes": 60,
        "reason": "Unauthorized claim attempt"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/attendance/claim-ot/{emp_id}"),
            ws_id,
            normal_user_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_behalf_assignments_crud_and_claims() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Create target employee
    let target_emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Create clerk user & employee
    let clerk_user_id = factory
        .user()
        .with_email("clerk_explicit@example.com")
        .create()
        .await;
    let employee_role_id = uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap();
    factory
        .add_member(ws_id, clerk_user_id, employee_role_id)
        .await;
    let clerk_emp_id = factory
        .employee()
        .with_user(clerk_user_id)
        .with_workspace(ws_id)
        .create()
        .await;

    // Add 'overtime:claim_on_behalf' permission to employee role
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO workspace_role_permissions (role_id, permission_name) VALUES ($1, 'overtime:claim_on_behalf') ON CONFLICT DO NOTHING",
        diesel::sql_types::Uuid, employee_role_id
    );

    // Prepare claim date & daily summary for target employee
    let date = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO attendance_daily_summaries (id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, ot_status, created_at, updated_at) VALUES ($1, $2, $3, $4, 500, 60, $5::ot_status_type, NOW(), NOW())",
        diesel::sql_types::Uuid, uuid::Uuid::new_v4(),
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, target_emp_id,
        diesel::sql_types::Date, date,
        diesel::sql_types::Text, "pending_employee"
    );

    // 1. Without assignment, clerk's claim attempt must fail (FORBIDDEN)
    let claim_body = serde_json::json!({
        "date": date.to_string(),
        "claimed_minutes": 60,
        "reason": "Behalf claim test"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/attendance/claim-ot/{target_emp_id}"),
            ws_id,
            clerk_user_id,
            Some(claim_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 2. Admin creates behalf assignment (Clerk -> Target Employee)
    let assign_body = serde_json::json!({
        "assignee_employee_id": clerk_emp_id.to_string(),
        "target_employee_id": target_emp_id.to_string(),
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments"),
            ws_id,
            admin_id,
            Some(assign_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let created_assignment = body_json(resp).await;
    let assignment_id = created_assignment["id"].as_str().unwrap().to_string();

    // 3. Clerk lists behalf assignments
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list = body_json(resp).await;
    assert!(list.as_array().unwrap().len() > 0);

    // 4. Now, clerk's claim attempt must succeed (OK)
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/attendance/claim-ot/{target_emp_id}"),
            ws_id,
            clerk_user_id,
            Some(claim_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 5. Admin deletes behalf assignment
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!(
                "/api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments/{assignment_id}"
            ),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_behalf_assignments_department_claims() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Create target employee & department
    let target_emp_id = factory.employee().with_workspace(ws_id).create().await;
    let dept_id = factory.department().with_workspace(ws_id).create().await;

    // Link target employee to department
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO employee_department_history (employee_id, department_id, effective_date) VALUES ($1, $2, CURRENT_DATE)",
        diesel::sql_types::Uuid, target_emp_id,
        diesel::sql_types::Uuid, dept_id
    );

    // Create clerk user & employee
    let clerk_user_id = factory
        .user()
        .with_email("clerk_dept@example.com")
        .create()
        .await;
    let employee_role_id = uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap();
    factory
        .add_member(ws_id, clerk_user_id, employee_role_id)
        .await;
    let clerk_emp_id = factory
        .employee()
        .with_user(clerk_user_id)
        .with_workspace(ws_id)
        .create()
        .await;

    // Add 'overtime:claim_on_behalf' permission to employee role
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO workspace_role_permissions (role_id, permission_name) VALUES ($1, 'overtime:claim_on_behalf') ON CONFLICT DO NOTHING",
        diesel::sql_types::Uuid, employee_role_id
    );

    // Prepare claim date & daily summary for target employee
    let date = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO attendance_daily_summaries (id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, ot_status, created_at, updated_at) VALUES ($1, $2, $3, $4, 500, 60, $5::ot_status_type, NOW(), NOW())",
        diesel::sql_types::Uuid, uuid::Uuid::new_v4(),
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, target_emp_id,
        diesel::sql_types::Date, date,
        diesel::sql_types::Text, "pending_employee"
    );

    // 1. Without department assignment, clerk's claim attempt must fail (FORBIDDEN)
    let claim_body = serde_json::json!({
        "date": date.to_string(),
        "claimed_minutes": 60,
        "reason": "Behalf dept claim test"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/attendance/claim-ot/{target_emp_id}"),
            ws_id,
            clerk_user_id,
            Some(claim_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 2. Admin creates behalf assignment (Clerk -> Target Department)
    let assign_body = serde_json::json!({
        "assignee_employee_id": clerk_emp_id.to_string(),
        "target_department_id": dept_id.to_string(),
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments"),
            ws_id,
            admin_id,
            Some(assign_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let created_assignment = body_json(resp).await;
    let assignment_id = created_assignment["id"].as_str().unwrap().to_string();

    // 3. Now, clerk's claim attempt must succeed (OK) because target employee is in the department
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/attendance/claim-ot/{target_emp_id}"),
            ws_id,
            clerk_user_id,
            Some(claim_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 4. Admin deletes behalf assignment
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!(
                "/api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments/{assignment_id}"
            ),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
