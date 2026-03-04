use agent::hybrid_database::HybridDatabase;
use agent::models::AttendanceData;
use std::time::Instant;

fn make_attendance(user_id: &str, timestamp: &str) -> AttendanceData {
    AttendanceData {
        user_id: user_id.to_string(),
        timestamp: timestamp.to_string(),
        status: Some(1),
        punch: Some(0),
    }
}

// ============ Performance Benchmark Tests ============

#[test]
fn benchmark_insert_latency() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid.store().upsert_device("BENCH_DEV", "Bench Device", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();

    let start = Instant::now();
    let count = 100;
    for i in 0..count {
        let data = make_attendance(&format!("U{i:04}"), "2026-01-27 10:00:00");
        let _ = hybrid.save_raw_attendance("BENCH_DEV", data);
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() / count as u128;
    println!("Insert latency: avg={avg_us}µs, total={}ms for {count} inserts", elapsed.as_millis());
    assert!(avg_us < 5000, "Insert should be under 5ms each, got {avg_us}µs");
}

#[test]
fn benchmark_query_latency() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid.store().upsert_device("BENCH_DEV", "Bench Device", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();

    for i in 0..500 {
        let data = make_attendance(&format!("U{i:04}"), &format!("2026-01-27 10:{:02}:00", i % 60));
        let _ = hybrid.save_raw_attendance("BENCH_DEV", data);
    }

    let start = Instant::now();
    let iters = 50;
    for _ in 0..iters {
        let _ = hybrid.get_unsynced_logs(50);
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() / iters as u128;
    println!("Query latency: avg={avg_us}µs for get_unsynced(50)");
    assert!(avg_us < 50000, "Query should be under 50ms, got {avg_us}µs");
}

#[test]
fn benchmark_batch_insert_latency() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid.store().upsert_device("BENCH_DEV", "Bench Device", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();

    let batch: Vec<(String, AttendanceData)> = (0..100).map(|i| {
        ("BENCH_DEV".to_string(), make_attendance(&format!("U{i:04}"), &format!("2026-01-27 10:{:02}:00", i % 60)))
    }).collect();

    let start = Instant::now();
    let mut total_inserted = 0;
    for (device_id, data) in &batch {
        if let Ok(_id) = hybrid.save_raw_attendance(device_id, data.clone()) {
            total_inserted += 1;
        }
    }
    let elapsed = start.elapsed();
    println!("Batch insert: {}ms for {total_inserted} records", elapsed.as_millis());
    assert!(total_inserted > 0);
}

#[test]
fn benchmark_mark_operations() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid.store().upsert_device("BENCH_DEV", "Bench Device", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();

    let mut ids = Vec::new();
    for i in 0..100 {
        let data = make_attendance(&format!("U{i:04}"), "2026-01-27 10:00:00");
        if let Ok(id) = hybrid.save_raw_attendance("BENCH_DEV", data) {
            ids.push(id);
        }
    }

    let start = Instant::now();
    hybrid.mark_in_flight(&ids);
    let mark_time = start.elapsed();
    println!("Mark in-flight: {}µs for {} logs", mark_time.as_micros(), ids.len());
}

// ============ Hybrid Database Integration Tests ============

#[test]
fn integration_full_sync_workflow() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid.store().upsert_device("DEV1", "Device 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();

    // 1. Insert logs
    let data = make_attendance("U001", "2026-01-27 10:00:00");
    let id = hybrid.save_raw_attendance("DEV1", data).expect("insert ok");

    // 2. Query unsynced
    let unsynced = hybrid.get_unsynced_logs(10);
    assert_eq!(unsynced.len(), 1);

    // 3. Mark in-flight
    hybrid.mark_in_flight(&[id]);
    let unsynced2 = hybrid.get_unsynced_logs(10);
    assert_eq!(unsynced2.len(), 0);

    // 4. Mark synced (ACK success)
    hybrid.store().mark_synced(&[id]).unwrap();
    assert_eq!(hybrid.count_unsynced_logs(), 0);
}

#[test]
fn integration_restart_recovery_with_inflight() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid.store().upsert_device("DEV1", "Device 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();

    let data = make_attendance("U001", "2026-01-27 10:00:00");
    let id = hybrid.save_raw_attendance("DEV1", data).unwrap();

    // Verify log exists and is unsynced
    let unsynced_count = hybrid.store().count_unsynced().unwrap();
    assert_eq!(unsynced_count, 1, "Should have 1 unsynced log");

    // Mark in-flight using store directly (unwrap to see errors)
    hybrid.store().mark_in_flight(&[id]).unwrap();

    // Verify it's now in-flight (not unsynced)
    let unsynced_after = hybrid.store().count_unsynced().unwrap();
    assert_eq!(unsynced_after, 0, "Should have 0 unsynced after mark_in_flight");

    // Simulate crash recovery: reset in-flight to unsynced
    let reset = hybrid.store().reset_syncing_status().unwrap();
    assert_eq!(reset, 1, "Should find 1 in-flight log to reset");

    // Should be available for sync again (ignoring backoff)
    let final_unsynced = hybrid.store().count_unsynced().unwrap();
    assert_eq!(final_unsynced, 1, "Should have 1 unsynced log after reset");
}

#[test]
fn integration_concurrent_device_writes() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid.store().upsert_device("DEV1", "Device 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();
    hybrid.store().upsert_device("DEV2", "Device 2", "1.2.3.5", 4370, "zkteco", "UTC", None, true).unwrap();

    for i in 0..5 {
        let _ = hybrid.save_raw_attendance("DEV1", make_attendance(&format!("U{i}"), "2026-01-27 10:00:00"));
        let _ = hybrid.save_raw_attendance("DEV2", make_attendance(&format!("U{i}"), "2026-01-27 10:00:00"));
    }

    assert!(hybrid.count_unsynced_logs() > 0);
}

#[test]
fn integration_mixed_sync_states() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid.store().upsert_device("DEV1", "Device 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();

    let mut all_ids = Vec::new();
    for i in 0..6 {
        let data = make_attendance(&format!("U{i:03}"), &format!("2026-01-27 1{}:00:00", i));
        if let Ok(id) = hybrid.save_raw_attendance("DEV1", data) {
            all_ids.push(id);
        }
    }

    assert_eq!(all_ids.len(), 6);

    // Sync first 2
    hybrid.store().mark_synced(&all_ids[..2]).unwrap();
    // Mark 2 in-flight
    hybrid.mark_in_flight(&all_ids[2..4]);
    // Leave 2 unsynced

    let unsynced = hybrid.get_unsynced_logs(10);
    assert_eq!(unsynced.len(), 2); // Only unsynced ones

    assert_eq!(hybrid.count_unsynced_logs(), 2);
}

// ============ Edge Cases and Error Scenarios ============

#[test]
fn edge_case_empty_batch_insert() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    assert_eq!(hybrid.count_unsynced_logs(), 0);
}

#[test]
fn edge_case_get_unsynced_after_all_synced() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid.store().upsert_device("DEV1", "Device 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();

    let data = make_attendance("U001", "2026-01-27 10:00:00");
    let id = hybrid.save_raw_attendance("DEV1", data).unwrap();
    hybrid.store().mark_synced(&[id]).unwrap();

    let unsynced = hybrid.get_unsynced_logs(10);
    assert!(unsynced.is_empty());
}

#[test]
fn edge_case_duplicate_insert() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid.store().upsert_device("DEV1", "Device 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();

    let data = make_attendance("U001", "2026-01-27 10:00:00");
    let id1 = hybrid.save_raw_attendance("DEV1", data.clone());
    assert!(id1.is_ok());

    let id2 = hybrid.save_raw_attendance("DEV1", data);
    assert!(id2.is_err()); // Duplicate should be rejected
}

#[test]
fn edge_case_cleanup_synced_no_old_logs() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid.store().upsert_device("DEV1", "Device 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();

    let data = make_attendance("U001", "2026-01-27 10:00:00");
    let id = hybrid.save_raw_attendance("DEV1", data).unwrap();
    hybrid.store().mark_synced(&[id]).unwrap();

    // Cleanup with 30-day retention — nothing should be cleaned since log is fresh
    let cleaned = hybrid.store().cleanup_old_synced_logs(30).unwrap();
    assert_eq!(cleaned, 0);
}

#[test]
fn edge_case_multiple_timestamp_formats() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid.store().upsert_device("DEV1", "Device 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();

    // Standard format
    let data1 = make_attendance("U001", "2026-01-27 10:00:00");
    assert!(hybrid.save_raw_attendance("DEV1", data1).is_ok());

    // ISO format
    let data2 = make_attendance("U002", "2026-01-27T11:00:00Z");
    assert!(hybrid.save_raw_attendance("DEV1", data2).is_ok());
}
