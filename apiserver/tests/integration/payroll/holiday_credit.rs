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
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

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
        "INSERT INTO salary_component_definitions (workspace_id, name, code, is_base_for_ot, is_taxable) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(workspace_id)
    .bind("Lương cơ bản")
    .bind("BASE")
    .bind(true)
    .bind(true)
    .fetch_one(&pool)
    .await
    .unwrap();

    // 2. Assign Salary (Effective from 1st)
    // 26 standard days assumed per month. So $2600 / 26 = $100 per day.
    sqlx::query(
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
    )
    .bind(employee_id)
    .bind(component_id)
    .bind(Decimal::from(2600))
    .bind(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap())
    .execute(&pool)
    .await
    .unwrap();

    // 3. Create a paid holiday on 2026-05-01 (Labor Day)
    let holiday_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    sqlx::query(
        "INSERT INTO holidays (workspace_id, name, date, holiday_type, is_paid, ot_multiplier) 
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(workspace_id)
    .bind("Labor Day")
    .bind(holiday_date)
    .bind("national")
    .bind(true)
    .bind(Decimal::from(2)) // 200% logic
    .execute(&pool)
    .await
    .unwrap();

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
            std::sync::Arc::new(
                apiserver::repositories::payroll_repository::SqlxPayrollRepository::new(
                    pool.clone(),
                ),
            );
        let dr: std::sync::Arc<
            dyn apiserver::repositories::department_repository::DepartmentRepository,
        > = std::sync::Arc::new(
            apiserver::repositories::department_repository::SqlxDepartmentRepository::new(
                pool.clone(),
            ),
        );
        let tr: std::sync::Arc<
            dyn apiserver::repositories::time_entry_repository::TimeEntryRepository,
        > = std::sync::Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(
            pool.clone(),
        ));
        let wr: std::sync::Arc<
            dyn apiserver::repositories::workspace_repository::WorkspaceRepository,
        > = std::sync::Arc::new(apiserver::repositories::SqlxWorkspaceRepository {
            pool: pool.clone(),
        });
        let hr: std::sync::Arc<
            dyn apiserver::repositories::holiday_repository::HolidayRepositoryTrait,
        > = std::sync::Arc::new(
            apiserver::repositories::holiday_repository::SqlxHolidayRepository::new(pool.clone()),
        );
        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: pr,
            department_repo: dr,
            time_entry_repo: tr,
            workspace_repo: wr,
            holiday_repo: hr,
            roster_repo: Arc::new(apiserver::repositories::roster::SqlxRosterRepository::new(
                pool.clone(),
            )),
            pool: pool.clone(),
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
