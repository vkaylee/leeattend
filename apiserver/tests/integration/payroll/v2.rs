//! Payroll multi-segment legacy support

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::payroll_repository;
use apiserver::services::payroll_service::PayrollService;
use chrono::{NaiveDate, TimeZone, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;

#[tokio::test]
async fn test_calculate_payroll_multi_segment() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // Set work_unit_minutes to 60 for consistent hourly-like testing
    sqlx::query(
        "UPDATE attendance_policy_versions SET rules = '{\"work_unit_minutes\": 60, \"rounding_policy\": \"none\"}'::jsonb WHERE policy_id = (SELECT id FROM attendance_policies WHERE workspace_id = $1 AND is_default = true)",
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

    // 1. Create 'BASE' component
    let component_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO salary_component_definitions (workspace_id, name, code) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(workspace_id)
    .bind("Lương cơ bản")
    .bind("BASE")
    .fetch_one(&pool)
    .await
    .unwrap();

    // 2. Assign Salary 1 (Effective from 1st) - $2600
    sqlx::query(
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
    )
    .bind(employee_id)
    .bind(component_id)
    .bind(Decimal::from(2600))
    .bind(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .execute(&pool)
    .await
    .unwrap();

    // 3. Assign Salary 2 (Effective from 16th) - $5200 (Double)
    sqlx::query(
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
    )
    .bind(employee_id)
    .bind(component_id)
    .bind(Decimal::from(5200))
    .bind(NaiveDate::from_ymd_opt(2026, 1, 16).unwrap())
    .execute(&pool)
    .await
    .unwrap();

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
        let pr: std::sync::Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> =
            std::sync::Arc::new(apiserver::repositories::DieselPayrollRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            ));
        let dr: std::sync::Arc<
            dyn apiserver::repositories::department_repository::DepartmentRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let tr: std::sync::Arc<
            dyn apiserver::repositories::time_entry_repository::TimeEntryRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let wr: std::sync::Arc<
            dyn apiserver::repositories::workspace_repository::WorkspaceRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let hr: std::sync::Arc<
            dyn apiserver::repositories::holiday_repository::HolidayRepositoryTrait,
        > = std::sync::Arc::new(apiserver::repositories::DieselHolidayRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: pr,
            department_repo: dr,
            time_entry_repo: tr,
            workspace_repo: wr,
            holiday_repo: hr,
            roster_repo: Arc::new(apiserver::repositories::DieselRosterRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            )),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    pool.clone(),
                    diesel_pools.clone(),
                ),
            ),

            pool: pool.clone(),
        })
    }
    .calculate_monthly_payslip(employee_id, workspace_id, 2026, 1)
    .await
    .unwrap();

    // Verification
    // Segment 1: Base=2600, StdDays=26 -> DailyRate=100. Worked 1 day -> Pay=100.
    // Segment 2: Base=5200, StdDays=26 -> DailyRate=200. Worked 1 day -> Pay=200.
    // Total Base Pay = 300.
    assert_eq!(payslip.base_pay_amount.round_dp(2), Decimal::from(300));

    // 6. Verify Segments are stored
    let segments = payroll_repository::get_payslip_segments(&pool, payslip.id)
        .await
        .unwrap();
    assert_eq!(segments.len(), 2);

    let seg1 = &segments[0];
    assert_eq!(seg1.applied_base_salary, Decimal::from(2600));
    assert_eq!(seg1.amount.round_dp(2), Decimal::from(100));

    let seg2 = &segments[1];
    assert_eq!(seg2.applied_base_salary, Decimal::from(5200));
    assert_eq!(seg2.amount.round_dp(2), Decimal::from(200));
}
