// =============================================================================
// RedbStore Operations Integration Tests
// =============================================================================
// Tests for untested RedbStore operations: crash recovery, stale sync reset,
// dead letter queue, signature management, backoff filtering, and parsing errors.
//
// All tests use in-memory Redb (":memory:") via HybridDatabase for isolation.
// =============================================================================

use agent::hybrid_database::HybridDatabase;
use agent::models::AttendanceData;

// --- Helpers ---

fn setup() -> HybridDatabase {
    HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed")
}

fn make_attendance(user_id: &str, timestamp: &str) -> AttendanceData {
    AttendanceData {
        user_id: user_id.into(),
        timestamp: timestamp.into(),
        status: Some(1),
        punch: Some(1),
    }
}

fn setup_with_device(device_id: &str) -> HybridDatabase {
    let db = setup();
    db.store()
        .upsert_device(device_id, "Test Device", "127.0.0.1", 4370, "zkteco", "UTC", None, true)
        .unwrap();
    db
}

// =============================================================================
// 1. count_unsynced
// =============================================================================

#[test]
fn test_count_unsynced() {
    let db = setup_with_device("DEV1");
    let store = db.store();

    assert_eq!(store.count_unsynced().unwrap(), 0);

    let id1 = db.save_raw_attendance("DEV1", make_attendance("U1", "2026-01-01 10:00:00")).unwrap();
    let id2 = db.save_raw_attendance("DEV1", make_attendance("U2", "2026-01-01 11:00:00")).unwrap();
    let _id3 = db.save_raw_attendance("DEV1", make_attendance("U3", "2026-01-01 12:00:00")).unwrap();

    assert_eq!(store.count_unsynced().unwrap(), 3);

    // Mark one synced
    store.mark_in_flight(&[id1]).unwrap();
    store.mark_synced(&[id1]).unwrap();
    assert_eq!(store.count_unsynced().unwrap(), 2);

    // Mark another in-flight (should not count as unsynced)
    store.mark_in_flight(&[id2]).unwrap();
    assert_eq!(store.count_unsynced().unwrap(), 1);
}

// =============================================================================
// 2. reset_syncing_status — crash recovery
// =============================================================================

#[test]
fn test_reset_syncing_status_crash_recovery() {
    let db = setup_with_device("DEV_CR");
    let store = db.store();

    let id1 = db.save_raw_attendance("DEV_CR", make_attendance("U1", "2026-01-01 10:00:00")).unwrap();
    let id2 = db.save_raw_attendance("DEV_CR", make_attendance("U2", "2026-01-01 11:00:00")).unwrap();
    let id3 = db.save_raw_attendance("DEV_CR", make_attendance("U3", "2026-01-01 12:00:00")).unwrap();

    // Mark all in-flight (simulating crash mid-sync)
    store.mark_in_flight(&[id1, id2, id3]).unwrap();
    assert_eq!(store.count_unsynced().unwrap(), 0);

    // Crash recovery: reset all in-flight back to unsynced
    let reset = store.reset_syncing_status().unwrap();
    assert_eq!(reset, 3);
    assert_eq!(store.count_unsynced().unwrap(), 3);

    // Second call should return 0 (nothing to reset)
    let reset2 = store.reset_syncing_status().unwrap();
    assert_eq!(reset2, 0);
}

// =============================================================================
// 3. reset_stale_syncing — timeout-based reset
// =============================================================================

#[test]
fn test_reset_stale_syncing() {
    let db = setup_with_device("DEV_STALE");
    let store = db.store();

    let id1 = db.save_raw_attendance("DEV_STALE", make_attendance("U1", "2026-01-01 10:00:00")).unwrap();

    // Mark in-flight (updated_at = now)
    store.mark_in_flight(&[id1]).unwrap();

    // With a very large timeout, nothing should be reset (too recent)
    let reset = store.reset_stale_syncing(99999).unwrap();
    assert_eq!(reset, 0);

    // With a negative timeout, now - updated_at (== 0) > -1, so it IS stale
    let reset = store.reset_stale_syncing(-1).unwrap();
    assert_eq!(reset, 1);
    assert_eq!(store.count_unsynced().unwrap(), 1);
}

// =============================================================================
// 4. get_max_id
// =============================================================================

