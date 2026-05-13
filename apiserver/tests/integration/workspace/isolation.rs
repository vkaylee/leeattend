//! Workspace isolation integration tests.
//! Verifies that repository queries with `workspace_id` correctly scope data,
//! preventing cross-workspace data leaks.

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::{
    department_repository, time_entry_repository, working_pattern_repository,
    DieselTimeEntryRepository, TimeEntryRepository,
};
use chrono::{Duration, NaiveDate, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Helper: create a workspace with an employee and return (`workspace_id`, `employee_id`).
async fn create_workspace_with_employee(factory: &TestFactory) -> (Uuid, Uuid) {
    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    (ws_id, emp_id)
}

// ─── Time Entry Repository Isolation ─────────────────────────────────────

#[tokio::test]
async fn test_get_employee_time_entries_isolated_by_workspace() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    // Create two workspaces each with an employee
    let (ws_a, emp_a) = create_workspace_with_employee(&factory).await;
    let (ws_b, _emp_b) = create_workspace_with_employee(&factory).await;

    // Insert a time entry for emp_a in ws_a
    let now = Utc::now();
    factory
        .time_entry()
        .with_employee(emp_a)
        .with_punch_in(now - Duration::hours(2))
        .with_punch_out(now)
        .create()
        .await;

    let start = now - Duration::hours(24);
    let end = now + Duration::hours(1);

    // Query with correct workspace → should find 1 entry
    let entries_a =
        time_entry_repository::get_employee_time_entries(&pool, ws_a, emp_a, start, end)
            .await
            .unwrap();
    assert_eq!(
        entries_a.len(),
        1,
        "Should find 1 entry in the correct workspace"
    );

    // Query with wrong workspace → should find 0 entries
    let entries_b =
        time_entry_repository::get_employee_time_entries(&pool, ws_b, emp_a, start, end)
            .await
            .unwrap();
    assert_eq!(
        entries_b.len(),
        0,
        "Should find 0 entries when querying with wrong workspace_id"
    );
}

#[tokio::test]
async fn test_get_latest_active_entry_isolated_by_workspace() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let (ws_a, emp_a) = create_workspace_with_employee(&factory).await;
    let (ws_b, _emp_b) = create_workspace_with_employee(&factory).await;

    // Insert a time entry WITHOUT punch_out (active) for emp_a in ws_a
    let now = Utc::now();
    factory
        .time_entry()
        .with_employee(emp_a)
        .with_punch_in(now - Duration::hours(1))
        .create()
        .await;

    let repo = Arc::new(DieselTimeEntryRepository::new(
        pool.clone(),
        diesel_pools.clone(),
    ));

    // Correct workspace → should find the active entry
    let active_a = repo.get_latest_active_entry(ws_a, emp_a).await.unwrap();
    assert!(
        active_a.is_some(),
        "Active entry should exist in correct workspace"
    );

    // Wrong workspace → should NOT find it
    let active_b = repo.get_latest_active_entry(ws_b, emp_a).await.unwrap();
    assert!(
        active_b.is_none(),
        "Active entry should NOT be visible from wrong workspace"
    );
}

#[tokio::test]
async fn test_get_workspace_time_entries_batch_isolated() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let (ws_a, emp_a) = create_workspace_with_employee(&factory).await;
    let (ws_b, _emp_b) = create_workspace_with_employee(&factory).await;

    let now = Utc::now();
    factory
        .time_entry()
        .with_employee(emp_a)
        .with_punch_in(now - Duration::hours(3))
        .with_punch_out(now - Duration::hours(1))
        .create()
        .await;

    let start = now - Duration::hours(24);
    let end = now + Duration::hours(1);

    // Batch query with correct workspace
    let batch_a =
        time_entry_repository::get_workspace_time_entries_batch(&pool, ws_a, &[emp_a], start, end)
            .await
            .unwrap();
    assert_eq!(batch_a.get(&emp_a).map_or(0, std::vec::Vec::len), 1);

    // Batch query with wrong workspace
    let batch_b =
        time_entry_repository::get_workspace_time_entries_batch(&pool, ws_b, &[emp_a], start, end)
            .await
            .unwrap();
    assert_eq!(
        batch_b.get(&emp_a).map_or(0, std::vec::Vec::len),
        0,
        "Batch query should return 0 entries for wrong workspace"
    );
}

// ─── Department Repository Isolation ────────────────────────────────────

#[tokio::test]
async fn test_department_history_isolated_by_workspace() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let (ws_a, emp_a) = create_workspace_with_employee(&factory).await;
    let (ws_b, _emp_b) = create_workspace_with_employee(&factory).await;

    // Create a department in ws_a and transfer emp_a to it
    let dept_id = factory.department().with_workspace(ws_a).create().await;
    department_repository::transfer_employee(
        &pool,
        emp_a,
        dept_id,
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        None,
    )
    .await
    .unwrap();

    // Correct workspace → should find 1 history record
    let history_a = department_repository::get_employee_department_history(&pool, ws_a, emp_a)
        .await
        .unwrap();
    assert_eq!(
        history_a.len(),
        1,
        "Department history should exist in correct workspace"
    );

    // Wrong workspace → should find 0
    let history_b = department_repository::get_employee_department_history(&pool, ws_b, emp_a)
        .await
        .unwrap();
    assert_eq!(
        history_b.len(),
        0,
        "Department history should NOT be visible from wrong workspace"
    );
}

#[tokio::test]
async fn test_get_current_department_isolated_by_workspace() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let (ws_a, emp_a) = create_workspace_with_employee(&factory).await;
    let (ws_b, _emp_b) = create_workspace_with_employee(&factory).await;

    let dept_id = factory.department().with_workspace(ws_a).create().await;
    department_repository::transfer_employee(
        &pool,
        emp_a,
        dept_id,
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        None,
    )
    .await
    .unwrap();

    // Correct workspace
    let current_a = department_repository::get_current_department(&pool, ws_a, emp_a)
        .await
        .unwrap();
    assert!(
        current_a.is_some(),
        "Current department should exist in correct workspace"
    );

    // Wrong workspace
    let current_b = department_repository::get_current_department(&pool, ws_b, emp_a)
        .await
        .unwrap();
    assert!(
        current_b.is_none(),
        "Current department should NOT be visible from wrong workspace"
    );
}

// ─── Working Pattern Repository Isolation ───────────────────────────────

#[tokio::test]
async fn test_employee_pattern_history_isolated_by_workspace() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let (ws_a, emp_a) = create_workspace_with_employee(&factory).await;
    let (ws_b, _emp_b) = create_workspace_with_employee(&factory).await;

    // Create a working pattern and assign to emp_a in ws_a
    let pattern_id = sqlx::query_scalar(
        "INSERT INTO working_patterns (workspace_id, name) VALUES ($1, $2) RETURNING id",
    )
    .bind(ws_a)
    .bind("Day Shift Pattern")
    .fetch_one(&pool)
    .await
    .unwrap();

    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    working_pattern_repository::assign_pattern(&pool, ws_a, emp_a, pattern_id, start_date, None)
        .await
        .unwrap();

    // Correct workspace → should find 1
    let history_a = working_pattern_repository::get_employee_pattern_history(&pool, ws_a, emp_a)
        .await
        .unwrap();
    assert_eq!(
        history_a.len(),
        1,
        "Pattern history should exist in correct workspace"
    );

    // Wrong workspace → should find 0
    let history_b = working_pattern_repository::get_employee_pattern_history(&pool, ws_b, emp_a)
        .await
        .unwrap();
    assert_eq!(
        history_b.len(),
        0,
        "Pattern history should NOT be visible from wrong workspace"
    );
}
