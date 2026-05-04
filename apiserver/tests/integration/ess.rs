//! Employee Self-Service (ESS) API integration tests

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

/// Helper to create a user linked to an employee record (required by `EmployeeContext`)
async fn create_employee_user(
    factory: &TestFactory,
    test_db: &TestDatabase,
    ws_id: Uuid,
    email: &str,
) -> (Uuid, Uuid) {
    let user_id = factory.user().with_email(email).create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Link employee to user by setting user_id on the employee record
    sqlx::query("UPDATE employees SET user_id = $1 WHERE id = $2")
        .bind(user_id)
        .bind(emp_id)
        .execute(&test_db.pool())
        .await
        .unwrap();

    (user_id, emp_id)
}

#[tokio::test]
async fn test_ess_get_profile_smoke() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let (user_id, _emp_id) =
        create_employee_user(&factory, &test_db, ws_id, "ess@example.com").await;

    // Also need to add user as workspace member
    factory
        .add_member(
            ws_id,
            user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let resp = app
        .clone()
        .oneshot(req("GET", "/api/ess/profile", ws_id, user_id, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let profile = body_json(resp).await;
    assert!(profile["employee"].is_object() || profile["id"].is_string());
}

#[tokio::test]
async fn test_ess_emergency_contacts_crud() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let (user_id, _emp_id) =
        create_employee_user(&factory, &test_db, ws_id, "contacts@example.com").await;
    factory
        .add_member(
            ws_id,
            user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    // 1. Get contacts (empty)
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            "/api/ess/emergency-contacts",
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let contacts = body_json(resp).await;
    assert!(contacts.as_array().unwrap().is_empty());

    // 2. Add contact
    let body = serde_json::json!({
        "contact_name": "Nguyen Van A",
        "relationship": "Spouse",
        "phone": "+84912345678"
    });
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/ess/emergency-contacts",
            ws_id,
            user_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = body_json(resp).await;
    let contact_id = created["id"].as_str().unwrap();

    // 3. List contacts (should have 1)
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            "/api/ess/emergency-contacts",
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let contacts = body_json(resp).await;
    assert_eq!(contacts.as_array().unwrap().len(), 1);

    // 4. Delete contact
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/ess/emergency-contacts/{contact_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_ess_get_documents_empty() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let (user_id, _) = create_employee_user(&factory, &test_db, ws_id, "docs@example.com").await;
    factory
        .add_member(
            ws_id,
            user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let resp = app
        .clone()
        .oneshot(req("GET", "/api/ess/documents", ws_id, user_id, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_ess_get_payslips_empty() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let (user_id, _) =
        create_employee_user(&factory, &test_db, ws_id, "payslips@example.com").await;
    factory
        .add_member(
            ws_id,
            user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let resp = app
        .clone()
        .oneshot(req("GET", "/api/ess/payslips", ws_id, user_id, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_ess_get_attendance_empty() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let (user_id, _) = create_employee_user(&factory, &test_db, ws_id, "att@example.com").await;
    factory
        .add_member(
            ws_id,
            user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            "/api/ess/attendance?start_date=2026-01-01&end_date=2026-01-31",
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_ess_get_leave_balances() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let (user_id, _) = create_employee_user(&factory, &test_db, ws_id, "leave@example.com").await;
    factory
        .add_member(
            ws_id,
            user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let resp = app
        .clone()
        .oneshot(req("GET", "/api/ess/leave/balances", ws_id, user_id, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_ess_get_leave_requests_empty() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let (user_id, _) = create_employee_user(&factory, &test_db, ws_id, "lr@example.com").await;
    factory
        .add_member(
            ws_id,
            user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let resp = app
        .clone()
        .oneshot(req("GET", "/api/ess/leave/requests", ws_id, user_id, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_ess_get_shift_swaps_empty() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let (user_id, _) = create_employee_user(&factory, &test_db, ws_id, "swaps@example.com").await;
    factory
        .add_member(
            ws_id,
            user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let resp = app
        .clone()
        .oneshot(req("GET", "/api/ess/shift-swaps", ws_id, user_id, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_ess_no_employee_record_returns_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // User without employee record
    let user_id = factory
        .user()
        .with_email("noemp@example.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let resp = app
        .clone()
        .oneshot(req("GET", "/api/ess/profile", ws_id, user_id, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_ess_time_tracking_endpoints() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let (user_id, _emp_id) =
        create_employee_user(&factory, &test_db, ws_id, "time@example.com").await;

    factory
        .add_member(
            ws_id,
            user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    // 1. Get time entries (should be empty initially)
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            "/api/ess/time-entries?start_date=2026-01-01T00:00:00Z&end_date=2026-12-31T23:59:59Z",
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let entries = body_json(resp).await;
    assert!(entries.as_array().unwrap().is_empty());

    // 2. Punch in
    let punch_in_body = serde_json::json!({
        "entry_type": "work",
        "note": "Starting shift"
    });
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/ess/punch",
            ws_id,
            user_id,
            Some(punch_in_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let entry = body_json(resp).await;
    let entry_id = entry["id"].as_str().unwrap();

    // 3. Punch out
    let punch_out_body = serde_json::json!({
        "note": "Ending shift"
    });
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/ess/punch",
            ws_id,
            user_id,
            Some(punch_out_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 4. Get time entries again (should have 1)
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            "/api/ess/time-entries?start_date=2000-01-01T00:00:00Z&end_date=2100-12-31T23:59:59Z",
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let entries = body_json(resp).await;
    assert_eq!(entries.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_ess_punch_out_without_punch_in() {
    let (test_db, factory, _state, app) = setup_app_tower().await;
    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let email = "no_punch_in@leeattend.com";
    let (user_id, _) = create_employee_user(&factory, &test_db, ws_id, email).await;

    let punch_out_body = serde_json::json!({
        "note": "Ending shift"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/ess/punch",
            ws_id,
            user_id,
            Some(punch_out_body.to_string()),
        ))
        .await
        .unwrap();

    // With the generic punch API, punching without an active session just creates a new log
    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_ess_punch_in_already_active() {
    let (test_db, factory, _state, app) = setup_app_tower().await;
    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let email = "double_punch@leeattend.com";
    let (user_id, _) = create_employee_user(&factory, &test_db, ws_id, email).await;

    let punch_in_body = serde_json::json!({
        "entry_type": "work",
        "note": "Starting shift"
    });

    // 1. First Punch In
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/ess/punch",
            ws_id,
            user_id,
            Some(punch_in_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 2. Second Punch In (should fail)
    let resp2 = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/ess/punch",
            ws_id,
            user_id,
            Some(punch_in_body.to_string()),
        ))
        .await
        .unwrap();

    // With the generic punch API, the second punch should also succeed (and log as punch OUT)
    assert_eq!(resp2.status(), StatusCode::CREATED);
}
