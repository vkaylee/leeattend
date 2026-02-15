//! Payroll traceability tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::{employee_repository, payroll_repository, department_repository};
use apiserver::services::payroll_service::PayrollService;
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[tokio::test]
async fn test_payroll_std_days_source_traceability() {
    let db = TestDatabase::new().await;
    let pool = db.pool();
    let factory = TestFactory::new(pool.clone());

    // 1. Setup Workspace & Employee
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let dept_id = factory.department().with_workspace(workspace_id).with_name("Engineering").create().await;
    let employee_id = factory.employee().with_workspace(workspace_id).create().await;

    // Assign employee to department
    sqlx::query("INSERT INTO employee_department_history (employee_id, department_id, effective_date) VALUES ($1, $2, $3)")
        .bind(employee_id)
        .bind(dept_id)
        .bind(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    // 2. Setup Base Salary
    let base_comp = payroll_repository::create_salary_component_definition(
        &pool, 
        workspace_id, 
        "Base Salary".to_string(), 
        "BASE".to_string(), 
        false, 
        true
    ).await.unwrap();

    payroll_repository::assign_employee_salary_component(
        &pool,
        employee_id,
        base_comp.id,
        Decimal::from(10_000_000),
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    ).await.unwrap();

    // 3. Scenario A: System Default (No overrides)
    let payslip_a = PayrollService::calculate_monthly_payslip(&pool, employee_id, workspace_id, 2024, 1).await.unwrap();
    let segments_a = payroll_repository::get_payslip_segments(&pool, payslip_a.id).await.unwrap();
    assert_eq!(segments_a.len(), 1);
    assert_eq!(segments_a[0].std_days_source.as_deref(), Some("system"));
    assert_eq!(segments_a[0].applied_std_days, Decimal::from(26));

    // 4. Scenario B: Workspace Override
    payroll_repository::update_payroll_settings(
        &pool,
        workspace_id,
        "fixed".to_string(),
        Decimal::from(24),
        5,
        Decimal::from(8),
        NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()
    ).await.unwrap();

    let payslip_b = PayrollService::calculate_monthly_payslip(&pool, employee_id, workspace_id, 2024, 2).await.unwrap();
    let segments_b = payroll_repository::get_payslip_segments(&pool, payslip_b.id).await.unwrap();
    assert_eq!(segments_b.len(), 1);
    assert_eq!(segments_b[0].std_days_source.as_deref(), Some("workspace"));
    assert_eq!(segments_b[0].applied_std_days, Decimal::from(24));

    // 5. Scenario C: Department Override
    department_repository::upsert_department_payroll_config(
        &pool,
        dept_id,
        Decimal::from(22),
        Decimal::from(8),
        NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()
    ).await.unwrap();

    let payslip_c = PayrollService::calculate_monthly_payslip(&pool, employee_id, workspace_id, 2024, 3).await.unwrap();
    let segments_c = payroll_repository::get_payslip_segments(&pool, payslip_c.id).await.unwrap();
    assert_eq!(segments_c.len(), 1);
    assert_eq!(segments_c[0].std_days_source.as_deref(), Some("department"));
    assert_eq!(segments_c[0].applied_std_days, Decimal::from(22));

    // 6. Scenario D: Individual Override
    employee_repository::upsert_employee_payroll_config(
        &pool,
        employee_id,
        Decimal::from(20),
        Decimal::from(8),
        NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()
    ).await.unwrap();

    let payslip_d = PayrollService::calculate_monthly_payslip(&pool, employee_id, workspace_id, 2024, 4).await.unwrap();
    let segments_d = payroll_repository::get_payslip_segments(&pool, payslip_d.id).await.unwrap();
    assert_eq!(segments_d.len(), 1);
    assert_eq!(segments_d[0].std_days_source.as_deref(), Some("individual"));
    assert_eq!(segments_d[0].applied_std_days, Decimal::from(20));
}
