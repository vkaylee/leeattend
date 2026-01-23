mod common ;

use apiserver::models::{OvertimeRule};
use apiserver::repositories::overtime_policy_repository::OvertimePolicyRepository;
use chrono::Utc;
use common::db::TestDatabase;
use common::factory::TestFactory;
use uuid::Uuid;

#[tokio::test]
async fn test_overtime_policy_resolution() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let date = Utc::now().date_naive();

    // 1. Create Policies
    let global_pol = OvertimePolicyRepository::create_policy_definition(&pool, ws_id, "Global", None).await.unwrap();
    let emp_pol = OvertimePolicyRepository::create_policy_definition(&pool, ws_id, "Employee Specific", None).await.unwrap();

    // 2, Assign Global
    OvertimePolicyRepository::create_policy_assignment(
        &pool,
        ws_id,
        "workspace",
        None,
        global_pol.id,
        date,
    ).await.unwrap();

    let resolved = OvertimePolicyRepository::resolve_policy_for_employee(&pool, emp_id, ws_id, date).await.unwrap();
    assert_eq!(resolved, Some(global_pol.id));

    // 3. Assign Employee Override
    OvertimePolicyRepository::create_policy_assignment(
        &pool,
        ws_id,
        "employee",
        Some(emp_id),
        emp_pol.id,
        date,
    ).await.unwrap();

    let resolved2 = OvertimePolicyRepository::resolve_policy_for_employee(&pool, emp_id, ws_id, date).await.unwrap();
    assert_eq!(resolved2, Some(emp_pol.id));
}

#[tokio::test]
async fn test_create_policy_with_rules() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let pol = OvertimePolicyRepository::create_policy_definition(&pool, ws_id, "Night Shift", Some("Test")).await.unwrap();
    
    let rule = OvertimeRule {
        id: Uuid::new_v4(),
        policy_id: pol.id,
        priority: 10,
        multiplier: rust_decimal::Decimal::new(20, 1), // 2.0
        time_start: Some(chrono::NaiveTime::from_hms_opt(22, 0, 0).unwrap()),
        time_end: Some(chrono::NaiveTime::from_hms_opt(6, 0, 0).unwrap()),
        days_of_week: 127,
        is_holiday: false,
        created_at: None,
        updated_at: None,
    };
    
    OvertimePolicyRepository::add_rule(&pool, &rule).await.unwrap();

    let detail = OvertimePolicyRepository::get_policy_with_rules(&pool, pol.id).await.unwrap().unwrap();
    assert_eq!(detail.0.name, "Night Shift");
    assert_eq!(detail.1.len(), 1);
    assert_eq!(detail.1[0].multiplier, rust_decimal::Decimal::new(20, 1));
}