#[test]
fn test_get_max_id() {
    let db = setup_with_device("DEV_MAX");
    let store = db.store();

    // Empty DB
    assert_eq!(store.get_max_id().unwrap(), 0);

    let _id1 = db.save_raw_attendance("DEV_MAX", make_attendance("U1", "2026-01-01 10:00:00")).unwrap();
    let id2 = db.save_raw_attendance("DEV_MAX", make_attendance("U2", "2026-01-01 11:00:00")).unwrap();

    assert_eq!(store.get_max_id().unwrap(), id2);

    // After syncing (removing from active), max_id should update
    store.mark_in_flight(&[id2]).unwrap();
    store.mark_synced(&[id2]).unwrap();

    // Only id1 remains in active storage
    let max = store.get_max_id().unwrap();
    assert!(max > 0, "max_id should still be > 0 after removing one log");
}

// =============================================================================
// 5. Device signature load/unload
// =============================================================================

#[test]
fn test_device_signature_load_unload() {
    let db = setup_with_device("DEV_SIG");
    let store = db.store();

    // Insert a log (automatically creates signature via insert)
    let _id = db.save_raw_attendance("DEV_SIG", make_attendance("U1", "2026-01-01 10:00:00")).unwrap();

    // Trying to insert same data again should be rejected (Duplicate error)
    let result = db.save_raw_attendance("DEV_SIG", make_attendance("U1", "2026-01-01 10:00:00"));
    assert!(result.is_err(), "Duplicate insert should return Err");

    // Remove signatures for device (also removes logs and indices)
    let removed = store.remove_signatures_for_device("DEV_SIG").unwrap();
    assert_eq!(removed, 1);

    // Verify has_signature returns false
    assert!(!store.has_signature("DEV_SIG", "U1", 1735689600, Some(1), Some(1)).unwrap());

    // Now the same data can be inserted again (signature gone)
    let id2 = db.save_raw_attendance("DEV_SIG", make_attendance("U1", "2026-01-01 10:00:00")).unwrap();
    assert!(id2 > 0, "Should insert successfully after signature removal");

    // Redb persists signatures on insert — duplicate should be rejected immediately
    let result = db.save_raw_attendance("DEV_SIG", make_attendance("U1", "2026-01-01 10:00:00"));
    assert!(result.is_err(), "Duplicate should be rejected (signatures persist in redb)");
}

// =============================================================================
// 6. move_expired_logs_to_dlq
// =============================================================================

#[test]
fn test_move_expired_to_dlq() {
    let db = setup_with_device("DEV_DLQ");
    let store = db.store();

    let id = db.save_raw_attendance("DEV_DLQ", make_attendance("U1", "2026-01-01 10:00:00")).unwrap();

    // Simulate multiple retries by marking in-flight then resetting
    // mark_in_flight increments retry_count each time
    for _ in 0..4 {
        store.mark_in_flight(&[id]).unwrap();
        store.reset_to_unsynced(id).unwrap();
    }

    // Verify retry_count is >= 4
    let logs = store.get_all_logs().unwrap();
    let log = logs.iter().find(|l| l.id == id).unwrap();
    assert!(log.retry_count >= 4, "retry_count should be >= 4, got {}", log.retry_count);

    // Move expired logs (max_retries=3) to DLQ
    let moved = store.move_expired_logs_to_dlq(3).unwrap();
    assert_eq!(moved, 1);

    // Log should be gone from active
    assert_eq!(store.count_unsynced().unwrap(), 0);

    // But visible in failed logs
    let failed = store.get_failed_logs().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].device_id, "DEV_DLQ");
    assert_eq!(failed[0].user_id, "U1");
    assert!(failed[0].failure_reason.contains("exceeded"));
}

// =============================================================================
// 7. retry_failed_log
// =============================================================================

#[test]
fn test_retry_failed_log() {
    let db = setup_with_device("DEV_RETRY");
    let store = db.store();

    let id = db.save_raw_attendance("DEV_RETRY", make_attendance("U1", "2026-01-01 10:00:00")).unwrap();

    // Exceed retries and move to DLQ
    for _ in 0..4 {
        store.mark_in_flight(&[id]).unwrap();
        store.reset_to_unsynced(id).unwrap();
    }
    store.move_expired_logs_to_dlq(3).unwrap();
    assert_eq!(store.count_failed_logs().unwrap(), 1);
    assert_eq!(store.count_unsynced().unwrap(), 0);

    // Retry the failed log
    let retried = store.retry_failed_log(id).unwrap();
    assert!(retried, "Should successfully retry");

    // Failed logs should be empty
    assert_eq!(store.count_failed_logs().unwrap(), 0);

    // Log should be back in active queue, unsynced
    assert_eq!(store.count_unsynced().unwrap(), 1);
}

// =============================================================================
// 8. delete_failed_log and purge_failed_logs
// =============================================================================

