//! Leave accrual integration tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::services::leave_accrual_service::LeaveAccrualService;
use chrono::{NaiveDate, TimeZone, Utc};
use std::sync::Arc;

#[tokio::test]
async fn test_accrual_scenarios() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    // 1. Setup
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Hire date: Jan 1, 2025
    let emp = factory
        .employee()
        .with_workspace(ws_id)
        .with_hire_date(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
        .create()
        .await;

    // Init balance for 2026
    factory
        .leave_balance()
        .with_employee(emp)
        .with_entitlement(12)
        .create()
        .await;

    // Policy: Monthly 1.0
    let policy_monthly = factory
        .leave_policy()
        .with_workspace(ws_id)
        .with_name("Monthly 1.0")
        .with_config(serde_json::json!({
            "frequency": "monthly",
            "accrual_amount": 1.0,
            "accrual_timing": "start_of_period"
        }))
        .create()
        .await;

    factory
        .leave_policy()
        .assign_to(
            ws_id,
            policy_monthly,
            "employee",
            Some(emp),
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            None,
        )
        .await;

    // Run Accrual for Jan 1, 2026
    let employee_repo: Arc<dyn apiserver::repositories::EmployeeRepository> = Arc::new(
        apiserver::repositories::SqlxEmployeeRepository::new(pool.clone()),
    );
    let department_repo: Arc<
        dyn apiserver::repositories::department_repository::DepartmentRepository,
    > = Arc::new(
        apiserver::repositories::department_repository::SqlxDepartmentRepository::new(pool.clone()),
    );
    let leave_policy_repo: Arc<
        dyn apiserver::repositories::leave_policy_repository::LeavePolicyRepository,
    > = Arc::new(
        apiserver::repositories::leave_policy_repository::SqlxLeavePolicyRepository::new(
            pool.clone(),
        ),
    );
    let accrual_service = LeaveAccrualService::new(
        employee_repo,
        department_repo,
        leave_policy_repo,
        Arc::new(apiserver::repositories::leave_accrual_log_repository::SqlxLeaveAccrualLogRepository::new(pool.clone())),
        pool.clone(),
    );
    let logs = accrual_service
        .process_workspace_accruals(ws_id, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), false)
        .await
        .unwrap();

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].amount.to_string(), "1.00");
}

#[tokio::test]
async fn test_performance_based_accrual() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp = factory.employee().with_workspace(ws_id).create().await;

    // Policy: Monthly 1.0 with 14 days threshold
    let policy = factory
        .leave_policy()
        .with_workspace(ws_id)
        .with_name("Performance Policy")
        .with_config(serde_json::json!({
            "frequency": "monthly",
            "accrual_amount": 1.0,
            "accrual_timing": "start_of_period",
            "min_work_days_threshold": 14.0
        }))
        .create()
        .await;

    factory
        .leave_policy()
        .assign_to(
            ws_id,
            policy,
            "employee",
            Some(emp),
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            None,
        )
        .await;

    // 10 work days (below threshold)
    for i in 1..=10 {
        let dt = Utc.with_ymd_and_hms(2026, 1, i, 8, 0, 0).unwrap();
        sqlx::query!(
            "INSERT INTO time_entries (employee_id, workspace_id, punch_in_time, entry_type) VALUES ($1, $2, $3, 'work')",
            emp, ws_id, dt
        ).execute(&pool).await.unwrap();
    }

    // Run accrual for Feb 1 (checking Jan)
    let employee_repo: Arc<dyn apiserver::repositories::EmployeeRepository> = Arc::new(
        apiserver::repositories::SqlxEmployeeRepository::new(pool.clone()),
    );
    let department_repo: Arc<
        dyn apiserver::repositories::department_repository::DepartmentRepository,
    > = Arc::new(
        apiserver::repositories::department_repository::SqlxDepartmentRepository::new(pool.clone()),
    );
    let leave_policy_repo: Arc<
        dyn apiserver::repositories::leave_policy_repository::LeavePolicyRepository,
    > = Arc::new(
        apiserver::repositories::leave_policy_repository::SqlxLeavePolicyRepository::new(
            pool.clone(),
        ),
    );
    let accrual_service = LeaveAccrualService::new(
        employee_repo,
        department_repo,
        leave_policy_repo,
        Arc::new(apiserver::repositories::leave_accrual_log_repository::SqlxLeaveAccrualLogRepository::new(pool.clone())),
        pool.clone(),
    );
    let logs = accrual_service
        .process_workspace_accruals(ws_id, NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(), false)
        .await
        .unwrap();

    assert!(logs.is_empty());

    // Add 5 days (total 15 > 14)
    for i in 11..=15 {
        let dt = Utc.with_ymd_and_hms(2026, 1, i, 8, 0, 0).unwrap();
        sqlx::query!(
            "INSERT INTO time_entries (employee_id, workspace_id, punch_in_time, entry_type) VALUES ($1, $2, $3, 'work')",
            emp, ws_id, dt
        ).execute(&pool).await.unwrap();
    }

    // Retry
    let logs = accrual_service
        .process_workspace_accruals(ws_id, NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(), false)
        .await
        .unwrap();

    assert_eq!(logs.len(), 1);
}

