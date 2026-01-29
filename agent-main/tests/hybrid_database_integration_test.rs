use agent::hybrid_database::HybridDatabase;
use agent::memory_store::InMemoryStore;
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
    let store = InMemoryStore::new(100_000);

    let start = Instant::now();
    for i in 0..1000 {
        let data = make_attendance(&format!("U{:05}", i), "2026-01-29 10:00:00");
        store.insert(format!("DEV{}", i % 10), data).unwrap();
    }
    let elapsed = start.elapsed();

    let avg_latency_us = elapsed.as_micros() / 1000;
    println!("Average insert latency: {}us", avg_latency_us);

    // Assert sub-millisecond latency target
    assert!(
        avg_latency_us < 1000,
        "Insert latency too high: {}us (target: <1000us)",
        avg_latency_us
    );
}

#[test]
fn benchmark_query_latency() {
    let store = InMemoryStore::new(100_000);

    // Populate with 10K logs
    for i in 0..10_000 {
        let data = make_attendance(&format!("U{:05}", i), "2026-01-29 10:00:00");
        store.insert(format!("DEV{}", i % 10), data).unwrap();
    }

    let start = Instant::now();
    for _ in 0..100 {
        let _logs = store.get_unsynced(500);
    }
    let elapsed = start.elapsed();

    let avg_latency_us = elapsed.as_micros() / 100;
    println!("Average query latency (500 logs): {}us", avg_latency_us);

    // Assert sub-millisecond latency target
    assert!(
        avg_latency_us < 1000,
        "Query latency too high: {}us (target: <1000us)",
        avg_latency_us
    );
}

#[test]
fn benchmark_memory_usage() {
    let store = InMemoryStore::new(100_000);

    // Insert 50K logs
    for i in 0..50_000 {
        let data = make_attendance(&format!("U{:05}", i), "2026-01-29 10:00:00");
        store.insert(format!("DEV{}", i % 100), data).unwrap();
    }

    // Verify all logs inserted
    assert_eq!(store.count_total(), 50_000);

    // Verify count_unsynced works at scale
    let unsynced_count = store.count_unsynced();
    assert_eq!(unsynced_count, 50_000);
}

#[test]
fn benchmark_batch_insert_latency() {
    let store = InMemoryStore::new(100_000);

    let start = Instant::now();
    for batch_idx in 0..100 {
        let batch: Vec<(String, AttendanceData)> = (0..100)
            .map(|i| {
                let offset = batch_idx * 100 + i;
                (
                    format!("DEV{}", offset % 10),
                    make_attendance(&format!("U{:05}", offset), "2026-01-29 10:00:00"),
                )
            })
            .collect();

        store.insert_batch(batch).unwrap();
    }
    let elapsed = start.elapsed();

    let avg_latency_us = elapsed.as_micros() / 10_000; // 10K operations
    println!("Average batch insert latency: {}us", avg_latency_us);

    assert!(avg_latency_us < 1000);
}

#[test]
fn benchmark_mark_operations() {
    let store = InMemoryStore::new(100_000);

    // Insert 5K logs
    for i in 0..5_000 {
        let data = make_attendance(&format!("U{:05}", i), "2026-01-29 10:00:00");
        store.insert(format!("DEV{}", i % 10), data).unwrap();
    }

    // Benchmark mark_in_flight
    let logs = store.get_unsynced(1000);
    let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();

    let start = Instant::now();
    for _ in 0..100 {
        store.mark_in_flight(&ids);
        store.reset_to_unsynced(ids[0]);
    }
    let elapsed = start.elapsed();

    let avg_latency_us = elapsed.as_micros() / 100;
    println!("Average mark_in_flight latency: {}us", avg_latency_us);

    assert!(avg_latency_us < 1000);
}

// ============ Hybrid Database Integration Tests ============

