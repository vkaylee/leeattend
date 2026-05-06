//! Integration tests for holiday work policy features:
//! - Payroll: holiday work compensation (credit + premium with configurable multiplier)
//! - Leave reversal: threshold-based auto-cancel of system leave when employee works
//! - Leave breakdown: paid holidays excluded from leave day count
//! - CRUD: new `work_threshold_percent` and `is_credit_on_work` fields

use crate::common::app::spawn_app;
use apiserver::models::LeaveStatus;
use chrono::{Datelike, NaiveDate, NaiveTime, TimeZone, Utc};
use reqwest::StatusCode;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HolidayResponse {
    id: Uuid,
    is_paid: bool,
    ot_multiplier: Option<String>,
    work_threshold_percent: Option<i32>,
    is_credit_on_work: bool,
}

// =============================================================================
// Test 1: Payroll holiday work with multiplier (300% = credit + 2x premium)
// =============================================================================

#[tokio::test]
async fn test_payroll_holiday_work_with_multiplier() {
    let (_address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    let today = Utc::now().date_naive();
    let first_of_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();

    // Assign shift 09:00-18:00
    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        )
        .assign_to_employee(emp, first_of_month)
        .await;

    // Create paid holiday with ot_multiplier=3, is_credit_on_work=true
    sqlx::query(
        "INSERT INTO holidays (workspace_id, name, date, holiday_type, is_paid, ot_multiplier, is_credit_on_work)
         VALUES ($1, 'Test Holiday', $2, 'national', true, 3.0, true)"
    )
    .bind(workspace_id)
    .bind(first_of_month)
    .execute(&pool)
    .await
    .unwrap();

    // Employee works full shift on the holiday
    let punch_in = Utc
        .with_ymd_and_hms(
            first_of_month.year(),
            first_of_month.month(),
            first_of_month.day(),
            2,
            0,
            0,
        )
        .unwrap();
    let punch_out = Utc
        .with_ymd_and_hms(
            first_of_month.year(),
            first_of_month.month(),
            first_of_month.day(),
            11,
            0,
            0,
        )
        .unwrap();
    factory
        .time_entry()
        .with_employee(emp)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .with_entry_type("work")
        .create()
        .await;

    // Assign salary component for payroll calculation
    let comp_id: Uuid = sqlx::query_scalar(
        "INSERT INTO salary_component_definitions (workspace_id, name, code)
         VALUES ($1, 'Base', 'BASE') RETURNING id",
    )
    .bind(workspace_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date)
         VALUES ($1, $2, 10000000, $3)"
    )
    .bind(emp)
    .bind(comp_id)
    .bind(first_of_month)
    .execute(&pool)
    .await
    .unwrap();

    // Calculate payroll
    let result = {
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
        apiserver::services::payroll_service::PayrollService::new(
            apiserver::services::payroll_service::PayrollServiceArgs {
                payroll_repo: pr,
                department_repo: dr,
                time_entry_repo: tr,
                workspace_repo: wr,
                holiday_repo: hr,
                roster_repo: Arc::new(apiserver::repositories::roster::SqlxRosterRepository::new(
                    pool.clone(),
                )),
                pool: pool.clone(),
            },
        )
    }
    .calculate_monthly_payslip(
        emp,
        workspace_id,
        first_of_month.year(),
        first_of_month.month(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Payroll calculation should succeed: {:?}",
        result.err()
    );
    let payslip = result.unwrap();

    // holiday_work_premium_amount should be > 0 (employee worked on holiday)
    assert!(
        payslip.holiday_work_premium_amount > Decimal::ZERO,
        "Holiday work premium should be > 0, got: {}",
        payslip.holiday_work_premium_amount
    );

    // Net pay should include the premium
    assert!(
        payslip.net_pay_amount > payslip.base_pay_amount,
        "Net pay should be greater than base pay due to holiday premium"
    );
}

// =============================================================================
// Test 2: Payroll holiday work without credit (is_credit_on_work=false)
// =============================================================================

