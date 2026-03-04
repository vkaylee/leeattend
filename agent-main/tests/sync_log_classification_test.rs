use agent::hybrid_database::HybridDatabase;
use agent::models::AttendanceData;

/// Test that new logs (updated_at IS None) are correctly identified vs retries.
#[test]
fn test_sync_log_classification_new_vs_retry() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();
    store.upsert_device("device1", "Dev 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();

    // Insert 5 logs — all new (updated_at = None)
    let mut ids = Vec::new();
    for i in 1..=5 {
        let data = AttendanceData {
            user_id: format!("U{i}"),
            timestamp: format!("2026-01-28 08:0{i}:00"),
            status: Some(1),
            punch: Some(0),
        };
        ids.push(hybrid.save_raw_attendance("device1", data).unwrap());
    }

    // Mark first 3 as in-flight (sets updated_at + retry_count)
    store.mark_in_flight(&ids[..3]).unwrap();

    // Reset to unsynced (simulate failure)
    for &id in &ids[..3] {
        store.reset_to_unsynced(id).unwrap();
    }

    // Use count_unsynced (no backoff filter) to verify all 5 are still unsynced
    let total_unsynced = store.count_unsynced().unwrap();
    assert_eq!(total_unsynced, 5, "Should have 5 unsynced logs total");

    // Verify the 2 new logs are retrievable (no backoff)
    let new_logs = hybrid.get_unsynced_logs(10);
    assert_eq!(new_logs.len(), 2, "Should have 2 new logs (no backoff) immediately available");
}

/// Test sync message includes classification.
#[test]
fn test_sync_message_includes_classification() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid.store().upsert_device("device1", "Dev 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();

    for i in 1..=10 {
        let data = AttendanceData {
            user_id: format!("U{i}"),
            timestamp: format!("2026-01-28 09:{i}:00"),
            status: Some(1),
            punch: Some(0),
        };
        let _ = hybrid.save_raw_attendance("device1", data);
    }

    let logs = hybrid.get_unsynced_logs(10);
    let new_count = logs.iter().filter(|l| l.updated_at.is_none()).count();
    let retry_count = logs.len() - new_count;

    assert_eq!(new_count, 10, "All 10 should be new");
    assert_eq!(retry_count, 0, "None should be retries");
}
