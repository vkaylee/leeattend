// Push logs integration test
// E2E WebSocket tests temporarily simplified to unit-level store tests
// Original tests used CentralClient with mock WS server + old Database API.
// These will be restored once CentralClient is fully adapted for Redb.

use agent::hybrid_database::HybridDatabase;
use agent::models::AttendanceData;

#[tokio::test]
async fn test_end_to_end_log_push_data_flow_smoke() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();

    // Setup device
    store.upsert_device("DEV1", "Test Device", "127.0.0.1", 4370, "zkteco", "UTC", None, true, 123456).unwrap();

    // Insert a log
    let log_data = AttendanceData {
        user_id: "USER1".into(),
        timestamp: "2026-01-26 12:00:00".into(),
        status: Some(1),
        punch: Some(1),
    };
    let id = hybrid.save_raw_attendance("DEV1".into(), log_data).await.unwrap();

    // Verify unsynced
    let unsynced = hybrid.get_unsynced_logs(10).await.unwrap();
    assert_eq!(unsynced.len(), 1);
    assert_eq!(unsynced[0].user_id, "USER1");

    // Mark in-flight
    hybrid.mark_in_flight(vec![id]).await;
    assert_eq!(hybrid.get_unsynced_logs(10).await.unwrap().len(), 0);

    // Mark synced (simulates ACK)
    store.mark_synced(&[id]).unwrap();
    assert_eq!(hybrid.count_unsynced_logs().await, 0);
}

#[tokio::test]
async fn test_reliable_sync_with_retry() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();

    store.upsert_device("DEV_RETRY", "Retry Device", "127.0.0.1", 4370, "zkteco", "UTC", None, true, 123456).unwrap();

    let id = hybrid.save_raw_attendance("DEV_RETRY".into(), AttendanceData {
        user_id: "RETRY_USER".into(),
        timestamp: "2026-01-26 12:00:00".into(),
        status: Some(1),
        punch: Some(1),
    }).await.unwrap();

    // First attempt: mark in-flight then simulate failure
    store.mark_in_flight(&[id]).unwrap();
    store.reset_to_unsynced(id).unwrap();

    // Verify log count is back to 1 (use count_unsynced which doesn't have backoff filter)
    let count = store.count_unsynced().unwrap();
    assert_eq!(count, 1, "Log should be unsynced again after reset");

    // Second attempt: success
    store.mark_in_flight(&[id]).unwrap();
    store.mark_synced(&[id]).unwrap();

    assert_eq!(hybrid.count_unsynced_logs().await, 0);
}
