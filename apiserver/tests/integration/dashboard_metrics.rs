//! Executive KPI dashboard integration tests.
//!
//! Covers the security-sensitive surface of `GET /api/v1/dashboard/executive-overview`:
//! mandatory `dashboard:read` authorization, overtime cost vs. hours fallback gated
//! on `payroll:sensitive:read` (no cost leakage), and cross-tenant isolation.

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::body::Body;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

const EMPLOYEE_ROLE: &str = "a0000000-0000-0000-0000-000000000006";

fn request(ws_id: Uuid, user_id: Uuid) -> axum::http::Request<Body> {
    axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/dashboard/executive-overview")
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .body(Body::empty())
        .unwrap()
}

fn breakdown_request(ws_id: Uuid, user_id: Uuid, query: &str) -> axum::http::Request<Body> {
    axum::http::Request::builder()
        .method("GET")
        .uri(format!("/api/v1/dashboard/overtime-breakdown?{query}"))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .body(Body::empty())
        .unwrap()
}

fn find_metric<'a>(metrics: &'a [serde_json::Value], key: &str) -> Option<&'a serde_json::Value> {
    metrics.iter().find(|json| json["key"] == key)
}

fn metric_value(metric: &serde_json::Value) -> Option<f64> {
    metric["value"].as_str().and_then(|s| s.parse::<f64>().ok())
}

#[tokio::test]
async fn unauthorized_member_without_dashboard_read_is_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Employee role has ESS permissions but no `dashboard:read`.
    let employee_user = factory.user().create().await;
    factory
        .add_member(
            ws_id,
            employee_user,
            Uuid::parse_str(EMPLOYEE_ROLE).unwrap(),
        )
        .await;

    let response = app
        .clone()
        .oneshot(request(ws_id, employee_user))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "a member without dashboard:read must be forbidden"
    );
}

#[tokio::test]
async fn overtime_falls_back_to_hours_without_payroll_sensitive_read() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Grant `dashboard:read` to an employee-role user WITHOUT `payroll:sensitive:read`.
    let viewer_user = factory.user().create().await;
    factory
        .add_member(ws_id, viewer_user, Uuid::parse_str(EMPLOYEE_ROLE).unwrap())
        .await;
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO workspace_member_extra_permissions (workspace_id, user_id, permission, expires_at) VALUES ($1, $2, 'dashboard:read', $3)",
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, viewer_user,
        diesel::sql_types::Timestamptz, chrono::Utc::now() + chrono::Duration::days(1)
    );

    // Seed a paid payslip so a payroll-sensitive caller WOULD see a cost.
    seed_payslip(&factory, &test_db, ws_id, rust_decimal::Decimal::from(1500)).await;

    let response = app
        .clone()
        .oneshot(request(ws_id, viewer_user))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let metrics = json["metrics"].as_array().unwrap();

    assert!(
        find_metric(metrics, "overtime_hours").is_some(),
        "unauthorized-for-cost callers must receive overtime_hours"
    );
    assert!(
        find_metric(metrics, "overtime_cost").is_none(),
        "overtime_cost must never leak to callers without payroll:sensitive:read"
    );
}

#[tokio::test]
async fn overtime_cost_emitted_for_payroll_sensitive_caller() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    seed_payslip(&factory, &test_db, ws_id, rust_decimal::Decimal::from(1500)).await;

    let response = app.clone().oneshot(request(ws_id, owner_id)).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let metrics = json["metrics"].as_array().unwrap();

    let overtime_cost = find_metric(metrics, "overtime_cost")
        .expect("payroll-sensitive owner must receive overtime_cost");
    assert_eq!(overtime_cost["unit"], "currency");
    assert_eq!(
        metric_value(overtime_cost),
        Some(1500.0),
        "overtime cost must reflect the seeded payslip amount"
    );
    assert!(
        find_metric(metrics, "overtime_hours").is_none(),
        "overtime_hours must not coexist with overtime_cost"
    );
}

#[tokio::test]
async fn cross_tenant_cost_isolation() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    // Workspace A: owner + a paid payslip worth 1500.
    let owner_a = factory.user().create().await;
    let ws_a = factory.workspace().create_with_owner(owner_a).await;
    seed_payslip(&factory, &test_db, ws_a, rust_decimal::Decimal::from(1500)).await;

    // Workspace B: owner, no payslips.
    let owner_b = factory.user().create().await;
    let ws_b = factory.workspace().create_with_owner(owner_b).await;

    let response = app.clone().oneshot(request(ws_b, owner_b)).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let metrics = json["metrics"].as_array().unwrap();

    // Tenant B has no payslips, so cost is unresolved -> hours fallback, and the
    // 1500 from tenant A must not appear.
    assert!(
        find_metric(metrics, "overtime_cost").is_none(),
        "tenant B must not receive an overtime_cost metric from tenant A's payroll"
    );
    assert!(find_metric(metrics, "overtime_hours").is_some());
}

// ── Overtime breakdown endpoint (`GET /api/v1/dashboard/overtime-breakdown`) ─────

