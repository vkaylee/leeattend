//! Roster cache build data tests
//!
//! Tests for `build_roster_data` — the async loader that fetches all
//! roster-related data for a workspace in parallel queries.

use crate::common::app::create_test_redis_pool;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::cache::{build_roster_data, RosterCache};
use chrono::NaiveDate;
use redis::AsyncCommands;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Empty / baseline
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_empty_workspace_all_maps_empty() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let workspace_id = factory.workspace().create().await;

    let data = build_roster_data(&diesel_pools, workspace_id)
        .await
        .unwrap();

    assert!(data.shifts.is_empty());
    assert!(data.fixed_assignments.is_empty());
    assert!(data.pattern_assignments.is_empty());
    assert!(data.active_rules.is_empty());
    assert!(data.rule_patterns.is_empty());
    assert!(data.dept_history.is_empty());
}

// ---------------------------------------------------------------------------
// Shifts
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_shifts_populated() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let workspace_id = factory.workspace().create().await;
    let s1 = factory.shift().with_workspace(workspace_id).create().await;
    let s2 = factory.shift().with_workspace(workspace_id).create().await;

    let data = build_roster_data(&diesel_pools, workspace_id)
        .await
        .unwrap();

    assert_eq!(data.shifts.len(), 2);
    assert!(data.shifts.contains_key(&s1));
    assert!(data.shifts.contains_key(&s2));
}

#[tokio::test]
async fn test_shift_fields_mapped_correctly() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let ws = factory.workspace().create().await;
    let start = chrono::NaiveTime::from_hms_opt(8, 30, 0).unwrap();
    let end = chrono::NaiveTime::from_hms_opt(17, 30, 0).unwrap();

    let shift_id = factory
        .shift()
        .with_workspace(ws)
        .with_name("Morning Shift")
        .with_times(start, end)
        .with_break_duration(45)
        .with_grace_period(10)
        .with_allow_early_ot(true)
        .create()
        .await;

    let data = build_roster_data(&diesel_pools, ws).await.unwrap();
    let shift = data.shifts.get(&shift_id).expect("shift must exist");
    assert_eq!(shift.name(), "Morning Shift");

    // Time/break/policy fields now live on intervals, not RosterShift
    let intervals = data
        .shift_intervals
        .get(&shift_id)
        .expect("intervals must exist");
    let iv = &intervals[0];
    assert_eq!(iv.start_time, start);
    assert_eq!(iv.end_time, end);
    assert_eq!(iv.break_duration_minutes, 45);
    assert_eq!(iv.early_punch_in_grace_period_minutes, 10);
    assert!(iv.allow_early_ot);
}

// ---------------------------------------------------------------------------
// Fixed assignments (employee_shifts)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fixed_assignments_grouped_by_employee() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let ws = factory.workspace().create().await;
    let emp_a = factory.employee().with_workspace(ws).create().await;
    let emp_b = factory.employee().with_workspace(ws).create().await;

    let emp_a_employment: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        emp_a
    );
    let emp_b_employment: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        emp_b
    );

    let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let start2 = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();

    // emp_a gets 2 shifts, emp_b gets 1
    factory
        .shift()
        .with_workspace(ws)
        .assign_to_employee(emp_a, start)
        .await;
    factory
        .shift()
        .with_workspace(ws)
        .assign_to_employee(emp_a, start2)
        .await;
    factory
        .shift()
        .with_workspace(ws)
        .assign_to_employee(emp_b, start)
        .await;

    let data = build_roster_data(&diesel_pools, ws).await.unwrap();

    assert_eq!(data.fixed_assignments.len(), 2, "two employees");
    assert_eq!(data.fixed_assignments[&emp_a_employment].len(), 2);
    assert_eq!(data.fixed_assignments[&emp_b_employment].len(), 1);

    // Verify the assignment fields
    let assignment = &data.fixed_assignments[&emp_b_employment][0];
    assert_eq!(assignment.start_date, start);
    assert!(assignment.end_date.is_none());
}

// ---------------------------------------------------------------------------
// Department history
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_dept_history_populated() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let ws = factory.workspace().create().await;
    let dept = factory.department().with_workspace(ws).create().await;
    let emp = factory.employee().with_workspace(ws).create().await;
    let eff_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    crate::test_execute!(
        &pool.api,
        "UPDATE employments SET department_id = $2, start_date = $3 WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        emp,
        diesel::sql_types::Uuid,
        dept,
        diesel::sql_types::Date,
        eff_date
    );

    let data = build_roster_data(&diesel_pools, ws).await.unwrap();

    assert_eq!(data.dept_history.len(), 1);
    let entries = &data.dept_history[&emp];
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].department_id, dept);
    assert_eq!(entries[0].effective_date, eff_date);
}

// ---------------------------------------------------------------------------
// Cross-workspace isolation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_cross_workspace_isolation() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let ws_a = factory.workspace().create().await;
    let ws_b = factory.workspace().create().await;

    // Create shifts in both workspaces
    factory.shift().with_workspace(ws_a).create().await;
    factory.shift().with_workspace(ws_a).create().await;
    factory.shift().with_workspace(ws_b).create().await;

    let data_a = build_roster_data(&diesel_pools, ws_a).await.unwrap();
    let data_b = build_roster_data(&diesel_pools, ws_b).await.unwrap();

    assert_eq!(data_a.shifts.len(), 2, "ws_a should have 2 shifts");
    assert_eq!(data_b.shifts.len(), 1, "ws_b should have 1 shift");

    // Verify no cross-contamination
    for shift_id in data_a.shifts.keys() {
        assert!(
            !data_b.shifts.contains_key(shift_id),
            "ws_b must not contain ws_a's shifts"
        );
    }
}

