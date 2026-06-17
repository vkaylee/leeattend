//! Payroll API and repository integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::Payslip;
use apiserver::repositories::{payroll_repository, PayrollRepository};
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::Utc;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_payroll_payslips_smoke() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    let payslip = Payslip::new(
        Uuid::new_v4(),
        employment_id,
        ws_id,
        Utc::now().date_naive(),
        Utc::now().date_naive(),
        Decimal::from_i32(4500).unwrap(),
        Decimal::from_i32(200).unwrap(),
        Decimal::from_i32(100).unwrap(),
        Decimal::from_i32(4600).unwrap(),
        160,
        10,
        "paid".to_string(),
        Decimal::ZERO,
        Utc::now(),
        Utc::now(),
    );

    // 1. Create payslip
    let payroll_repo = apiserver::repositories::DieselPayrollRepository::new(diesel_pools.clone());
    let created = payroll_repo.create_payslip(&payslip).await.unwrap();
    assert_eq!(created.net_pay_amount(), Decimal::from_i32(4600).unwrap());

    // 2. Get by employee
    let list = payroll_repo
        .get_payslips_by_employee(ws_id, employee_id)
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
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    // Create a payslip
    let payslip = Payslip::new(
        Uuid::new_v4(),
        employment_id,
        ws_id,
        Utc::now().date_naive(),
        Utc::now().date_naive(),
        Decimal::from_i32(1000).unwrap(),
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::from_i32(1000).unwrap(),
        40,
        0,
        "draft".to_string(),
        Decimal::ZERO,
        Utc::now(),
        Utc::now(),
    );
    _state
        .payroll()
        .repo()
        .create_payslip(&payslip)
        .await
        .unwrap();

    // Create a 'BASE' salary component
    let component_def = _state
        .payroll()
        .repo()
        .create_salary_component_definition(
            ws_id,
            "Lương cơ bản".to_string(),
            "BASE".to_string(),
            None,
        )
        .await
        .unwrap();

    // Assign it to employee
    _state
        .payroll()
        .repo()
        .assign_employment_salary_component(
            ws_id,
            employment_id,
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
                    "/api/v1/workspaces/{ws_id}/employees/{emp_id}/payslips"
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
                    "/api/v1/workspaces/{ws_id}/employees/{emp_id}/calculate-payroll?year=2026&month=1"
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
