//! Payroll API and repository integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::Payslip;
use apiserver::repositories::payroll_repository;
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::Utc;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_payroll_payslips() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;

    let payslip = Payslip {
        id: Uuid::new_v4(),
        employee_id,
        period_start: Utc::now().date_naive(),
        period_end: Utc::now().date_naive(),
        base_pay_amount: Decimal::from_i32(4500).unwrap(),
        overtime_pay_amount: Decimal::from_i32(200).unwrap(),
        deductions_amount: Decimal::from_i32(100).unwrap(),
        net_pay_amount: Decimal::from_i32(4600).unwrap(),
        standard_units: 160,
        overtime_units: 10,
        status: "paid".to_string(),
        holiday_work_premium_amount: Decimal::ZERO,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // 1. Create payslip
    let created = payroll_repository::create_payslip(&pool, &payslip)
        .await
        .unwrap();
    assert_eq!(created.net_pay_amount, Decimal::from_i32(4600).unwrap());

    // 2. Get by employee
    let list = payroll_repository::get_payslips_by_employee(&pool, employee_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn test_get_employee_payslips_api() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;
    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let emp_id: Uuid = factory.employee().with_workspace(ws_id).create().await;

    // Create a payslip
    let payslip = Payslip {
        id: Uuid::new_v4(),
        employee_id: emp_id,
        period_start: Utc::now().date_naive(),
        period_end: Utc::now().date_naive(),
        base_pay_amount: Decimal::from_i32(1000).unwrap(),
        overtime_pay_amount: Decimal::ZERO,
        deductions_amount: Decimal::ZERO,
        net_pay_amount: Decimal::from_i32(1000).unwrap(),
        standard_units: 40,
        overtime_units: 0,
        status: "draft".to_string(),
        holiday_work_premium_amount: Decimal::ZERO,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    payroll_repository::create_payslip(&factory.pool, &payslip)
        .await
        .unwrap();

    // Create a 'BASE' salary component
    let component_def = payroll_repository::create_salary_component_definition(
        &factory.pool,
        ws_id,
        "Lương cơ bản".to_string(),
        "BASE".to_string(),
        true,
        true,
    )
    .await
    .unwrap();

    // Assign it to employee
    payroll_repository::assign_employee_salary_component(
        &factory.pool,
        emp_id,
        component_def.id,
        Decimal::from_i32(2000).unwrap(),
        Utc::now().date_naive(),
    )
    .await
    .unwrap();

    // Test GET payslips
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{ws_id}/employees/{emp_id}/payslips"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test calculate payroll API
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{ws_id}/employees/{emp_id}/calculate-payroll?year=2026&month=1"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
