//! Workspace isolation integration tests.
//! Verifies that repository queries with `workspace_id` correctly scope data,
//! preventing cross-workspace data leaks.

#![allow(clippy::unwrap_used)]
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::diesel::employee::EmployeeEntity;
use apiserver::repositories::{
    department_repository::{self, DepartmentRepository},
    time_entry_repository, working_pattern_repository, DieselTimeEntryRepository,
    TimeEntryRepository,
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
    let pool = test_db.diesel_pools.clone();
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

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_a
    );

    // Query with correct workspace → should find 1 entry
    let repo = apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone());
    let entries_a = repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_a),
            employment_id,
            start,
            end,
        )
        .await
        .unwrap();
    assert_eq!(
        entries_a.len(),
        1,
        "Should find 1 entry in the correct workspace"
    );

    // Query with wrong workspace → should find 0 entries
    let entries_b = repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_b),
            employment_id,
            start,
            end,
        )
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
    let pool = test_db.diesel_pools.clone();
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

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_a
    );

    let repo = Arc::new(DieselTimeEntryRepository::new(diesel_pools.clone()));

    // Correct workspace → should find the active entry
    let active_a = repo
        .get_latest_active_entry(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_a),
            employment_id,
        )
        .await
        .unwrap();
    assert!(
        active_a.is_some(),
        "Active entry should exist in correct workspace"
    );

    // Wrong workspace → should NOT find it
    let active_b = repo
        .get_latest_active_entry(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_b),
            employment_id,
        )
        .await
        .unwrap();
    assert!(
        active_b.is_none(),
        "Active entry should NOT be visible from wrong workspace"
    );
}

#[tokio::test]
async fn test_get_workspace_time_entries_batch_isolated() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
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

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_a
    );

    // Batch query with correct workspace
    let repo = apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone());
    let batch_a = repo
        .get_workspace_time_entries_batch_by_employments(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_a),
            &[employment_id],
            start,
            end,
        )
        .await
        .unwrap();
    assert_eq!(batch_a.get(&employment_id).map_or(0, std::vec::Vec::len), 1);

    // Batch query with wrong workspace
    let batch_b = repo
        .get_workspace_time_entries_batch_by_employments(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_b),
            &[employment_id],
            start,
            end,
        )
        .await
        .unwrap();
    assert_eq!(
        batch_b.get(&employment_id).map_or(0, std::vec::Vec::len),
        0,
        "Batch query should return 0 entries for wrong workspace"
    );
}

// ─── Department Repository Isolation ────────────────────────────────────

#[tokio::test]
async fn test_department_history_isolated_by_workspace() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let (ws_a, emp_a) = create_workspace_with_employee(&factory).await;
    let (ws_b, _emp_b) = create_workspace_with_employee(&factory).await;

    // Create a department in ws_a and transfer emp_a to it
    let dept_id = factory.department().with_workspace(ws_a).create().await;
    let dept_repo = apiserver::repositories::DieselDepartmentRepository::new(diesel_pools.clone());
    dept_repo
        .transfer_employee(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_a),
            emp_a,
            dept_id,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            None,
        )
        .await
        .unwrap();

    // Correct workspace → should find 1 history record
    let history_a = dept_repo
        .get_employee_department_history(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_a),
            emp_a,
        )
        .await
        .unwrap();
    assert_eq!(
        history_a.len(),
        1,
        "Department history should exist in correct workspace"
    );

    // Wrong workspace → should find 0
    let history_b = dept_repo
        .get_employee_department_history(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_b),
            emp_a,
        )
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
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let (ws_a, emp_a) = create_workspace_with_employee(&factory).await;
    let (ws_b, _emp_b) = create_workspace_with_employee(&factory).await;

    let dept_id = factory.department().with_workspace(ws_a).create().await;
    let dept_repo = apiserver::repositories::DieselDepartmentRepository::new(diesel_pools.clone());
    dept_repo
        .transfer_employee(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_a),
            emp_a,
            dept_id,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            None,
        )
        .await
        .unwrap();

    // Correct workspace
    let current_a = dept_repo
        .get_current_department(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_a),
            emp_a,
        )
        .await
        .unwrap();
    assert!(
        current_a.is_some(),
        "Current department should exist in correct workspace"
    );

    // Wrong workspace
    let current_b = dept_repo
        .get_current_department(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_b),
            emp_a,
        )
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
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let (ws_a, emp_a) = create_workspace_with_employee(&factory).await;
    let (ws_b, _emp_b) = create_workspace_with_employee(&factory).await;

    // Create a working pattern and assign to emp_a in ws_a
    let pattern_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "INSERT INTO working_patterns (workspace_id, name) VALUES ($1, $2) RETURNING id",
        diesel::sql_types::Uuid,
        ws_a,
        diesel::sql_types::Text,
        "Day Shift Pattern"
    );

    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let repo = apiserver::repositories::DieselWorkingPatternRepository::new(diesel_pools.clone());
    use apiserver::repositories::working_pattern_repository::WorkingPatternRepositoryTrait;
    repo.assign_pattern(ws_a, emp_a, pattern_id, start_date, None, None)
        .await
        .unwrap();

    // Correct workspace → should find 1
    let history_a = repo
        .get_employee_pattern_history(ws_a, emp_a)
        .await
        .unwrap();
    assert_eq!(
        history_a.len(),
        1,
        "Pattern history should exist in correct workspace"
    );

    // Wrong workspace → should find 0
    let history_b = repo
        .get_employee_pattern_history(ws_b, emp_a)
        .await
        .unwrap();
    assert_eq!(
        history_b.len(),
        0,
        "Pattern history should NOT be visible from wrong workspace"
    );
}