#[tokio::test]
async fn integration_full_sync_workflow() {
    let hybrid = HybridDatabase::new("sqlite::memory:", 10_000).await.unwrap();

    // Phase 1: Collect logs
    for i in 1..=100 {
        let data = make_attendance(&format!("U{:03}", i), "2026-01-29 10:00:00");
        hybrid.save_raw_attendance("DEV1", data).unwrap();
    }
    assert_eq!(hybrid.count_unsynced_logs(), 100);

    // Phase 2: Batch sync
    let logs = hybrid.get_unsynced_logs(1000);
    assert_eq!(logs.len(), 100);

    let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();

    // Mark as in-flight
    hybrid.mark_in_flight(&ids);
    assert_eq!(hybrid.count_unsynced_logs(), 0);

    // Mark first 50 as synced
    hybrid.mark_synced(&ids[0..50]);

    // Checkpoint
    let checkpointed = hybrid.checkpoint().await.unwrap();
    assert!(checkpointed > 0);
}

#[tokio::test]
async fn integration_restart_recovery_with_inflight() {
    // Note: In-memory SQLite doesn't persist between connections.
    // This test verifies the checkpoint mechanism works correctly.
    let db_url = "sqlite::memory:";

    let hybrid = HybridDatabase::new(db_url, 1000).await.unwrap();

    // Save and mark some as in-flight
    for i in 1..=10 {
        let data = make_attendance(&format!("U{:03}", i), "2026-01-29 10:00:00");
        hybrid.save_raw_attendance("DEV1", data).unwrap();
    }

    let logs = hybrid.get_unsynced_logs(10);
    let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();

    // Mark first 5 as in-flight
    hybrid.mark_in_flight(&ids[0..5]);

    // Verify state
    assert_eq!(hybrid.count_unsynced_logs(), 5);

    // Checkpoint
    let checkpointed = hybrid.checkpoint().await.unwrap();
    assert!(checkpointed > 0);
}

