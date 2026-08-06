// ── TECH_DEBT: Enterprise-grade lint upgrade (2026-05-03) ──
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(missing_docs)]

//! Verifies the cached unsynced counter stays exact across every state
//! transition, now that `count_unsynced` is O(1) instead of a full SYNC_INDEX
//! scan. `SYNC_INDEX.len()` is NOT a valid substitute: the table holds both
//! state 0 (unsynced) and state 2 (in-flight) rows, and `mark_synced` deletes
//! rows outright, so state 1 is never persisted.

use agent::models::AttendanceData;
use agent::redb_store::RedbStore;

fn make_store() -> RedbStore {
    RedbStore::new(":memory:", 90).expect("in-memory store")
}

fn make_data(user_id: &str) -> AttendanceData {
    AttendanceData {
        user_id: user_id.into(),
        timestamp: "2026-01-01 10:00:00".into(),
        status: Some(1),
        punch: Some(1),
    }
}

fn insert(store: &RedbStore, user_id: &str, ts: i64) -> i64 {
    store.insert("DEV1", &make_data(user_id), ts).unwrap()
}

/// Assert the cached count matches a fresh scan of ground truth.
fn assert_consistent(store: &RedbStore, expected: usize, label: &str) {
    assert_eq!(
        store.count_unsynced().unwrap(),
        expected,
        "cached count wrong after {label}"
    );
    let truth = store.reconcile_unsynced_count().unwrap();
    assert_eq!(
        truth, expected,
        "ground-truth scan disagrees after {label} (cache had drifted)"
    );
}

#[test]
fn test_count_unsynced_across_full_lifecycle() {
    let store = make_store();

    // The exact sequence from the task: insert → mark_in_flight → mark_synced
    // → reset_to_unsynced.
    let id = insert(&store, "U1", 1000);
    assert_consistent(&store, 1, "insert");

    store.mark_in_flight(&[id]).unwrap();
    assert_consistent(&store, 0, "mark_in_flight (0 → 2)");

    store.mark_synced(&[id]).unwrap();
    assert_consistent(&store, 0, "mark_synced (row removed)");

    // Row no longer exists, so this is a no-op and must not resurrect the count.
    store.reset_to_unsynced(id).unwrap();
    assert_consistent(&store, 0, "reset_to_unsynced on a synced (removed) row");
}

#[test]
fn test_count_unsynced_reset_after_in_flight() {
    let store = make_store();
    let id = insert(&store, "U1", 1000);

    store.mark_in_flight(&[id]).unwrap();
    assert_consistent(&store, 0, "mark_in_flight");

    // Failed ACK path: 2 → 0 must increment again.
    store.reset_to_unsynced(id).unwrap();
    assert_consistent(&store, 1, "reset_to_unsynced (2 → 0)");
}

#[test]
fn test_repeated_transitions_do_not_drift() {
    let store = make_store();
    let id = insert(&store, "U1", 1000);
    assert_consistent(&store, 1, "insert");

    // Idempotence: repeated identical transitions must not double-count.
    for i in 0..5 {
        store.mark_in_flight(&[id]).unwrap();
        store.mark_in_flight(&[id]).unwrap(); // already in-flight
        assert_consistent(&store, 0, &format!("double mark_in_flight round {i}"));

        store.reset_to_unsynced(id).unwrap();
        store.reset_to_unsynced(id).unwrap(); // already unsynced
        assert_consistent(&store, 1, &format!("double reset_to_unsynced round {i}"));
    }
}

#[test]
fn test_count_unsynced_batch_and_partial_sync() {
    let store = make_store();
    let ids: Vec<i64> = (0..10)
        .map(|i| insert(&store, &format!("U{i}"), (i + 1) * 1000))
        .collect();
    assert_consistent(&store, 10, "10 inserts");

    store.mark_in_flight(&ids[..4]).unwrap();
    assert_consistent(&store, 6, "4 of 10 in flight");

    store.mark_synced(&ids[..2]).unwrap();
    assert_consistent(&store, 6, "2 synced from in-flight");

    store.reset_to_unsynced(ids[2]).unwrap();
    assert_consistent(&store, 7, "1 reset from in-flight");
}

#[test]
fn test_mark_synced_directly_from_unsynced() {
    let store = make_store();
    let id = insert(&store, "U1", 1000);
    assert_consistent(&store, 1, "insert");

    // An ACK can arrive for a row never marked in-flight; removing a state-0
    // row must decrement.
    store.mark_synced(&[id]).unwrap();
    assert_consistent(&store, 0, "mark_synced straight from state 0");
}

