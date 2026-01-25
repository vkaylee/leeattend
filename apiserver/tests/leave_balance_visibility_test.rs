mod common;

use apiserver::models::{AccrualFrequency, LeavePolicyAssignment, LeavePolicyConfig};
use apiserver::repositories::{leave_balance_repository, leave_policy_repository};
use chrono::{Utc, Datelike};
use common::db::TestDatabase;
use common::factory::TestFactory;
use uuid::Uuid;

#[tokio::test]
async fn test_balance_visibility_after_assignment() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Get leave type
    let vacation_id = sqlx::query!("SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'", ws_id)
        .fetch_one(&pool)
        .await
        .unwrap()
        .id;

    // Create Policy
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
        "Super Vacation Policy",
        None,
        vacation_id,
        config,
    )
    .await
    .unwrap();

    // Assign Policy to Employee
    let assignment = LeavePolicyAssignment {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        assignee_type: "employee".to_string(),
        assignee_id: Some(emp_id),
        policy_id: policy.id,
        effective_from: Utc::now().date_naive(),
        effective_to: None,
        created_at: Utc::now(),
    };
    leave_policy_repository::create_policy_assignment(&pool, &assignment).await.unwrap();

    // Verify Balances Visibility
    // Note: No accrual run has happened, so no leave_balances record exists for this employee/type.
    let year = Utc::now().date_naive().year();
    let (balances, total) = leave_balance_repository::get_workspace_balances(
        &pool,
        ws_id,
        year,
        20,
        0,
        None
    ).await.unwrap();

    assert_eq!(total, 4, "Should find 4 balance rows (1 emp * 4 seeded types)");
    assert_eq!(balances.len(), 4);
    
    let balance = balances.iter().find(|b| b.leave_type_id == vacation_id).unwrap();
    assert_eq!(balance.employee_id, emp_id);
    assert_eq!(balance.leave_type_id, vacation_id);
    assert_eq!(balance.policy_name, Some("Super Vacation Policy".to_string()));
    assert_eq!(balance.entitlement, 0); // Correctly 0
}
