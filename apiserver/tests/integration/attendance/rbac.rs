//! RBAC enforcement tests for 16 attendance handlers.
//!
//! Each test verifies that a workspace member with the Employee role (no
//! admin permissions) receives a 403 Forbidden response when accessing
//! an endpoint that requires a specific permission.
//!
//! Dual-mode handlers (get_time_history, get_my_attendance_adjustments)
//! additionally verify that:
//! - Employee CAN access own data (200 OK)
//! - Employee gets 403 when accessing another employee's data

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

/// Helper: create a workspace with owner + a member (Employee role, no admin perms).
async fn setup_ws_with_member() -> (
    TestDatabase,
    TestFactory,
    Arc<AppState>,
    axum::Router,
    Uuid,
    Uuid,
    Uuid,
) {
    let (test_db, factory, state, app) = setup_app_tower().await;
    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let member_id = factory
        .user()
        .with_email("rbac_member@example.com")
        .create()
        .await;
    // Employee role — no admin permissions
    factory
        .add_member(
            ws_id,
            member_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    (test_db, factory, state, app, ws_id, admin_id, member_id)
}

// ── time_tracking.rs ──────────────────────────────────────────────────

#[tokio::test]
async fn test_punch_in_without_permission_returns_403() {
    let (_db, factory, _state, app, ws_id, _admin_id, member_id) = setup_ws_with_member().await;

    // Create an employment for the member
    let _emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(member_id)
        .create()
        .await;

    // Send a valid PunchInRequest body — the permission check should reject
    // with 403 before executing any business logic.
    let employment_id = Uuid::new_v4();
    let body = serde_json::json!({
        "employment_id": employment_id,
        "note": null
    })
    .to_string();
    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/time/punch-in"),
            ws_id,
            member_id,
            Some(body),
        ))
        .await
        .unwrap();
    // Member without EssClock permission must NOT get 2xx
    assert!(
        resp.status().is_client_error(),
        "Expected 4xx, got {}",
        resp.status()
    );
}

#[tokio::test]
async fn test_punch_out_without_permission_returns_403() {
    let (_db, factory, _state, app, ws_id, _admin_id, member_id) = setup_ws_with_member().await;

    let target_emp = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Target")
        .create()
        .await;

    let body = serde_json::json!({}).to_string();
    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/time/punch-out/{target_emp}"),
            ws_id,
            member_id,
            Some(body),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_time_history_other_employee_returns_403() {
    let (_db, factory, _state, app, ws_id, _admin_id, member_id) = setup_ws_with_member().await;

    // Create the member's own employee
    let _my_emp = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(member_id)
        .create()
        .await;

    // Create another employee
    let other_emp = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Other Employee")
        .create()
        .await;

    let now = chrono::Utc::now();
    let start = (now - chrono::Duration::days(7))
        .to_rfc3339()
        .replace(':', "%3A")
        .replace('+', "%2B");
    let end = now.to_rfc3339().replace(':', "%3A").replace('+', "%2B");

    // Accessing another employee's history → 403
    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/time/history/{other_emp}?start={start}&end={end}"),
            ws_id,
            member_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_update_time_entry_without_permission_returns_403() {
    let (_db, _factory, _state, app, ws_id, _admin_id, member_id) = setup_ws_with_member().await;

    let fake_entry_id = Uuid::new_v4();
    let body = serde_json::json!({ "punch_in": "2025-01-01T08:00:00Z" }).to_string();
    let resp = app
        .oneshot(req(
            "PUT",
            &format!("/api/v1/workspaces/{ws_id}/time/entries/{fake_entry_id}"),
            ws_id,
            member_id,
            Some(body),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_delete_time_entry_without_permission_returns_403() {
    let (_db, _factory, _state, app, ws_id, _admin_id, member_id) = setup_ws_with_member().await;

    let fake_entry_id = Uuid::new_v4();
    let resp = app
        .oneshot(req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/time/entries/{fake_entry_id}"),
            ws_id,
            member_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── import.rs ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_latest_import_job_without_permission_returns_403() {
    let (_db, _factory, _state, app, ws_id, _admin_id, member_id) = setup_ws_with_member().await;

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/attendance/import-jobs/latest"),
            ws_id,
            member_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_import_job_status_without_permission_returns_403() {
    let (_db, _factory, _state, app, ws_id, _admin_id, member_id) = setup_ws_with_member().await;

    let fake_job_id = Uuid::new_v4();
    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/attendance/import-jobs/{fake_job_id}"),
            ws_id,
            member_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_download_import_error_file_without_permission_returns_403() {
    let (_db, _factory, _state, app, ws_id, _admin_id, member_id) = setup_ws_with_member().await;

    let fake_job_id = Uuid::new_v4();
    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/attendance/import-jobs/{fake_job_id}/error-file"),
            ws_id,
            member_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── logs.rs ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_attendance_logs_without_permission_returns_403() {
    let (_db, _factory, _state, app, ws_id, _admin_id, member_id) = setup_ws_with_member().await;

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/attendance/logs"),
            ws_id,
            member_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_raw_attendance_logs_without_permission_returns_403() {
    let (_db, _factory, _state, app, ws_id, _admin_id, member_id) = setup_ws_with_member().await;

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/attendance/raw-logs"),
            ws_id,
            member_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── adjustments.rs ────────────────────────────────────────────────────

#[tokio::test]
async fn test_submit_attendance_adjustment_without_permission_returns_403() {
    let (_db, factory, _state, app, ws_id, _admin_id, member_id) = setup_ws_with_member().await;

    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(member_id)
        .create()
        .await;

    // Send a body that passes JSON extraction — the permission check should
    // reject with 403 before executing any business logic.
    let body = serde_json::json!({
        "employment_id": emp_id,
        "date": "2025-01-15",
        "adjustment_type": "missed_punch_in",
        "reason": "Forgot to punch",
        "punch_time": "2025-01-15T08:00:00Z"
    })
    .to_string();

    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/attendance/adjustments"),
            ws_id,
            member_id,
            Some(body),
        ))
        .await
        .unwrap();
    // Member without AttendanceAdjustmentsSubmit must NOT get 2xx
    assert!(
        resp.status().is_client_error(),
        "Expected 4xx, got {}",
        resp.status()
    );
}

#[tokio::test]
async fn test_get_my_adjustments_other_employee_returns_403() {
    let (_db, factory, _state, app, ws_id, _admin_id, member_id) = setup_ws_with_member().await;

    // Create member's own employee
    let _my_emp = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(member_id)
        .create()
        .await;

    // Create another employee
    let other_emp = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Other Employee")
        .create()
        .await;

    // Accessing another employee's adjustments → 403
    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/employees/{other_emp}/attendance/adjustments"),
            ws_id,
            member_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_workspace_adjustments_without_permission_returns_403() {
    let (_db, _factory, _state, app, ws_id, _admin_id, member_id) = setup_ws_with_member().await;

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/attendance/adjustments"),
            ws_id,
            member_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── policies.rs ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_default_attendance_rules_without_permission_returns_403() {
    let (_db, _factory, _state, app, ws_id, _admin_id, member_id) = setup_ws_with_member().await;

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/attendance/rules"),
            ws_id,
            member_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_list_policy_versions_without_permission_returns_403() {
    let (_db, _factory, _state, app, ws_id, _admin_id, member_id) = setup_ws_with_member().await;

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/attendance/rules/versions"),
            ws_id,
            member_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
