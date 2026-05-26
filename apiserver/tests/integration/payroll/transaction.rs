//! Payroll transaction boundary tests

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{Payslip, CreatePayslipSegmentArgs};
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;
use apiserver::repositories::payroll_repository::PayrollRepository;

#[tokio::test]
async fn test_persist_calculated_payslip_data_transaction_rollback() {
    let (_, _, user_id, factory, _): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .create()
        .await;

    let payroll_repo = apiserver::repositories::diesel_payroll_repository::DieselPayrollRepository::new(factory.diesel_pools.clone());
    let payslip_id = Uuid::new_v4();

    let payslip = Payslip {
        id: payslip_id,
        workspace_id: ws_id,
        employee_id: emp_id,
        period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        period_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        base_pay_amount: Decimal::from(1000),
        overtime_pay_amount: Decimal::ZERO,
        holiday_work_premium_amount: Decimal::ZERO,
        deductions_amount: Decimal::ZERO,
        net_pay_amount: Decimal::from(1000),
        status: "draft".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        standard_units: 100,
        overtime_units: 0,
    };

    // Valid segment
    let valid_segment = CreatePayslipSegmentArgs {
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        department_id: None,
        applied_base_salary: Decimal::from(500),
        applied_std_days: Decimal::from(10),
        hours_per_day: Decimal::from(8),
        actual_work_days: Decimal::from(10),
        paid_work_days: Decimal::from(10),
        amount: Decimal::from(500),
        std_days_source: Some("policy".to_string()),
    };

    // Invalid segment - intentionally use an invalid department_id that doesn't exist
    // to trigger a foreign key violation and fail the transaction
    let invalid_segment = CreatePayslipSegmentArgs {
        start_date: NaiveDate::from_ymd_opt(2026, 1, 16).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        department_id: Some(Uuid::new_v4()), // FK violation!
        applied_base_salary: Decimal::from(500),
        applied_std_days: Decimal::from(10),
        hours_per_day: Decimal::from(8),
        actual_work_days: Decimal::from(10),
        paid_work_days: Decimal::from(10),
        amount: Decimal::from(500),
        std_days_source: Some("policy".to_string()),
    };

    let segments = vec![valid_segment, invalid_segment];
    let incentives = vec![];
    let rules = vec![];

    let result = payroll_repo.persist_calculated_payslip_data(
        ws_id,
        &payslip,
        &segments,
        &incentives,
        &rules
    ).await;

    // Must return an error due to FK violation
    assert!(result.is_err(), "Expected transaction to fail due to FK violation");

    // Check that the payslip was rolled back (does not exist in DB)
    let fetched_payslips = payroll_repo.get_payslips_by_employee(ws_id, emp_id).await.unwrap();
    assert!(fetched_payslips.is_empty(), "Payslip should have been rolled back, but it exists!");
}
