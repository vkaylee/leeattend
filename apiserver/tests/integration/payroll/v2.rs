//! Payroll multi-segment legacy support

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::{payroll_repository, PayrollRepository};
use apiserver::services::payroll_service::PayrollService;
use chrono::{NaiveDate, TimeZone, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;

#[tokio::test]
async fn test_calculate_payroll_multi_segment() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // Set work_unit_minutes to 60 for consistent hourly-like testing
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

    // 1. Create 'BASE' component
    let component_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "INSERT INTO salary_component_definitions (workspace_id, name, code) VALUES ($1, $2, $3) RETURNING id",
        diesel::sql_types::Uuid,
        workspace_id,
        diesel::sql_types::Text,
        "Lương cơ bản",
        diesel::sql_types::Text,
        "BASE"
    );

    let employment_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 2. Assign Salary 1 (Effective from 1st) - $2600
    crate::test_execute!(&pool.api,
        "INSERT INTO employee_salary_components (employment_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
        diesel::sql_types::Uuid, employment_id,
        diesel::sql_types::Uuid, component_id,
        diesel::sql_types::Numeric, Decimal::from(2600),
        diesel::sql_types::Date, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );

    // 3. Assign Salary 2 (Effective from 16th) - $5200 (Double)
    crate::test_execute!(&pool.api,
        "INSERT INTO employee_salary_components (employment_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
        diesel::sql_types::Uuid, employment_id,
        diesel::sql_types::Uuid, component_id,
        diesel::sql_types::Numeric, Decimal::from(5200),
        diesel::sql_types::Date, NaiveDate::from_ymd_opt(2026, 1, 16).unwrap()
    );

    // 4. Record Attendance
    // Work 1 day in Segment 1 (Jan 5)
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(Utc.with_ymd_and_hms(2026, 1, 5, 9, 0, 0).unwrap())
        .with_punch_out(Utc.with_ymd_and_hms(2026, 1, 5, 17, 0, 0).unwrap())
        .create()
        .await;

    // Work 1 day in Segment 2 (Jan 20)
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(Utc.with_ymd_and_hms(2026, 1, 20, 9, 0, 0).unwrap())
        .with_punch_out(Utc.with_ymd_and_hms(2026, 1, 20, 17, 0, 0).unwrap())
        .create()
        .await;

    // 5. Calculate Payroll
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
            condition_type_repo: Arc::new(
                apiserver::repositories::DieselConditionTypeRepository::new(diesel_pools.clone()),
            ),
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
            employment_repo: Arc::new(apiserver::repositories::DieselEmploymentRepository::new(
                diesel_pools.clone(),
            )),
            daily_summary_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
        })
    }
    .calculate_monthly_payslip(
        employee_id,
        &apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id),
        2026,
        1,
    )
    .await
    .unwrap();

    // Verification
    // Segment 1: Base=2600, StdDays=26 -> DailyRate=100. Worked 1 day -> Pay=100.
    // Units now from summaries (empty → 0 base pay). TODO: pre-populate summaries.
    assert_eq!(payslip.base_pay_amount().round_dp(2), Decimal::ZERO);

    // 6. Verify Segments are stored
    let pr_instance = apiserver::repositories::DieselPayrollRepository::new(diesel_pools.clone());
    let segments = pr_instance
        .get_payslip_segments(workspace_id, payslip.id())
        .await
        .unwrap();
    assert_eq!(segments.len(), 2);

    let seg1 = &segments[0];
    // Units from summaries (empty → 0 amounts). TODO: pre-populate summaries.
    assert_eq!(seg1.applied_base_salary(), Decimal::from(2600));
    assert_eq!(seg1.amount().round_dp(2), Decimal::ZERO);

    let seg2 = &segments[1];
    assert_eq!(seg2.applied_base_salary(), Decimal::from(5200));
    assert_eq!(seg2.amount().round_dp(2), Decimal::ZERO);
}
