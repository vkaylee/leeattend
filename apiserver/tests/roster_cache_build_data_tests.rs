mod common;

use apiserver::cache::build_roster_data;
use apiserver::repositories::{
    department_repository, shift_repository, working_pattern_repository,
};
use chrono::NaiveDate;
use common::db::TestDatabase;
use common::factory::TestFactory;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn setup() -> (TestDatabase, TestFactory, Uuid) {
    let db = TestDatabase::new().await;
    let factory = TestFactory::new(db.pool());
    let ws_id = factory.workspace().create().await;
    (db, factory, ws_id)
}

fn jan1() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Empty workspace -> every map in RosterData is empty.
#[tokio::test]
async fn test_empty_workspace_all_maps_empty() {
    let (db, _factory, ws_id) = setup().await;
    let data = build_roster_data(&db.pool(), ws_id).await.unwrap();

    assert!(data.shifts.is_empty());
    assert!(data.fixed_assignments.is_empty());
    assert!(data.pattern_assignments.is_empty());
    assert!(data.dept_patterns.is_empty());
    assert!(data.dept_fixed_shifts.is_empty());
    assert!(data.active_rules.is_empty());
    assert!(data.rule_patterns.is_empty());
    assert!(data.dept_history.is_empty());
}

/// Shifts are fetched and keyed by shift_id.
#[tokio::test]
async fn test_shifts_populated() {
    let (db, factory, ws_id) = setup().await;
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Morning")
        .create()
        .await;

    let data = build_roster_data(&db.pool(), ws_id).await.unwrap();

    assert_eq!(data.shifts.len(), 1);
    let shift = data.shifts.get(&shift_id).unwrap();
    assert_eq!(shift.name, "Morning");
}

/// Fixed assignments are grouped by employee_id.
#[tokio::test]
async fn test_fixed_assignments_grouped_by_employee() {
    let (db, factory, ws_id) = setup().await;
    let emp_a = factory.employee().with_workspace(ws_id).create().await;
    let emp_b = factory.employee().with_workspace(ws_id).create().await;

    let shift_a = factory
        .shift()
        .with_workspace(ws_id)
        .assign_to_employee(emp_a, jan1())
        .await;
    let shift_b = factory
        .shift()
        .with_workspace(ws_id)
        .assign_to_employee(emp_b, jan1())
        .await;

    let data = build_roster_data(&db.pool(), ws_id).await.unwrap();

    assert_eq!(data.fixed_assignments.len(), 2);
    assert_eq!(data.fixed_assignments[&emp_a][0].shift_id, shift_a);
    assert_eq!(data.fixed_assignments[&emp_b][0].shift_id, shift_b);
}

/// Pattern assignments include day_of_week details from the JOIN.
#[tokio::test]
async fn test_pattern_assignments_with_details() {
    let (db, factory, ws_id) = setup().await;
    let emp = factory.employee().with_workspace(ws_id).create().await;
    let shift_id = factory.shift().with_workspace(ws_id).create().await;

    let pool = db.pool();
    let pat =
        working_pattern_repository::create_pattern(&pool, ws_id, "Weekly", None)
            .await
            .unwrap();
    working_pattern_repository::add_pattern_detail(&pool, pat.id, 1, Some(shift_id))
        .await
        .unwrap();
    working_pattern_repository::assign_pattern(&pool, ws_id, emp, pat.id, jan1(), None)
        .await
        .unwrap();

    let data = build_roster_data(&pool, ws_id).await.unwrap();

    let entries = data.pattern_assignments.get(&emp).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].pattern_id, pat.id);
    assert_eq!(entries[0].day_of_week, 1);
    assert_eq!(entries[0].shift_id, Some(shift_id));
}

/// dept_patterns populated when department has a default working pattern.
#[tokio::test]
async fn test_dept_patterns_via_default_working_pattern() {
    let (db, factory, ws_id) = setup().await;
    let emp = factory.employee().with_workspace(ws_id).create().await;
    let shift_id = factory.shift().with_workspace(ws_id).create().await;

    let pool = db.pool();
    let pat =
        working_pattern_repository::create_pattern(&pool, ws_id, "DPat", None)
            .await
            .unwrap();
    working_pattern_repository::add_pattern_detail(&pool, pat.id, 1, Some(shift_id))
        .await
        .unwrap();

    let dept = department_repository::create_department(
        &pool,
        ws_id,
        "Engineering".to_string(),
        None,
        None,
        Some(pat.id),
    )
    .await
    .unwrap();
    department_repository::transfer_employee(&pool, emp, dept.id, jan1(), None)
        .await
        .unwrap();

    let data = build_roster_data(&pool, ws_id).await.unwrap();

    let entries = data.dept_patterns.get(&emp).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].shift_id, Some(shift_id));
}

