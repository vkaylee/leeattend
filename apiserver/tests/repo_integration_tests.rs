mod common;

use apiserver::repositories::{roster_repository, shift_repository};
use chrono::{Duration, NaiveDate, NaiveTime, Utc};
use common::db::TestDatabase;
use common::factory::TestFactory;
use uuid::Uuid;

#[tokio::test]
async fn test_get_effective_shift_priority_integrated() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let s_dept = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Dept")
        .create()
        .await;
    let s_fixed = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Fixed")
        .create()
        .await;
    let s_roster = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Roster")
        .create()
        .await;

    let date = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();

    // 1. Dept Priority
    let dept_id: Uuid = sqlx::query_scalar(
        "INSERT INTO departments (workspace_id, name) VALUES ($1, 'Dev') RETURNING id",
    )
    .bind(ws_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO employee_department_history (id, employee_id, department_id, effective_date) VALUES ($1, $2, $3, $4)").bind(Uuid::new_v4()).bind(emp_id).bind(dept_id).bind(date - Duration::days(1)).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO department_default_shifts (department_id, shift_id) VALUES ($1, $2)")
        .bind(dept_id)
        .bind(s_dept)
        .execute(&pool)
        .await
        .unwrap();

    let (res, _) = roster_repository::get_effective_shift(&pool, emp_id, date)
        .await
        .unwrap();
    assert_eq!(res, Some(s_dept));

    // 2. Fixed Overrides Dept
    sqlx::query("INSERT INTO employee_shifts (id, employee_id, shift_id, start_date) VALUES ($1, $2, $3, $4)").bind(Uuid::new_v4()).bind(emp_id).bind(s_fixed).bind(date).execute(&pool).await.unwrap();
    let (res, _) = roster_repository::get_effective_shift(&pool, emp_id, date)
        .await
        .unwrap();
    assert_eq!(res, Some(s_fixed));

    // 3. Roster Overrides Fixed
    sqlx::query(
        "INSERT INTO shift_rosters (id, employee_id, shift_id, date) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(emp_id)
    .bind(s_roster)
    .bind(date)
    .execute(&pool)
    .await
    .unwrap();
    let (res, _) = roster_repository::get_effective_shift(&pool, emp_id, date)
        .await
        .unwrap();
    assert_eq!(res, Some(s_roster));
}

#[tokio::test]
async fn test_shift_repository_basic_ops_integrated() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let shift = shift_repository::create_shift(
        &pool,
        ws_id,
        "Test Shift".to_string(),
        NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        60,
        None,
        None,
        true,
        false,
        0,
        false,
        true,
        None,
    )
    .await
    .unwrap();

    // Get by ID
    let fetched = shift_repository::get_shift_by_id(&pool, shift.id)
        .await
        .unwrap();
    assert!(fetched.is_some());

    // Archive
    shift_repository::archive_shift(&pool, shift.id)
        .await
        .unwrap();
    let fetched = shift_repository::get_shift_by_id(&pool, shift.id)
        .await
        .unwrap();
    assert!(!fetched.unwrap().is_active);
}

#[tokio::test]
async fn test_roster_repository_delete() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    sqlx::query("INSERT INTO shift_rosters (id, employee_id, date) VALUES ($1, $2, $3)")
        .bind(Uuid::new_v4())
        .bind(emp_id)
        .bind(date)
        .execute(&pool)
        .await
        .unwrap();

    roster_repository::delete_roster(&pool, emp_id, date)
        .await
        .unwrap();

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM shift_rosters WHERE employee_id = $1")
            .bind(emp_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_leave_balance_repo_upsert() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let balance = apiserver::models::LeaveBalance {
        id: Uuid::new_v4(),
        employee_id: emp_id,
        leave_type: "vacation".to_string(),
        year: 2026,
        entitlement: 15,
        accrued_amount: rust_decimal::Decimal::ZERO,
        carried_over: 2,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    apiserver::repositories::leave_balance_repository::upsert_leave_balance(&pool, &balance)
        .await
        .unwrap();

    // Upsert again (Update)
    let mut balance2 = balance.clone();
    balance2.entitlement = 20;
    apiserver::repositories::leave_balance_repository::upsert_leave_balance(&pool, &balance2)
        .await
        .unwrap();

    let fetched = apiserver::repositories::leave_balance_repository::get_employee_balances(
        &pool, emp_id, 2026,
    )
    .await
    .unwrap();
    assert_eq!(fetched.len(), 1);
    assert_eq!(fetched[0].entitlement, 20);
}

#[tokio::test]
async fn test_get_employee_active_shift_integrated() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let shift_id = factory.shift().with_workspace(ws_id).create().await;

    let date = Utc::now().date_naive();
    sqlx::query(
        "INSERT INTO employee_shifts (employee_id, shift_id, start_date) VALUES ($1, $2, $3)",
    )
    .bind(emp_id)
    .bind(shift_id)
    .bind(date)
    .execute(&pool)
    .await
    .unwrap();

    let active = shift_repository::get_employee_active_shift(&pool, emp_id, date)
        .await
        .unwrap();
    assert_eq!(active.unwrap().id, shift_id);
}

#[tokio::test]
async fn test_update_assignments_transaction() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let old_shift = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Old")
        .create()
        .await;
    let new_shift = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("New")
        .create()
        .await;

    let start_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    sqlx::query(
        "INSERT INTO employee_shifts (employee_id, shift_id, start_date) VALUES ($1, $2, $3)",
    )
    .bind(emp_id)
    .bind(old_shift)
    .bind(start_date)
    .execute(&pool)
    .await
    .unwrap();

    let mut tx = pool.begin().await.unwrap();
    shift_repository::update_assignments_to_new_version_tx(
        &mut tx,
        old_shift,
        new_shift,
        NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    // Verify
    let active = shift_repository::get_employee_active_shift(
        &pool,
        emp_id,
        NaiveDate::from_ymd_opt(2026, 1, 16).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(active.unwrap().id, new_shift);
}
