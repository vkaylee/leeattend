mod common;

use apiserver::services::payroll_service::PayrollService;
use chrono::{Datelike, Local, NaiveDate, NaiveTime, TimeZone, Utc};
use common::db::TestDatabase;
use common::factory::TestFactory;
use rust_decimal::prelude::*;
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_test_db() -> (TestDatabase, TestFactory) {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.pool());
    (test_db, factory)
}

async fn setup_employee_with_salary(
    factory: &TestFactory,
    pool: &PgPool,
    test_date: NaiveDate,
) -> (Uuid, Uuid, Decimal) {
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // Set work_unit_minutes to 60 for consistent hourly-like testing
    sqlx::query(
        "UPDATE workspaces SET work_unit_minutes = 60, rounding_policy = 'none' WHERE id = $1",
    )
    .bind(workspace_id)
    .execute(pool)
    .await
    .unwrap();

    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_user(user_id)
        .create()
        .await;

    let base_salary = Decimal::from_f64(100.0).unwrap(); // 100 units per hour (since unit is 60m)

    // Create a 'BASE' salary component
    let component_id: Uuid = sqlx::query_scalar(
        "INSERT INTO salary_component_definitions (workspace_id, name, code, is_base_for_ot, is_taxable) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(workspace_id)
    .bind("Lương cơ bản")
    .bind("BASE")
    .bind(true)
    .bind(true)
    .fetch_one(pool)
    .await
    .unwrap();

    // Assign to employee
    sqlx::query(
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
    )
    .bind(employee_id)
    .bind(component_id)
    .bind(base_salary)
    .bind(test_date)
    .execute(pool)
    .await
    .unwrap();

    (employee_id, workspace_id, base_salary)
}

async fn setup_employee_with_monthly_salary(
    factory: &TestFactory,
    pool: &PgPool,
    test_date: NaiveDate,
) -> (Uuid, Uuid, Decimal) {
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // Set work_unit_minutes to 60 for consistent hourly-like testing
    sqlx::query(
        "UPDATE workspaces SET work_unit_minutes = 60, rounding_policy = 'none' WHERE id = $1",
    )
    .bind(workspace_id)
    .execute(pool)
    .await
    .unwrap();

    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_user(user_id)
        .create()
        .await;

    let base_salary = Decimal::from_f64(3000.0).unwrap();
    
    // Effective from start of the month to ensure coverage
    let effective_date = test_date.with_day(1).unwrap();

    // Create a 'BASE' salary component
    let component_id: Uuid = sqlx::query_scalar(
        "INSERT INTO salary_component_definitions (workspace_id, name, code, is_base_for_ot, is_taxable) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(workspace_id)
    .bind("Lương cơ bản")
    .bind("BASE")
    .bind(true)
    .bind(true)
    .fetch_one(pool)
    .await
    .unwrap();

    // Assign to employee
    sqlx::query(
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
    )
    .bind(employee_id)
    .bind(component_id)
    .bind(base_salary)
    .bind(effective_date)
    .execute(pool)
    .await
    .unwrap();

    (employee_id, workspace_id, base_salary)
}

#[tokio::test]
async fn test_calculate_payroll_monthly() {
    let (test_db, factory) = setup_test_db().await;
    let pool = test_db.pool();
    let test_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    let (employee_id, workspace_id, _base_salary) =
        setup_employee_with_monthly_salary(&factory, &pool, test_date).await;

    let payslip =
        PayrollService::calculate_monthly_payslip(&pool, employee_id, workspace_id, 2026, 1)
            .await
            .unwrap();

    // New Logic: No attendance = No Pay
    assert_eq!(payslip.base_pay_amount, Decimal::ZERO);
}

#[tokio::test]
async fn test_calculate_payroll_monthly_with_overtime() {
    let (test_db, factory) = setup_test_db().await;
    let pool = test_db.pool();
    let test_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    let (employee_id, workspace_id, base_salary) =
        setup_employee_with_monthly_salary(&factory, &pool, test_date).await;

    // Create a shift: 8 hours work (09:00 - 17:00)
    let _shift_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .with_break_duration(0)
        .assign_to_employee(employee_id, test_date)
        .await;

    // Work 10 hours (09:00 - 19:00) -> 2 hours overtime
    let punch_in = Local
        .with_ymd_and_hms(2026, 1, 15, 9, 0, 0)
        .unwrap()
        .with_timezone(&Utc);
    let punch_out = Local
        .with_ymd_and_hms(2026, 1, 15, 19, 0, 0)
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

    // Standard Days = 26 (Default). Units Per Day = 8 (since unit=60m).
    // Daily Rate = 3000 / 26 = 115.3846...
    // Worked 1 day (8 standard hours) -> 115.38
    let expected_base = (base_salary / Decimal::from(26)).round_dp(2);
    assert_eq!(payslip.base_pay_amount.round_dp(2), expected_base);

    // 2 hours overtime = 2 units.
    // Monthly Standard Units = 26 * 8 = 208.
    // Hourly Rate (Unit Rate) = 3000 / 208 = 14.423...
    // OT = 2 * 14.423 * 1.5 = 43.269... -> 43.27
    let expected_ot =
        ((base_salary / Decimal::from(208)) * Decimal::from_f64(2.0 * 1.5).unwrap()).round_dp(2);
    assert_eq!(payslip.overtime_pay_amount.round_dp(2), expected_ot);
}

#[tokio::test]
async fn test_early_punch_in_grace_period() {
    let (test_db, factory) = setup_test_db().await;
    let pool = test_db.pool();

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

    let punch_in_time_case1 = Local
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            8,
            0,
            0,
        )
        .unwrap()
        .with_timezone(&Utc);
    let punch_out_time_case1 = Local
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            17,
            0,
            0,
        )
        .unwrap()
        .with_timezone(&Utc);
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
        payslip_case1.overtime_units,
        0,
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

    let punch_in_time_case2 = Local
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            7,
            45,
            0,
        )
        .unwrap()
        .with_timezone(&Utc);
    let punch_out_time_case2 = Local
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            17,
            0,
            0,
        )
        .unwrap()
        .with_timezone(&Utc);
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
        payslip_case2.overtime_units,
        0,
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

    let punch_in_time_case3 = Local
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            7,
            30,
            0,
        )
        .unwrap()
        .with_timezone(&Utc);
    let punch_out_time_case3 = Local
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            17,
            0,
            0,
        )
        .unwrap()
        .with_timezone(&Utc);
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
        payslip_case3.overtime_units,
        0,
        "Case 3: No early punch-in overtime expected exactly at grace period start"
    );

    // Case 4: Punch in before grace period start (07:00)
    factory.cleanup().await;
    let (employee_id, workspace_id, _base_salary) =
        setup_employee_with_salary(&factory, &pool, test_date).await;
    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(shift_start_time, shift_end_time)
        .with_break_duration(break_duration_minutes)
        .with_grace_period(grace_period_minutes)
        .with_allow_early_ot(true)
        .assign_to_employee(employee_id, test_date)
        .await;

    let punch_in_time_case4 = Local
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            7,
            0,
            0,
        )
        .unwrap()
        .with_timezone(&Utc);
    let punch_out_time_case4 = Local
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            17,
            0,
            0,
        )
        .unwrap()
        .with_timezone(&Utc);
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
    // Overtime minutes: 07:00 to 08:00 = 60 minutes (1.0 hour) = 1 Unit
    assert_eq!(
        payslip_case4.overtime_units, 1,
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

    let punch_in_time_case5 = Local
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            8,
            15,
            0,
        )
        .unwrap()
        .with_timezone(&Utc);
    let punch_out_time_case5 = Local
        .with_ymd_and_hms(
            test_date.year(),
            test_date.month(),
            test_date.day(),
            17,
            0,
            0,
        )
        .unwrap()
        .with_timezone(&Utc);
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
        payslip_case5.overtime_units,
        0,
        "Case 5: No early punch-in overtime expected when punching in after shift start"
    );

    factory.cleanup().await;
}

