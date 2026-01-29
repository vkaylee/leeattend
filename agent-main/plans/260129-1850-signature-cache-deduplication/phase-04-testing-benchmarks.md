---
phase: 4
title: "Testing and Benchmarks"
status: complete
effort: 2h
blocked_by: [phase-03]
completed: 2026-01-29
---

# Phase 4: Testing & Benchmarks

## Context

**Related files:**
- `/home/elt1541/leeattend/agent-main/src/memory_store.rs`
- `/home/elt1541/leeattend/agent-main/src/hybrid_database.rs`
- `/home/elt1541/leeattend/agent-main/src/database.rs`

**Prerequisite:** Phases 1-3 complete

## Overview

Comprehensive testing and benchmarking to validate signature cache implementation.

## Test Categories

### 1. Unit Tests (memory_store.rs)

| Test Name | Description | Priority |
|-----------|-------------|----------|
| `test_compute_signature_deterministic` | Same input → same hash | P1 |
| `test_compute_signature_different_inputs` | Different inputs → different hash | P1 |
| `test_insert_rejects_duplicate` | Duplicate log rejected | P1 |
| `test_cleanup_preserves_signatures` | Cleanup keeps signatures | P1 |
| `test_batch_insert_skips_duplicates` | Batch handles duplicates | P1 |
| `test_load_from_disk_populates_signatures` | Startup populates index | P1 |
| `test_signature_count` | count_signatures() works | P2 |
| `test_has_signature` | has_signature() works | P2 |

### 2. Integration Tests (hybrid_database.rs)

| Test Name | Description | Priority |
|-----------|-------------|----------|
| `test_full_lifecycle_with_dedup` | Insert → sync → cleanup → reject duplicate | P1 |
| `test_startup_recovery_with_signatures` | Restart loads signatures correctly | P1 |
| `test_concurrent_insert_dedup` | Multi-thread duplicate handling | P2 |

### 3. Performance Benchmarks

| Benchmark | Target | Methodology |
|-----------|--------|-------------|
| Startup time (100K logs) | < 2s | `Instant::now()` measurement |
| Insert throughput | > 100K/s | Batch insert timing |
| Memory usage (100K sigs) | < 5MB | External monitoring |
| Duplicate check latency | < 1μs | Micro-benchmark |

## Implementation Steps

### Step 1: Core Unit Tests (30 min)

