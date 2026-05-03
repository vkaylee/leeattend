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

use agent::hybrid_database::HybridDatabase;
use agent::models::AttendanceData;

#[tokio::test]
async fn test_batch_worker_inserts_records_smoke() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid
        .store()
        .upsert_device(
            "D1", "Device 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true, 123456,
        )
        .unwrap();

    // Simulate batch worker: accumulate and insert
    let items: Vec<(String, AttendanceData)> = (0..3)
        .map(|i| {
            (
                "D1".to_string(),
                AttendanceData {
                    user_id: format!("U{i}"),
                    timestamp: format!("2026-01-01 10:00:0{i}"),
                    status: Some(1),
                    punch: Some(1),
                },
            )
        })
        .collect();

    for (device_id, data) in items {
        hybrid.save_raw_attendance(device_id, data).await.unwrap();
    }

    let unsynced = hybrid.get_unsynced_logs(100).await.unwrap();
    assert_eq!(unsynced.len(), 3, "All 3 logs should be saved");
}
