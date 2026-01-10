mod common;

use common::db::TestDatabase;
use common::factory::TestFactory;
use apiserver::repositories::payroll_repository;
use apiserver::models::Payslip;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::Utc;
use apiserver::agent_manager::AgentManager;
use apiserver::AppState;
use std::sync::Arc;
use tower::ServiceExt;
use axum::http::StatusCode;

async fn setup_app() -> (TestDatabase, TestFactory, axum::Router) {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());
    
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository::new(pool.clone()));
    let workspace_repo = Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_manager = AgentManager::new();
    let agent_service = Arc::new(apiserver::services::AgentService::new(agent_repo.clone(), agent_manager.clone()));
    
    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret_payroll".to_string(),
    });
    
    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_payroll_salary_history() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;

    // 1. Add salary history
    let salary = payroll_repository::add_salary_history(
        &pool,
        employee_id,
        Decimal::from_i32(5000).unwrap(),
        "monthly".to_string(),
        "USD".to_string(),
        Utc::now().date_naive()
    ).await.unwrap();
    assert_eq!(salary.base_salary, Decimal::from_i32(5000).unwrap());

    // 2. Get latest salary
    let latest = payroll_repository::get_latest_salary(&pool, employee_id, Utc::now().date_naive()).await.unwrap();
    assert!(latest.is_some());
    assert_eq!(latest.unwrap().base_salary, Decimal::from_i32(5000).unwrap());

    // 3. Get history list
    let history = payroll_repository::get_salary_history(&pool, employee_id).await.unwrap();
    assert_eq!(history.len(), 1);
}

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
        status: "paid".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // 1. Create payslip
    let created = payroll_repository::create_payslip(&pool, &payslip).await.unwrap();
    assert_eq!(created.net_pay_amount, Decimal::from_i32(4600).unwrap());

    // 2. Get by employee
    let list = payroll_repository::get_payslips_by_employee(&pool, employee_id).await.unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn test_get_employee_payslips_api() {
    let (_test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

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
        status: "draft".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    payroll_repository::create_payslip(&factory.pool, &payslip).await.unwrap();

    // Add salary history so calculation doesn't fail with 400
    payroll_repository::add_salary_history(
        &factory.pool,
        emp_id,
        Decimal::from_i32(2000).unwrap(),
        "monthly".to_string(),
        "USD".to_string(),
        Utc::now().date_naive()
    ).await.unwrap();

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/employees/{}/payslips", ws_id, emp_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test calculate payroll API
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/employees/{}/calculate-payroll?year=2026&month=1", ws_id, emp_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}