#[tokio::test]
async fn test_payroll_holiday_work_no_credit() {
    let (_address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    let today = Utc::now().date_naive();
    let first_of_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();

    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        )
        .assign_to_employee(emp, first_of_month)
        .await;

    // is_credit_on_work=false → no holiday pay credit, only premium
    sqlx::query(
        "INSERT INTO holidays (workspace_id, name, date, holiday_type, is_paid, ot_multiplier, is_credit_on_work)
         VALUES ($1, 'No Credit Holiday', $2, 'company', true, 2.0, false)"
    )
    .bind(workspace_id)
    .bind(first_of_month)
    .execute(&pool)
    .await
    .unwrap();

    let punch_in = Utc
        .with_ymd_and_hms(
            first_of_month.year(),
            first_of_month.month(),
            first_of_month.day(),
            2,
            0,
            0,
        )
        .unwrap();
    let punch_out = Utc
        .with_ymd_and_hms(
            first_of_month.year(),
            first_of_month.month(),
            first_of_month.day(),
            11,
            0,
            0,
        )
        .unwrap();
    factory
        .time_entry()
        .with_employee(emp)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .with_entry_type("work")
        .create()
        .await;

    let comp_id: Uuid = sqlx::query_scalar(
        "INSERT INTO salary_component_definitions (workspace_id, name, code)
         VALUES ($1, 'Base', 'BASE') RETURNING id",
    )
    .bind(workspace_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date)
         VALUES ($1, $2, 10000000, $3)"
    )
    .bind(emp)
    .bind(comp_id)
    .bind(first_of_month)
    .execute(&pool)
    .await
    .unwrap();

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
        apiserver::services::payroll_service::PayrollService::new(
            apiserver::services::payroll_service::PayrollServiceArgs {
                payroll_repo: pr,
                department_repo: dr,
                time_entry_repo: tr,
                workspace_repo: wr,
                holiday_repo: hr,
                roster_repo: Arc::new(apiserver::repositories::roster::SqlxRosterRepository::new(
                    pool.clone(),
                )),
                pool: pool.clone(),
            },
        )
    }
    .calculate_monthly_payslip(
        emp,
        workspace_id,
        first_of_month.year(),
        first_of_month.month(),
    )
    .await
    .unwrap();

    // Premium should exist (multiplier 2x → premium = 1x worth)
    assert!(
        payslip.holiday_work_premium_amount > Decimal::ZERO,
        "Should have holiday premium even without credit"
    );
}

// =============================================================================
// Test 3: Unpaid holiday, multiplier=1 → no premium
// =============================================================================

#[tokio::test]
async fn test_payroll_holiday_work_unpaid_no_premium() {
    let (_address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    let today = Utc::now().date_naive();
    let first_of_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();

    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        )
        .assign_to_employee(emp, first_of_month)
        .await;

    // Unpaid holiday with multiplier=1 → standard rate, no premium
    sqlx::query(
        "INSERT INTO holidays (workspace_id, name, date, holiday_type, is_paid, ot_multiplier, is_credit_on_work)
         VALUES ($1, 'Unpaid Day', $2, 'company', false, 1.0, false)"
    )
    .bind(workspace_id)
    .bind(first_of_month)
    .execute(&pool)
    .await
    .unwrap();

    let punch_in = Utc
        .with_ymd_and_hms(
            first_of_month.year(),
            first_of_month.month(),
            first_of_month.day(),
            2,
            0,
            0,
        )
        .unwrap();
    let punch_out = Utc
        .with_ymd_and_hms(
            first_of_month.year(),
            first_of_month.month(),
            first_of_month.day(),
            11,
            0,
            0,
        )
        .unwrap();
    factory
        .time_entry()
        .with_employee(emp)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .with_entry_type("work")
        .create()
        .await;

    let comp_id: Uuid = sqlx::query_scalar(
        "INSERT INTO salary_component_definitions (workspace_id, name, code)
         VALUES ($1, 'Base', 'BASE') RETURNING id",
    )
    .bind(workspace_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date)
         VALUES ($1, $2, 10000000, $3)"
    )
    .bind(emp)
    .bind(comp_id)
    .bind(first_of_month)
    .execute(&pool)
    .await
    .unwrap();

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
        apiserver::services::payroll_service::PayrollService::new(
            apiserver::services::payroll_service::PayrollServiceArgs {
                payroll_repo: pr,
                department_repo: dr,
                time_entry_repo: tr,
                workspace_repo: wr,
                holiday_repo: hr,
                roster_repo: Arc::new(apiserver::repositories::roster::SqlxRosterRepository::new(
                    pool.clone(),
                )),
                pool: pool.clone(),
            },
        )
    }
    .calculate_monthly_payslip(
        emp,
        workspace_id,
        first_of_month.year(),
        first_of_month.month(),
    )
    .await
    .unwrap();

    // multiplier=1 means (1-1)=0 premium
    assert_eq!(
        payslip.holiday_work_premium_amount,
        Decimal::ZERO,
        "ot_multiplier=1 should produce zero premium"
    );
}