```rust
// memory_store.rs - add to #[cfg(test)] mod tests

#[test]
fn test_compute_signature_deterministic() {
    let ts = NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

    // Multiple calls with same input
    let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", ts);
    let sig2 = InMemoryStore::compute_signature("DEV1", "USER001", ts);
    let sig3 = InMemoryStore::compute_signature("DEV1", "USER001", ts);

    assert_eq!(sig1, sig2);
    assert_eq!(sig2, sig3);
}

#[test]
fn test_compute_signature_different_devices() {
    let ts = NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

    let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", ts);
    let sig2 = InMemoryStore::compute_signature("DEV2", "USER001", ts);

    assert_ne!(sig1, sig2, "Different device should produce different signature");
}

#[test]
fn test_compute_signature_different_users() {
    let ts = NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

    let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", ts);
    let sig2 = InMemoryStore::compute_signature("DEV1", "USER002", ts);

    assert_ne!(sig1, sig2, "Different user should produce different signature");
}

#[test]
fn test_compute_signature_different_timestamps() {
    let ts1 = NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let ts2 = NaiveDateTime::parse_from_str("2026-01-29 10:00:01", "%Y-%m-%d %H:%M:%S").unwrap();

    let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", ts1);
    let sig2 = InMemoryStore::compute_signature("DEV1", "USER001", ts2);

    assert_ne!(sig1, sig2, "Different timestamp should produce different signature");
}

#[test]
fn test_insert_rejects_exact_duplicate() {
    let store = InMemoryStore::new(1000);

    let data = AttendanceData {
        user_id: "U001".to_string(),
        timestamp: "2026-01-29 10:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };

    // First insert succeeds
    let result1 = store.insert("DEV1".to_string(), data.clone());
    assert!(result1.is_ok());

    // Second insert fails
    let result2 = store.insert("DEV1".to_string(), data);
    assert!(matches!(result2, Err(StoreError::Duplicate)));

    // Only 1 log in store
    assert_eq!(store.count_total(), 1);
    assert_eq!(store.count_signatures(), 1);
}

#[test]
fn test_insert_allows_different_timestamp() {
    let store = InMemoryStore::new(1000);

    let data1 = AttendanceData {
        user_id: "U001".to_string(),
        timestamp: "2026-01-29 10:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };
    let data2 = AttendanceData {
        user_id: "U001".to_string(),
        timestamp: "2026-01-29 10:00:01".to_string(), // 1 second later
        status: Some(1),
        punch: Some(0),
    };

    // Both should succeed
    assert!(store.insert("DEV1".to_string(), data1).is_ok());
    assert!(store.insert("DEV1".to_string(), data2).is_ok());

    assert_eq!(store.count_total(), 2);
    assert_eq!(store.count_signatures(), 2);
}

#[test]
fn test_cleanup_synced_preserves_signatures() {
    let store = InMemoryStore::new(1000);

    let data = AttendanceData {
        user_id: "U001".to_string(),
        timestamp: "2026-01-29 10:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };

    // Insert and sync
    let id = store.insert("DEV1".to_string(), data.clone()).unwrap();
    store.mark_in_flight(&[id]);
    store.mark_synced(&[id]);

    // Set old created_at
    if let Some(mut log) = store.logs.get_mut(&id) {
        log.created_at = chrono::Utc::now().naive_utc() - chrono::Duration::hours(2);
    }

    // Cleanup (1 hour threshold)
    let cleaned = store.cleanup_synced(3600);
    assert_eq!(cleaned, 1);

    // Log removed, but signature remains
    assert_eq!(store.count_total(), 0);
    assert_eq!(store.count_signatures(), 1);

    // Duplicate still rejected
    let result = store.insert("DEV1".to_string(), data);
    assert!(matches!(result, Err(StoreError::Duplicate)));
}

#[test]
fn test_batch_insert_handles_duplicates() {
    let store = InMemoryStore::new(1000);

    let batch = vec![
        ("DEV1".to_string(), AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        }),
        ("DEV1".to_string(), AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(), // Duplicate of first
            status: Some(1),
            punch: Some(0),
        }),
        ("DEV1".to_string(), AttendanceData {
            user_id: "U002".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(), // Different user
            status: Some(1),
            punch: Some(0),
        }),
    ];

    let inserted = store.insert_batch(batch).unwrap();

    assert_eq!(inserted, 2, "Should insert 2, skip 1 duplicate");
    assert_eq!(store.count_total(), 2);
    assert_eq!(store.count_signatures(), 2);
}

#[test]
fn test_concurrent_inserts_no_duplicates() {
    use std::sync::Arc;
    use std::thread;

    let store = Arc::new(InMemoryStore::new(10000));
    let mut handles = vec![];

    // 10 threads trying to insert same log
    for _ in 0..10 {
        let store_clone = store.clone();
        let handle = thread::spawn(move || {
            let data = AttendanceData {
                user_id: "CONCURRENT_USER".to_string(),
                timestamp: "2026-01-29 12:00:00".to_string(),
                status: Some(1),
                punch: Some(0),
            };
            store_clone.insert("CONCURRENT_DEV".to_string(), data)
        });
        handles.push(handle);
    }

    let mut success_count = 0;
    let mut duplicate_count = 0;

    for handle in handles {
        match handle.join().unwrap() {
            Ok(_) => success_count += 1,
            Err(StoreError::Duplicate) => duplicate_count += 1,
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    // Exactly 1 should succeed, 9 should be duplicates
    assert_eq!(success_count, 1, "Exactly 1 insert should succeed");
    assert_eq!(duplicate_count, 9, "9 should be duplicates");
    assert_eq!(store.count_total(), 1);
    assert_eq!(store.count_signatures(), 1);
}
```

### Step 2: Integration Tests (30 min)

