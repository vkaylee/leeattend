mod common;

use apiserver::services::payroll_service::PayrollService;
use chrono::{NaiveDate, NaiveTime, TimeZone, Local, Utc};
use common::factory::TestFactory;
use rust_decimal::prelude::*;
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_test_db() -> PgPool {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres")
}

async fn setup_employee_with_salary(
    factory: &TestFactory,
    pool: &PgPool,
    test_date: NaiveDate,
) -> (Uuid, Uuid, Decimal) {
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_user(user_id)
        .create()
        .await;

    let base_salary = Decimal::from_f64(100.0).unwrap();
    
    // Set work_unit_minutes to 60 so that 'hourly' salary is per hour
    sqlx::query("UPDATE workspaces SET work_unit_minutes = 60, rounding_policy = 'none' WHERE id = $1")
        .bind(workspace_id)
        .execute(pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO salary_history (employee_id, base_salary, salary_type, currency, effective_date) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(employee_id)
    .bind(base_salary)
    .bind("hourly")
    .bind("VND")
    .bind(test_date)
    .execute(pool)
    .await
    .unwrap();

    (employee_id, workspace_id, base_salary)
}

#[tokio::test]
async fn test_payroll_with_explicit_breaks() {
    let pool = setup_test_db().await;
    let factory = TestFactory::new(pool.clone());
    let test_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    // Shift: 08:00 - 17:00, 60m default break
    let (employee_id, workspace_id, _base_salary) = setup_employee_with_salary(&factory, &pool, test_date).await;
    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(NaiveTime::from_hms_opt(8, 0, 0).unwrap(), NaiveTime::from_hms_opt(17, 0, 0).unwrap())
        .with_break_duration(60)
        .assign_to_employee(employee_id, test_date)
        .await;

    // Case 1: Employee takes a 90 minute break (Longer than 60m default)
    // Work entry: 08:00 - 17:00
    let punch_in = Local.with_ymd_and_hms(2026, 1, 15, 8, 0, 0).unwrap().with_timezone(&Utc);
    let punch_out = Local.with_ymd_and_hms(2026, 1, 15, 17, 0, 0).unwrap().with_timezone(&Utc);
    factory.time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .with_entry_type("work")
        .create().await;

    // Break entry: 12:00 - 13:30 (90 minutes)
    let break_in = Local.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap().with_timezone(&Utc);
    let break_out = Local.with_ymd_and_hms(2026, 1, 15, 13, 30, 0).unwrap().with_timezone(&Utc);
    factory.time_entry()
        .with_employee(employee_id)
        .with_punch_in(break_in)
        .with_punch_out(break_out)
        .with_entry_type("break")
        .create().await;

    let payslip = PayrollService::calculate_monthly_payslip(&pool, employee_id, workspace_id, 2026, 1).await.unwrap();
    
    // Calculation: 
    // Total gross: 9 hours (540 mins)
    // Actual break: 90 mins
    // Net worked: 450 mins = 7.5 hours
    // Integer Work Units (60m block): Floor(450 / 60) = 7 units
    // Expected pay: 7 * 100 = 700.0
    assert_eq!(payslip.base_pay_amount.to_f64().unwrap(), 700.0, "Should use explicit 90m break instead of 60m default");

    factory.cleanup().await;
}