/// dept_fixed_shifts populated when department has a row in department_default_shifts.
#[tokio::test]
async fn test_dept_fixed_shifts_via_department_default_shifts() {
    let (db, factory, ws_id) = setup().await;
    let emp = factory.employee().with_workspace(ws_id).create().await;
    let shift_id = factory.shift().with_workspace(ws_id).create().await;

    let pool = db.pool();
    let dept = department_repository::create_department(
        &pool,
        ws_id,
        "HR".to_string(),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    department_repository::transfer_employee(&pool, emp, dept.id, jan1(), None)
        .await
        .unwrap();

    sqlx::query!(
        "INSERT INTO department_default_shifts (department_id, shift_id) VALUES ($1, $2)",
        dept.id,
        shift_id,
    )
    .execute(&pool)
    .await
    .unwrap();

    let data = build_roster_data(&pool, ws_id).await.unwrap();

    let entries = data.dept_fixed_shifts.get(&emp).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].shift_id, shift_id);
}

/// dept_history records department transfers.
#[tokio::test]
async fn test_dept_history_populated() {
    let (db, factory, ws_id) = setup().await;
    let emp = factory.employee().with_workspace(ws_id).create().await;

    let pool = db.pool();
    let dept = department_repository::create_department(
        &pool,
        ws_id,
        "Sales".to_string(),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    department_repository::transfer_employee(&pool, emp, dept.id, jan1(), None)
        .await
        .unwrap();

    let data = build_roster_data(&pool, ws_id).await.unwrap();

    let history = data.dept_history.get(&emp).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].department_id, dept.id);
}

/// active_rules and rule_patterns are fetched for active assignment rules.
#[tokio::test]
async fn test_active_rules_and_rule_patterns() {
    let (db, factory, ws_id) = setup().await;
    let shift_id = factory.shift().with_workspace(ws_id).create().await;

    let pool = db.pool();
    let pat =
        working_pattern_repository::create_pattern(&pool, ws_id, "RulePat", None)
            .await
            .unwrap();
    working_pattern_repository::add_pattern_detail(&pool, pat.id, 1, Some(shift_id))
        .await
        .unwrap();
    working_pattern_repository::add_pattern_detail(&pool, pat.id, 0, None)
        .await
        .unwrap();

    let rule_id = factory
        .assignment_rule()
        .with_workspace(ws_id)
        .with_pattern(pat.id)
        .with_name("Mon Rule")
        .create()
        .await;

    let data = build_roster_data(&pool, ws_id).await.unwrap();

    assert_eq!(data.active_rules.len(), 1);
    assert_eq!(data.active_rules[0].id, rule_id);
    assert_eq!(data.rule_patterns[&(pat.id, 1)], Some(shift_id));
    assert!(data.rule_patterns[&(pat.id, 0)].is_none());
}

/// Data from workspace B must not appear when querying workspace A.
#[tokio::test]
async fn test_workspace_isolation() {
    let (db, factory, ws_a) = setup().await;
    let ws_b = factory.workspace().create().await;

    // Seed a shift only in workspace B
    factory
        .shift()
        .with_workspace(ws_b)
        .with_name("B Only")
        .create()
        .await;

    let data = build_roster_data(&db.pool(), ws_a).await.unwrap();
    assert!(data.shifts.is_empty(), "Workspace A must not see B's shifts");
}

/// Two fixed-shift assignments to the same employee are ordered by created_at DESC.
#[tokio::test]
async fn test_fixed_assignment_order_by_created_at_desc() {
    let (db, factory, ws_id) = setup().await;
    let emp = factory.employee().with_workspace(ws_id).create().await;

    let pool = db.pool();
    let shift_old = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Old")
        .create()
        .await;
    let shift_new = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("New")
        .create()
        .await;

    // Insert older assignment first
    shift_repository::assign_employee_shift(
        &pool,
        emp,
        shift_old,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        None,
    )
    .await
    .unwrap();

    // Small delay to guarantee created_at ordering
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    shift_repository::assign_employee_shift(
        &pool,
        emp,
        shift_new,
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        None,
    )
    .await
    .unwrap();

    let data = build_roster_data(&pool, ws_id).await.unwrap();
    let assignments = &data.fixed_assignments[&emp];
    assert_eq!(assignments.len(), 2);
    // ORDER BY created_at DESC -> newest first
    assert_eq!(assignments[0].shift_id, shift_new);
    assert_eq!(assignments[1].shift_id, shift_old);
}
