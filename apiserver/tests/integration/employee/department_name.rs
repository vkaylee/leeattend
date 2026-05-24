//! Tests that Employee queries return the current `department_name`.

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::{
    diesel_employee_repository::DieselEmployeeRepository, EmployeeRepository,
};
use chrono::NaiveDate;
use uuid::Uuid;

async fn setup_db() -> (TestDatabase, TestFactory) {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.diesel_pools.clone());
    (test_db, factory)
}

/// Helper: insert a row in `employee_department_history` to assign an employee
/// to a department.
async fn transfer(
    pool: &apiserver::diesel_pool::DieselPools,
    employee_id: Uuid,
    department_id: Uuid,
    date: NaiveDate,
) {
    crate::test_execute!(
        &pool.api,
        "INSERT INTO employee_department_history (employee_id, department_id, effective_date) \
         VALUES ($1, $2, $3)",
        diesel::sql_types::Uuid,
        employee_id,
        diesel::sql_types::Uuid,
        department_id,
        diesel::sql_types::Date,
        date
    );
}

// ── get_employee_by_id ──────────────────────────────────────────────

#[tokio::test]
async fn test_employee_without_department_has_null_department_name() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let emp = repo
        .get_employee_by_id(emp_id, &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id))
        .await
        .unwrap()
        .unwrap();
    assert!(
        emp.department_name.is_none(),
        "Employee with no department should have department_name = None"
    );
}

#[tokio::test]
async fn test_employee_with_department_returns_department_name() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let dept_id = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Engineering")
        .create()
        .await;

    let today = chrono::Utc::now().date_naive();
    transfer(&pool, emp_id, dept_id, today).await;

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let emp = repo
        .get_employee_by_id(emp_id, &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        emp.department_name.as_deref(),
        Some("Engineering"),
        "Should return the current department name"
    );
}

// ── get_all_employees (paginated list) ──────────────────────────────

#[tokio::test]
async fn test_get_all_employees_includes_department_name() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

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

    let dept_id = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Sales")
        .create()
        .await;

    let today = chrono::Utc::now().date_naive();
    transfer(&pool, emp1, dept_id, today).await;
    // emp2 has no department

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let (employees, total) = repo
        .get_all_employees(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            apiserver::models::EmployeeFilterOptions {
                search_query: None,
                page: 1,
                limit: 50,
                sort_by: None,
                sort_dir: None,
                department_ids: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(total, 2);

    let alice = employees
        .iter()
        .find(|e| e.id == emp1)
        .expect("Alice should be in the results");
    let bob = employees
        .iter()
        .find(|e| e.id == emp2)
        .expect("Bob should be in the results");

    assert_eq!(alice.department_name.as_deref(), Some("Sales"));
    assert!(bob.department_name.is_none());
}

// ── Latest department wins ──────────────────────────────────────────

#[tokio::test]
async fn test_department_name_uses_latest_transfer() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let dept_a = factory
        .department()
        .with_workspace(ws_id)
        .with_name("OldDept")
        .create()
        .await;
    let dept_b = factory
        .department()
        .with_workspace(ws_id)
        .with_name("NewDept")
        .create()
        .await;

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

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let emp = repo
        .get_employee_by_id(emp_id, &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        emp.department_name.as_deref(),
        Some("NewDept"),
        "Should use the most recent transfer (by effective_date DESC)"
    );
}

// ── find_by_number ──────────────────────────────────────────────────

#[tokio::test]
async fn test_find_by_number_includes_department_name() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let _emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("DN001")
        .create()
        .await;

    let dept_id = factory
        .department()
        .with_workspace(ws_id)
        .with_name("R&D")
        .create()
        .await;

    let today = chrono::Utc::now().date_naive();
    transfer(&pool, _emp_id, dept_id, today).await;

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let emp = repo.find_by_number(&apiserver::diesel_pool::TenantAuthToken::system_override(ws_id), "DN001").await.unwrap().unwrap();
    assert_eq!(emp.department_name.as_deref(), Some("R&D"));
}

// ── get_active_employees ────────────────────────────────────────────

#[tokio::test]
async fn test_get_active_employees_includes_department_name() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Active Dept Worker")
        .create()
        .await;

    let dept_id = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Operations")
        .create()
        .await;

    let today = chrono::Utc::now().date_naive();
    transfer(&pool, emp_id, dept_id, today).await;

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let actives = repo.get_active_employees(&apiserver::diesel_pool::TenantAuthToken::system_override(ws_id)).await.unwrap();
    assert_eq!(actives.len(), 1);
    assert_eq!(actives[0].department_name.as_deref(), Some("Operations"));
}
