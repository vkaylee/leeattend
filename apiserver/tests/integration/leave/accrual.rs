//! Leave accrual integration tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::services::leave_accrual_service::LeaveAccrualService;
use chrono::{NaiveDate, TimeZone, Utc};

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
    let logs = LeaveAccrualService::process_workspace_accruals(
        &pool,
        ws_id,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        false,
    )
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

    factory.leave_policy().assign_to(
        ws_id,
        policy,
        "employee",
        Some(emp),
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        None
    ).await;

    // 10 work days (below threshold)
    for i in 1..=10 {
        let dt = Utc.with_ymd_and_hms(2026, 1, i, 8, 0, 0).unwrap();
        sqlx::query!(
            "INSERT INTO time_entries (employee_id, workspace_id, punch_in_time, entry_type) VALUES ($1, $2, $3, 'work')",
            emp, ws_id, dt
        ).execute(&pool).await.unwrap();
    }

    // Run accrual for Feb 1 (checking Jan)
    let logs = LeaveAccrualService::process_workspace_accruals(
        &pool,
        ws_id,
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        false
    ).await.unwrap();

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
    let logs = LeaveAccrualService::process_workspace_accruals(
        &pool,
        ws_id,
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        false
    ).await.unwrap();

    assert_eq!(logs.len(), 1);
}
