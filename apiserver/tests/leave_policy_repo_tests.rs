mod common;

use apiserver::models::{AccrualFrequency, LeavePolicyAssignment, LeavePolicyConfig};
use apiserver::repositories::leave_policy_repository;
use chrono::Utc;
use common::db::TestDatabase;
use common::factory::TestFactory;
use uuid::Uuid;

#[tokio::test]
async fn test_find_active_policy_priority() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let dept_id = Uuid::new_v4(); // Mock dept_id

    let config = LeavePolicyConfig {
        frequency: AccrualFrequency::Monthly,
        accrual_amount: 1.0,
        accrual_timing: "start_of_period".to_string(),
        seniority_bonus: vec![],
        max_balance: None,
        reset_month: None,
    };

    // 1. Create Global Policy
    let global_policy = leave_policy_repository::create_policy_definition(
        &pool,
        ws_id,
        "Global",
        None,
        "vacation",
        config.clone(),
    )
    .await
    .unwrap();

    let global_assign = LeavePolicyAssignment {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        assignee_type: "workspace".to_string(),
        assignee_id: None,
        policy_id: global_policy.id,
        effective_from: Utc::now().date_naive(),
        effective_to: None,
        created_at: Utc::now(),
    };
    leave_policy_repository::create_policy_assignment(&pool, &global_assign)
        .await
        .unwrap();

    // Verify global is active
    let found = leave_policy_repository::find_active_assignment_for_employee(
        &pool,
        ws_id,
        emp_id,
        Some(dept_id),
        Utc::now().date_naive(),
    )
    .await
    .unwrap();
    assert_eq!(found.unwrap().0.id, global_policy.id);

    // 2. Create Dept Policy
    let dept_policy = leave_policy_repository::create_policy_definition(
        &pool,
        ws_id,
        "Dept",
        None,
        "vacation",
        config.clone(),
    )
    .await
    .unwrap();

    let dept_assign = LeavePolicyAssignment {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        assignee_type: "department".to_string(),
        assignee_id: Some(dept_id),
        policy_id: dept_policy.id,
        effective_from: Utc::now().date_naive(),
        effective_to: None,
        created_at: Utc::now(),
    };
    leave_policy_repository::create_policy_assignment(&pool, &dept_assign)
        .await
        .unwrap();

    // Verify dept takes priority over global
    let found = leave_policy_repository::find_active_assignment_for_employee(
        &pool,
        ws_id,
        emp_id,
        Some(dept_id),
        Utc::now().date_naive(),
    )
    .await
    .unwrap();
    assert_eq!(found.unwrap().0.id, dept_policy.id);

    // 3. Create Employee Policy
    let emp_policy = leave_policy_repository::create_policy_definition(
        &pool,
        ws_id,
        "Employee",
        None,
        "vacation",
        config.clone(),
    )
    .await
    .unwrap();

    let emp_assign = LeavePolicyAssignment {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        assignee_type: "employee".to_string(),
        assignee_id: Some(emp_id),
        policy_id: emp_policy.id,
        effective_from: Utc::now().date_naive(),
        effective_to: None,
        created_at: Utc::now(),
    };
    leave_policy_repository::create_policy_assignment(&pool, &emp_assign)
        .await
        .unwrap();

    // Verify employee takes priority over all
    let found = leave_policy_repository::find_active_assignment_for_employee(
        &pool,
        ws_id,
        emp_id,
        Some(dept_id),
        Utc::now().date_naive(),
    )
    .await
    .unwrap();
    assert_eq!(found.unwrap().0.id, emp_policy.id);

    // 4. No policy found (use a fresh workspace)
    let ws_id_2 = factory.workspace().create().await;
    let other_emp = factory.employee().with_workspace(ws_id_2).create().await;
    let found = leave_policy_repository::find_active_assignment_for_employee(
        &pool,
        ws_id_2,
        other_emp,
        None,
        Utc::now().date_naive(),
    )
    .await
    .unwrap();
    assert!(found.is_none());
}