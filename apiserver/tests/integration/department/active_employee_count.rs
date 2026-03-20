//! Tests that Department queries return correct active_employee_count.

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::department_repository;
use chrono::NaiveDate;
use uuid::Uuid;

async fn setup_db() -> (TestDatabase, TestFactory) {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.pool());
    (test_db, factory)
}

/// Helper: assign an employee to a department via employee_department_history.
async fn transfer(pool: &sqlx::PgPool, employee_id: Uuid, department_id: Uuid, date: NaiveDate) {
    sqlx::query(
        "INSERT INTO employee_department_history (employee_id, department_id, effective_date) \
         VALUES ($1, $2, $3)",
    )
    .bind(employee_id)
    .bind(department_id)
    .bind(date)
    .execute(pool)
    .await
    .unwrap();
}

/// Helper: deactivate an employee.
async fn deactivate_employee(pool: &sqlx::PgPool, employee_id: Uuid) {
    sqlx::query("UPDATE employees SET is_active = false WHERE id = $1")
        .bind(employee_id)
        .execute(pool)
        .await
        .unwrap();
}

// ── get_departments ─────────────────────────────────────────────────

#[tokio::test]
async fn test_department_with_no_employees_has_zero_count() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let _dept_id = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Empty Dept")
        .create()
        .await;

    let depts = department_repository::get_departments(&pool, ws_id)
        .await
        .unwrap();
    assert_eq!(depts.len(), 1);
    assert_eq!(
        depts[0].active_employee_count, 0,
        "Department with no employees should have count = 0"
    );
}

#[tokio::test]
async fn test_department_counts_active_employees() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let dept_id = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Engineering")
        .create()
        .await;

    // Create 3 employees and assign them to the department
    let today = chrono::Utc::now().date_naive();
    for _ in 0..3 {
        let emp_id = factory.employee().with_workspace(ws_id).create().await;
        transfer(&pool, emp_id, dept_id, today).await;
    }

    let depts = department_repository::get_departments(&pool, ws_id)
        .await
        .unwrap();
    assert_eq!(depts.len(), 1);
    assert_eq!(
        depts[0].active_employee_count, 3,
        "Should count 3 active employees"
    );
}

#[tokio::test]
async fn test_department_excludes_inactive_employees() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let dept_id = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Sales")
        .create()
        .await;

    let today = chrono::Utc::now().date_naive();

    // 2 active employees
    let emp1 = factory.employee().with_workspace(ws_id).create().await;
    let emp2 = factory.employee().with_workspace(ws_id).create().await;
    // 1 inactive employee
    let emp3 = factory.employee().with_workspace(ws_id).create().await;

    transfer(&pool, emp1, dept_id, today).await;
    transfer(&pool, emp2, dept_id, today).await;
    transfer(&pool, emp3, dept_id, today).await;

    deactivate_employee(&pool, emp3).await;

    let depts = department_repository::get_departments(&pool, ws_id)
        .await
        .unwrap();
    assert_eq!(depts.len(), 1);
    assert_eq!(
        depts[0].active_employee_count, 2,
        "Should only count active employees (excluding the deactivated one)"
    );
}

#[tokio::test]
async fn test_department_uses_latest_transfer_only() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let dept_a = factory
        .department()
        .with_workspace(ws_id)
        .with_name("DeptA")
        .create()
        .await;
    let dept_b = factory
        .department()
        .with_workspace(ws_id)
        .with_name("DeptB")
        .create()
        .await;

    // Employee transferred from DeptA to DeptB
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    transfer(
        &pool,
        emp_id,
        dept_a,
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
    )
    .await;
    transfer(
        &pool,
        emp_id,
        dept_b,
        NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
    )
    .await;

    let depts = department_repository::get_departments(&pool, ws_id)
        .await
        .unwrap();

    let dept_a_row = depts.iter().find(|d| d.id == dept_a).unwrap();
    let dept_b_row = depts.iter().find(|d| d.id == dept_b).unwrap();

    assert_eq!(
        dept_a_row.active_employee_count, 0,
        "DeptA should have 0 — employee was transferred out"
    );
    assert_eq!(
        dept_b_row.active_employee_count, 1,
        "DeptB should have 1 — employee's latest assignment"
    );
}

// ── get_department_by_id ────────────────────────────────────────────

#[tokio::test]
async fn test_get_department_by_id_includes_employee_count() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let dept_id = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Ops")
        .create()
        .await;

    let today = chrono::Utc::now().date_naive();
    let emp1 = factory.employee().with_workspace(ws_id).create().await;
    let emp2 = factory.employee().with_workspace(ws_id).create().await;
    transfer(&pool, emp1, dept_id, today).await;
    transfer(&pool, emp2, dept_id, today).await;

    let dept = department_repository::get_department_by_id(&pool, ws_id, dept_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        dept.active_employee_count, 2,
        "get_department_by_id should also return active_employee_count"
    );
}

// ── create/update return default 0 ──────────────────────────────────

#[tokio::test]
async fn test_create_department_returns_zero_count() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let dept = department_repository::create_department(
        &pool,
        ws_id,
        "New Dept".to_string(),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    assert_eq!(
        dept.active_employee_count, 0,
        "Newly created department should have count = 0 (via sqlx default)"
    );
}

#[tokio::test]
async fn test_multiple_departments_independent_counts() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let dept_eng = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Engineering")
        .create()
        .await;
    let dept_hr = factory
        .department()
        .with_workspace(ws_id)
        .with_name("HR")
        .create()
        .await;
    let dept_empty = factory
        .department()
        .with_workspace(ws_id)
        .with_name("ZZZ Empty")
        .create()
        .await;

    let today = chrono::Utc::now().date_naive();

    // 3 employees in Engineering
    for _ in 0..3 {
        let emp = factory.employee().with_workspace(ws_id).create().await;
        transfer(&pool, emp, dept_eng, today).await;
    }
    // 1 employee in HR
    let hr_emp = factory.employee().with_workspace(ws_id).create().await;
    transfer(&pool, hr_emp, dept_hr, today).await;
    // Empty department has no employees

    let depts = department_repository::get_departments(&pool, ws_id)
        .await
        .unwrap();

    let eng = depts.iter().find(|d| d.id == dept_eng).unwrap();
    let hr = depts.iter().find(|d| d.id == dept_hr).unwrap();
    let empty = depts.iter().find(|d| d.id == dept_empty).unwrap();

    assert_eq!(eng.active_employee_count, 3);
    assert_eq!(hr.active_employee_count, 1);
    assert_eq!(empty.active_employee_count, 0);
}
