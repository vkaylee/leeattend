mod common;

use apiserver::models::AssignmentRuleConditions;
use apiserver::repositories::{
    department_repository, roster_repository, shift_repository, working_pattern_repository,
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

/// 2026-01-05 — confirmed Monday (dow=1)
fn monday() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, 5).unwrap()
}
/// 2026-01-11 — confirmed Sunday (dow=0)
fn sunday() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, 11).unwrap()
}
fn jan1() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
}

// ---------------------------------------------------------------------------
// Test 1 — P1 roster | P2 pattern (incl. NULL day-off) | no-assignment employee
//   Exercises: roster_map, pattern_assignments_map, source="none"
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_multi_source_and_pattern_day_off() {
    let (_db, factory, ws_id) = setup().await;
    let pool = factory.pool.clone();

    let shift_a = factory.shift().with_workspace(ws_id).with_name("A").create().await;
    let shift_b = factory.shift().with_workspace(ws_id).with_name("B").create().await;

    // emp1 — roster entry on Monday → P1
    let emp1 = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Roster")
        .create()
        .await;
    sqlx::query!(
        "INSERT INTO shift_rosters (id, employee_id, shift_id, date, status) VALUES ($1, $2, $3, $4, 'confirmed')",
        Uuid::new_v4(), emp1, shift_a, monday()
    )
    .execute(&pool)
    .await
    .unwrap();

    // emp2 — pattern: Mon=shift_b, Sun=NULL (day off) → P2
    let emp2 = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Pattern")
        .create()
        .await;
    let pat =
        working_pattern_repository::create_pattern(&pool, ws_id, "Pat", None).await.unwrap();
    working_pattern_repository::add_pattern_detail(&pool, pat.id, 1, Some(shift_b))
        .await
        .unwrap(); // Mon
    working_pattern_repository::add_pattern_detail(&pool, pat.id, 0, None)
        .await
        .unwrap(); // Sun = off
    working_pattern_repository::assign_pattern(&pool, ws_id, emp2, pat.id, jan1(), None)
        .await
        .unwrap();

    // emp3 — zero assignments
    let emp3 = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("NoAssign")
        .create()
        .await;

    // --- Monday ---
    let sched =
        roster_repository::get_workspace_schedules(&pool, ws_id, monday(), monday(), None)
            .await
            .unwrap();

    let e1 = sched.iter().find(|s| s.employee_id == emp1).unwrap();
    assert_eq!(e1.schedules[0].shift_id, Some(shift_a));
    assert_eq!(e1.schedules[0].source, "roster");

    let e2 = sched.iter().find(|s| s.employee_id == emp2).unwrap();
    assert_eq!(e2.schedules[0].shift_id, Some(shift_b));
    assert_eq!(e2.schedules[0].source, "pattern");

    let e3 = sched.iter().find(|s| s.employee_id == emp3).unwrap();
    assert!(e3.schedules[0].shift_id.is_none());
    assert_eq!(e3.schedules[0].source, "none");

    // --- Sunday: emp2 pattern day-off (NULL shift_id in pattern_assignments_map path) ---
    let sched_sun =
        roster_repository::get_workspace_schedules(&pool, ws_id, sunday(), sunday(), None)
            .await
            .unwrap();
    let e2_sun = sched_sun.iter().find(|s| s.employee_id == emp2).unwrap();
    assert!(
        e2_sun.schedules[0].shift_id.is_none(),
        "Sunday: pattern detail is NULL → day off"
    );
    assert_eq!(e2_sun.schedules[0].source, "pattern");
}