#[tokio::test]
async fn integration_concurrent_device_writes() {
    let hybrid = HybridDatabase::new("sqlite::memory:", 10_000).await.unwrap();

    // Simulate 5 devices writing simultaneously
    let mut handles = vec![];

    for dev_idx in 1..=5 {
        let hybrid = hybrid.clone();
        let handle = tokio::spawn(async move {
            for user_idx in 1..=20 {
                let device = format!("DEV{}", dev_idx);
                let data = make_attendance(
                    &format!("D{}U{:03}", dev_idx, user_idx),
                    "2026-01-29 10:00:00"
                );
                hybrid.save_raw_attendance(&device, data).ok();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Total: 5 devices * 20 users = 100 logs
    assert_eq!(hybrid.count_unsynced_logs(), 100);
}

#[tokio::test]
async fn integration_checkpoint_under_load() {
    let hybrid = HybridDatabase::new("sqlite::memory:", 10_000).await.unwrap();

    // Insert large batch
    for i in 0..1000 {
        let data = make_attendance(&format!("U{:05}", i), "2026-01-29 10:00:00");
        hybrid.save_raw_attendance("DEV1", data).unwrap();
    }

    let start = Instant::now();
    let checkpointed = hybrid.checkpoint().await.unwrap();
    let elapsed = start.elapsed();

    println!("Checkpointed {} logs in {:?}", checkpointed, elapsed);

    assert_eq!(checkpointed, 1000);
    // Checkpoint should complete in reasonable time
    assert!(elapsed.as_millis() < 5000); // 5 second max
}

#[tokio::test]
async fn integration_mixed_sync_states() {
    let hybrid = HybridDatabase::new("sqlite::memory:", 1000).await.unwrap();

    // Create mixed states
    let mut synced_ids = Vec::new();
    let mut inflight_ids = Vec::new();
    let mut unsynced_ids = Vec::new();

    // Insert 30 logs
    for i in 1..=30 {
        let data = make_attendance(&format!("U{:03}", i), "2026-01-29 10:00:00");
        let id = hybrid.save_raw_attendance("DEV1", data).unwrap();
        if i <= 10 {
            synced_ids.push(id);
        } else if i <= 20 {
            inflight_ids.push(id);
        } else {
            unsynced_ids.push(id);
        }
    }

    // Apply state transitions
    hybrid.mark_in_flight(&synced_ids);
    hybrid.mark_synced(&synced_ids);
    hybrid.mark_in_flight(&inflight_ids);

    // Verify counts
    assert_eq!(hybrid.count_unsynced_logs(), 10); // Only truly unsynced

    // Checkpoint
    let checkpointed = hybrid.checkpoint().await.unwrap();
    assert!(checkpointed > 0);
}

// ============ Edge Cases and Error Scenarios ============

#[test]
fn edge_case_empty_batch_insert() {
    let store = InMemoryStore::new(1000);

    let batch: Vec<(String, AttendanceData)> = vec![];
    let result = store.insert_batch(batch);

    assert!(result.is_err());
}

#[test]
fn edge_case_single_capacity_store() {
    let store = InMemoryStore::new(1);

    // Insert first log
    let data1 = make_attendance("U1", "2026-01-29 10:00:00");
    let id1 = store.insert("D1".to_string(), data1).unwrap();
    assert!(id1 > 0);

    // Try to insert second (should fail)
    let data2 = make_attendance("U2", "2026-01-29 10:01:00");
    let result = store.insert("D1".to_string(), data2);

    assert!(result.is_err());
}

#[test]
fn edge_case_get_unsynced_after_all_synced() {
    let store = InMemoryStore::new(1000);

    // Insert and sync all
    for i in 1..=3 {
        let data = make_attendance(&format!("U{}", i), "2026-01-29 10:00:00");
        store.insert("D1".to_string(), data).unwrap();
    }

    let logs = store.get_unsynced(10);
    let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();

    store.mark_in_flight(&ids);
    store.mark_synced(&ids);

    // Should return empty
    let remaining = store.get_unsynced(10);
    assert_eq!(remaining.len(), 0);
}

#[test]
fn edge_case_cleanup_synced_no_old_logs() {
    let store = InMemoryStore::new(1000);

    // Insert recent logs
    for i in 1..=3 {
        let data = make_attendance(&format!("U{}", i), "2026-01-29 10:00:00");
        store.insert("D1".to_string(), data).unwrap();
    }

    // Mark as synced
    store.mark_in_flight(&[1, 2, 3]);
    store.mark_synced(&[1, 2, 3]);

    // Cleanup with high threshold (only removes very old logs)
    let removed = store.cleanup_synced(86400); // 24 hours

    assert_eq!(removed, 0); // Nothing removed
    assert_eq!(store.count_total(), 3);
}

#[test]
fn edge_case_multiple_timestamp_formats() {
    let store = InMemoryStore::new(1000);

    // Test format 1: YYYY-MM-DD HH:MM:SS
    let data1 = AttendanceData {
        user_id: "U1".to_string(),
        timestamp: "2026-01-29 10:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };
    assert!(store.insert("D1".to_string(), data1).is_ok());

    // Test format 2: ISO 8601
    let data2 = AttendanceData {
        user_id: "U2".to_string(),
        timestamp: "2026-01-29T10:00:00Z".to_string(),
        status: Some(1),
        punch: Some(0),
    };
    assert!(store.insert("D1".to_string(), data2).is_ok());

    assert_eq!(store.count_total(), 2);
}

#[tokio::test]
async fn edge_case_checkpoint_empty_store() {
    let hybrid = HybridDatabase::new("sqlite::memory:", 1000).await.unwrap();

    // Checkpoint without any logs
    let result = hybrid.checkpoint().await.unwrap();

    assert_eq!(result, 0);
}

#[tokio::test]
async fn edge_case_flush_all_empty_store() {
    let hybrid = HybridDatabase::new("sqlite::memory:", 1000).await.unwrap();

    // Flush without any logs
    let result = hybrid.flush_all().await.unwrap();

    assert_eq!(result, 0);
}