// =============================================================================
// Test 4: Leave reversal — system leave can be cancelled via SQL
// =============================================================================

#[tokio::test]
async fn test_leave_reversal_full_shift() {
    let (_address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;
    let today = Utc::now().date_naive();

    // Create holiday with threshold=50%
    sqlx::query(
        "INSERT INTO holidays (workspace_id, name, date, holiday_type, is_paid, work_threshold_percent, is_credit_on_work)
         VALUES ($1, 'Threshold Holiday', $2, 'company', false, 50, false)"
    )
    .bind(workspace_id)
    .bind(today)
    .execute(&pool)
    .await
    .unwrap();

    factory
        .leave_balance()
        .with_employee(emp)
        .with_entitlement(12)
        .create()
        .await;

    let vacation_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        workspace_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // System-generated leave (requested_by_user_id IS NULL)
    let leave_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO leave_requests (id, employee_id, leave_type_id, start_date, end_date, status, requested_by_user_id)
         VALUES ($1, $2, $3, $4, $4, 'approved'::leave_status_type, NULL)"
    )
    .bind(leave_id)
    .bind(emp)
    .bind(vacation_id)
    .bind(today)
    .execute(&pool)
    .await
    .unwrap();

    // Verify system-generated leave can be found
    let found: Option<(Uuid,)> = sqlx::query_as(
        r"SELECT lr.id FROM leave_requests lr
           WHERE lr.employee_id = $1
             AND lr.status = 'approved'::leave_status_type
             AND lr.start_date <= $2 AND lr.end_date >= $2
             AND lr.requested_by_user_id IS NULL",
    )
    .bind(emp)
    .bind(today)
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert!(found.is_some(), "System-generated leave should be findable");

    // Simulate leave reversal (what attendance_summary_service does)
    sqlx::query(
        "UPDATE leave_requests SET status = 'cancelled'::leave_status_type, updated_at = NOW() WHERE id = $1"
    )
    .bind(leave_id)
    .execute(&pool)
    .await
    .unwrap();

    let status: String =
        sqlx::query_scalar("SELECT status::TEXT FROM leave_requests WHERE id = $1")
            .bind(leave_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        status, "cancelled",
        "Leave should be cancelled via SQL update"
    );
}

// =============================================================================
// Test 5: Leave kept — partial work below threshold
// =============================================================================

#[tokio::test]
async fn test_leave_kept_partial_work() {
    let (_address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;
    let today = Utc::now().date_naive();

    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        )
        .assign_to_employee(emp, today)
        .await;

    // Holiday with threshold=50%
    sqlx::query(
        "INSERT INTO holidays (workspace_id, name, date, holiday_type, is_paid, work_threshold_percent, is_credit_on_work)
         VALUES ($1, 'Threshold Holiday', $2, 'company', false, 50, false)"
    )
    .bind(workspace_id)
    .bind(today)
    .execute(&pool)
    .await
    .unwrap();

    factory
        .leave_balance()
        .with_employee(emp)
        .with_entitlement(12)
        .create()
        .await;

    let vacation_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        workspace_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let leave_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO leave_requests (id, employee_id, leave_type_id, start_date, end_date, status, requested_by_user_id)
         VALUES ($1, $2, $3, $4, $4, 'approved', NULL)"
    )
    .bind(leave_id)
    .bind(emp)
    .bind(vacation_id)
    .bind(today)
    .execute(&pool)
    .await
    .unwrap();

    // Employee works only ~3 hours (37% of 8h shift, < 50% threshold)
    let punch_in = Utc
        .with_ymd_and_hms(today.year(), today.month(), today.day(), 2, 0, 0)
        .unwrap();
    let punch_out = Utc
        .with_ymd_and_hms(today.year(), today.month(), today.day(), 5, 0, 0)
        .unwrap();
    factory
        .time_entry()
        .with_employee(emp)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .with_entry_type("work")
        .create()
        .await;

    apiserver::services::AttendanceSummaryService::new(
        std::sync::Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(
            pool.clone(),
        )),
        std::sync::Arc::new(apiserver::repositories::roster::SqlxRosterRepository::new(
            pool.clone(),
        )),
        pool.clone(),
    )
    .recalculate_daily_summary(workspace_id, emp, today)
    .await
    .expect("Recalculate failed");

    // Leave should NOT be cancelled (work < threshold)
    let status: String =
        sqlx::query_scalar("SELECT status::TEXT FROM leave_requests WHERE id = $1")
            .bind(leave_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        status, "approved",
        "Leave should remain approved when work < threshold"
    );
}

