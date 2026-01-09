mod common;

use apiserver::services::payroll_service::PayrollService;
use chrono::{Datelike, NaiveDate, NaiveTime, TimeZone, Utc};
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

    let base_salary = Decimal::from_f64(100.0).unwrap(); // 100 units per hour
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
#[ignore = "Needs database running and migrations applied"]
async fn test_early_punch_in_grace_period() {
    let pool = setup_test_db().await;
    let factory = TestFactory::new(pool.clone());

    let test_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    // Shift settings
    let shift_start_time = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
    let shift_end_time = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
    let break_duration_minutes = 60;
    let grace_period_minutes = 30; // Grace period starts at 07:30

    // Case 1: Punch in exactly at shift start (08:00)
    factory.cleanup().await;
    let (employee_id, workspace_id, _) =
        setup_employee_with_salary(&factory, &pool, test_date).await;
    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(shift_start_time, shift_end_time)
        .with_break_duration(break_duration_minutes)
        .with_grace_period(grace_period_minutes)
        .assign_to_employee(employee_id, test_date)
        .await;

    let punch_in_time_case1 = Utc
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            8,
            0,
            0,
        )
        .unwrap();
    let punch_out_time_case1 = Utc
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            17,
            0,
            0,
        )
        .unwrap();
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in_time_case1)
        .with_punch_out(punch_out_time_case1)
        .create()
        .await;

    let payslip_case1 = PayrollService::calculate_monthly_payslip(
        &pool,
        employee_id,
        workspace_id,
        test_date.year(),
        test_date.month(),
    )
    .await
    .unwrap();
    assert_eq!(
        payslip_case1.overtime_pay_amount.to_f64().unwrap(),
        0.0,
        "Case 1: No early punch-in overtime expected"
    );

    // Case 2: Punch in within grace period (07:45)
    factory.cleanup().await;
    let (employee_id, workspace_id, _) =
        setup_employee_with_salary(&factory, &pool, test_date).await;
    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(shift_start_time, shift_end_time)
        .with_break_duration(break_duration_minutes)
        .with_grace_period(grace_period_minutes)
        .assign_to_employee(employee_id, test_date)
        .await;

    let punch_in_time_case2 = Utc
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            7,
            45,
            0,
        )
        .unwrap();
    let punch_out_time_case2 = Utc
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            17,
            0,
            0,
        )
        .unwrap();
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in_time_case2)
        .with_punch_out(punch_out_time_case2)
        .create()
        .await;

    let payslip_case2 = PayrollService::calculate_monthly_payslip(
        &pool,
        employee_id,
        workspace_id,
        test_date.year(),
        test_date.month(),
    )
    .await
    .unwrap();
    assert_eq!(
        payslip_case2.overtime_pay_amount.to_f64().unwrap(),
        0.0,
        "Case 2: No early punch-in overtime expected within grace period"
    );

    // Case 3: Punch in exactly at grace period start (07:30)
    factory.cleanup().await;
    let (employee_id, workspace_id, _) =
        setup_employee_with_salary(&factory, &pool, test_date).await;
    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(shift_start_time, shift_end_time)
        .with_break_duration(break_duration_minutes)
        .with_grace_period(grace_period_minutes)
        .assign_to_employee(employee_id, test_date)
        .await;

    let punch_in_time_case3 = Utc
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            7,
            30,
            0,
        )
        .unwrap();
    let punch_out_time_case3 = Utc
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            17,
            0,
            0,
        )
        .unwrap();
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in_time_case3)
        .with_punch_out(punch_out_time_case3)
        .create()
        .await;

    let payslip_case3 = PayrollService::calculate_monthly_payslip(
        &pool,
        employee_id,
        workspace_id,
        test_date.year(),
        test_date.month(),
    )
    .await
    .unwrap();
    assert_eq!(
        payslip_case3.overtime_pay_amount.to_f64().unwrap(),
        0.0,
        "Case 3: No early punch-in overtime expected exactly at grace period start"
    );

    // Case 4: Punch in before grace period start (07:00)
    factory.cleanup().await;
    let (employee_id, workspace_id, base_salary) =
        setup_employee_with_salary(&factory, &pool, test_date).await;
    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(shift_start_time, shift_end_time)
        .with_break_duration(break_duration_minutes)
        .with_grace_period(grace_period_minutes)
        .assign_to_employee(employee_id, test_date)
        .await;

    let punch_in_time_case4 = Utc
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            7,
            0,
            0,
        )
        .unwrap();
    let punch_out_time_case4 = Utc
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            17,
            0,
            0,
        )
        .unwrap();
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in_time_case4)
        .with_punch_out(punch_out_time_case4)
        .create()
        .await;

    let payslip_case4 = PayrollService::calculate_monthly_payslip(
        &pool,
        employee_id,
        workspace_id,
        test_date.year(),
        test_date.month(),
    )
    .await
    .unwrap();
    // Actual punch-in at 07:00. Shift starts at 08:00. Grace period is 30 mins (starts 07:30).
    // Since 07:00 < 07:30, effective punch-in is 07:00.
    // Overtime minutes: 07:00 to 08:00 = 60 minutes (1.0 hour)
    let expected_overtime_pay_case4 =
        base_salary * Decimal::from_f64(1.0).unwrap() * Decimal::from_f64(1.5).unwrap();
    assert_eq!(
        payslip_case4.overtime_pay_amount, expected_overtime_pay_case4,
        "Case 4: Early punch-in overtime expected before grace period"
    );

    // Case 5: Punch in after shift start (08:15)
    factory.cleanup().await;
    let (employee_id, workspace_id, _) =
        setup_employee_with_salary(&factory, &pool, test_date).await;
    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(shift_start_time, shift_end_time)
        .with_break_duration(break_duration_minutes)
        .with_grace_period(grace_period_minutes)
        .assign_to_employee(employee_id, test_date)
        .await;

    let punch_in_time_case5 = Utc
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            8,
            15,
            0,
        )
        .unwrap();
    let punch_out_time_case5 = Utc
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            17,
            0,
            0,
        )
        .unwrap();
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in_time_case5)
        .with_punch_out(punch_out_time_case5)
        .create()
        .await;

    let payslip_case5 = PayrollService::calculate_monthly_payslip(
        &pool,
        employee_id,
        workspace_id,
        test_date.year(),
        test_date.month(),
    )
    .await
    .unwrap();
    assert_eq!(
        payslip_case5.overtime_pay_amount.to_f64().unwrap(),
        0.0,
        "Case 5: No early punch-in overtime expected when punching in after shift start"
    );

    factory.cleanup().await;
}
