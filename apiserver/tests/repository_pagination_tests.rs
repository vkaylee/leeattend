mod common;

use apiserver::repositories::{employee_repository, time_entry_repository};
use chrono::{Duration, Utc};
use common::db::TestDatabase;
use common::factory::TestFactory;

#[tokio::test]
async fn test_get_paginated_employee_ids_basic() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create test employees with predictable names for sorting
    let _emp1 = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Alice Anderson")
        .create()
        .await;
    let _emp2 = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Bob Brown")
        .create()
        .await;
    let _emp3 = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Charlie Chen")
        .create()
        .await;

    // Test basic pagination - page 1, limit 2, ascending
    let (employees, total) = employee_repository::get_paginated_employee_ids(
        &pool,
        ws_id,
        None,
        1,
        2,
        false,
    )
    .await
    .unwrap();

    assert_eq!(total, 3, "Should count 3 total active employees");
    assert_eq!(employees.len(), 2, "Should return 2 employees for limit=2");
    assert_eq!(employees[0].full_name, "Alice Anderson", "First should be Alice (ASC)");
    assert_eq!(employees[1].full_name, "Bob Brown", "Second should be Bob (ASC)");

    // Test page 2 - should get remaining employee
    let (employees, total) = employee_repository::get_paginated_employee_ids(
        &pool,
        ws_id,
        None,
        2,
        2,
        false,
    )
    .await
    .unwrap();

    assert_eq!(total, 3, "Total should still be 3");
    assert_eq!(employees.len(), 1, "Page 2 should have 1 employee");
    assert_eq!(employees[0].full_name, "Charlie Chen", "Should be Charlie");
}

#[tokio::test]
async fn test_get_paginated_employee_ids_sort_desc() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Alice Anderson")
        .create()
        .await;
    factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Bob Brown")
        .create()
        .await;
    factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Charlie Chen")
        .create()
        .await;

    // Test descending order
    let (employees, total) = employee_repository::get_paginated_employee_ids(
        &pool,
        ws_id,
        None,
        1,
        2,
        true,
    )
    .await
    .unwrap();

    assert_eq!(total, 3, "Should count 3 total employees");
    assert_eq!(employees.len(), 2, "Should return 2 employees");
    assert_eq!(employees[0].full_name, "Charlie Chen", "First should be Charlie (DESC)");
    assert_eq!(employees[1].full_name, "Bob Brown", "Second should be Bob (DESC)");
}

#[tokio::test]
async fn test_get_paginated_employee_ids_with_filter() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let emp1 = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Alice")
        .create()
        .await;
    let emp2 = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Bob")
        .create()
        .await;
    let _emp3 = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Charlie")
        .create()
        .await;

    // Filter to only emp1 and emp2
    let filter_ids = vec![emp1, emp2];
    let (employees, total) = employee_repository::get_paginated_employee_ids(
        &pool,
        ws_id,
        Some(&filter_ids),
        1,
        10,
        false,
    )
    .await
    .unwrap();

    assert_eq!(total, 2, "Should count only filtered employees");
    assert_eq!(employees.len(), 2, "Should return only filtered employees");

    let returned_ids: Vec<_> = employees.iter().map(|e| e.id).collect();
    assert!(returned_ids.contains(&emp1), "Should include emp1");
    assert!(returned_ids.contains(&emp2), "Should include emp2");
}

#[tokio::test]
async fn test_get_paginated_employee_ids_inactive_excluded() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create active employee
    factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Active Employee")
        .create()
        .await;

    // Create inactive employee
    let inactive_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Inactive Employee")
        .create()
        .await;

    // Mark as inactive
    sqlx::query("UPDATE employees SET is_active = false WHERE id = $1")
        .bind(inactive_id)
        .execute(&pool)
        .await
        .unwrap();

    // Query should only return active employees
    let (employees, total) = employee_repository::get_paginated_employee_ids(
        &pool,
        ws_id,
        None,
        1,
        10,
        false,
    )
    .await
    .unwrap();

    assert_eq!(total, 1, "Should only count active employees");
    assert_eq!(employees.len(), 1, "Should only return active employees");
    assert_eq!(employees[0].full_name, "Active Employee");
}

#[tokio::test]
async fn test_get_paginated_employee_ids_empty_result() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // No employees created
    let (employees, total) = employee_repository::get_paginated_employee_ids(
        &pool,
        ws_id,
        None,
        1,
        10,
        false,
    )
    .await
    .unwrap();

    assert_eq!(total, 0, "Should count 0 employees");
    assert_eq!(employees.len(), 0, "Should return empty list");
}

#[tokio::test]
async fn test_get_workspace_time_entries_batch_basic() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let emp1 = factory.employee().with_workspace(ws_id).create().await;
    let emp2 = factory.employee().with_workspace(ws_id).create().await;

    let now = Utc::now();
    let from = now - Duration::days(1);
    let to = now + Duration::days(1);

    // Create time entries for emp1
    factory
        .time_entry()
        .with_employee(emp1)
        .with_punch_in(now - Duration::hours(8))
        .with_punch_out(now - Duration::hours(4))
        .create()
        .await;
    factory
        .time_entry()
        .with_employee(emp1)
        .with_punch_in(now - Duration::hours(2))
        .with_punch_out(now)
        .create()
        .await;

    // Create time entry for emp2
    factory
        .time_entry()
        .with_employee(emp2)
        .with_punch_in(now - Duration::hours(6))
        .with_punch_out(now - Duration::hours(3))
        .create()
        .await;

    // Batch fetch
    let employee_ids = vec![emp1, emp2];
    let result = time_entry_repository::get_workspace_time_entries_batch(
        &pool,
        &employee_ids,
        from,
        to,
    )
    .await
    .unwrap();

    assert_eq!(result.len(), 2, "Should have entries for 2 employees");
    assert_eq!(result.get(&emp1).unwrap().len(), 2, "emp1 should have 2 entries");
    assert_eq!(result.get(&emp2).unwrap().len(), 1, "emp2 should have 1 entry");
}

