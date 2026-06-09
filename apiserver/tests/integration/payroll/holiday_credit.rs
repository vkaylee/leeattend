//! Tests for payroll auto-credit of paid holidays

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::services::payroll_service::PayrollService;
use chrono::{NaiveDate, TimeZone, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;

#[tokio::test]
async fn test_payroll_credits_paid_holiday() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    crate::test_execute!(
        &diesel_pools.api,
        "UPDATE attendance_policy_versions SET rules = '{\"work_unit_minutes\": 60, \"rounding_policy\": \"none\"}'::jsonb WHERE policy_id = (SELECT id FROM attendance_policies WHERE workspace_id = $1 AND is_default = true)",
        diesel::sql_types::Uuid, workspace_id
    );

    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_user(user_id)
        .create()
        .await;

    let employment_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 1. Create 'BASE' component
    let component_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &diesel_pools.api,
        "INSERT INTO salary_component_definitions (workspace_id, name, code) VALUES ($1, $2, $3) RETURNING id",
        diesel::sql_types::Uuid, workspace_id,
        diesel::sql_types::Text, "Lương cơ bản",
        diesel::sql_types::Text, "BASE"
    );

    crate::test_execute!(
        &diesel_pools.api,
        "INSERT INTO employee_salary_components (employment_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
        diesel::sql_types::Uuid, employment_id,
        diesel::sql_types::Uuid, component_id,
        diesel::sql_types::Numeric, Decimal::from(2600),
        diesel::sql_types::Date, NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()
    );

    let holiday_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    crate::test_execute!(
        &diesel_pools.api,
        "INSERT INTO holidays (id, workspace_id, name, date, holiday_type, is_paid, ot_multiplier) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        diesel::sql_types::Uuid, uuid::Uuid::new_v4(),
        diesel::sql_types::Uuid, workspace_id,
        diesel::sql_types::Text, "Labor Day",
        diesel::sql_types::Date, holiday_date,
        diesel::sql_types::Text, "national",
        diesel::sql_types::Bool, true,
        diesel::sql_types::Numeric, Decimal::from(2)
    );

    // 4. Record Attendance for ONE day only (2026-05-02)
    // The employee will NOT clock in on the holiday (2026-05-01).
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(Utc.with_ymd_and_hms(2026, 5, 2, 9, 0, 0).unwrap())
        .with_punch_out(Utc.with_ymd_and_hms(2026, 5, 2, 17, 0, 0).unwrap())
        .create()
        .await;

    // 5. Calculate Payroll for May 2026
    let payslip = {
        let pr: std::sync::Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> =
            std::sync::Arc::new(apiserver::repositories::DieselPayrollRepository::new(
                diesel_pools.clone(),
            ));
        let dr: std::sync::Arc<
            dyn apiserver::repositories::department_repository::DepartmentRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            diesel_pools.clone(),
        ));
        let tr: std::sync::Arc<
            dyn apiserver::repositories::time_entry_repository::TimeEntryRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
            diesel_pools.clone(),
        ));
        let wr: std::sync::Arc<
            dyn apiserver::repositories::workspace_repository::WorkspaceRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            diesel_pools.clone(),
        ));
        let hr: std::sync::Arc<dyn apiserver::repositories::holiday_repository::HolidayRepository> =
            std::sync::Arc::new(apiserver::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            ));
        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: pr,
            department_repo: dr,
            time_entry_repo: tr,
            workspace_repo: wr,
            holiday_repo: hr,
            roster_repo: Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            leave_repo: Arc::new(apiserver::repositories::DieselLeaveRequestRepository::new(
                diesel_pools.clone(),
            )),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            employee_repo: Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
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
            contract_repo: Arc::new(
                apiserver::repositories::DieselEmployeeContractRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            employment_repo: Arc::new(apiserver::repositories::DieselEmploymentRepository::new(
                diesel_pools.clone(),
            )),
        })
    }
    .calculate_monthly_payslip(employee_id, workspace_id, 2026, 5)
    .await
    .unwrap();

    // Verification
    // Worked 1 actual day (8 hours) -> standard_units: 8
    // + Auto-credited 1 paid holiday (8 hours) -> standard_units: 8
    // Total standard units should be 16 (equivalent to 2 work days).
    assert_eq!(payslip.standard_units, 16);

    // Dailyrate = 100.
    // 2 standard days = Pay = 200.
    assert_eq!(payslip.base_pay_amount.round_dp(2), Decimal::from(200));
}
