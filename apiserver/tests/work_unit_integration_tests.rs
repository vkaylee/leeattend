mod common;

use apiserver::services::payroll_service::PayrollService;
use chrono::{Datelike, Local, NaiveTime, TimeZone, Utc};
use common::db::TestDatabase;
use common::factory::TestFactory;
use rust_decimal::prelude::*;

#[tokio::test]
async fn test_work_unit_integer_integrity_flow() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    // 1. Setup Workspace with 30-minute units and Floor rounding
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    sqlx::query(
        "UPDATE workspaces SET work_unit_minutes = 30, rounding_policy = 'floor' WHERE id = $1",
    )
    .bind(ws_id)
    .execute(&pool)
    .await
    .unwrap();

    let employee_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(user_id)
        .create()
        .await;

    // Set salary: 300,000 VND per month
    // Std units per month: 26 days * 8 hours (16 units/day) = 416 units
    // Unit rate = 300,000 / 416 = 721.1538...
    let base_salary = Decimal::from(300000);
    
    // Create a 'BASE' salary component
    let component_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO salary_component_definitions (workspace_id, name, code, is_base_for_ot, is_taxable) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(ws_id)
    .bind("Lương cơ bản")
    .bind("BASE")
    .bind(true)
    .bind(true)
    .fetch_one(&pool)
    .await
    .unwrap();

    // Assign to employee
    sqlx::query(
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
    )
    .bind(employee_id)
    .bind(component_id)
    .bind(base_salary)
    .bind(Utc::now().date_naive())
    .execute(&pool)
    .await
    .unwrap();

    // 2. Create a shift and a 45-minute work session
    // Shift: 08:00 - 17:00
    let test_date = Utc::now().date_naive();
    factory
        .shift()
        .with_workspace(ws_id)
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .with_break_duration(0)
        .assign_to_employee(employee_id, test_date)
        .await;

    // Punch: 08:00 to 08:45 (45 minutes)
    // Rule: Floor(45/30) = 1 Unit.
    // If it were minutes, it would be 45. If it were fractional units, 1.5.
    // The business rule REQUIRES 1.
    let punch_in = Local
        .from_local_datetime(&test_date.and_time(NaiveTime::from_hms_opt(8, 0, 0).unwrap()))
        .single()
        .unwrap()
        .with_timezone(&Utc);
    let punch_out = Local
        .from_local_datetime(&test_date.and_time(NaiveTime::from_hms_opt(8, 45, 0).unwrap()))
        .single()
        .unwrap()
        .with_timezone(&Utc);

    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create()
        .await;

    // 3. Verify Report Service returns 1 unit
    let reports = apiserver::services::report_service::ReportService::get_daily_attendance_report(
        &pool,
        ws_id,
        test_date,
        test_date,
        None,
        None,
    )
    .await
    .unwrap();
    let reports_data = reports.data;

    // Assert
    assert_eq!(reports_data.len(), 1);
    assert_eq!(
        reports_data[0].work_units, 1,
        "Should have 1 work unit (1 minute)"
    );

    // 4. Verify Payroll Service uses 1 unit for calculation
    // Total units = 1 standard unit + 0 OT
    // Unit rate = 300,000 / (26 * 16) = 721.1538...
    // Total pay should be base_salary (monthly) since it's standard hours.
    // Wait, PayrollService for monthly salary returns the full base_pay if standard hours are worked?
    // No, current logic returns full base_pay for monthly + OT pay.
    // We want to check OT specifically to see the unit rate application.

    // Let's add 45 mins of OT (08:00 - 17:45)
    // Std: 08:00 - 17:00 = 9h = 540m = 18 units.
    // OT: 17:00 - 17:45 = 45m = 1 unit (Floor).

    // Cleanup previous entry
    sqlx::query("DELETE FROM time_entries WHERE employee_id = $1")
        .bind(employee_id)
        .execute(&pool)
        .await
        .unwrap();

    let punch_out_ot = Local
        .from_local_datetime(&test_date.and_time(NaiveTime::from_hms_opt(17, 45, 0).unwrap()))
        .single()
        .unwrap()
        .with_timezone(&Utc);
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out_ot)
        .create()
        .await;

    let payslip = PayrollService::calculate_monthly_payslip(
        &pool,
        employee_id,
        ws_id,
        test_date.year(),
        test_date.month(),
    )
    .await
    .unwrap();

    // OT Units should be exactly 1
    assert_eq!(payslip.overtime_units, 1);

    let expected_unit_rate = base_salary / Decimal::from(26 * 16); // 208 hours / 0.5 = 416 units
    let expected_ot_pay =
        (expected_unit_rate * Decimal::from(1) * Decimal::from_f64(1.5).unwrap()).round_dp(2);

    assert_eq!(payslip.overtime_pay_amount.round_dp(2), expected_ot_pay);
}
