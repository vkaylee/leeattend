use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{LeaveRequest, LeaveStatus, OtRequestStatus, OtStatus, OvertimeRequest};
use apiserver::AppState;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

fn req(method: &str, uri: &str, ws_id: Uuid, token: &str, body: Option<String>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("Authorization", format!("Bearer {}", token));

    if body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }

    builder
        .body(match body {
            Some(b) => Body::from(b),
            None => Body::empty(),
        })
        .unwrap()
}

async fn body_json(resp: axum::http::Response<Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 100000)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_leave_request_workflow_flow() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_user_id).await;

    // Create regular employee who doesn't have auto-approve permission
    let emp_user_id = factory
        .user()
        .with_email("emp_workflow@test.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            emp_user_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(), // member role
        )
        .await;

    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO workspace_role_permissions (role_id, permission_name) VALUES
        ('a0000000-0000-0000-0000-000000000006', 'leave:requests:submit')
        ON CONFLICT DO NOTHING"
    );

    crate::test_execute!(
        &factory.diesel_pools.api,
        "UPDATE workspace_roles SET scope_type = 'global' WHERE id = 'a0000000-0000-0000-0000-000000000006'"
    );

    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(emp_user_id)
        .create()
        .await;

    let admin_token = apiserver::services::auth_service::AuthService::generate_token(
        admin_user_id,
        "test_secret_tower",
        60,
    )
    .unwrap();

    let emp_token = apiserver::services::auth_service::AuthService::generate_token(
        emp_user_id,
        "test_secret_tower",
        60,
    )
    .unwrap();

    let vacation_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    // Scenario A: Approve Leave Request
    let submit_payload = serde_json::json!({
        "employee_id": emp_id,
        "leave_type_id": vacation_id,
        "start_date": "2027-08-10",
        "end_date": "2027-08-11",
        "reason": "Trip A",
        "duration_type": "full_day"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/leave/submit"),
            ws_id,
            &emp_token,
            Some(submit_payload.to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let created: LeaveRequest = serde_json::from_value(body_json(resp).await).unwrap();
    assert_eq!(created.status, LeaveStatus::Pending);

    // Approve workflow step
    let approve_payload = serde_json::json!({
        "entity_type": "leave_request",
        "entity_id": created.id,
        "action": "approve",
        "note": "approved!"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/approval-workflows/approve"),
            ws_id,
            &admin_token,
            Some(approve_payload.to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // Verify it is Approved
    let status_str: String = crate::test_query_scalar_string!(
        &factory.diesel_pools.api,
        "SELECT status::text AS val FROM leave_requests WHERE id = $1",
        diesel::sql_types::Uuid,
        created.id
    );
    assert_eq!(status_str, "approved");

    // Scenario B: Reject Leave Request
    let submit_payload_b = serde_json::json!({
        "employee_id": emp_id,
        "leave_type_id": vacation_id,
        "start_date": "2027-08-15",
        "end_date": "2027-08-16",
        "reason": "Trip B",
        "duration_type": "full_day"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/leave/submit"),
            ws_id,
            &emp_token,
            Some(submit_payload_b.to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let created_b: LeaveRequest = serde_json::from_value(body_json(resp).await).unwrap();
    assert_eq!(created_b.status, LeaveStatus::Pending);

    // Reject workflow step
    let reject_payload = serde_json::json!({
        "entity_type": "leave_request",
        "entity_id": created_b.id,
        "action": "reject",
        "note": "denied!"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/approval-workflows/approve"),
            ws_id,
            &admin_token,
            Some(reject_payload.to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // Verify it is Rejected
    let status_str_b: String = crate::test_query_scalar_string!(
        &factory.diesel_pools.api,
        "SELECT status::text AS val FROM leave_requests WHERE id = $1",
        diesel::sql_types::Uuid,
        created_b.id
    );
    assert_eq!(status_str_b, "rejected");
}

#[tokio::test]
async fn test_ot_request_workflow_flow() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_user_id).await;

    let emp_user_id = factory.user().create().await;
    factory
        .add_member(
            ws_id,
            emp_user_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(emp_user_id)
        .create()
        .await;

    let admin_token = apiserver::services::auth_service::AuthService::generate_token(
        admin_user_id,
        "test_secret_tower",
        60,
    )
    .unwrap();

    // Explicit permissions
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO workspace_role_permissions (role_id, permission_name) VALUES
        ('a0000000-0000-0000-0000-000000000001', 'overtime:request:create')
        ON CONFLICT DO NOTHING"
    );

    // Scenario A: Approve OT Request
    let ot_payload = serde_json::json!({
        "department_id": null,
        "planned_date": "2026-05-15",
        "planned_start_time": "18:00:00",
        "planned_end_time": "20:00:00",
        "planned_minutes": 120,
        "reason": "Project deadline",
        "employees": [emp_id]
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/overtime/requests"),
            ws_id,
            &admin_token,
            Some(ot_payload.to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let created: OvertimeRequest = serde_json::from_value(body_json(resp).await).unwrap();

    // Consent
    let consent_payload = serde_json::json!({ "status": "accepted" });
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!(
                "/api/v1/workspaces/{ws_id}/overtime/requests/{}/consent/{}",
                created.id, emp_id
            ),
            ws_id,
            &admin_token,
            Some(consent_payload.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Submit
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!(
                "/api/v1/workspaces/{ws_id}/overtime/requests/{}/submit",
                created.id
            ),
            ws_id,
            &admin_token,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Approve workflow step
    let approve_payload = serde_json::json!({
        "entity_type": "ot_request",
        "entity_id": created.id,
        "action": "approve",
        "note": "approved!"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/approval-workflows/approve"),
            ws_id,
            &admin_token,
            Some(approve_payload.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify it is Approved
    let status_str: String = crate::test_query_scalar_string!(
        &factory.diesel_pools.api,
        "SELECT status::text AS val FROM overtime_requests WHERE id = $1",
        diesel::sql_types::Uuid,
        created.id
    );
    assert_eq!(status_str, "approved");
}

#[tokio::test]
async fn test_ot_claim_workflow_flow() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_user_id).await;

    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(admin_user_id)
        .create()
        .await;

    let admin_token = apiserver::services::auth_service::AuthService::generate_token(
        admin_user_id,
        "test_secret_tower",
        60,
    )
    .unwrap();

    let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
    let summary_id = uuid::Uuid::new_v4();

    // Create a daily summary in pending_employee
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO attendance_daily_summaries (id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, ot_status, created_at, updated_at) VALUES ($1, $2, $3, $4, 500, 60, $5::ot_status_type, NOW(), NOW())",
        diesel::sql_types::Uuid, summary_id,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, emp_id,
        diesel::sql_types::Date, date,
        diesel::sql_types::Text, "pending_employee"
    );

    // Claim OT
    let claim_payload = serde_json::json!({
        "date": date.to_string(),
        "claimed_minutes": 60,
        "reason": "Deadline work"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/attendance/claim-ot/{emp_id}"),
            ws_id,
            &admin_token,
            Some(claim_payload.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify it is submitted to workflow (status becomes pending_manager in db, and workflow instance is created)
    let ot_status_str: String = crate::test_query_scalar_string!(
        &factory.diesel_pools.api,
        "SELECT ot_status::text AS val FROM attendance_daily_summaries WHERE id = $1",
        diesel::sql_types::Uuid,
        summary_id
    );
    assert_eq!(ot_status_str, "pending_manager");

    // Approve workflow step
    let approve_payload = serde_json::json!({
        "entity_type": "ot_claim",
        "entity_id": summary_id,
        "action": "approve",
        "note": "claiming looks good"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/approval-workflows/approve"),
            ws_id,
            &admin_token,
            Some(approve_payload.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify status becomes Approved
    let ot_status_str: String = crate::test_query_scalar_string!(
        &factory.diesel_pools.api,
        "SELECT ot_status::text AS val FROM attendance_daily_summaries WHERE id = $1",
        diesel::sql_types::Uuid,
        summary_id
    );
    assert_eq!(ot_status_str, "approved");
}