#[test]
fn test_reset_stale_syncing_restores_count() {
    let store = make_store();
    let ids: Vec<i64> = (0..3)
        .map(|i| insert(&store, &format!("U{i}"), (i + 1) * 1000))
        .collect();

    store.mark_in_flight(&ids).unwrap();
    assert_consistent(&store, 0, "all in flight");

    // A negative timeout makes every in-flight row stale, even when the
    // timestamp was written in the same wall-clock second.
    let reset = store.reset_stale_syncing(-1).unwrap();
    assert_eq!(reset, 3);
    assert_consistent(&store, 3, "reset_stale_syncing");
}

#[test]
fn test_reset_syncing_status_restores_count() {
    let store = make_store();
    let ids: Vec<i64> = (0..3)
        .map(|i| insert(&store, &format!("U{i}"), (i + 1) * 1000))
        .collect();

    store.mark_in_flight(&ids).unwrap();
    assert_consistent(&store, 0, "all in flight");

    let reset = store.reset_syncing_status().unwrap();
    assert_eq!(reset, 3);
    assert_consistent(&store, 3, "reset_syncing_status (crash recovery)");
}

#[test]
fn test_dlq_move_decrements_count() {
    let store = make_store();
    let id = insert(&store, "U1", 1000);
    let keep = insert(&store, "U2", 2000);
    assert_consistent(&store, 2, "2 inserts");

    // Drive retry_count past the limit, ending in state 0.
    for _ in 0..4 {
        store.mark_in_flight(&[id]).unwrap();
        store.reset_to_unsynced(id).unwrap();
    }
    assert_consistent(&store, 2, "retries exhausted, still unsynced");

    let moved = store.move_expired_logs_to_dlq(3).unwrap();
    assert_eq!(moved, 1);
    assert_consistent(&store, 1, "expired log moved to DLQ");

    // Returning it to the active queue must increment again.
    assert!(store.retry_failed_log(id).unwrap());
    assert_consistent(&store, 2, "retry_failed_log re-queued");
    assert!(keep > 0);
}

#[test]
fn test_batch_insert_counts_only_unique() {
    let store = make_store();
    let batch: Vec<(String, AttendanceData, i64)> = vec![
        ("DEV1".into(), make_data("U1"), 1000),
        ("DEV1".into(), make_data("U1"), 1000), // duplicate, skipped
        ("DEV1".into(), make_data("U2"), 2000),
    ];
    let inserted = store.insert_batch(&batch).unwrap();
    assert_eq!(inserted, 2);
    assert_consistent(&store, 2, "insert_batch with one duplicate");
}

#[test]
fn test_remove_signatures_for_device_adjusts_count() {
    let store = make_store();
    let d1a = store.insert("DEV1", &make_data("U1"), 1000).unwrap();
    store.insert("DEV1", &make_data("U2"), 2000).unwrap();
    store.insert("DEV2", &make_data("U3"), 3000).unwrap();
    assert_consistent(&store, 3, "3 inserts across 2 devices");

    // One DEV1 row is in-flight, so only the state-0 rows may be subtracted.
    store.mark_in_flight(&[d1a]).unwrap();
    assert_consistent(&store, 2, "one DEV1 row in flight");

    store.remove_signatures_for_device("DEV1").unwrap();
    assert_consistent(&store, 1, "DEV1 purged (1 unsynced + 1 in-flight removed)");
}

#[test]
fn test_count_survives_reopen_via_reseed() {
    // A file-backed store proves the counter is rebuilt from the table on open,
    // which is what makes in-memory drift self-correcting across restarts.
    let dir = std::env::temp_dir().join(format!("leeattend_count_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("reseed.redb");
    let path_str = path.to_string_lossy().to_string();

    let id_in_flight;
    {
        let store = RedbStore::new(&path_str, 90).unwrap();
        for i in 0..5 {
            insert(&store, &format!("U{i}"), (i + 1) * 1000);
        }
        id_in_flight = store.get_unsynced(1).unwrap()[0].id;
        store.mark_in_flight(&[id_in_flight]).unwrap();
        assert_eq!(store.count_unsynced().unwrap(), 4);
    }

    // Reopen: the seed scan must count state-0 rows only, excluding the
    // in-flight row that still occupies a SYNC_INDEX slot.
    {
        let store = RedbStore::new(&path_str, 90).unwrap();
        assert_eq!(
            store.count_unsynced().unwrap(),
            4,
            "reopened store must exclude the state-2 row (len() would report 5)"
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}
