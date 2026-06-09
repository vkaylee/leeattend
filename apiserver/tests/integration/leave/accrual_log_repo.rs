use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::diesel_leave_accrual_log_repository::DieselLeaveAccrualLogRepository;
use apiserver::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

#[tokio::test]
async fn test_leave_accrual_log_repo_crud() {
    let _test_db = TestDatabase::new().await;
    let factory = TestFactory::new(_test_db.diesel_pools.clone());

    let repo = DieselLeaveAccrualLogRepository::new(factory.diesel_pools.clone());

    // Create required data
    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Create leave type
    let leave_type_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    // Create employee
    let employee_id = factory.employee().with_workspace(ws_id).create().await;

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    let accrual_date = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();

    // 1. Insert Accrual Log
    let log = repo
        .insert_accrual_log(
            ws_id,
            employment_id,
            None, // assignment_id
            accrual_date,
            Decimal::new(15, 1), // 1.5
            leave_type_id,
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(log.amount(), Decimal::new(15, 1));
    assert_eq!(log.employment_id(), employment_id);
    assert_eq!(log.leave_type_id(), leave_type_id);

    // 2. Find by employment, date, type
    let found = repo
        .find_by_employment_date_type(employment_id, leave_type_id, accrual_date, ws_id)
        .await
        .unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, log.id());
    assert_eq!(found.amount, Decimal::new(15, 1));

    // 3. Update Accrual Amount
    let updated = repo
        .update_accrual_amount(log.id(), ws_id, Decimal::new(20, 1))
        .await
        .unwrap();
    assert_eq!(updated.amount(), Decimal::new(20, 1));
    // Notes should contain (Re-calculated)
    assert!(updated
        .notes()
        .unwrap_or_default()
        .contains("(Re-calculated)"));

    // 4. Get Employment Leave History Logs
    let history = repo
        .get_employment_leave_history_logs(ws_id, employment_id, 2024, Some(leave_type_id))
        .await
        .unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].amount, Decimal::new(20, 1));

    // 5. Upsert Balance Accrual
    repo.upsert_balance_accrual(
        ws_id,
        employment_id,
        2024,
        leave_type_id,
        Decimal::new(50, 1),
    )
    .await
    .unwrap();

    // 6. Adjust Balance Accrual
    repo.adjust_balance_accrual(
        ws_id,
        employment_id,
        2024,
        leave_type_id,
        Decimal::new(10, 1),
    )
    .await
    .unwrap();

    // Check balance
    let balance_amt: bigdecimal::BigDecimal = crate::test_query_scalar_bigdecimal!(
        &factory.diesel_pools.api,
        "SELECT lb.accrued_amount as val FROM leave_balances lb JOIN employments emp ON lb.employment_id = emp.id WHERE emp.employee_id = $1",
        diesel::sql_types::Uuid,
        employee_id
    );
    use std::str::FromStr;
    assert_eq!(
        Decimal::from_str(&balance_amt.to_string()).unwrap(),
        Decimal::new(60, 1)
    ); // 5.0 + 1.0 = 6.0

    // 7. Test get_workspace_timezone
    let tz = repo.get_workspace_timezone(ws_id).await.unwrap();
    assert_eq!(tz, "Asia/Ho_Chi_Minh");

    // 8. Test get_assignment_effective_from
    let policy_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "INSERT INTO leave_policy_definitions (id, workspace_id, leave_type_id, name, rule_config)
         VALUES ($1, $2, $3, 'Test Policy', '{}'::jsonb) RETURNING id",
        diesel::sql_types::Uuid,
        Uuid::new_v4(),
        diesel::sql_types::Uuid,
        ws_id,
        diesel::sql_types::Uuid,
        leave_type_id
    );

    let assignment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "INSERT INTO leave_policy_assignments (id, workspace_id, assignee_type, assignee_id, policy_id, effective_from)
         VALUES ($1, $2, 'employee', $3, $4, '2024-01-01') RETURNING id",
        diesel::sql_types::Uuid,
        Uuid::new_v4(),
        diesel::sql_types::Uuid,
        ws_id,
        diesel::sql_types::Uuid,
        employee_id,
        diesel::sql_types::Uuid,
        policy_id
    );

    let effective_from = repo
        .get_assignment_effective_from(assignment_id, ws_id)
        .await
        .unwrap();
    assert_eq!(effective_from, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

    // 9. Test persist_accrual
    // Get tenant auth token
    let auth_token = apiserver::diesel_pool::TenantAuthToken::system_override(ws_id);

    let persist_date = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();

    // First time insert
    let persisted = repo
        .persist_accrual(
            &auth_token,
            ws_id,
            employment_id,
            assignment_id,
            leave_type_id,
            persist_date,
            3.5, // 3.5 days
            false,
        )
        .await
        .unwrap();

    assert!(persisted.is_some());
    let p = persisted.unwrap();
    assert_eq!(p.amount(), Decimal::new(35, 1));
    assert_eq!(p.accrual_date(), persist_date);

    // Call again with same amount and force=false should return None
    let persisted2 = repo
        .persist_accrual(
            &auth_token,
            ws_id,
            employment_id,
            assignment_id,
            leave_type_id,
            persist_date,
            3.5,
            false,
        )
        .await
        .unwrap();
    assert!(persisted2.is_none());

    // Call again with different amount and force=true should return Some and update
    let persisted3 = repo
        .persist_accrual(
            &auth_token,
            ws_id,
            employment_id,
            assignment_id,
            leave_type_id,
            persist_date,
            5.0,
            true,
        )
        .await
        .unwrap();
    assert!(persisted3.is_some());
    let p3 = persisted3.unwrap();
    assert_eq!(p3.amount(), Decimal::new(50, 1));

    // And balance should be updated accordingly (6.0 + 3.5 + (5.0 - 3.5) = 11.0)
    let balance_amt_new: bigdecimal::BigDecimal = crate::test_query_scalar_bigdecimal!(
        &factory.diesel_pools.api,
        "SELECT lb.accrued_amount as val FROM leave_balances lb JOIN employments emp ON lb.employment_id = emp.id WHERE emp.employee_id = $1",
        diesel::sql_types::Uuid,
        employee_id
    );
    assert_eq!(
        Decimal::from_str(&balance_amt_new.to_string()).unwrap(),
        Decimal::new(110, 1)
    );
}