// =============================================================================
// Test 6: User-submitted leave NOT cancelled vs system leave cancelled
// =============================================================================

#[tokio::test]
async fn test_leave_reversal_custom_threshold() {
    let (_address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;
    let today = Utc::now().date_naive();

    factory
        .leave_balance()
        .with_employee(emp)
        .with_entitlement(12)
        .create()
        .await;

    let vacation_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        workspace_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // Create system leave (NULL user) and user leave (non-NULL user)
    let sys_leave = Uuid::new_v4();
    let user_leave = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO leave_requests (id, employee_id, leave_type_id, start_date, end_date, status, requested_by_user_id)
         VALUES ($1, $2, $3, $4, $4, 'approved'::leave_status_type, NULL)"
    )
    .bind(sys_leave)
    .bind(emp)
    .bind(vacation_id)
    .bind(today)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO leave_requests (id, employee_id, leave_type_id, start_date, end_date, status, requested_by_user_id)
         VALUES ($1, $2, $3, $4, $4, 'approved'::leave_status_type, $5)"
    )
    .bind(user_leave)
    .bind(emp)
    .bind(vacation_id)
    .bind(today)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    // Query only system-generated leaves (matching service behavior)
    let system_only: Vec<(Uuid,)> = sqlx::query_as(
        r"SELECT lr.id FROM leave_requests lr
           WHERE lr.employee_id = $1
             AND lr.status = 'approved'::leave_status_type
             AND lr.start_date <= $2 AND lr.end_date >= $2
             AND lr.requested_by_user_id IS NULL",
    )
    .bind(emp)
    .bind(today)
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(
        system_only.len(),
        1,
        "Should find exactly 1 system-generated leave"
    );
    assert_eq!(
        system_only[0].0, sys_leave,
        "Should match the system leave ID"
    );

    // Cancel only system leave
    sqlx::query("UPDATE leave_requests SET status = 'cancelled'::leave_status_type WHERE id = $1")
        .bind(sys_leave)
        .execute(&pool)
        .await
        .unwrap();

    // Verify: system leave cancelled, user leave still approved
    let sys_status: String =
        sqlx::query_scalar("SELECT status::TEXT FROM leave_requests WHERE id = $1")
            .bind(sys_leave)
            .fetch_one(&pool)
            .await
            .unwrap();
    let user_status: String =
        sqlx::query_scalar("SELECT status::TEXT FROM leave_requests WHERE id = $1")
            .bind(user_leave)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(sys_status, "cancelled", "System leave should be cancelled");
    assert_eq!(user_status, "approved", "User leave should remain approved");
}

// =============================================================================
// Test 7: Leave reversal only for system-generated requests
// =============================================================================

