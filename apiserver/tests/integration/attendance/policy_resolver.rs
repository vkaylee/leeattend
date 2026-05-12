use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::domain::policy::resolver::PolicyResolver;
use apiserver::models::EarlyArrivalStrategy;
use chrono::NaiveDate;
use uuid::Uuid;

#[tokio::test]
async fn test_resolve_attendance_policy_hierarchy() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(
        &std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/leeattend".to_string()
        }),
    );

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Clean up auto-generated policies
    sqlx::query!(
        "DELETE FROM attendance_policies WHERE workspace_id = $1",
        ws_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // 1. Create a department
    let dept_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO departments (id, workspace_id, name) VALUES ($1, $2, 'Engineering')",
        dept_id,
        ws_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // 2. Create an employee
    let user_id = factory
        .user()
        .with_email("test_resolver@example.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            user_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let employee_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(user_id)
        .create()
        .await;

    // Assign employee to department
    sqlx::query!(
        "INSERT INTO employee_department_history (employee_id, department_id, effective_date) VALUES ($1, $2, '2026-01-01')",
        employee_id,
        dept_id
    )
    .execute(&pool)
    .await
    .unwrap();

    let test_date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();

    // === CASE 1: Default Policy ===
    let default_policy_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO attendance_policies (id, workspace_id, name, is_default) VALUES ($1, $2, 'Default', true)",
        default_policy_id, ws_id
    ).execute(&pool).await.unwrap();

    sqlx::query!(
        "INSERT INTO attendance_policy_versions (policy_id, rules, valid_from) VALUES ($1, $2, '2000-01-01 00:00:00+00')",
        default_policy_id,
        serde_json::json!({ "work_unit_minutes": 10, "rounding_policy": "floor", "early_arrival_strategy": "block" })
    ).execute(&pool).await.unwrap();

    let rules = PolicyResolver::resolve_attendance_policy(&pool, employee_id, ws_id, test_date)
        .await
        .unwrap();
    assert_eq!(rules.work_unit_minutes, 10);
    assert_eq!(rules.early_arrival_strategy, EarlyArrivalStrategy::Block);

    // === CASE 2: Group (Department) Policy ===
    let group_policy_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO attendance_policies (id, workspace_id, name, is_default) VALUES ($1, $2, 'Dept Policy', false)",
        group_policy_id, ws_id
    ).execute(&pool).await.unwrap();

    sqlx::query!(
        "INSERT INTO attendance_policy_versions (policy_id, rules, valid_from) VALUES ($1, $2, '2000-01-01 00:00:00+00')",
        group_policy_id,
        serde_json::json!({ "work_unit_minutes": 20, "rounding_policy": "ceil", "early_arrival_strategy": "allow" })
    ).execute(&pool).await.unwrap();

    sqlx::query!(
        "INSERT INTO attendance_policy_assignments (policy_id, target_type, target_id, valid_from) VALUES ($1, 'GROUP', $2, '2026-05-01 00:00:00+00')",
        group_policy_id, dept_id
    ).execute(&pool).await.unwrap();

    let rules = PolicyResolver::resolve_attendance_policy(&pool, employee_id, ws_id, test_date)
        .await
        .unwrap();
    assert_eq!(rules.work_unit_minutes, 20, "Should inherit from GROUP");
    assert_eq!(rules.early_arrival_strategy, EarlyArrivalStrategy::Allow);

    // === CASE 3: User Override Policy ===
    let user_policy_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO attendance_policies (id, workspace_id, name, is_default) VALUES ($1, $2, 'User Policy', false)",
        user_policy_id, ws_id
    ).execute(&pool).await.unwrap();

    sqlx::query!(
        "INSERT INTO attendance_policy_versions (policy_id, rules, valid_from) VALUES ($1, $2, '2000-01-01 00:00:00+00')",
        user_policy_id,
        serde_json::json!({ "work_unit_minutes": 30, "rounding_policy": "round", "early_arrival_strategy": "clamp" })
    ).execute(&pool).await.unwrap();

    sqlx::query!(
        "INSERT INTO attendance_policy_assignments (policy_id, target_type, target_id, valid_from) VALUES ($1, 'USER', $2, '2026-05-15 00:00:00+00')",
        user_policy_id, employee_id
    ).execute(&pool).await.unwrap();

    let rules = PolicyResolver::resolve_attendance_policy(&pool, employee_id, ws_id, test_date)
        .await
        .unwrap();
    assert_eq!(rules.work_unit_minutes, 30, "Should be overridden by USER");
    assert_eq!(rules.early_arrival_strategy, EarlyArrivalStrategy::Clamp);
}