#[tokio::test]
async fn breakdown_unauthorized_member_without_dashboard_read_is_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Employee role has no `dashboard:read`.
    let employee_user = factory.user().create().await;
    factory
        .add_member(
            ws_id,
            employee_user,
            Uuid::parse_str(EMPLOYEE_ROLE).unwrap(),
        )
        .await;

    let response = app
        .clone()
        .oneshot(breakdown_request(ws_id, employee_user, ""))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "a member without dashboard:read must be forbidden from the breakdown"
    );
}

#[tokio::test]
async fn breakdown_hours_returned_without_payroll_sensitive_read() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Grant `dashboard:read` without `payroll:sensitive:read`.
    let viewer_user = factory.user().create().await;
    factory
        .add_member(ws_id, viewer_user, Uuid::parse_str(EMPLOYEE_ROLE).unwrap())
        .await;
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO workspace_member_extra_permissions (workspace_id, user_id, permission, expires_at) VALUES ($1, $2, 'dashboard:read', $3)",
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, viewer_user,
        diesel::sql_types::Timestamptz, chrono::Utc::now() + chrono::Duration::days(1)
    );

    let (dept_id, employment_id) =
        seed_employee_in_department(&factory, &test_db, ws_id, "Engineering").await;
    let today = chrono::Utc::now().date_naive();
    seed_attendance_summary(&test_db, ws_id, employment_id, today, 120, 60).await;
    // A paid payslip exists, but the caller lacks the cost permission.
    seed_payslip_for_employment(
        &test_db,
        ws_id,
        employment_id,
        rust_decimal::Decimal::from(500),
    )
    .await;

    let response = app
        .clone()
        .oneshot(breakdown_request(ws_id, viewer_user, ""))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let rows = json["rows"].as_array().unwrap();
    let row = rows
        .iter()
        .find(|r| r["department_id"] == dept_id.to_string());
    let row = row.expect("engineering department must appear in the breakdown");

    assert_eq!(row["system_ot_hours"], 2.0, "120 system minutes → 2h");
    assert_eq!(row["approved_ot_hours"], 1.0, "60 approved minutes → 1h");
    assert_eq!(row["unapproved_ot_hours"], 1.0, "2h − 1h = 1h unapproved");
    assert_eq!(
        row["cost_available"], false,
        "cost must not be available to a caller without payroll:sensitive:read"
    );
    assert!(
        row["overtime_cost"].is_null(),
        "overtime_cost must be null (absent) for a restricted caller"
    );
}

#[tokio::test]
async fn breakdown_cost_emitted_for_payroll_sensitive_caller_with_department_hours() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let (_dept_id, employment_id) =
        seed_employee_in_department(&factory, &test_db, ws_id, "Engineering").await;
    let today = chrono::Utc::now().date_naive();
    seed_attendance_summary(&test_db, ws_id, employment_id, today, 120, 60).await;
    seed_payslip_for_employment(
        &test_db,
        ws_id,
        employment_id,
        rust_decimal::Decimal::from(750),
    )
    .await;

    let response = app
        .clone()
        .oneshot(breakdown_request(ws_id, owner_id, ""))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let rows = json["rows"].as_array().unwrap();
    let row = rows
        .iter()
        .find(|r| r["department_name"] == "Engineering")
        .expect("engineering department must appear");

    assert_eq!(row["system_ot_hours"], 2.0);
    assert_eq!(
        row["cost_available"], true,
        "payroll-sensitive caller with a paid payslip sees cost"
    );
    assert_eq!(
        row["overtime_cost"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok()),
        Some(750.0),
        "overtime cost must equal the seeded paid payslip amount"
    );
    assert_eq!(
        json["workspace"]["currency"], "VND",
        "workspace currency comes from the employers table (VND default in test fixtures)"
    );
    assert_eq!(json["contract_version"], "overtime-breakdown.v1");
}

#[tokio::test]
async fn breakdown_top_n_limits_rows_and_sorts_by_system_ot() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let today = chrono::Utc::now().date_naive();

    let (_, emp_a) = seed_employee_in_department(&factory, &test_db, ws_id, "Alpha").await;
    seed_attendance_summary(&test_db, ws_id, emp_a, today, 300, 150).await;

    let (_, emp_b) = seed_employee_in_department(&factory, &test_db, ws_id, "Beta").await;
    seed_attendance_summary(&test_db, ws_id, emp_b, today, 60, 30).await;

    let response = app
        .clone()
        .oneshot(breakdown_request(ws_id, owner_id, "top_n=1"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let rows = json["rows"].as_array().unwrap();

    assert_eq!(rows.len(), 1, "top_n=1 must return exactly one department");
    assert_eq!(
        rows[0]["department_name"], "Alpha",
        "the department with the most system OT hours must rank first"
    );
    assert_eq!(rows[0]["system_ot_hours"], 5.0, "300 system minutes → 5h");
}

#[tokio::test]
async fn breakdown_cross_tenant_isolation() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    // Workspace A: a paid payslip with 900 overtime.
    let owner_a = factory.user().create().await;
    let ws_a = factory.workspace().create_with_owner(owner_a).await;
    let (_dept_a, emp_a) = seed_employee_in_department(&factory, &test_db, ws_a, "Eng").await;
    seed_payslip_for_employment(&test_db, ws_a, emp_a, rust_decimal::Decimal::from(900)).await;

    // Workspace B: no payroll, no summaries.
    let owner_b = factory.user().create().await;
    let ws_b = factory.workspace().create_with_owner(owner_b).await;

    let response = app
        .clone()
        .oneshot(breakdown_request(ws_b, owner_b, ""))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let rows = json["rows"].as_array().unwrap();

    assert!(
        rows.iter().all(|r| r["overtime_cost"].is_null()),
        "tenant B must not see tenant A's payroll cost"
    );
    assert!(
        rows.iter().all(|r| r["system_ot_hours"] == 0.0),
        "tenant B has no attendance summaries → all department rows show zero hours"
    );
}

