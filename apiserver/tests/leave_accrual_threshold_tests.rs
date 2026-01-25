mod common;

use apiserver::services::leave_accrual_service::LeaveAccrualService;
use chrono::{NaiveDate, Utc, TimeZone};
use common::db::TestDatabase;
use common::factory::TestFactory;

#[tokio::test]
async fn test_leave_accrual_min_work_days_threshold() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    // 1. Setup
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Create a policy with 14 days threshold
    let policy_id = factory.leave_policy()
        .with_workspace(ws_id)
        .with_name("Threshold Policy")
        .with_config(serde_json::json!({
            "frequency": "monthly",
            "accrual_amount": 1.0,
            "accrual_timing": "start_of_period",
            "min_work_days_threshold": 14.0
        }))
        .create()
        .await;

    // Assign policy starting Jan 1st, 2026
    factory.leave_policy().assign_to(
        ws_id,
        policy_id,
        "employee",
        Some(emp_id),
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        None
    ).await;

    // --- Scenario 1: Only 10 work days in Jan (Below 14) ---
    for day in 1..=10 {
        let date = Utc.with_ymd_and_hms(2026, 1, day, 9, 0, 0).unwrap();
        factory.time_entry()
            .with_employee(emp_id)
            .with_punch_in(date)
            .with_punch_out(date + chrono::Duration::hours(8))
            .with_entry_type("work")
            .create()
            .await;
    }

    // Run accrual for Feb 1st. It should check January.
    let run_date = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
    let logs = LeaveAccrualService::process_workspace_accruals(&pool, ws_id, run_date, false).await.unwrap();

    // Expect 0 logs because threshold not met
    assert!(logs.is_empty(), "Should not accrue leave when work days (10) < threshold (14)");

    // --- Scenario 2: Add 5 more work days (Total 15, Above 14) ---
    for day in 11..=15 {
        let date = Utc.with_ymd_and_hms(2026, 1, day, 9, 0, 0).unwrap();
        factory.time_entry()
            .with_employee(emp_id)
            .with_punch_in(date)
            .with_punch_out(date + chrono::Duration::hours(8))
            .with_entry_type("work")
            .create()
            .await;
    }

    // Run accrual for Feb 1st again.
    let logs_v2 = LeaveAccrualService::process_workspace_accruals(&pool, ws_id, run_date, false).await.unwrap();

    // Expect 1 log now
    assert_eq!(logs_v2.len(), 1, "Should accrue leave when work days (15) >= threshold (14)");
    assert_eq!(logs_v2[0].amount.to_string(), "1.00");
    assert_eq!(logs_v2[0].accrual_date, run_date);
}
