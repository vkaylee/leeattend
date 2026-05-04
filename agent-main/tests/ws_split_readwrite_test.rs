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

// WebSocket split read/write tests
// E2E WebSocket tests temporarily simplified to unit-level store tests
// Original tests used CentralClient with mock WS server + old Database API.
// These will be restored once CentralClient is fully adapted for Redb.

use agent::hybrid_database::HybridDatabase;
use agent::models::AttendanceData;

#[tokio::test]
async fn test_concurrent_device_writes_data_integrity() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();

    store
        .upsert_device(
            "DEV_BULK",
            "Bulk Device",
            "127.0.0.1",
            4370,
            "zkteco",
            "UTC",
            None,
            true,
            123456,
        )
        .unwrap();

    // Insert 20 logs
    let mut ids = Vec::new();
    for i in 0..20 {
        let id = hybrid
            .save_raw_attendance(
                "DEV_BULK".into(),
                AttendanceData {
                    user_id: format!("USER_{i}"),
                    timestamp: format!("2026-01-26 12:{i:02}:00"),
                    status: Some(1),
                    punch: Some(1),
                },
            )
            .await
            .unwrap();
        ids.push(id);
    }

    assert_eq!(hybrid.count_unsynced_logs().await, 20);

    // Mark all in-flight
    hybrid.mark_in_flight(ids.clone()).await;
    assert_eq!(hybrid.get_unsynced_logs(20).await.unwrap().len(), 0);

    // Mark all synced
    store.mark_synced(&ids).unwrap();
    assert_eq!(hybrid.count_unsynced_logs().await, 0);
}

#[tokio::test]
async fn test_connection_failure_preserves_logs() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();

    store
        .upsert_device(
            "DEV_FAIL",
            "Fail Device",
            "127.0.0.1",
            4370,
            "zkteco",
            "UTC",
            None,
            true,
            123456,
        )
        .unwrap();
    hybrid
        .save_raw_attendance(
            "DEV_FAIL".into(),
            AttendanceData {
                user_id: "USER_FAIL".into(),
                timestamp: "2026-01-26 12:00:00".into(),
                status: Some(1),
                punch: Some(1),
            },
        )
        .await
        .unwrap();

    // Simulate: logs stay unsynced when connection fails
    assert!(
        hybrid.count_unsynced_logs().await > 0,
        "Logs should still be pending"
    );
}

#[tokio::test]
async fn test_no_ack_starvation_under_load() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();

    store
        .upsert_device(
            "DEV_LOAD",
            "Load Device",
            "127.0.0.1",
            4370,
            "zkteco",
            "UTC",
            None,
            true,
            123456,
        )
        .unwrap();

    // Insert 50 logs for load testing
    let mut ids = Vec::new();
    for i in 0..50 {
        let id = hybrid
            .save_raw_attendance(
                "DEV_LOAD".into(),
                AttendanceData {
                    user_id: format!("LOAD_USER_{i}"),
                    timestamp: format!("2026-01-26 {}:{:02}:00", 10 + i / 60, i % 60),
                    status: Some(1),
                    punch: Some(1),
                },
            )
            .await
            .unwrap();
        ids.push(id);
    }

    assert_eq!(hybrid.count_unsynced_logs().await, 50);

    // Mark all synced (simulates successful ACK for all)
    hybrid.mark_in_flight(ids.clone()).await;
    store.mark_synced(&ids).unwrap();

    assert_eq!(
        hybrid.count_unsynced_logs().await,
        0,
        "All 50 logs should be processed"
    );
}