// ---------------------------------------------------------------------------
// Assignment rules + rule patterns
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_assignment_rules_and_rule_patterns() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let ws = factory.workspace().create().await;
    let shift_id = factory.shift().with_workspace(ws).create().await;

    // Create a working pattern
    let pattern_id: uuid::Uuid = crate::test_query_scalar_uuid!(&pool.api,
        "INSERT INTO working_patterns (workspace_id, name) VALUES ($1, 'Test Pattern') RETURNING id",
        diesel::sql_types::Uuid, ws
    );

    // Add pattern details (Monday = shift, Tuesday = off)
    crate::test_execute!(&pool.api,
        "INSERT INTO working_pattern_details (pattern_id, schedule_day, template_id) VALUES ($1, 1, $2), ($1, 2, NULL)",
        diesel::sql_types::Uuid, pattern_id,
        diesel::sql_types::Uuid, shift_id
    );

    // Create an assignment rule that references this pattern
    let _rule_id = factory
        .assignment_rule()
        .with_workspace(ws)
        .with_pattern(pattern_id)
        .with_name("Office Rule")
        .with_priority(10)
        .create()
        .await;

    let data = build_roster_data(&diesel_pools, ws).await.unwrap();

    // Active rules
    assert_eq!(data.active_rules.len(), 1);
    assert_eq!(data.active_rules[0].name(), "Office Rule");
    assert_eq!(data.active_rules[0].priority(), 10);
    assert_eq!(data.active_rules[0].pattern_id(), pattern_id);

    // Rule patterns (pattern_id, day_of_week) -> shift_id
    assert_eq!(data.rule_patterns.len(), 2);
    assert_eq!(
        data.rule_patterns[&(pattern_id, 1)],
        vec![Some(shift_id)],
        "Monday should have the shift"
    );
    assert_eq!(
        data.rule_patterns[&(pattern_id, 2)],
        vec![None],
        "Tuesday should be off"
    );
}

// ---------------------------------------------------------------------------
// Concurrent build produces consistent result (Arc)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_build_returns_arc_with_consistent_data() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let ws = factory.workspace().create().await;
    factory
        .shift()
        .with_workspace(ws)
        .with_name("Day Shift")
        .create()
        .await;

    // Build twice — both should be Ok and contain the same shift
    let data1 = build_roster_data(&diesel_pools, ws).await.unwrap();
    let data2 = build_roster_data(&diesel_pools, ws).await.unwrap();

    assert_eq!(data1.shifts.len(), data2.shifts.len());
    for (id, shift) in &data1.shifts {
        let other = data2
            .shifts
            .get(id)
            .expect("same shift should exist in both");
        assert_eq!(shift.name(), other.name());
    }
}

// ---------------------------------------------------------------------------
// Redis-backed cache behavior
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_redis_cache_reuses_and_refreshes_workspace_snapshot() {
    let Some(redis) = create_test_redis_pool().await else {
        return;
    };

    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.diesel_pools.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let workspace_id = factory.workspace().create().await;
    let first_shift = factory.shift().with_workspace(workspace_id).create().await;
    let cache = RosterCache::new(redis);

    let first = cache
        .get_roster_data(workspace_id, &diesel_pools)
        .await
        .expect("initial roster cache load should succeed");
    let second = cache
        .get_roster_data(workspace_id, &diesel_pools)
        .await
        .expect("cached roster load should succeed");

    assert!(Arc::ptr_eq(&first, &second));
    assert!(first.shifts.contains_key(&first_shift));

    let second_shift = factory.shift().with_workspace(workspace_id).create().await;
    cache.invalidate(workspace_id).await;

    let refreshed = cache
        .get_roster_data(workspace_id, &diesel_pools)
        .await
        .expect("invalidated roster cache load should succeed");
    assert!(!Arc::ptr_eq(&first, &refreshed));
    assert!(refreshed.shifts.contains_key(&second_shift));
}

#[tokio::test]
async fn test_redis_cache_version_is_monotonic_and_workspace_scoped() {
    let Some(redis) = create_test_redis_pool().await else {
        return;
    };

    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.diesel_pools.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let workspace_a = factory.workspace().create().await;
    let workspace_b = factory.workspace().create().await;
    factory.shift().with_workspace(workspace_a).create().await;
    factory.shift().with_workspace(workspace_b).create().await;
    let cache = RosterCache::new(redis.clone());

    let data_b = cache
        .get_roster_data(workspace_b, &diesel_pools)
        .await
        .expect("workspace B cache load should succeed");
    let key_a = format!("roster:version:{workspace_a}");
    let key_b = format!("roster:version:{workspace_b}");
    let mut conn = redis.conn();
    conn.set::<_, _, ()>(&key_a, i64::MAX - 2)
        .await
        .expect("seeded Redis version should be writable");
    let before_b: Option<i64> = conn
        .get(&key_b)
        .await
        .expect("workspace B version should be readable");
    drop(conn);

    cache.invalidate(workspace_a).await;
    cache.invalidate(workspace_a).await;

    let mut conn = redis.conn();
    let after_a: i64 = conn
        .get(&key_a)
        .await
        .expect("workspace A version should be readable");
    let after_b: Option<i64> = conn
        .get(&key_b)
        .await
        .expect("workspace B version should remain readable");
    drop(conn);

    assert_eq!(after_a, i64::MAX);
    assert_eq!(after_b, before_b);

    let data_b_again = cache
        .get_roster_data(workspace_b, &diesel_pools)
        .await
        .expect("workspace B cached load should succeed");
    assert!(Arc::ptr_eq(&data_b, &data_b_again));
}