```rust
// hybrid_database.rs tests

#[tokio::test]
async fn test_full_dedup_lifecycle() {
    let hybrid = HybridDatabase::new("sqlite::memory:", 1000).await.unwrap();

    let data = make_attendance("U001", "2026-01-29 10:00:00");

    // 1. Insert
    let id = hybrid.save_raw_attendance("DEV1", data.clone()).unwrap();
    assert!(id > 0);

    // 2. Sync
    hybrid.mark_in_flight(&[id]);
    hybrid.mark_synced(&[id]);

    // 3. Checkpoint
    hybrid.checkpoint().await.unwrap();

    // 4. Cleanup (set old timestamp first)
    // ... manual timestamp manipulation ...

    // 5. Try duplicate - should fail
    let result = hybrid.save_raw_attendance("DEV1", data);
    assert!(matches!(result, Err(StoreError::Duplicate)));
}

#[tokio::test]
async fn test_signatures_survive_restart_simulation() {
    // This test simulates what happens on restart
    // by loading from a pre-populated database

    // Phase 1: Create and checkpoint
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_url = format!("sqlite:{}", temp_db.path().display());

    {
        let hybrid = HybridDatabase::new(&db_url, 1000).await.unwrap();

        for i in 1..=10 {
            let data = make_attendance(&format!("U{:03}", i), "2026-01-29 10:00:00");
            hybrid.save_raw_attendance("DEV1", data).unwrap();
        }

        // Checkpoint to disk
        hybrid.checkpoint().await.unwrap();

        // Mark some as synced
        let logs = hybrid.get_unsynced_logs(5);
        let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        hybrid.mark_in_flight(&ids);
        hybrid.mark_synced(&ids);

        // Final checkpoint
        hybrid.flush_all().await.unwrap();
    }

    // Phase 2: Simulate restart
    {
        let hybrid = HybridDatabase::new(&db_url, 1000).await.unwrap();

        // All 10 signatures should be loaded
        assert_eq!(hybrid.memory_store().count_signatures(), 10);

        // Try to insert duplicate of synced log
        let data = make_attendance("U001", "2026-01-29 10:00:00");
        let result = hybrid.save_raw_attendance("DEV1", data);
        assert!(matches!(result, Err(StoreError::Duplicate)));
    }
}
```

