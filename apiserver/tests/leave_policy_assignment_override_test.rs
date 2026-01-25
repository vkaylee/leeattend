mod common;

use apiserver::models::{AccrualFrequency, LeavePolicyAssignment, LeavePolicyConfig};
use apiserver::repositories::leave_policy_repository;
use chrono::{NaiveDate, Datelike};
use common::db::TestDatabase;
use common::factory::TestFactory;
use uuid::Uuid;

#[tokio::test]
async fn test_assign_policy_override_and_future_cleanup() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    
    // Get leave type
    let vacation_id = sqlx::query!("SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'", ws_id)
        .fetch_one(&pool)
        .await
        .unwrap()
        .id;

    let config = LeavePolicyConfig {
        frequency: AccrualFrequency::Monthly,
        accrual_amount: 1.0,
        accrual_timing: "start_of_period".to_string(),
        seniority_bonus: vec![],
        max_balance: None,
        reset_month: None,
        min_work_days_threshold: None,
        carry_over_limit: None,
    };

    let policy = leave_policy_repository::create_policy_definition(
        &pool,
        ws_id,
        "Standard Policy",
        None,
        vacation_id,
        config,
    )
    .await
    .unwrap();

    // 1. Assign Policy from Jan 1st
    let assign_jan = LeavePolicyAssignment {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        assignee_type: "workspace".to_string(),
        assignee_id: None,
        policy_id: policy.id,
        effective_from: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        effective_to: None,
        created_at: chrono::Utc::now(),
    };
    leave_policy_repository::create_policy_assignment(&pool, &assign_jan).await.unwrap();

    // Verify
    let assignments = leave_policy_repository::get_policy_assignments(&pool, ws_id).await.unwrap();
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].effective_from, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
    assert_eq!(assignments[0].effective_to, None);

    // 2. Assign Policy from June 1st (Future split)
    let assign_jun = LeavePolicyAssignment {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        assignee_type: "workspace".to_string(),
        assignee_id: None,
        policy_id: policy.id,
        effective_from: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        effective_to: None,
        created_at: chrono::Utc::now(),
    };
    leave_policy_repository::create_policy_assignment(&pool, &assign_jun).await.unwrap();

    // Verify: Jan assignment is capped at Jun 1st. Jun assignment is active.
    let assignments = leave_policy_repository::get_policy_assignments(&pool, ws_id).await.unwrap();
    assert_eq!(assignments.len(), 2);
    // Sort by effective_from
    // Note: get_policy_assignments returns sorted by created_at DESC.
    // We should sort by effective_from to be sure
    let mut sorted_assignments = assignments.clone();
    sorted_assignments.sort_by_key(|a| a.effective_from);
    
    let jan_assign = &sorted_assignments[0];
    let jun_assign = &sorted_assignments[1];

    assert_eq!(jan_assign.effective_from.month(), 1);
    assert_eq!(jun_assign.effective_from.month(), 6);

    assert_eq!(jan_assign.effective_to, Some(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()));
    assert_eq!(jun_assign.effective_to, None);

    // 3. Assign Policy from March 1st (Should CUT Jan short, and DELETE June)
    let assign_mar = LeavePolicyAssignment {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        assignee_type: "workspace".to_string(),
        assignee_id: None,
        policy_id: policy.id,
        effective_from: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        effective_to: None,
        created_at: chrono::Utc::now(),
    };
    leave_policy_repository::create_policy_assignment(&pool, &assign_mar).await.unwrap();

    // Verify
    let assignments = leave_policy_repository::get_policy_assignments(&pool, ws_id).await.unwrap();
    assert_eq!(assignments.len(), 2); // Should be 2 (Jan + Mar). June should be gone.

    let mut sorted_assignments = assignments.clone();
    sorted_assignments.sort_by_key(|a| a.effective_from);

    let jan_assign = &sorted_assignments[0];
    let mar_assign = &sorted_assignments[1];
    
    assert_eq!(jan_assign.effective_from.month(), 1);
    assert_eq!(mar_assign.effective_from.month(), 3);
    
    assert!(assignments.iter().find(|a| a.effective_from.month() == 6).is_none(), "June assignment should be deleted");

    assert_eq!(jan_assign.effective_to, Some(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()));
    assert_eq!(mar_assign.effective_to, None);
}