#[tokio::test]
async fn test_calculate_payroll_no_salary() {
    let (test_db, factory) = setup_test_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let result = PayrollService::calculate_monthly_payslip(&pool, emp_id, ws_id, 2026, 1).await;

    // After cleanup, no salary history fallback means payroll is calculated with 0 base pay
    assert!(result.is_ok());
    let payslip = result.unwrap();
    assert_eq!(payslip.base_pay_amount, Decimal::ZERO);
}

#[tokio::test]
async fn test_payslip_calculation_with_mixed_ot_base_components_integrated() {
    let (test_db, factory) = setup_test_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

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

    let test_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    
    let base_salary_id: Uuid = sqlx::query_scalar(
        "INSERT INTO salary_component_definitions (workspace_id, name, code, is_base_for_ot, is_taxable) 
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(workspace_id)
    .bind("Lương cơ bản")
    .bind("BASE")
    .bind(true)
    .bind(true)
    .fetch_one(&pool)
    .await
    .unwrap();

    let allowance_id: Uuid = sqlx::query_scalar(
        "INSERT INTO salary_component_definitions (workspace_id, name, code, is_base_for_ot, is_taxable) 
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(workspace_id)
    .bind("Phụ cấp ăn trưa")
    .bind("LUNCH")
    .bind(false)
    .bind(true)
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date) 
         VALUES ($1, $2, $3, $4), ($1, $5, $6, $7)",
    )
    .bind(employee_id)
    .bind(base_salary_id)
    .bind(Decimal::new(1000000, 0))
    .bind(test_date)
    .bind(allowance_id)
    .bind(Decimal::new(500000, 0))
    .bind(test_date)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO payroll_settings (workspace_id, fixed_std_days, hours_per_day, effective_date) 
         VALUES ($1, $2, $3, $4)",
    )
    .bind(workspace_id)
    .bind(Decimal::new(25, 0))
    .bind(Decimal::new(8, 0))
    .bind(test_date)
    .execute(&pool)
    .await
    .unwrap();

    let shift_id = factory.shift().with_workspace(workspace_id).create().await;
    sqlx::query("INSERT INTO shift_rosters (employee_id, shift_id, date) VALUES ($1, $2, $3)")
        .bind(employee_id).bind(shift_id).bind(test_date).execute(&pool).await.unwrap();

    let tz = chrono_tz::UTC;
    let punch_in = tz.from_local_datetime(&test_date.and_hms_opt(8, 0, 0).unwrap()).single().unwrap().with_timezone(&Utc);
    let punch_out = tz.from_local_datetime(&test_date.and_hms_opt(19, 0, 0).unwrap()).single().unwrap().with_timezone(&Utc);

    sqlx::query(
        "INSERT INTO time_entries (employee_id, workspace_id, punch_in_time, punch_out_time, entry_type) 
         VALUES ($1, $2, $3, $4, 'work')",
    )
    .bind(employee_id).bind(workspace_id).bind(punch_in).bind(punch_out).execute(&pool).await.unwrap();

    let payslip = PayrollService::calculate_monthly_payslip(&pool, employee_id, workspace_id, 2026, 1)
        .await
        .expect("Should calculate payslip");

    assert_eq!(payslip.base_pay_amount, Decimal::new(60000, 0));
    assert_eq!(payslip.overtime_pay_amount, Decimal::new(15000, 0));
    assert_eq!(payslip.net_pay_amount, Decimal::new(75000, 0));
}