### Step 3: Performance Benchmarks (30 min)

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    #[ignore] // Run with: cargo test --release benchmark -- --ignored --nocapture
    fn benchmark_insert_throughput() {
        let store = InMemoryStore::new(200_000);
        let count = 100_000;

        let start = Instant::now();

        for i in 0..count {
            let data = AttendanceData {
                user_id: format!("U{:06}", i),
                timestamp: format!("2026-01-29 {:02}:{:02}:00", i / 3600 % 24, i / 60 % 60),
                status: Some(1),
                punch: Some(0),
            };
            let _ = store.insert(format!("DEV{}", i % 10), data);
        }

        let elapsed = start.elapsed();
        let throughput = count as f64 / elapsed.as_secs_f64();

        println!("Insert throughput: {:.0} logs/sec", throughput);
        println!("Total time: {:?}", elapsed);
        println!("Signatures: {}", store.count_signatures());

        assert!(throughput > 100_000.0, "Should exceed 100K logs/sec");
    }

    #[test]
    #[ignore]
    fn benchmark_duplicate_check_latency() {
        let store = InMemoryStore::new(100_000);

        // Pre-populate with 50K logs
        for i in 0..50_000 {
            let data = AttendanceData {
                user_id: format!("U{:06}", i),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(0),
            };
            let _ = store.insert(format!("DEV{}", i % 10), data);
        }

        // Benchmark duplicate check
        let iterations = 100_000;
        let ts = NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let start = Instant::now();
        for i in 0..iterations {
            let _ = store.has_signature(&format!("DEV{}", i % 10), &format!("U{:06}", i % 50_000), ts);
        }
        let elapsed = start.elapsed();

        let avg_ns = elapsed.as_nanos() / iterations as u128;
        println!("Duplicate check: {} ns/op", avg_ns);

        assert!(avg_ns < 1000, "Should be < 1μs per check");
    }

    #[tokio::test]
    #[ignore]
    async fn benchmark_startup_time() {
        let temp_db = tempfile::NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_db.path().display());

        // Create DB with 100K logs
        {
            let hybrid = HybridDatabase::new(&db_url, 200_000).await.unwrap();

            for batch_start in (0..100_000).step_by(1000) {
                let batch: Vec<_> = (batch_start..batch_start + 1000)
                    .map(|i| {
                        (
                            format!("DEV{}", i % 10),
                            AttendanceData {
                                user_id: format!("U{:06}", i),
                                timestamp: format!("2026-01-29 {:02}:{:02}:00", i / 3600 % 24, i / 60 % 60),
                                status: Some(1),
                                punch: Some(0),
                            },
                        )
                    })
                    .collect();

                let _ = hybrid.save_raw_attendance_batch(batch);
            }

            hybrid.checkpoint().await.unwrap();
        }

        // Benchmark startup
        let start = Instant::now();
        let hybrid = HybridDatabase::new(&db_url, 200_000).await.unwrap();
        let elapsed = start.elapsed();

        println!("Startup time (100K logs): {:?}", elapsed);
        println!("Signatures loaded: {}", hybrid.memory_store().count_signatures());

        assert!(elapsed.as_secs() < 2, "Startup should be < 2s");
    }
}
```

### Step 4: Memory Usage Test (15 min)

```rust
#[test]
#[ignore]
fn estimate_memory_usage() {
    use std::mem::size_of;

    // DashSet<u64> overhead estimate
    let entry_size = size_of::<u64>() + 16; // u64 + DashSet overhead per entry
    let overhead_per_shard = 64; // Approximate shard overhead
    let num_shards = 64; // Default DashMap shard count

    let entries = 100_000u64;
    let estimated_bytes = (entries * entry_size as u64) + (num_shards * overhead_per_shard);
    let estimated_mb = estimated_bytes as f64 / (1024.0 * 1024.0);

    println!("Estimated memory for {} signatures: {:.2} MB", entries, estimated_mb);
    println!("  Entry size: {} bytes", entry_size);
    println!("  Total entries size: {} bytes", entries * entry_size as u64);
    println!("  Shard overhead: {} bytes", num_shards * overhead_per_shard);

    assert!(estimated_mb < 5.0, "Should be < 5MB for 100K signatures");
}
```

## Todo List

- [ ] Write `test_compute_signature_deterministic`
- [ ] Write `test_compute_signature_different_inputs` (3 variants)
- [ ] Write `test_insert_rejects_exact_duplicate`
- [ ] Write `test_insert_allows_different_timestamp`
- [ ] Write `test_cleanup_synced_preserves_signatures`
- [ ] Write `test_batch_insert_handles_duplicates`
- [ ] Write `test_concurrent_inserts_no_duplicates`
- [ ] Write `test_full_dedup_lifecycle`
- [ ] Write `test_signatures_survive_restart_simulation`
- [ ] Write `benchmark_insert_throughput`
- [ ] Write `benchmark_duplicate_check_latency`
- [ ] Write `benchmark_startup_time`
- [ ] Write `estimate_memory_usage`
- [ ] Run all tests: `cargo test`
- [ ] Run benchmarks: `cargo test --release benchmark -- --ignored --nocapture`
- [ ] Document results in this file

## Success Criteria

| Metric | Target | Actual |
|--------|--------|--------|
| All existing tests | PASS | TBD |
| New unit tests | PASS | TBD |
| Insert throughput | > 100K/s | TBD |
| Duplicate check | < 1μs | TBD |
| Startup (100K) | < 2s | TBD |
| Memory (100K sigs) | < 5MB | TBD |

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Benchmark flakiness | Medium | Low | Run multiple times, use --release |
| Memory estimation wrong | Low | Low | Use external tools (valgrind, heaptrack) |

## Next Steps

After Phase 4 complete:
- Merge to main
- Monitor production metrics
- Consider future optimizations (signature eviction, bloom filter)

## Unresolved Questions

1. **Signature capacity limit:** Should we cap at 1M and evict old signatures?
2. **Hash collision logging:** Worth adding collision detection counter?
3. **Metrics export:** Should signature count be exposed via /metrics endpoint?