#[tokio::test]
async fn test_get_workspace_time_entries_batch_empty_input() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();

    let now = Utc::now();
    let from = now - Duration::days(1);
    let to = now + Duration::days(1);

    // Empty employee_ids should return empty map
    let result = time_entry_repository::get_workspace_time_entries_batch(
        &pool,
        &[],
        from,
        to,
    )
    .await
    .unwrap();

    assert_eq!(result.len(), 0, "Should return empty map for empty input");
}

#[tokio::test]
async fn test_get_workspace_time_entries_batch_date_filtering() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let now = Utc::now();

    // Entry within range
    factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(now - Duration::hours(2))
        .with_punch_out(now)
        .create()
        .await;

    // Entry outside range (too old)
    factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(now - Duration::days(10))
        .with_punch_out(now - Duration::days(10) + Duration::hours(1))
        .create()
        .await;

    // Entry outside range (future)
    factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(now + Duration::days(10))
        .with_punch_out(now + Duration::days(10) + Duration::hours(1))
        .create()
        .await;

    // Batch fetch with specific range
    let from = now - Duration::days(1);
    let to = now + Duration::days(1);
    let result = time_entry_repository::get_workspace_time_entries_batch(
        &pool,
        &[emp_id],
        from,
        to,
    )
    .await
    .unwrap();

    assert_eq!(result.len(), 1, "Should have entries for 1 employee");
    assert_eq!(
        result.get(&emp_id).unwrap().len(),
        1,
        "Should only include entry within date range"
    );
}

#[tokio::test]
async fn test_get_workspace_time_entries_batch_grouping() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let emp1 = factory.employee().with_workspace(ws_id).create().await;
    let emp2 = factory.employee().with_workspace(ws_id).create().await;
    let emp3 = factory.employee().with_workspace(ws_id).create().await;

    let now = Utc::now();
    let from = now - Duration::days(1);
    let to = now + Duration::days(1);

    // Create multiple entries for each employee
    for emp in &[emp1, emp2, emp3] {
        for i in 0..3 {
            factory
                .time_entry()
                .with_employee(*emp)
                .with_punch_in(now - Duration::hours(6 - i))
                .with_punch_out(now - Duration::hours(4 - i))
                .create()
                .await;
        }
    }

    // Batch fetch
    let result = time_entry_repository::get_workspace_time_entries_batch(
        &pool,
        &[emp1, emp2, emp3],
        from,
        to,
    )
    .await
    .unwrap();

    // Verify grouping by employee_id
    assert_eq!(result.len(), 3, "Should have entries grouped for 3 employees");
    assert_eq!(result.get(&emp1).unwrap().len(), 3, "emp1 should have 3 entries");
    assert_eq!(result.get(&emp2).unwrap().len(), 3, "emp2 should have 3 entries");
    assert_eq!(result.get(&emp3).unwrap().len(), 3, "emp3 should have 3 entries");

    // Verify all entries for emp1 actually belong to emp1
    for entry in result.get(&emp1).unwrap() {
        assert_eq!(entry.employee_id, emp1, "Entry should belong to emp1");
    }
}

#[tokio::test]
async fn test_get_workspace_time_entries_batch_ordering() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let now = Utc::now();
    let from = now - Duration::days(1);
    let to = now + Duration::days(1);

    // Create entries in specific order
    let t1 = factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(now - Duration::hours(6))
        .with_punch_out(now - Duration::hours(5))
        .create()
        .await;

    let t2 = factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(now - Duration::hours(3))
        .with_punch_out(now - Duration::hours(2))
        .create()
        .await;

    let t3 = factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(now - Duration::hours(1))
        .with_punch_out(now)
        .create()
        .await;

    // Batch fetch
    let result = time_entry_repository::get_workspace_time_entries_batch(
        &pool,
        &[emp_id],
        from,
        to,
    )
    .await
    .unwrap();

    let entries = result.get(&emp_id).unwrap();
    assert_eq!(entries.len(), 3, "Should have 3 entries");

    // Verify DESC ordering (newest first)
    assert_eq!(entries[0].id, t3, "First should be most recent entry");
    assert_eq!(entries[1].id, t2, "Second should be middle entry");
    assert_eq!(entries[2].id, t1, "Third should be oldest entry");
}

#[tokio::test]
async fn test_get_workspace_time_entries_batch_no_entries_for_employee() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let emp1 = factory.employee().with_workspace(ws_id).create().await;
    let emp2 = factory.employee().with_workspace(ws_id).create().await;

    let now = Utc::now();
    let from = now - Duration::days(1);
    let to = now + Duration::days(1);

    // Only create entry for emp1
    factory
        .time_entry()
        .with_employee(emp1)
        .with_punch_in(now - Duration::hours(2))
        .with_punch_out(now)
        .create()
        .await;

    // Batch fetch including emp2 (who has no entries)
    let result = time_entry_repository::get_workspace_time_entries_batch(
        &pool,
        &[emp1, emp2],
        from,
        to,
    )
    .await
    .unwrap();

    // emp2 should not appear in the map (HashMap only contains keys with entries)
    assert_eq!(result.len(), 1, "Should only have entry for emp1");
    assert!(result.contains_key(&emp1), "Should contain emp1");
    assert!(!result.contains_key(&emp2), "Should NOT contain emp2 (no entries)");
}
