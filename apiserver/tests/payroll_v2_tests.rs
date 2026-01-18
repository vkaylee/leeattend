mod common;

use apiserver::services::payroll_service::PayrollService;
use apiserver::repositories::payroll_repository;
use chrono::{NaiveDate};
use common::db::TestDatabase;
use common::factory::TestFactory;
use rust_decimal::prelude::*;

#[tokio::test]
async fn test_calculate_payroll_multi_segment() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // Set work_unit_minutes to 60 for consistent hourly-like testing
    sqlx::query(
        "UPDATE workspaces SET work_unit_minutes = 60, rounding_policy = 'none' WHERE id = $1",
    )
    .bind(workspace_id)
    .execute(&pool)
    .await
    .unwrap();

    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_user(user_id)
        .create()
        .await;

    // 1. Create 'BASE' component
    let component_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO salary_component_definitions (workspace_id, name, code, is_base_for_ot, is_taxable) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(workspace_id)
    .bind("Lương cơ bản")
    .bind("BASE")
    .bind(true)
    .bind(true)
    .fetch_one(&pool)
    .await
    .unwrap();

    // 2. Assign Salary 1 (Effective from 1st) - $2600
    sqlx::query(
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
    )
    .bind(employee_id)
    .bind(component_id)
    .bind(Decimal::from(2600))
    .bind(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .execute(&pool)
    .await
    .unwrap();

    // 3. Assign Salary 2 (Effective from 16th) - $5200 (Double)
    sqlx::query(
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
    )
    .bind(employee_id)
    .bind(component_id)
    .bind(Decimal::from(5200))
    .bind(NaiveDate::from_ymd_opt(2026, 1, 16).unwrap())
    .execute(&pool)
    .await
    .unwrap();

    // 4. Record Attendance
    // Work 1 day in Segment 1 (Jan 5)
    factory.time_entry()
        .with_employee(employee_id)
        .with_punch_in(chrono::TimeZone::from_utc_datetime(&chrono::Utc, &NaiveDate::from_ymd_opt(2026, 1, 5).unwrap().and_hms_opt(9, 0, 0).unwrap()))
        .with_punch_out(chrono::TimeZone::from_utc_datetime(&chrono::Utc, &NaiveDate::from_ymd_opt(2026, 1, 5).unwrap().and_hms_opt(17, 0, 0).unwrap()))
        .create().await;

    // Work 1 day in Segment 2 (Jan 20)
    factory.time_entry()
        .with_employee(employee_id)
        .with_punch_in(chrono::TimeZone::from_utc_datetime(&chrono::Utc, &NaiveDate::from_ymd_opt(2026, 1, 20).unwrap().and_hms_opt(9, 0, 0).unwrap()))
        .with_punch_out(chrono::TimeZone::from_utc_datetime(&chrono::Utc, &NaiveDate::from_ymd_opt(2026, 1, 20).unwrap().and_hms_opt(17, 0, 0).unwrap()))
        .create().await;

    // 5. Calculate Payroll
    let payslip = PayrollService::calculate_monthly_payslip(&pool, employee_id, workspace_id, 2026, 1)
        .await
        .unwrap();

    // Verification
    // Segment 1: Base=2600, StdDays=26 -> DailyRate=100. Worked 1 day -> Pay=100.
    // Segment 2: Base=5200, StdDays=26 -> DailyRate=200. Worked 1 day -> Pay=200.
    // Total Base Pay = 300.
    assert_eq!(payslip.base_pay_amount.round_dp(2), Decimal::from(300));

    // 6. Verify Segments are stored
    let segments = payroll_repository::get_payslip_segments(&pool, payslip.id).await.unwrap();
    assert_eq!(segments.len(), 2);
    
    let seg1 = &segments[0];
    assert_eq!(seg1.applied_base_salary, Decimal::from(2600));
    assert_eq!(seg1.amount.round_dp(2), Decimal::from(100));

    let seg2 = &segments[1];
    assert_eq!(seg2.applied_base_salary, Decimal::from(5200));
    assert_eq!(seg2.amount.round_dp(2), Decimal::from(200));
}

#[tokio::test]
async fn test_3tier_standard_days_resolution() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let department_id = factory.department().with_workspace(workspace_id).create().await;
    let employee_id = factory.employee().with_workspace(workspace_id).create().await;

    // Hierarchy: Individual > Department > Global > Default (26)

    // 1. Initial State: Global Default (26)
    let res = apiserver::domain::policy::resolver::PolicyResolver::resolve_standard_days(
        &pool, employee_id, Some(department_id), workspace_id, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    ).await.unwrap();
    assert_eq!(res, Decimal::from(26));

    // 2. Set Global Override (24) effective Jan 1
    apiserver::repositories::payroll_repository::update_payroll_settings(
        &pool, workspace_id, "fixed".to_string(), Decimal::from(24), 6, Decimal::from(8), NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    ).await.unwrap();

    let res = apiserver::domain::policy::resolver::PolicyResolver::resolve_standard_days(
        &pool, employee_id, Some(department_id), workspace_id, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    ).await.unwrap();
    assert_eq!(res, Decimal::from(24));

    // 3. Set Department Override (22 days, 8 hours) effective Feb 1
    apiserver::repositories::department_repository::upsert_department_payroll_config(
        &pool, department_id, Decimal::from(22), Decimal::from(8), NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()
    ).await.unwrap();

    // Jan should still be 24 (Global)
    let res_jan = apiserver::domain::policy::resolver::PolicyResolver::resolve_standard_days(
        &pool, employee_id, Some(department_id), workspace_id, NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()
    ).await.unwrap();
    assert_eq!(res_jan, Decimal::from(24));

    // Feb should be 22 (Dept)
    let res_feb = apiserver::domain::policy::resolver::PolicyResolver::resolve_standard_days(
        &pool, employee_id, Some(department_id), workspace_id, NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()
    ).await.unwrap();
    assert_eq!(res_feb, Decimal::from(22));

    // 4. Set Individual Override (20 days, 8 hours) effective Mar 1
    apiserver::repositories::employee_repository::upsert_employee_payroll_config(
        &pool, employee_id, Decimal::from(20), Decimal::from(8), NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
    ).await.unwrap();

    // Mar should be 20 (Individual)
    let res_mar = apiserver::domain::policy::resolver::PolicyResolver::resolve_standard_days(
        &pool, employee_id, Some(department_id), workspace_id, NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
    ).await.unwrap();
    assert_eq!(res_mar, Decimal::from(20));
}
