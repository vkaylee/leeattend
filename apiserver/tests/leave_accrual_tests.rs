use apiserver::services::leave_accrual_service::LeaveAccrualService;
use chrono::NaiveDate;

mod common;
use common::db::TestDatabase;

#[tokio::test]
async fn test_accrual_scenarios() {
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

    // Init balance for 2026
    factory
        .leave_balance()
        .with_employee(emp)
        .with_entitlement(12)
        .create()
        .await;

    // 2. Create Policies

    // Policy A: Standard 1.0 Monthly
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

    // Policy B: Seniority Bonus (+1 after 1 year)
    let policy_seniority = factory
        .leave_policy()
        .with_workspace(ws_id)
        .with_name("Seniority +1")
        .with_config(serde_json::json!({
            "frequency": "monthly",
            "accrual_amount": 1.0,
            "accrual_timing": "start_of_period",
            "seniority_bonus": [
                { "months": 12, "bonus_amount": 1.0 }
            ]
        }))
        .create()
        .await;

    // 3. Assign Policy A (Jan 1, 2026 -> Mar 1, 2026)
    factory
        .leave_policy()
        .assign_to(
            ws_id,
            policy_monthly,
            "employee",
            Some(emp),
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            Some(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()),
        )
        .await;

    // 4. Run Accrual for Feb 1, 2026
    // Should catch up Jan 1st and Feb 1st: Total 2 logs
    let logs = LeaveAccrualService::process_workspace_accruals(
        &pool,
        ws_id,
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        false,
    )
    .await
    .unwrap();

    assert_eq!(logs.len(), 2);
    assert_eq!(logs[0].amount.to_string(), "1.00"); // Jan 1st
    assert_eq!(logs[1].amount.to_string(), "1.00"); // Feb 1st

    // 5. Assign Policy B (Effective Mar 1, 2026 -> Apr 1, 2026)
    factory
        .leave_policy()
        .assign_to(
            ws_id,
            policy_seniority,
            "employee",
            Some(emp),
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            Some(NaiveDate::from_ymd_opt(2026, 4, 1).unwrap()),
        )
        .await;

    // 6. Run Accrual for Mar 1, 2026
    // Hire date Jan 1, 2025. By Mar 1, 2026, service > 1 year.
    // Should get 1.0 (base) + 1.0 (bonus) = 2.0
    let logs_mar = LeaveAccrualService::process_workspace_accruals(
        &pool,
        ws_id,
        NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        false,
    )
    .await
    .unwrap();

    assert_eq!(logs_mar.len(), 1);
    assert_eq!(logs_mar[0].amount.to_string(), "2.00");

    // Verify Balance
    let balance: rust_decimal::Decimal =
        sqlx::query_scalar("SELECT accrued_amount FROM leave_balances WHERE employee_id = $1")
            .bind(emp)
            .fetch_one(&pool)
            .await
            .unwrap();

    // 1.0 (Jan) + 1.0 (Feb) + 2.0 (Mar) = 4.0
    assert_eq!(balance.to_string(), "4.00");

    // 7. Test Quarterly Policy
    let policy_quarterly = factory
        .leave_policy()
        .with_workspace(ws_id)
        .with_name("Quarterly")
        .with_config(serde_json::json!({
            "frequency": "quarterly",
            "accrual_amount": 3.0,
            "accrual_timing": "start_of_period"
        }))
        .create()
        .await;

    factory
        .leave_policy()
        .assign_to(
            ws_id,
            policy_quarterly,
            "employee",
            Some(emp),
            NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            None,
        )
        .await;

    // Run for Apr 1, 2026 (Start of Q2) -> Should run
    let logs_apr = LeaveAccrualService::process_workspace_accruals(
        &pool,
        ws_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        false,
    )
    .await
    .unwrap();

    println!("DEBUG LOGS APR: {:?}", logs_apr);

    assert_eq!(logs_apr.len(), 1);
    assert_eq!(logs_apr[0].amount.to_string(), "3.00");

    // Run for May 1, 2026 -> Should NOT run (not start of quarter)
    let logs_may = LeaveAccrualService::process_workspace_accruals(
        &pool,
        ws_id,
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        false,
    )
    .await
    .unwrap();

    assert_eq!(logs_may.len(), 0);
}

#[tokio::test]
async fn test_accrual_creates_balance_if_missing() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = common::factory::TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Employee hired in 2027
    let emp = factory
        .employee()
        .with_workspace(ws_id)
        .with_hire_date(NaiveDate::from_ymd_opt(2027, 1, 1).unwrap())
        .create()
        .await;

    // NO leave_balance created for 2027 yet

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
            NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            None,
        )
        .await;

    // Run Accrual for Jan 1, 2027
    let logs = LeaveAccrualService::process_workspace_accruals(
        &pool,
        ws_id,
        NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
        false,
    )
    .await
    .unwrap();

    assert_eq!(logs.len(), 1);

    // Verify Balance was CREATED for 2027
    let balance = sqlx::query!(
        "SELECT accrued_amount, year FROM leave_balances WHERE employee_id = $1 AND year = 2027",
        emp
    )
    .fetch_one(&pool)
    .await
    .expect("Balance should have been created");

    assert_eq!(balance.accrued_amount.to_string(), "1.00");
    assert_eq!(balance.year, 2027);
}
