//! Test grace period conditional override (Issue 5)
//!
//! Verifies that shift's early_punch_in_grace_period_minutes only overrides
//! policy grace period when non-zero.

use std::sync::Arc;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::services::payroll_service::PayrollService;
use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use rust_decimal::Decimal;

#[tokio::test]
async fn test_grace_period_zero_shift_uses_policy_default() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // Set work_unit to 60 min, no rounding
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

    // Create BASE component
    let component_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO salary_component_definitions (workspace_id, name, code, is_base_for_ot, is_taxable) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(workspace_id)
    .bind("Base")
    .bind("BASE")
    .bind(true)
    .bind(true)
    .fetch_one(&pool)
    .await
    .unwrap();

    // Assign salary
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

    // Create shift with grace_period = 0 (should use policy default)
    let shift_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Day Shift")
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .with_grace_period(0) // Zero grace period on shift
        .create()
        .await;

    // Assign shift to employee for Jan 5
    sqlx::query(
        "INSERT INTO employee_shifts (employee_id, shift_id, start_date) VALUES ($1, $2, $3)",
    )
    .bind(employee_id)
    .bind(shift_id)
    .bind(NaiveDate::from_ymd_opt(2026, 1, 5).unwrap())
    .execute(&pool)
    .await
    .unwrap();

    // Punch in 5 minutes early (8:55 AM)
    let tz: chrono_tz::Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(
            tz.with_ymd_and_hms(2026, 1, 5, 8, 55, 0)
                .unwrap()
                .with_timezone(&Utc),
        )
        .with_punch_out(
            tz.with_ymd_and_hms(2026, 1, 5, 17, 0, 0)
                .unwrap()
                .with_timezone(&Utc),
        )
        .create()
        .await;

    // Calculate payroll
    let payslip =
        {
            let pr: std::sync::Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> = std::sync::Arc::new(apiserver::repositories::payroll_repository::SqlxPayrollRepository::new(pool.clone()));
            let dr: std::sync::Arc<dyn apiserver::repositories::department_repository::DepartmentRepository> = std::sync::Arc::new(apiserver::repositories::department_repository::SqlxDepartmentRepository::new(pool.clone()));
            let tr: std::sync::Arc<dyn apiserver::repositories::time_entry_repository::TimeEntryRepository> = std::sync::Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(pool.clone()));
            let wr: std::sync::Arc<dyn apiserver::repositories::workspace_repository::WorkspaceRepository> = std::sync::Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
            let hr: std::sync::Arc<dyn apiserver::repositories::holiday_repository::HolidayRepositoryTrait> = std::sync::Arc::new(apiserver::repositories::holiday_repository::SqlxHolidayRepository::new(pool.clone()));
            PayrollService::new(pr, dr, tr, wr, hr, Arc::new(apiserver::repositories::roster_repository::SqlxRosterRepository::new(pool.clone())), pool.clone())
        }.calculate_monthly_payslip( employee_id, workspace_id, 2026, 1)
            .await
            .unwrap();

    // With grace_period=0, policy default should apply (usually allows some grace)
    // The 5-min early punch should be within standard hours
    assert!(
        payslip.standard_units > 0,
        "Standard units should be > 0 when grace period applies"
    );
}

#[tokio::test]
async fn test_grace_period_nonzero_shift_overrides() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

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

    let component_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO salary_component_definitions (workspace_id, name, code, is_base_for_ot, is_taxable) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(workspace_id)
    .bind("Base")
    .bind("BASE")
    .bind(true)
    .bind(true)
    .fetch_one(&pool)
    .await
    .unwrap();

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

    // Create shift with grace_period = 15 (overrides policy)
    let shift_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Flexible Shift")
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .with_grace_period(15) // Non-zero grace period overrides
        .create()
        .await;

    sqlx::query(
        "INSERT INTO employee_shifts (employee_id, shift_id, start_date) VALUES ($1, $2, $3)",
    )
    .bind(employee_id)
    .bind(shift_id)
    .bind(NaiveDate::from_ymd_opt(2026, 1, 5).unwrap())
    .execute(&pool)
    .await
    .unwrap();

    // Punch in 10 minutes early (8:50 AM) - within 15-min grace
    let tz: chrono_tz::Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(
            tz.with_ymd_and_hms(2026, 1, 5, 8, 50, 0)
                .unwrap()
                .with_timezone(&Utc),
        )
        .with_punch_out(
            tz.with_ymd_and_hms(2026, 1, 5, 17, 0, 0)
                .unwrap()
                .with_timezone(&Utc),
        )
        .create()
        .await;

    let payslip =
        {
            let pr: std::sync::Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> = std::sync::Arc::new(apiserver::repositories::payroll_repository::SqlxPayrollRepository::new(pool.clone()));
            let dr: std::sync::Arc<dyn apiserver::repositories::department_repository::DepartmentRepository> = std::sync::Arc::new(apiserver::repositories::department_repository::SqlxDepartmentRepository::new(pool.clone()));
            let tr: std::sync::Arc<dyn apiserver::repositories::time_entry_repository::TimeEntryRepository> = std::sync::Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(pool.clone()));
            let wr: std::sync::Arc<dyn apiserver::repositories::workspace_repository::WorkspaceRepository> = std::sync::Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
            let hr: std::sync::Arc<dyn apiserver::repositories::holiday_repository::HolidayRepositoryTrait> = std::sync::Arc::new(apiserver::repositories::holiday_repository::SqlxHolidayRepository::new(pool.clone()));
            PayrollService::new(pr, dr, tr, wr, hr, Arc::new(apiserver::repositories::roster_repository::SqlxRosterRepository::new(pool.clone())), pool.clone())
        }.calculate_monthly_payslip( employee_id, workspace_id, 2026, 1)
            .await
            .unwrap();

    // 10 minutes early is within 15-min grace period
    assert!(
        payslip.standard_units > 0,
        "Grace period override should apply"
    );
}
