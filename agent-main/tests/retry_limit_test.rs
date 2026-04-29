use agent::hybrid_database::HybridDatabase;
use agent::models::AttendanceData;

/// Test retry count increments on each mark_in_flight call
#[tokio::test]
async fn test_retry_count_increments_smoke() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();
    store
        .upsert_device(
            "device1", "Dev 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true, 123456,
        )
        .unwrap();

    let data = AttendanceData {
        user_id: "U001".to_string(),
        timestamp: "2026-01-28 08:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };
    let id = hybrid
        .save_raw_attendance("device1".into(), data)
        .await
        .unwrap();

    for expected_count in 1..=3i64 {
        store.mark_in_flight(&[id]).unwrap();
        store.reset_to_unsynced(id).unwrap();

        // Use count_unsynced (no backoff filter) to verify log still exists
        assert_eq!(
            store.count_unsynced().unwrap(),
            1,
            "Log should still be unsynced after retry {expected_count}"
        );
    }
}

/// Test logs exceeding max retry count are moved to DLQ
#[tokio::test]
async fn test_max_retry_limit_moves_to_dlq() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();
    store
        .upsert_device(
            "device1", "Dev 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true, 123456,
        )
        .unwrap();

    let data = AttendanceData {
        user_id: "U001".to_string(),
        timestamp: "2026-01-28 08:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };
    let id = hybrid
        .save_raw_attendance("device1".into(), data)
        .await
        .unwrap();

    // Simulate 10 failed attempts
    for _ in 0..10 {
        store.mark_in_flight(&[id]).unwrap();
        store.reset_to_unsynced(id).unwrap();
    }

    // Move expired logs (max_retries=10)
    let moved = store.move_expired_logs_to_dlq(10).unwrap();
    assert_eq!(moved, 1, "Should move 1 log to DLQ");

    let remaining = store.count_unsynced().unwrap();
    assert_eq!(remaining, 0, "Should have no unsynced logs");

    let failed = store.get_failed_logs().unwrap();
    assert_eq!(failed.len(), 1);
    assert!(failed[0].failure_reason.contains("Max retries"));
}

/// Test logs within limits are NOT moved to DLQ
#[tokio::test]
async fn test_valid_retries_not_moved_to_dlq() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();
    store
        .upsert_device(
            "device1", "Dev 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true, 123456,
        )
        .unwrap();

    let data = AttendanceData {
        user_id: "U001".to_string(),
        timestamp: "2026-01-28 10:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };
    let id = hybrid
        .save_raw_attendance("device1".into(), data)
        .await
        .unwrap();

    // Simulate 5 retries (below limit of 10)
    for _ in 0..5 {
        store.mark_in_flight(&[id]).unwrap();
        store.reset_to_unsynced(id).unwrap();
    }

    let moved = store.move_expired_logs_to_dlq(10).unwrap();
    assert_eq!(moved, 0, "Should NOT move any logs (within limits)");

    let remaining = store.count_unsynced().unwrap();
    assert_eq!(remaining, 1, "Log should still be unsynced");
}

/// Test configurable retry limits via agent_config
#[tokio::test]
async fn test_configurable_retry_limits() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();

    store.upsert_agent_config("max_retry_count", "5").unwrap();
    store
        .upsert_device(
            "config_device",
            "Cfg Dev",
            "1.2.3.4",
            4370,
            "zkteco",
            "UTC",
            None,
            true,
            123456,
        )
        .unwrap();

    assert_eq!(
        store.get_agent_config("max_retry_count").unwrap(),
        Some("5".to_string())
    );

    let data = AttendanceData {
        user_id: "UCONFIG".to_string(),
        timestamp: "2026-01-28 13:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };
    let id = hybrid
        .save_raw_attendance("config_device".into(), data)
        .await
        .unwrap();

    for _ in 0..5 {
        store.mark_in_flight(&[id]).unwrap();
        store.reset_to_unsynced(id).unwrap();
    }

    let moved = store.move_expired_logs_to_dlq(5).unwrap();
    assert_eq!(moved, 1, "Should move log with custom limit of 5");
}

/// Test DLQ preserves log metadata
#[tokio::test]
async fn test_dlq_preserves_metadata() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();
    store
        .upsert_device(
            "metadata_test",
            "Meta Dev",
            "1.2.3.4",
            4370,
            "zkteco",
            "UTC",
            None,
            true,
            123456,
        )
        .unwrap();

    let data = AttendanceData {
        user_id: "U999".to_string(),
        timestamp: "2026-01-28 12:30:00".to_string(),
        status: Some(1),
        punch: Some(3),
    };
    let id = hybrid
        .save_raw_attendance("metadata_test".into(), data)
        .await
        .unwrap();

    for _ in 0..10 {
        store.mark_in_flight(&[id]).unwrap();
        store.reset_to_unsynced(id).unwrap();
    }

    store.move_expired_logs_to_dlq(10).unwrap();

    let failed = store.get_failed_logs().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].device_id, "metadata_test");
    assert_eq!(failed[0].user_id, "U999");
    assert_eq!(failed[0].status, Some(1));
    assert_eq!(failed[0].punch, Some(3));
    assert_eq!(failed[0].retry_attempts, 10);
}