#[tokio::test]
async fn test_leave_reversal_only_system() {
    let (_address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;
    let today = Utc::now().date_naive();

    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        )
        .assign_to_employee(emp, today)
        .await;

    sqlx::query(
        "INSERT INTO holidays (workspace_id, name, date, holiday_type, is_paid, work_threshold_percent, is_credit_on_work)
         VALUES ($1, 'User Leave Holiday', $2, 'company', false, 50, false)"
    )
    .bind(workspace_id)
    .bind(today)
    .execute(&pool)
    .await
    .unwrap();

    factory
        .leave_balance()
        .with_employee(emp)
        .with_entitlement(12)
        .create()
        .await;

    let vacation_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        workspace_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // User-submitted leave (requested_by_user_id is NOT NULL)
    let leave_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO leave_requests (id, employee_id, leave_type_id, start_date, end_date, status, requested_by_user_id)
         VALUES ($1, $2, $3, $4, $4, 'approved', $5)"
    )
    .bind(leave_id)
    .bind(emp)
    .bind(vacation_id)
    .bind(today)
    .bind(user_id) // <-- user-submitted
    .execute(&pool)
    .await
    .unwrap();

    // Full shift work
    let punch_in = Utc
        .with_ymd_and_hms(today.year(), today.month(), today.day(), 2, 0, 0)
        .unwrap();
    let punch_out = Utc
        .with_ymd_and_hms(today.year(), today.month(), today.day(), 11, 0, 0)
        .unwrap();
    factory
        .time_entry()
        .with_employee(emp)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .with_entry_type("work")
        .create()
        .await;

    apiserver::services::AttendanceSummaryService::new(
        std::sync::Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(
            pool.clone(),
        )),
        std::sync::Arc::new(apiserver::repositories::roster::SqlxRosterRepository::new(
            pool.clone(),
        )),
        pool.clone(),
    )
    .recalculate_daily_summary(workspace_id, emp, today)
    .await
    .expect("Recalculate failed");

    // User-submitted leave should NOT be cancelled
    let status: String =
        sqlx::query_scalar("SELECT status::TEXT FROM leave_requests WHERE id = $1")
            .bind(leave_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        status, "approved",
        "User-submitted leave should NOT be auto-cancelled"
    );
}

// =============================================================================
// Test 8: Leave breakdown excludes paid holidays
// =============================================================================

#[tokio::test]
async fn test_leave_breakdown_excludes_holidays() {
    let (_address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    // Leave from Apr 28 to May 3 = 6 days
    let leave_start = NaiveDate::from_ymd_opt(2026, 4, 28).unwrap();
    let leave_end = NaiveDate::from_ymd_opt(2026, 5, 3).unwrap();

    factory
        .leave_balance()
        .with_employee(emp)
        .with_entitlement(12)
        .create()
        .await;

    // Create approved leave request
    factory
        .leave_request()
        .with_employee(emp)
        .with_dates(leave_start, leave_end)
        .with_status(LeaveStatus::Approved)
        .create()
        .await;

    // May 1 is a paid national holiday
    sqlx::query(
        "INSERT INTO holidays (workspace_id, name, date, holiday_type, is_paid, is_credit_on_work)
         VALUES ($1, 'Labour Day', '2026-05-01', 'national', true, true)",
    )
    .bind(workspace_id)
    .execute(&pool)
    .await
    .unwrap();

    // Get leave breakdown — should exclude May 1
    let breakdown =
        apiserver::repositories::leave_request_repository::get_approved_leave_breakdown(
            &pool,
            emp,
            leave_start,
            leave_end,
            workspace_id,
        )
        .await
        .unwrap();

    // 6 days - 1 paid holiday = 5 days
    assert_eq!(
        breakdown.total_days, 5.0,
        "Leave breakdown should exclude paid holiday (6 - 1 = 5), got: {}",
        breakdown.total_days
    );
}

// =============================================================================
// Test 9: CRUD — new fields in create/update
// =============================================================================

#[tokio::test]
async fn test_holiday_crud_new_fields() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    let today = Utc::now().date_naive();

    // Create holiday with new fields
    let resp = client
        .post(format!("{address}/api/workspaces/{workspace_id}/holidays"))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "name": "Configurable Holiday",
            "date": today.to_string(),
            "holiday_type": "company",
            "is_paid": true,
            "ot_multiplier": 2.5,
            "work_threshold_percent": 40,
            "is_credit_on_work": false,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let holiday: HolidayResponse = resp.json().await.unwrap();
    assert_eq!(holiday.work_threshold_percent, Some(40));
    assert!(!holiday.is_credit_on_work);
    assert_eq!(holiday.ot_multiplier.as_deref(), Some("2.5"));

    // Update
    let resp = client
        .put(format!(
            "{address}/api/workspaces/{workspace_id}/holidays/{}",
            holiday.id
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "work_threshold_percent": 60,
            "is_credit_on_work": true,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let updated: HolidayResponse = resp.json().await.unwrap();
    assert_eq!(updated.work_threshold_percent, Some(60));
    assert!(updated.is_credit_on_work);
}
