//! Repository-level integration tests for split employee sub-modules.
//!
//! These tests exercise free functions in `employee::core_queries`,
//! `employee::recruitment_queries`, and `employee::analytics_queries`
//! that are NOT exposed through REST endpoints.

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{EmploymentPeriod, WorkingCondition};
use apiserver::repositories::employee;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

// Helper: set up a clean DB + factory (no axum router needed)
async fn setup_db() -> (TestDatabase, TestFactory) {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.pool());
    (test_db, factory)
}

// ── Working Conditions ──────────────────────────────────────────────

#[tokio::test]
async fn test_working_condition_crud() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("WC Employee")
        .create()
        .await;

    // Create a working condition
    let wc = WorkingCondition {
        id: Uuid::new_v4(),
        employee_id: emp_id,
        type_: "late_arrival".to_string(),
        minute_allowance: 15,
        is_paid: true,
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: None,
    };
    employee::create_working_condition(&pool, &wc)
        .await
        .unwrap();

    // Read back
    let conditions = employee::get_working_conditions(&pool, ws_id, emp_id)
        .await
        .unwrap();
    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0].type_, "late_arrival");
    assert_eq!(conditions[0].minute_allowance, 15);
    assert!(conditions[0].is_paid);

    // Delete
    let deleted = employee::delete_working_condition(&pool, wc.id, emp_id)
        .await
        .unwrap();
    assert!(deleted);

    // Confirm empty
    let conditions = employee::get_working_conditions(&pool, ws_id, emp_id)
        .await
        .unwrap();
    assert!(conditions.is_empty());
}

#[tokio::test]
async fn test_delete_nonexistent_working_condition_returns_false() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let deleted = employee::delete_working_condition(&pool, Uuid::new_v4(), emp_id)
        .await
        .unwrap();
    assert!(!deleted, "Should return false for non-existent condition");
}

// ── Employment History ──────────────────────────────────────────────

#[tokio::test]
async fn test_employment_history_lifecycle() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_hire_date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
        .create()
        .await;

    // Factory already inserts an initial employment period
    let history = employee::get_employment_history(&pool, ws_id, emp_id)
        .await
        .unwrap();
    assert_eq!(history.len(), 1, "Should have initial period from factory");
    assert_eq!(
        history[0].hire_date,
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
    );

    // Add a rehire period
    let rehire_period = EmploymentPeriod {
        id: Uuid::new_v4(),
        employee_id: emp_id,
        hire_date: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        termination_date: None,
        is_rehire: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    employee::create_employment_period(&pool, &rehire_period)
        .await
        .unwrap();

    let history = employee::get_employment_history(&pool, ws_id, emp_id)
        .await
        .unwrap();
    assert_eq!(history.len(), 2, "Should have original + rehire period");
    // Ordered by hire_date DESC, so rehire is first
    assert!(history[0].is_rehire);
    assert_eq!(
        history[0].hire_date,
        NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()
    );
}

// ── Payroll Config ──────────────────────────────────────────────────

#[tokio::test]
async fn test_payroll_config_upsert_and_lookup() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let effective = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    // Insert
    let config = employee::upsert_employee_payroll_config(
        &pool,
        emp_id,
        Decimal::from(22),
        Decimal::from(8),
        effective,
    )
    .await
    .unwrap();
    assert_eq!(config.std_working_days, Decimal::from(22));
    assert_eq!(config.hours_per_day, Decimal::from(8));
    assert_eq!(config.effective_date, effective);

    // Upsert (same effective_date → update)
    let updated = employee::upsert_employee_payroll_config(
        &pool,
        emp_id,
        Decimal::from(20),
        Decimal::from(7),
        effective,
    )
    .await
    .unwrap();
    assert_eq!(updated.std_working_days, Decimal::from(20));
    assert_eq!(updated.hours_per_day, Decimal::from(7));

    // Lookup at exactly the effective date
    let found = employee::get_employee_payroll_config_at(&pool, emp_id, effective)
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().std_working_days, Decimal::from(20));

    // Lookup before effective date → should return None
    let before = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
    let not_found = employee::get_employee_payroll_config_at(&pool, emp_id, before)
        .await
        .unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_payroll_config_returns_closest_effective() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Insert two configs with different effective dates
    let jan = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let jul = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();

    employee::upsert_employee_payroll_config(
        &pool,
        emp_id,
        Decimal::from(22),
        Decimal::from(8),
        jan,
    )
    .await
    .unwrap();
    employee::upsert_employee_payroll_config(
        &pool,
        emp_id,
        Decimal::from(20),
        Decimal::from(7),
        jul,
    )
    .await
    .unwrap();

    // Lookup in March → should get Jan config
    let march = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
    let config = employee::get_employee_payroll_config_at(&pool, emp_id, march)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(config.std_working_days, Decimal::from(22));

    // Lookup in August → should get Jul config
    let august = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();
    let config = employee::get_employee_payroll_config_at(&pool, emp_id, august)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(config.std_working_days, Decimal::from(20));
}

// ── Active Employees & Search ───────────────────────────────────────

#[tokio::test]
async fn test_get_active_employees_excludes_inactive() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create 2 active employees
    let _e1 = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("ACTIVE01")
        .with_full_name("Active One")
        .create()
        .await;
    let e2 = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("INACTIVE02")
        .with_full_name("Inactive Two")
        .create()
        .await;

    // Deactivate one
    sqlx::query("UPDATE employees SET is_active = false WHERE id = $1")
        .bind(e2)
        .execute(&pool)
        .await
        .unwrap();

    let active = employee::get_active_employees(&pool, ws_id).await.unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].full_name, "Active One");
}

#[tokio::test]
async fn test_get_active_employee_ids_with_filter() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let e1 = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("FILT1")
        .create()
        .await;
    let _e2 = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("FILT2")
        .create()
        .await;
    let e3 = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("FILT3")
        .create()
        .await;

    // Request only e1 and e3
    let filter = vec![e1, e3];
    let ids = employee::get_active_employee_ids(&pool, ws_id, Some(&filter))
        .await
        .unwrap();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&e1));
    assert!(ids.contains(&e3));

    // Without filter → all 3
    let all_ids = employee::get_active_employee_ids(&pool, ws_id, None)
        .await
        .unwrap();
    assert_eq!(all_ids.len(), 3);
}

// ── Find by Number ──────────────────────────────────────────────────

#[tokio::test]
async fn test_find_by_number_normalizes_input() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let _emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("42")
        .with_full_name("Number Test")
        .create()
        .await;

    // Search with leading zeros → should still find due to normalization
    let found = employee::find_by_number(&pool, ws_id, "0042")
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().full_name, "Number Test");

    // Search non-existent
    let not_found = employee::find_by_number(&pool, ws_id, "999999")
        .await
        .unwrap();
    assert!(not_found.is_none());
}

// ── Existing Employee ID Check ──────────────────────────────────────

#[tokio::test]
async fn test_get_existing_employee_ids_by_numbers() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.pool();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("100")
        .create()
        .await;
    factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("200")
        .create()
        .await;

    let to_check = vec!["100".to_string(), "200".to_string(), "300".to_string()];
    let existing = employee::get_existing_employee_ids_by_numbers(&pool, ws_id, &to_check)
        .await
        .unwrap();
    assert_eq!(existing.len(), 2);
    assert!(existing.contains(&"100".to_string()));
    assert!(existing.contains(&"200".to_string()));

    // Empty input → empty result
    let empty_result = employee::get_existing_employee_ids_by_numbers(&pool, ws_id, &[])
        .await
        .unwrap();
    assert!(empty_result.is_empty());
}