#[tokio::test]
async fn breakdown_date_timezone_boundary_excludes_out_of_range_summary() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let (dept_id, employment_id) =
        seed_employee_in_department(&factory, &test_db, ws_id, "Finance").await;
    let today = chrono::Utc::now().date_naive();

    // A summary inside the window (today).
    seed_attendance_summary(&test_db, ws_id, employment_id, today, 120, 60).await;
    // A summary far outside the 30-day default window — must NOT leak into the breakdown.
    seed_attendance_summary(
        &test_db,
        ws_id,
        employment_id,
        today - chrono::Duration::days(60),
        5000,
        2500,
    )
    .await;

    let response = app
        .clone()
        .oneshot(breakdown_request(ws_id, owner_id, ""))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let rows = json["rows"].as_array().unwrap();
    let row = rows
        .iter()
        .find(|r| r["department_id"] == dept_id.to_string());
    let row = row.expect("finance department must appear");

    assert_eq!(
        row["system_ot_hours"], 2.0,
        "the out-of-window 5000-minute summary must be excluded (only the 120 in-window minutes count)"
    );
}

/// Create an active employee and a `paid` payslip with the given overtime amount.
async fn seed_payslip(
    factory: &TestFactory,
    test_db: &TestDatabase,
    ws_id: Uuid,
    overtime_amount: rust_decimal::Decimal,
) {
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 AND workspace_id = $2 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id,
        diesel::sql_types::Uuid,
        ws_id
    );
    seed_payslip_for_employment(test_db, ws_id, employment_id, overtime_amount).await;
}

/// Insert a `paid` payslip for an existing employment with the given overtime amount.
async fn seed_payslip_for_employment(
    test_db: &TestDatabase,
    ws_id: Uuid,
    employment_id: Uuid,
    overtime_amount: rust_decimal::Decimal,
) {
    let today = chrono::Utc::now().date_naive();
    // Overlap a ±1 day window so the workspace-local "today" period always matches.
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO payslips (period_start, period_end, base_pay_amount, overtime_pay_amount, net_pay_amount, status, workspace_id, employment_id) VALUES ($1, $2, $3, $4, $5, 'paid', $6, $7)",
        diesel::sql_types::Date, today - chrono::Duration::days(1),
        diesel::sql_types::Date, today + chrono::Duration::days(1),
        diesel::sql_types::Numeric, rust_decimal::Decimal::from(1000),
        diesel::sql_types::Numeric, overtime_amount,
        diesel::sql_types::Numeric, rust_decimal::Decimal::from(1000) + overtime_amount,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, employment_id
    );
}

/// Seed an `attendance_daily_summaries` row for the given employment (system + approved OT).
async fn seed_attendance_summary(
    test_db: &TestDatabase,
    ws_id: Uuid,
    employment_id: Uuid,
    date: chrono::NaiveDate,
    system_ot_minutes: i32,
    approved_ot_minutes: i32,
) {
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO attendance_daily_summaries (workspace_id, employment_id, date, actual_work_minutes, system_detected_ot_minutes, approved_ot_minutes, ot_status) VALUES ($1, $2, $3, $4, $5, $6, 'approved')",
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, employment_id,
        diesel::sql_types::Date, date,
        diesel::sql_types::Integer, 480,
        diesel::sql_types::Integer, system_ot_minutes,
        diesel::sql_types::Integer, approved_ot_minutes
    );
}

/// Create a department + active employee in it, returning (department_id, employment_id).
async fn seed_employee_in_department(
    factory: &TestFactory,
    test_db: &TestDatabase,
    ws_id: Uuid,
    department_name: &str,
) -> (Uuid, Uuid) {
    let department_id = factory
        .department()
        .with_workspace(ws_id)
        .with_name(department_name)
        .create()
        .await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 AND workspace_id = $2 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id,
        diesel::sql_types::Uuid,
        ws_id
    );
    // Move the employment into the target department.
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "UPDATE employments SET department_id = $1 WHERE id = $2",
        diesel::sql_types::Uuid,
        department_id,
        diesel::sql_types::Uuid,
        employment_id
    );
    (department_id, employment_id)
}
