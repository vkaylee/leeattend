mod common;

use apiserver::services::payroll_service::PayrollService;
use chrono::{NaiveDate, TimeZone, Local, Utc};
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
) -> (Uuid, Uuid) {
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_user(user_id)
        .create()
        .await;

    let base_salary = Decimal::from_f64(100.0).unwrap();
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

    (employee_id, workspace_id)
}

async fn update_workspace_rounding(pool: &PgPool, ws_id: Uuid, unit: i32, policy: &str) {
    sqlx::query("UPDATE workspaces SET work_unit_minutes = $1, rounding_policy = $2 WHERE id = $3")
        .bind(unit)
        .bind(policy)
        .bind(ws_id)
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_payroll_rounding_30m_floor() {
    let pool = setup_test_db().await;
    let factory = TestFactory::new(pool.clone());
    let test_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    let (employee_id, workspace_id) = setup_employee_with_salary(&factory, &pool, test_date).await;
    
    // Set rounding: 30m, floor
    update_workspace_rounding(&pool, workspace_id, 30, "floor").await;

    // Work: 08:00 - 10:45 (2h 45m = 165 mins)
    // Rounding 30m Floor: 165 / 30 = 5.5 -> Floor(5.5) = 5 units -> 5 * 30 = 150 mins (2.5 hours)
    let punch_in = Local.with_ymd_and_hms(2026, 1, 15, 8, 0, 0).unwrap().with_timezone(&Utc);
    let punch_out = Local.with_ymd_and_hms(2026, 1, 15, 10, 45, 0).unwrap().with_timezone(&Utc);
    factory.time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create().await;

    let payslip = PayrollService::calculate_monthly_payslip(&pool, employee_id, workspace_id, 2026, 1).await.unwrap();
    
    // 2.5 hours * 100 = 250.0
    assert_eq!(payslip.base_pay_amount.to_f64().unwrap(), 250.0);

    factory.cleanup().await;
}

#[tokio::test]
async fn test_payroll_rounding_30m_ceil() {
    let pool = setup_test_db().await;
    let factory = TestFactory::new(pool.clone());
    let test_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    let (employee_id, workspace_id) = setup_employee_with_salary(&factory, &pool, test_date).await;
    
    // Set rounding: 30m, ceil
    update_workspace_rounding(&pool, workspace_id, 30, "ceil").await;

    // Work: 08:00 - 08:15 (15 mins)
    // Rounding 30m Ceil: 15 / 30 = 0.5 -> Ceil(0.5) = 1 unit -> 30 mins (0.5 hours)
    let punch_in = Local.with_ymd_and_hms(2026, 1, 15, 8, 0, 0).unwrap().with_timezone(&Utc);
    let punch_out = Local.with_ymd_and_hms(2026, 1, 15, 8, 15, 0).unwrap().with_timezone(&Utc);
    factory.time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create().await;

    let payslip = PayrollService::calculate_monthly_payslip(&pool, employee_id, workspace_id, 2026, 1).await.unwrap();
    
    // 0.5 hours * 100 = 50.0
    assert_eq!(payslip.base_pay_amount.to_f64().unwrap(), 50.0);

    factory.cleanup().await;
}
