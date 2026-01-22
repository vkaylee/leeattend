use apiserver::services::leave_accrual_service::LeaveAccrualService;
use chrono::{NaiveDate, TimeZone, Utc};

mod common;
use common::db::TestDatabase;

#[tokio::test]
async fn test_performance_based_accrual() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = common::factory::TestFactory::new(pool.clone());

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

    // Policy: Monthly 1.0, with 14 days threshold
    let policy = factory
        .leave_policy()
        .with_workspace(ws_id)
        .with_name("Monthly with 14d threshold")
        .with_config(serde_json::json!({
            "frequency": "monthly",
            "accrual_amount": 1.0,
            "accrual_timing": "start_of_period",
            "min_work_days_threshold": 14.0
        }))
        .create()
        .await;

    // Assign policy
    factory.leave_policy().assign_to(
        ws_id,
        policy,
        "employee",
        Some(emp),
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(), // Effective from Feb 1
        None
    ).await;

    // Scenario 1: Jan 2026 - Employee works 10 days (Below 14)
    // We run accrual for Feb 1, 2026 (start_of_period) which checks Jan performance.
    for i in 1..=10 {
        let dt = Utc.with_ymd_and_hms(2026, 1, i, 8, 0, 0).unwrap();
        
        sqlx::query!(
            "INSERT INTO time_entries (employee_id, punch_in_time, entry_type) VALUES ($1, $2, 'work')",
            emp, dt
        ).execute(&pool).await.unwrap();
    }

    // Run accrual for 2026-02-01
    let logs = LeaveAccrualService::process_workspace_accruals(
        &pool,
        ws_id,
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        false
    ).await.unwrap();

    // Verify: No log for Feb 1 because Jan only has 10 days.
    assert!(logs.is_empty(), "Accrual should be skipped for 10 working days");

    // Scenario 2: Add 5 more days (Total 15, Above 14)
    for i in 11..=15 {
        let dt = Utc.with_ymd_and_hms(2026, 1, i, 8, 0, 0).unwrap();
        sqlx::query!(
            "INSERT INTO time_entries (employee_id, punch_in_time, entry_type) VALUES ($1, $2, 'work')",
            emp, dt
        ).execute(&pool).await.unwrap();
    }

    // Run accrual for 2026-02-01 again
    let logs = LeaveAccrualService::process_workspace_accruals(
        &pool,
        ws_id,
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        false
    ).await.unwrap();

    assert_eq!(logs.len(), 1, "Accrual should happen for 15 working days");
    assert_eq!(logs[0].accrual_date, NaiveDate::from_ymd_opt(2026, 2, 1).unwrap());
    assert_eq!(logs[0].amount.to_string(), "1.00");

    // Scenario 3: Approved Leave also counts
    // Let's clear and test with 10 work days + 5 leave days
    sqlx::query!("DELETE FROM leave_accrual_logs").execute(&pool).await.unwrap();
    sqlx::query!("DELETE FROM time_entries").execute(&pool).await.unwrap();
    sqlx::query!("DELETE FROM leave_balances").execute(&pool).await.unwrap();

    // 10 work days
    for i in 1..=10 {
        let dt = Utc.with_ymd_and_hms(2026, 1, i, 8, 0, 0).unwrap();
        sqlx::query!(
            "INSERT INTO time_entries (employee_id, punch_in_time, entry_type) VALUES ($1, $2, 'work')",
            emp, dt
        ).execute(&pool).await.unwrap();
    }

    // 5 leave days
    sqlx::query!(
        "INSERT INTO leave_requests (employee_id, leave_type, start_date, end_date, status) 
         VALUES ($1, 'vacation', '2026-01-20', '2026-01-24', 'approved')",
        emp
    ).execute(&pool).await.unwrap();

    // Run accrual for 2026-02-01
    let logs = LeaveAccrualService::process_workspace_accruals(
        &pool,
        ws_id,
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        false
    ).await.unwrap();

    assert_eq!(logs.len(), 1, "Accrual should happen for 10 work days + 5 leave days");
}

#[tokio::test]
async fn test_performance_based_accrual_end_of_period() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = common::factory::TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp = factory.employee().with_workspace(ws_id).create().await;

    // Policy: Monthly 1.0, with 14 days threshold, END OF PERIOD
    let policy = factory
        .leave_policy()
        .with_workspace(ws_id)
        .with_name("Monthly 14d EndOfPeriod")
        .with_config(serde_json::json!({
            "frequency": "monthly",
            "accrual_amount": 1.0,
            "accrual_timing": "end_of_period",
            "min_work_days_threshold": 14.0
        }))
        .create()
        .await;

    factory.leave_policy().assign_to(
        ws_id,
        policy,
        "employee",
        Some(emp),
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        None
    ).await;

    // 10 work days in Jan
    for i in 1..=10 {
        let dt = Utc.with_ymd_and_hms(2026, 1, i, 8, 0, 0).unwrap();
        sqlx::query!(
            "INSERT INTO time_entries (employee_id, punch_in_time, entry_type) VALUES ($1, $2, 'work')",
            emp, dt
        ).execute(&pool).await.unwrap();
    }

    // Run accrual for Jan 31, 2026
    let logs = LeaveAccrualService::process_workspace_accruals(
        &pool,
        ws_id,
        NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        false
    ).await.unwrap();

    assert!(logs.is_empty(), "Should skip Jan 31 accrual for 10 days");

    // Add 5 more days
    for i in 11..=15 {
        let dt = Utc.with_ymd_and_hms(2026, 1, i, 8, 0, 0).unwrap();
        sqlx::query!(
            "INSERT INTO time_entries (employee_id, punch_in_time, entry_type) VALUES ($1, $2, 'work')",
            emp, dt
        ).execute(&pool).await.unwrap();
    }

    // Run accrual for Jan 31, 2026 again
    let logs = LeaveAccrualService::process_workspace_accruals(
        &pool,
        ws_id,
        NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        false
    ).await.unwrap();

    assert_eq!(logs.len(), 1, "Should accrue Jan 31 for 15 days");
}