#[test]
fn test_delete_and_purge_failed_logs() {
    let db = setup_with_device("DEV_PURGE");
    let store = db.store();

    // Create 3 logs and push all to DLQ
    let id1 = db.save_raw_attendance("DEV_PURGE", make_attendance("U1", "2026-01-01 10:00:00")).unwrap();
    let id2 = db.save_raw_attendance("DEV_PURGE", make_attendance("U2", "2026-01-01 11:00:00")).unwrap();
    let id3 = db.save_raw_attendance("DEV_PURGE", make_attendance("U3", "2026-01-01 12:00:00")).unwrap();

    for id in [id1, id2, id3] {
        for _ in 0..4 {
            store.mark_in_flight(&[id]).unwrap();
            store.reset_to_unsynced(id).unwrap();
        }
    }
    store.move_expired_logs_to_dlq(3).unwrap();
    assert_eq!(store.count_failed_logs().unwrap(), 3);

    // Delete one
    let deleted = store.delete_failed_log(id1).unwrap();
    assert!(deleted);
    assert_eq!(store.count_failed_logs().unwrap(), 2);

    // Delete non-existent
    let deleted_again = store.delete_failed_log(id1).unwrap();
    assert!(!deleted_again);

    // Purge remaining
    let purged = store.purge_failed_logs().unwrap();
    assert_eq!(purged, 2);
    assert_eq!(store.count_failed_logs().unwrap(), 0);
}

// =============================================================================
// 9. record_parsing_error
// =============================================================================

#[test]
fn test_record_parsing_error() {
    let db = setup();
    let store = db.store();

    // Should not panic
    store.record_parsing_error("DEV1", "invalid-timestamp", "Failed to parse").unwrap();
    store.record_parsing_error("DEV2", "2026-13-99 25:61:99", "Invalid date components").unwrap();

    // Multiple errors should be recorded without issue
    for i in 0..10 {
        store
            .record_parsing_error("DEV_BATCH", &format!("bad-ts-{i}"), &format!("Error #{i}"))
            .unwrap();
    }
}

// =============================================================================
// 10. Backoff filtering in get_unsynced
// =============================================================================

#[test]
fn test_backoff_filtering() {
    let db = setup_with_device("DEV_BO");
    let store = db.store();

    let id = db.save_raw_attendance("DEV_BO", make_attendance("U1", "2026-01-01 10:00:00")).unwrap();

    // Fresh log should be returned (no backoff)
    let unsynced = db.get_unsynced_logs(10);
    assert_eq!(unsynced.len(), 1);

    // Simulate a retry: mark in-flight then reset
    // After reset, updated_at is set and retry_count > 0
    // get_unsynced applies backoff: now < updated_at + 5*(2^retry_count)
    store.mark_in_flight(&[id]).unwrap();
    store.reset_to_unsynced(id).unwrap();

    // After 1 retry, backoff = 5*2 = 10 seconds
    // Since we just set updated_at = now, the log should be in backoff
    let unsynced = db.get_unsynced_logs(10);
    assert_eq!(unsynced.len(), 0, "Log should be in backoff after retry");

    // But count_unsynced doesn't apply backoff (counts raw state)
    assert_eq!(store.count_unsynced().unwrap(), 1, "count_unsynced should still show 1");
}

// =============================================================================
// 11. Empty operations — edge cases
// =============================================================================

#[test]
fn test_empty_operations() {
    let db = setup();
    let store = db.store();

    // All operations on empty DB should succeed gracefully
    assert_eq!(store.count_unsynced().unwrap(), 0);
    assert_eq!(store.get_max_id().unwrap(), 0);
    assert_eq!(store.reset_syncing_status().unwrap(), 0);
    assert_eq!(store.reset_stale_syncing(60).unwrap(), 0);
    assert_eq!(store.move_expired_logs_to_dlq(3).unwrap(), 0);
    assert_eq!(store.count_failed_logs().unwrap(), 0);
    assert_eq!(store.purge_failed_logs().unwrap(), 0);
    assert_eq!(store.cleanup_old_synced_logs(30).unwrap(), 0);

    // Empty batch operations
    store.mark_in_flight(&[]).unwrap();
    store.mark_synced(&[]).unwrap();

    let (evicted, before, after) = store.evict_old_signatures().unwrap();
    assert_eq!(evicted, 0);
    assert_eq!(before, 0);
    assert_eq!(after, 0);

    // Non-existent operations
    assert!(!store.delete_failed_log(999).unwrap());
    assert!(!store.retry_failed_log(999).unwrap());
}