// ---------------------------------------------------------------------------
// Test 2 — P4 dept pattern | P5 fixed | P6 dept fixed shift
//   Exercises: dept_patterns_map, fixed_assignments_map, dept_fixed_shifts_map
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_dept_pattern_fixed_and_dept_fixed() {
    let (_db, factory, ws_id) = setup().await;
    let pool = factory.pool.clone();

    let shift_dp = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("DPat")
        .create()
        .await;
    let shift_fx = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Fix")
        .create()
        .await;
    let shift_df = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("DFix")
        .create()
        .await;

    // emp1 → P4: dept with default working pattern covering Monday
    let emp1 = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("DeptPat")
        .create()
        .await;
    let dp_pat =
        working_pattern_repository::create_pattern(&pool, ws_id, "DPPat", None).await.unwrap();
    working_pattern_repository::add_pattern_detail(&pool, dp_pat.id, 1, Some(shift_dp))
        .await
        .unwrap();
    let dept_pat = department_repository::create_department(
        &pool,
        ws_id,
        "IT".to_string(),
        None,
        None,
        Some(dp_pat.id),
    )
    .await
    .unwrap();
    department_repository::transfer_employee(&pool, emp1, dept_pat.id, jan1(), None)
        .await
        .unwrap();

    // emp2 → P5: individual fixed shift, no department
    let emp2 = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Fixed")
        .create()
        .await;
    shift_repository::assign_employee_shift(&pool, emp2, shift_fx, jan1(), None)
        .await
        .unwrap();

    // emp3 → P6: dept with default shift only (no pattern on dept)
    let emp3 = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("DeptFix")
        .create()
        .await;
    let dept_fix = department_repository::create_department(
        &pool,
        ws_id,
        "HR".to_string(),
        None,
        None,
        None, // no default pattern
    )
    .await
    .unwrap();
    department_repository::transfer_employee(&pool, emp3, dept_fix.id, jan1(), None)
        .await
        .unwrap();
    sqlx::query!(
        "INSERT INTO department_default_shifts (department_id, shift_id) VALUES ($1, $2)",
        dept_fix.id,
        shift_df
    )
    .execute(&pool)
    .await
    .unwrap();

    let sched =
        roster_repository::get_workspace_schedules(&pool, ws_id, monday(), monday(), None)
            .await
            .unwrap();

    let e1 = sched.iter().find(|s| s.employee_id == emp1).unwrap();
    assert_eq!(e1.schedules[0].shift_id, Some(shift_dp));
    assert_eq!(e1.schedules[0].source, "department_pattern");

    let e2 = sched.iter().find(|s| s.employee_id == emp2).unwrap();
    assert_eq!(e2.schedules[0].shift_id, Some(shift_fx));
    assert_eq!(e2.schedules[0].source, "fixed");

    let e3 = sched.iter().find(|s| s.employee_id == emp3).unwrap();
    assert_eq!(e3.schedules[0].shift_id, Some(shift_df));
    assert_eq!(e3.schedules[0].source, "department_fixed");
}

// ---------------------------------------------------------------------------
// Test 3 — P3 dynamic assignment rule
//   Exercises: rule_patterns_map (composite key), all_dept_history_map (dept
//   context passed to rule evaluator), and NULL-shift day-off via rule pattern.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_dynamic_rule_via_workspace_schedules() {
    let (_db, factory, ws_id) = setup().await;
    let pool = factory.pool.clone();

    let shift_rule = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("RuleShift")
        .create()
        .await;

    // emp: tagged "VIP", placed in a dept so all_dept_history_map is populated
    let emp = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("VIP")
        .create()
        .await;
    sqlx::query!(
        "UPDATE employees SET tags = $1 WHERE id = $2",
        &vec!["VIP".to_string()],
        emp
    )
    .execute(&pool)
    .await
    .unwrap();

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

    // pattern: Mon=shift_rule, Sun=NULL
    let pat =
        working_pattern_repository::create_pattern(&pool, ws_id, "RPat", None).await.unwrap();
    working_pattern_repository::add_pattern_detail(&pool, pat.id, 1, Some(shift_rule))
        .await
        .unwrap();
    working_pattern_repository::add_pattern_detail(&pool, pat.id, 0, None)
        .await
        .unwrap(); // Sun = off

    // rule: tag "VIP" → pattern (priority 100)
    factory
        .assignment_rule()
        .with_workspace(ws_id)
        .with_name("VIP Rule")
        .with_pattern(pat.id)
        .with_priority(100)
        .with_conditions(AssignmentRuleConditions {
            department_id: None,
            employee_id_numbers: None,
            tags: Some(vec!["VIP".to_string()]),
        })
        .create()
        .await;

    // Monday: rule matches, composite-key (pattern_id, 1) → shift_rule
    let sched =
        roster_repository::get_workspace_schedules(&pool, ws_id, monday(), monday(), None)
            .await
            .unwrap();
    let day = &sched.iter().find(|s| s.employee_id == emp).unwrap().schedules[0];
    assert_eq!(day.shift_id, Some(shift_rule));
    assert_eq!(day.source, "assignment_rule");

    // Sunday: rule matches, composite-key (pattern_id, 0) → None (day off)
    let sched_sun =
        roster_repository::get_workspace_schedules(&pool, ws_id, sunday(), sunday(), None)
            .await
            .unwrap();
    let day_sun = &sched_sun.iter().find(|s| s.employee_id == emp).unwrap().schedules[0];
    assert!(
        day_sun.shift_id.is_none(),
        "Sunday: rule pattern detail is NULL → day off"
    );
    assert_eq!(day_sun.source, "assignment_rule");

    // Wednesday: rule matches but pattern has no dow=3 entry → key absent in
    // rule_patterns_map → fall through P4/P5/P6 (none set up) → source "none"
    let wednesday = NaiveDate::from_ymd_opt(2026, 1, 7).unwrap(); // confirmed Wednesday
    let sched_wed =
        roster_repository::get_workspace_schedules(&pool, ws_id, wednesday, wednesday, None)
            .await
            .unwrap();
    let day_wed = &sched_wed.iter().find(|s| s.employee_id == emp).unwrap().schedules[0];
    assert!(
        day_wed.shift_id.is_none(),
        "Wednesday: pattern has no dow=3 detail → absent key → fall through to none"
    );
    assert_eq!(
        day_wed.source, "none",
        "Wednesday: absent composite key must not be treated as day off"
    );
}
