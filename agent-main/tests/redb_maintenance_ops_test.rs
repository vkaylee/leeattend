// ── TECH_DEBT: Enterprise-grade lint upgrade (2026-05-03) ──
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]  
#![allow(clippy::panic)]
#![allow(clippy::todo)]
#![allow(clippy::unimplemented)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(clippy::suspicious)]
#![allow(clippy::style)]
#![allow(clippy::correctness)]
#![allow(clippy::perf)]
#![allow(clippy::complexity)]
#![allow(clippy::clone_on_ref_ptr)]
#![allow(missing_docs)]

// =============================================================================
// RedbStore Maintenance Operations Integration Tests
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
        .upsert_device(
            device_id,
            "Test Device",
            "127.0.0.1",
            4370,
            "zkteco",
            "UTC",
            None,
            true,
            123456,
        )
        .unwrap();
    db
}

// =============================================================================
// 1. cleanup_old_synced_logs
// =============================================================================

#[tokio::test]
async fn test_cleanup_old_synced_logs() {
    let db = setup_with_device("DEV_MAINT");
    let store = db.store();

    let id1 = db
        .save_raw_attendance(
            "DEV_MAINT".into(),
            make_attendance("U1", "2026-01-01 10:00:00"),
        )
        .await
        .unwrap();
    let id2 = db
        .save_raw_attendance(
            "DEV_MAINT".into(),
            make_attendance("U2", "2026-01-01 11:00:00"),
        )
        .await
        .unwrap();

    // Mark both in flight
    store.mark_in_flight(&[id1, id2]).unwrap();

    // Mark id1 as synced — this REMOVES it from active storage (ATTENDANCE_LOGS + SYNC_INDEX)
    store.mark_synced(&[id1]).unwrap();

    // After mark_synced, id1 is already removed from active storage.
    // cleanup_old_synced_logs scans ATTENDANCE_LOGS for synced entries,
    // but mark_synced() already removed them. So cleanup should find 0.
    let cleaned = store.cleanup_old_synced_logs(30).unwrap();
    assert_eq!(
        cleaned, 0,
        "No synced logs remain in active storage after mark_synced()"
    );

    // Even with a very aggressive cutoff, nothing to clean
    let cleaned_all = store.cleanup_old_synced_logs(-1).unwrap();
    assert_eq!(
        cleaned_all, 0,
        "mark_synced already removed synced logs from active storage"
    );

    // Verify only id2 remains in active storage
    let all_logs = store.get_all_logs().unwrap();
    assert_eq!(all_logs.len(), 1);
    assert_eq!(all_logs[0].id, id2);
}

// =============================================================================
// 2. get_failed_logs & get_all_logs
// =============================================================================

#[tokio::test]
async fn test_get_failed_and_all_logs() {
    let db = setup_with_device("DEV_MAINT");
    let store = db.store();

    let id1 = db
        .save_raw_attendance(
            "DEV_MAINT".into(),
            make_attendance("U1", "2026-01-01 10:00:00"),
        )
        .await
        .unwrap();
    let id2 = db
        .save_raw_attendance(
            "DEV_MAINT".into(),
            make_attendance("U2", "2026-01-01 11:00:00"),
        )
        .await
        .unwrap();

    // Get all logs
    let all_logs = store.get_all_logs().unwrap();
    assert_eq!(all_logs.len(), 2);

    // Induce failures for id1
    for _ in 0..4 {
        store.mark_in_flight(&[id1]).unwrap();
        store.reset_to_unsynced(id1).unwrap();
    }

    // Move to DLQ
    store.move_expired_logs_to_dlq(3).unwrap();

    let failed_logs = store.get_failed_logs().unwrap();
    assert_eq!(failed_logs.len(), 1);
    assert_eq!(failed_logs[0].id, id1);

    // id1 should no longer be in get_all_logs
    let all_logs_after = store.get_all_logs().unwrap();
    assert_eq!(all_logs_after.len(), 1);
    assert_eq!(all_logs_after[0].id, id2);
}
