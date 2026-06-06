use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::EarlyArrivalStrategy;
use apiserver::services::PolicyResolverService;
use chrono::NaiveDate;
use uuid::Uuid;

#[tokio::test]
async fn test_resolve_attendance_policy_hierarchy() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let policy_repo =
        apiserver::repositories::DieselAttendancePolicyRepository::new(diesel_pools.clone());

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Clean up auto-generated policies
    crate::test_execute!(
        &pool.api,
        "DELETE FROM attendance_policies WHERE workspace_id = $1",
        diesel::sql_types::Uuid,
        ws_id
    );

    let dept_repo = apiserver::repositories::DieselDepartmentRepository::new(diesel_pools.clone());

    use apiserver::repositories::department_repository::DepartmentRepository;
    // 1. Create a department
    let dept = dept_repo
        .create_department(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            "Engineering".to_string(),
            None,
            None,
            None,
        )
        .await
        .unwrap();
    let dept_id = dept.id();

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
    dept_repo
        .transfer_employee(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employee_id,
            dept_id,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            None,
        )
        .await
        .unwrap();

    let test_date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();

    // === CASE 1: Default Policy ===
    let default_policy_id = Uuid::new_v4();
    crate::test_execute!(&pool.api,
        "INSERT INTO attendance_policies (id, workspace_id, name, is_default) VALUES ($1, $2, 'Default', true)",
        diesel::sql_types::Uuid, default_policy_id,
        diesel::sql_types::Uuid, ws_id
    );

    crate::test_execute!(&pool.api,
        "INSERT INTO attendance_policy_versions (policy_id, rules, valid_from) VALUES ($1, $2, '2000-01-01 00:00:00+00')",
        diesel::sql_types::Uuid, default_policy_id,
        diesel::sql_types::Jsonb, serde_json::json!({ "work_unit_minutes": 10, "rounding_policy": "floor", "early_arrival_strategy": "block" })
    );

    let rules = PolicyResolverService::resolve_attendance_policy(
        &policy_repo,
        employee_id,
        ws_id,
        test_date,
    )
    .await
    .unwrap();
    assert_eq!(rules.work_unit_minutes, 10);
    assert_eq!(rules.early_arrival_strategy, EarlyArrivalStrategy::Block);

    // === CASE 2: Group (Department) Policy ===
    let group_policy_id = Uuid::new_v4();
    crate::test_execute!(&pool.api,
        "INSERT INTO attendance_policies (id, workspace_id, name, is_default) VALUES ($1, $2, 'Dept Policy', false)",
        diesel::sql_types::Uuid, group_policy_id,
        diesel::sql_types::Uuid, ws_id
    );

    crate::test_execute!(&pool.api,
        "INSERT INTO attendance_policy_versions (policy_id, rules, valid_from) VALUES ($1, $2, '2000-01-01 00:00:00+00')",
        diesel::sql_types::Uuid, group_policy_id,
        diesel::sql_types::Jsonb, serde_json::json!({ "work_unit_minutes": 20, "rounding_policy": "ceil", "early_arrival_strategy": "allow" })
    );

    crate::test_execute!(&pool.api,
        "INSERT INTO attendance_policy_assignments (policy_id, target_type, target_id, valid_from) VALUES ($1, 'GROUP', $2, '2026-05-01 00:00:00+00')",
        diesel::sql_types::Uuid, group_policy_id,
        diesel::sql_types::Uuid, dept_id
    );

    let rules = PolicyResolverService::resolve_attendance_policy(
        &policy_repo,
        employee_id,
        ws_id,
        test_date,
    )
    .await
    .unwrap();
    assert_eq!(rules.work_unit_minutes, 20, "Should inherit from GROUP");
    assert_eq!(rules.early_arrival_strategy, EarlyArrivalStrategy::Allow);

    // === CASE 3: User Override Policy ===
    let user_policy_id = Uuid::new_v4();
    crate::test_execute!(&pool.api,
        "INSERT INTO attendance_policies (id, workspace_id, name, is_default) VALUES ($1, $2, 'User Policy', false)",
        diesel::sql_types::Uuid, user_policy_id,
        diesel::sql_types::Uuid, ws_id
    );

    crate::test_execute!(&pool.api,
        "INSERT INTO attendance_policy_versions (policy_id, rules, valid_from) VALUES ($1, $2, '2000-01-01 00:00:00+00')",
        diesel::sql_types::Uuid, user_policy_id,
        diesel::sql_types::Jsonb, serde_json::json!({ "work_unit_minutes": 30, "rounding_policy": "round", "early_arrival_strategy": "clamp" })
    );

    crate::test_execute!(&pool.api,
        "INSERT INTO attendance_policy_assignments (policy_id, target_type, target_id, valid_from) VALUES ($1, 'USER', $2, '2026-05-15 00:00:00+00')",
        diesel::sql_types::Uuid, user_policy_id,
        diesel::sql_types::Uuid, employee_id
    );

    let rules = PolicyResolverService::resolve_attendance_policy(
        &policy_repo,
        employee_id,
        ws_id,
        test_date,
    )
    .await
    .unwrap();
    assert_eq!(rules.work_unit_minutes, 30, "Should be overridden by USER");
    assert_eq!(rules.early_arrival_strategy, EarlyArrivalStrategy::Clamp);
}
