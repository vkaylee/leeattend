mod common;

use apiserver::services::payroll_service::PayrollService;
use chrono::{Local, NaiveDate, TimeZone, Utc};
use common::db::TestDatabase;
use common::factory::TestFactory;
use rust_decimal::prelude::*;
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_test_context() -> (TestDatabase, TestFactory) {
    let db = TestDatabase::new().await;
    let factory = TestFactory::new(db.pool());
    (db, factory)
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

    let base_salary = Decimal::from_f64(50.0).unwrap();
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
    let (db, factory) = setup_test_context().await;
    let pool = db.pool();
    let test_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    let (employee_id, workspace_id) = setup_employee_with_salary(&factory, &pool, test_date).await;

    // Set rounding: 30m, floor
    update_workspace_rounding(&pool, workspace_id, 30, "floor").await;

    // Work: 08:00 - 10:45 (2h 45m = 165 mins)
    // Rounding 30m Floor: 165 / 30 = 5.5 -> Floor(5.5) = 5 units -> 5 * 30 = 150 mins (2.5 hours)
    let punch_in = Local
        .with_ymd_and_hms(2026, 1, 15, 8, 0, 0)
        .unwrap()
        .with_timezone(&Utc);
    let punch_out = Local
        .with_ymd_and_hms(2026, 1, 15, 10, 45, 0)
        .unwrap()
        .with_timezone(&Utc);
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create()
        .await;

    let payslip =
        PayrollService::calculate_monthly_payslip(&pool, employee_id, workspace_id, 2026, 1)
            .await
            .unwrap();

    // 2.5 hours * 100 = 250.0
    assert_eq!(payslip.base_pay_amount.to_f64().unwrap(), 250.0);
}

#[tokio::test]
async fn test_payroll_rounding_30m_ceil() {
    let (db, factory) = setup_test_context().await;
    let pool = db.pool();
    let test_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    let (employee_id, workspace_id) = setup_employee_with_salary(&factory, &pool, test_date).await;

    // Set rounding: 30m, ceil
    update_workspace_rounding(&pool, workspace_id, 30, "ceil").await;

    // Work: 08:00 - 08:15 (15 mins)
    // Rounding 30m Ceil: 15 / 30 = 0.5 -> Ceil(0.5) = 1 unit -> 30 mins (0.5 hours)
    let punch_in = Local
        .with_ymd_and_hms(2026, 1, 15, 8, 0, 0)
        .unwrap()
        .with_timezone(&Utc);
    let punch_out = Local
        .with_ymd_and_hms(2026, 1, 15, 8, 15, 0)
        .unwrap()
        .with_timezone(&Utc);
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create()
        .await;

    let payslip =
        PayrollService::calculate_monthly_payslip(&pool, employee_id, workspace_id, 2026, 1)
            .await
            .unwrap();

    // 0.5 hours * 100 = 50.0
    assert_eq!(payslip.base_pay_amount.to_f64().unwrap(), 50.0);
}

#[tokio::test]
async fn test_payroll_rounding_30m_round() {
    let (db, factory) = setup_test_context().await;
    let pool = db.pool();
    let test_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    let (employee_id, workspace_id) = setup_employee_with_salary(&factory, &pool, test_date).await;

    // Set rounding: 30m, round
    update_workspace_rounding(&pool, workspace_id, 30, "round").await;

    // Work: 08:00 - 08:45 (45 mins)
    // Rounding 30m Round: 45 / 30 = 1.5 -> Round(1.5) = 2 units -> 60 mins (1.0 hour)
    let punch_in = Local
        .with_ymd_and_hms(2026, 1, 15, 8, 0, 0)
        .unwrap()
        .with_timezone(&Utc);
    let punch_out = Local
        .with_ymd_and_hms(2026, 1, 15, 8, 45, 0)
        .unwrap()
        .with_timezone(&Utc);
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create()
        .await;

    let payslip =
        PayrollService::calculate_monthly_payslip(&pool, employee_id, workspace_id, 2026, 1)
            .await
            .unwrap();

    // 1.0 hour * 100 = 100.0
    assert_eq!(payslip.base_pay_amount.to_f64().unwrap(), 100.0);
}
