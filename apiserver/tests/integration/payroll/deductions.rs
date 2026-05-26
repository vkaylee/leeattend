//! Test deductions in `net_pay` formula (Issue 7)
//!
//! Verifies that `net_pay` = `base_pay` + `overtime_pay` + incentive - deductions
//! Currently deductions = 0, so `net_pay` should equal `base_pay` + `overtime_pay` + incentive.

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::services::payroll_service::PayrollService;
use chrono::{NaiveDate, TimeZone, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;

#[tokio::test]
async fn test_net_pay_formula_with_zero_deductions() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    crate::test_execute!(&pool.api,
        "UPDATE attendance_policy_versions SET rules = '{\"work_unit_minutes\": 60, \"rounding_policy\": \"none\"}'::jsonb WHERE policy_id = (SELECT id FROM attendance_policies WHERE workspace_id = $1 AND is_default = true)",
        diesel::sql_types::Uuid, workspace_id
    );

    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_user(user_id)
        .create()
        .await;

    // Create BASE component
    let component_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "INSERT INTO salary_component_definitions (workspace_id, name, code) VALUES ($1, $2, $3) RETURNING id",
        diesel::sql_types::Uuid,
        workspace_id,
        diesel::sql_types::Text,
        "Base",
        diesel::sql_types::Text,
        "BASE"
    );

    crate::test_execute!(&pool.api,
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
        diesel::sql_types::Uuid, employee_id,
        diesel::sql_types::Uuid, component_id,
        diesel::sql_types::Numeric, Decimal::from(2600),
        diesel::sql_types::Date, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );

    // Create time entries to generate base pay
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(Utc.with_ymd_and_hms(2026, 1, 5, 9, 0, 0).unwrap())
        .with_punch_out(Utc.with_ymd_and_hms(2026, 1, 5, 17, 0, 0).unwrap())
        .create()
        .await;

    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(Utc.with_ymd_and_hms(2026, 1, 6, 9, 0, 0).unwrap())
        .with_punch_out(Utc.with_ymd_and_hms(2026, 1, 6, 17, 0, 0).unwrap())
        .create()
        .await;

    // Calculate payroll
    let payslip = {
        let pr = Arc::new(apiserver::repositories::DieselPayrollRepository::new(
            diesel_pools.clone(),
        ));
        let dr = Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            diesel_pools.clone(),
        ));
        let tr = Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
            diesel_pools.clone(),
        ));
        let wr = Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            diesel_pools.clone(),
        ));
        let hr = Arc::new(apiserver::repositories::DieselHolidayRepository::new(
            diesel_pools.clone(),
        ));
        let rr = Arc::new(apiserver::repositories::DieselRosterRepository::new(
            diesel_pools.clone(),
        ));
        let lr = Arc::new(apiserver::repositories::DieselLeaveRequestRepository::new(
            diesel_pools.clone(),
        ));
        let er = Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
            diesel_pools.clone(),
        ));
        let atr = Arc::new(
            apiserver::repositories::DieselAttendancePolicyRepository::new(diesel_pools.clone()),
        );

        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: pr,
            department_repo: dr,
            time_entry_repo: tr,
            workspace_repo: wr,
            holiday_repo: hr,
            roster_repo: rr,
            leave_repo: lr,
            employee_repo: er,
            policy_repo: atr,
            overtime_policy_repo: Arc::new(
                apiserver::repositories::DieselOvertimePolicyRepository::new(diesel_pools.clone()),
            ),
            salary_base_repo: std::sync::Arc::new(
                apiserver::repositories::DieselSalaryBaseRepository::new(diesel_pools.clone()),
            ),

            roster_provider: std::sync::Arc::new(
                apiserver::services::payroll_service::DirectRosterDataProvider(
                    std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                        diesel_pools.clone(),
                    )),
                ),
            ),
        })
    }
    .calculate_monthly_payslip(employee_id, workspace_id, 2026, 1)
    .await
    .unwrap();

    // Verify net_pay formula: net_pay = base_pay + overtime_pay - deductions
    // (Note: incentive is not a separate field in Payslip model)
    // Since deductions = 0 currently:
    let expected_net_pay = payslip.base_pay_amount + payslip.overtime_pay_amount;

    assert_eq!(
        payslip.net_pay_amount, expected_net_pay,
        "Net pay should equal base_pay + overtime_pay when deductions = 0"
    );

    // Also verify deductions is zero (current implementation)
    assert_eq!(
        payslip.deductions_amount,
        Decimal::ZERO,
        "Deductions should be zero in current implementation"
    );

    // Ensure base pay was actually calculated
    assert!(
        payslip.base_pay_amount > Decimal::ZERO,
        "Base pay should be > 0"
    );
}