#[tokio::test]
async fn test_accrual_recalculation() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp = factory.employee().with_workspace(ws_id).create().await;

    // Policy: Monthly 1.0
    let policy = factory
        .leave_policy()
        .with_workspace(ws_id)
        .with_name("Monthly Base")
        .with_config(serde_json::json!({
            "frequency": "monthly",
            "accrual_amount": 1.0,
            "accrual_timing": "start_of_period"
        }))
        .create()
        .await;

    factory
        .leave_policy()
        .assign_to(
            ws_id,
            policy,
            "employee",
            Some(emp),
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            None,
        )
        .await;

    let employee_repo: Arc<dyn apiserver::repositories::EmployeeRepository> = Arc::new(
        apiserver::repositories::SqlxEmployeeRepository::new(pool.clone()),
    );
    let department_repo: Arc<
        dyn apiserver::repositories::department_repository::DepartmentRepository,
    > = Arc::new(
        apiserver::repositories::department_repository::SqlxDepartmentRepository::new(pool.clone()),
    );
    let leave_policy_repo: Arc<
        dyn apiserver::repositories::leave_policy_repository::LeavePolicyRepository,
    > = Arc::new(
        apiserver::repositories::leave_policy_repository::SqlxLeavePolicyRepository::new(
            pool.clone(),
        ),
    );
    let accrual_service = LeaveAccrualService::new(
        employee_repo,
        department_repo,
        leave_policy_repo,
        Arc::new(apiserver::repositories::leave_accrual_log_repository::SqlxLeaveAccrualLogRepository::new(pool.clone())),
        pool.clone(),
    );

    // 1st run: Accrues 1.0
    let logs = accrual_service
        .process_workspace_accruals(ws_id, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), false)
        .await
        .unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].amount.to_string(), "1.00");

    // Change policy to 2.0
    sqlx::query!(
        "UPDATE leave_policy_definitions SET rule_config = $1 WHERE id = $2",
        serde_json::json!({
            "frequency": "monthly",
            "accrual_amount": 2.0,
            "accrual_timing": "start_of_period"
        }),
        policy
    )
    .execute(&pool)
    .await
    .unwrap();

    // 2nd run (force = true)
    let logs_2 = accrual_service
        .process_workspace_accruals(ws_id, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), true)
        .await
        .unwrap();

    assert_eq!(logs_2.len(), 1);
    assert_eq!(logs_2[0].amount.to_string(), "2.00");
    assert!(logs_2[0]
        .notes
        .as_deref()
        .unwrap_or("")
        .contains("Re-calculated"));
}
