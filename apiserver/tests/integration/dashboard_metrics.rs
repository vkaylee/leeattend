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
