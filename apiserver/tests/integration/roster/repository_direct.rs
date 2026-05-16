//! Repository-level integration tests for split roster sub-modules.
//!
//! These tests exercise functions in `roster::roster_crud` and
//! `roster::schedule_queries` that are NOT fully covered by the existing
//! API-level or cache tests.

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::enums::AssignmentHistoryType;
use apiserver::models::{RosterItem, ShiftRoster};
use apiserver::repositories::roster::RosterRepository;
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

// Helper: set up a clean DB + factory + diesel pools
async fn setup_db() -> (
    TestDatabase,
    TestFactory,
    apiserver::diesel_pool::DieselPools,
) {
    let test_db = TestDatabase::new().await;
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let factory = TestFactory::new(test_db.diesel_pools.clone());
    (test_db, factory, diesel_pools)
}

// ── Upsert & Get ────────────────────────────────────────────────────

#[tokio::test]
async fn test_upsert_roster_insert_and_update() {
    let (test_db, factory, diesel_pools) = setup_db().await;
    let pool = test_db.diesel_pools.clone();
    let repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Morning")
        .create()
        .await;
    let date = NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();

    // Insert
    let roster_in = ShiftRoster {
        id: Uuid::new_v4(),
        employee_id: emp_id,
        shift_id: Some(shift_id),
        date,
        status: "confirmed".to_string(),
        notes: Some("Initial".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let inserted = repo.upsert_roster(&roster_in).await.unwrap();
    assert_eq!(inserted.employee_id, emp_id);
    assert_eq!(inserted.shift_id, Some(shift_id));
    assert_eq!(inserted.date, date);
    assert_eq!(inserted.notes.as_deref(), Some("Initial"));

    // Update (same employee + date → ON CONFLICT)
    let shift_id2 = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Evening")
        .create()
        .await;
    let roster_update = ShiftRoster {
        id: Uuid::new_v4(), // different ID, but same (employee_id, date) conflict key
        employee_id: emp_id,
        shift_id: Some(shift_id2),
        date,
        status: "confirmed".to_string(),
        notes: Some("Updated".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let updated = repo.upsert_roster(&roster_update).await.unwrap();
    assert_eq!(updated.shift_id, Some(shift_id2), "shift should be updated");
    assert_eq!(updated.notes.as_deref(), Some("Updated"));
    // The id should be the original one (INSERT ... ON CONFLICT preserves original id)
    assert_eq!(updated.id, inserted.id);
}

#[tokio::test]
async fn test_upsert_roster_day_off() {
    let (test_db, factory, diesel_pools) = setup_db().await;
    let pool = test_db.diesel_pools.clone();
    let repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();

    let roster_off = ShiftRoster {
        id: Uuid::new_v4(),
        employee_id: emp_id,
        shift_id: None, // Day off
        date,
        status: "confirmed".to_string(),
        notes: Some("Holiday".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let inserted = repo.upsert_roster(&roster_off).await.unwrap();
    assert!(
        inserted.shift_id.is_none(),
        "Day off should have None shift_id"
    );
}

// ── Get Employee Rosters ────────────────────────────────────────────

#[tokio::test]
async fn test_get_employee_rosters_date_range() {
    let (test_db, factory, diesel_pools) = setup_db().await;
    let pool = test_db.diesel_pools.clone();
    let repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let shift_id = factory.shift().with_workspace(ws_id).create().await;

    // Insert 3 entries: Mar 1, 5, 10
    for day in [1, 5, 10] {
        let date = NaiveDate::from_ymd_opt(2026, 3, day).unwrap();
        let r = ShiftRoster {
            id: Uuid::new_v4(),
            employee_id: emp_id,
            shift_id: Some(shift_id),
            date,
            status: "confirmed".to_string(),
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        repo.upsert_roster(&r).await.unwrap();
    }

    // Query Mar 3–7 → should return only Mar 5
    let start = NaiveDate::from_ymd_opt(2026, 3, 3).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 3, 7).unwrap();
    let result = repo.get_employee_rosters(emp_id, start, end).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].date, NaiveDate::from_ymd_opt(2026, 3, 5).unwrap());

    // Query full range → all 3
    let start_full = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let end_full = NaiveDate::from_ymd_opt(2026, 3, 31).unwrap();
    let all = repo
        .get_employee_rosters(emp_id, start_full, end_full)
        .await
        .unwrap();
    assert_eq!(all.len(), 3);
    // Verify ordering (ASC by date)
    assert!(all[0].date < all[1].date && all[1].date < all[2].date);
}

#[tokio::test]
async fn test_get_employee_rosters_empty() {
    let (test_db, factory, diesel_pools) = setup_db().await;
    let pool = test_db.diesel_pools.clone();
    let repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
    let result = repo.get_employee_rosters(emp_id, start, end).await.unwrap();
    assert!(result.is_empty());
}

// ── Delete ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_roster() {
    let (test_db, factory, diesel_pools) = setup_db().await;
    let pool = test_db.diesel_pools.clone();
    let repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let shift_id = factory.shift().with_workspace(ws_id).create().await;
    let date = NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();

    // Insert then delete
    let r = ShiftRoster {
        id: Uuid::new_v4(),
        employee_id: emp_id,
        shift_id: Some(shift_id),
        date,
        status: "confirmed".to_string(),
        notes: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    repo.upsert_roster(&r).await.unwrap();
    repo.delete_roster(emp_id, date).await.unwrap();

    // Verify gone
    let start = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 3, 31).unwrap();
    let remaining = repo.get_employee_rosters(emp_id, start, end).await.unwrap();
    assert!(
        remaining.is_empty(),
        "Deleted roster should not be returned"
    );
}

#[tokio::test]
async fn test_delete_nonexistent_roster_is_no_op() {
    let (test_db, factory, diesel_pools) = setup_db().await;
    let pool = test_db.diesel_pools.clone();
    let repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();

    // Should not error
    repo.delete_roster(emp_id, date).await.unwrap();
}

// ── Bulk Upsert ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_bulk_upsert_roster() {
    let (test_db, factory, diesel_pools) = setup_db().await;
    let pool = test_db.diesel_pools.clone();
    let repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let shift_a = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Shift A")
        .create()
        .await;
    let shift_b = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Shift B")
        .create()
        .await;

    let items = vec![
        RosterItem {
            date: NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            shift_id: Some(shift_a),
            notes: Some("Day 1".to_string()),
        },
        RosterItem {
            date: NaiveDate::from_ymd_opt(2026, 4, 2).unwrap(),
            shift_id: Some(shift_b),
            notes: None,
        },
        RosterItem {
            date: NaiveDate::from_ymd_opt(2026, 4, 3).unwrap(),
            shift_id: None, // Day off
            notes: Some("Off".to_string()),
        },
    ];

    repo.bulk_upsert_roster(emp_id, items).await.unwrap();

    let start = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 4, 30).unwrap();
    let rosters = repo.get_employee_rosters(emp_id, start, end).await.unwrap();
    assert_eq!(rosters.len(), 3);
    assert_eq!(rosters[0].shift_id, Some(shift_a));
    assert_eq!(rosters[1].shift_id, Some(shift_b));
    assert!(
        rosters[2].shift_id.is_none(),
        "Third entry should be day off"
    );
}

#[tokio::test]
async fn test_bulk_upsert_overwrites_existing() {
    let (test_db, factory, diesel_pools) = setup_db().await;
    let pool = test_db.diesel_pools.clone();
    let repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
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
    let date = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();

    // Insert first entry
    let r = ShiftRoster {
        id: Uuid::new_v4(),
        employee_id: emp_id,
        shift_id: Some(shift_old),
        date,
        status: "confirmed".to_string(),
        notes: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    repo.upsert_roster(&r).await.unwrap();

    // Bulk upsert with same date → should overwrite
    let items = vec![RosterItem {
        date,
        shift_id: Some(shift_new),
        notes: Some("Overwritten".to_string()),
    }];
    repo.bulk_upsert_roster(emp_id, items).await.unwrap();

    let start = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
    let rosters = repo.get_employee_rosters(emp_id, start, end).await.unwrap();
    assert_eq!(rosters.len(), 1);
    assert_eq!(
        rosters[0].shift_id,
        Some(shift_new),
        "Shift should be overwritten"
    );
}

// ── Assignment History ──────────────────────────────────────────────

#[tokio::test]
async fn test_get_assignment_history_includes_roster_entries() {
    let (test_db, factory, diesel_pools) = setup_db().await;
    let pool = test_db.diesel_pools.clone();
    let repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Test Shift")
        .create()
        .await;

    // Create a roster entry
    let r = ShiftRoster {
        id: Uuid::new_v4(),
        employee_id: emp_id,
        shift_id: Some(shift_id),
        date: NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
        status: "confirmed".to_string(),
        notes: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    repo.upsert_roster(&r).await.unwrap();

    let history = repo.get_assignment_history(ws_id, 100).await.unwrap();
    assert!(
        !history.is_empty(),
        "History should contain the roster entry"
    );

    // Find the roster entry in history
    let roster_entry = history
        .iter()
        .find(|h| h.assignment_type == AssignmentHistoryType::Roster)
        .expect("Should have a 'roster' type entry");
    assert_eq!(roster_entry.employee_id, emp_id);
    assert_eq!(roster_entry.shift_name.as_deref(), Some("Test Shift"));
}

#[tokio::test]
async fn test_get_assignment_history_empty_workspace() {
    let (test_db, factory, diesel_pools) = setup_db().await;
    let pool = test_db.diesel_pools.clone();
    let repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    let ws_id = factory.workspace().create().await;
    let history = repo.get_assignment_history(ws_id, 100).await.unwrap();
    assert!(history.is_empty());
}

#[tokio::test]
async fn test_get_assignment_history_respects_limit() {
    let (test_db, factory, diesel_pools) = setup_db().await;
    let pool = test_db.diesel_pools.clone();
    let repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let shift_id = factory.shift().with_workspace(ws_id).create().await;

    // Create 5 roster entries
    for day in 1..=5 {
        let r = ShiftRoster {
            id: Uuid::new_v4(),
            employee_id: emp_id,
            shift_id: Some(shift_id),
            date: NaiveDate::from_ymd_opt(2026, 5, day).unwrap(),
            status: "confirmed".to_string(),
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        repo.upsert_roster(&r).await.unwrap();
    }

    let history = repo.get_assignment_history(ws_id, 3).await.unwrap();
    assert_eq!(history.len(), 3, "Should respect the limit parameter");
}

// ── Effective Shift (DB-backed) ─────────────────────────────────────

#[tokio::test]
async fn test_get_effective_shift_from_roster() {
    let (test_db, factory, diesel_pools) = setup_db().await;
    let pool = test_db.diesel_pools.clone();
    let repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Morning")
        .create()
        .await;
    let date = NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();

    // Assign via roster (P1 — highest priority)
    let r = ShiftRoster {
        id: Uuid::new_v4(),
        employee_id: emp_id,
        shift_id: Some(shift_id),
        date,
        status: "confirmed".to_string(),
        notes: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    repo.upsert_roster(&r).await.unwrap();

    let (resolved_shift, traces) = repo.get_effective_shift(ws_id, emp_id, date).await.unwrap();
    assert_eq!(resolved_shift, Some(shift_id));
    assert_eq!(traces.len(), 1);
    assert_eq!(traces[0].rule_name, "Manual Roster");
    assert_eq!(traces[0].priority, 100);
}

#[tokio::test]
async fn test_get_effective_shift_no_assignment_returns_none() {
    let (test_db, factory, diesel_pools) = setup_db().await;
    let pool = test_db.diesel_pools.clone();
    let repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();

    let (resolved_shift, traces) = repo.get_effective_shift(ws_id, emp_id, date).await.unwrap();
    assert!(resolved_shift.is_none(), "No assignment → None");
    assert!(traces.is_empty(), "No traces when nothing matched");
}

#[tokio::test]
async fn test_get_effective_shift_fixed_assignment() {
    let (test_db, factory, diesel_pools) = setup_db().await;
    let pool = test_db.diesel_pools.clone();
    let repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    // Assign a fixed shift (P5)
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Fixed Shift")
        .assign_to_employee(emp_id, start)
        .await;

    let date = NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();
    let (resolved_shift, traces) = repo.get_effective_shift(ws_id, emp_id, date).await.unwrap();
    assert_eq!(resolved_shift, Some(shift_id));
    assert_eq!(traces[0].rule_name, "Fixed Assignment");
    assert_eq!(traces[0].priority, 20);
}
