# Phase 6: Testing & Validation

## Context Links
- [Phase 5: Crash Recovery](./phase-05-crash-recovery.md)
- [Existing tests](/home/elt1541/leeattend/agent-main/src/database.rs) (lines 448-972)
- [Cargo.toml](/home/elt1541/leeattend/agent-main/Cargo.toml)

## Overview
- **Priority**: P1 (Critical)
- **Status**: pending
- **Effort**: 2h
- **Description**: Comprehensive testing to ensure all 100 existing tests pass and new functionality is validated

## Key Insights

### Test Categories

| Category | Count | Focus |
|----------|-------|-------|
| Unit Tests | ~30 | Individual component correctness |
| Integration Tests | ~15 | Component interaction |
| Performance Tests | ~5 | Latency and throughput |
| Crash Recovery Tests | ~5 | Data durability |
| Regression Tests | ~45 | Existing tests must pass |

### Current Test Structure
- All tests in `src/database.rs` (mod tests)
- Uses in-memory SQLite for isolation
- Async tests with tokio runtime

## Requirements

### Functional Requirements
1. All existing 100 tests must pass
2. New tests for InMemoryStore operations
3. New tests for HybridDatabase operations
4. New tests for checkpoint worker
5. Crash recovery tests

### Non-Functional Requirements
1. Tests run in <30 seconds total
2. No flaky tests (deterministic)
3. Test isolation (no shared state)

## Architecture

### Test Module Structure
```
src/
  memory_store.rs       # Unit tests (mod tests)
  hybrid_database.rs    # Integration tests (mod tests)
  crash_recovery.rs     # Unit tests (mod tests)

tests/
  integration_tests.rs  # End-to-end tests
  benchmark_tests.rs    # Performance benchmarks
```

## Related Code Files

### Files to Modify
- `src/memory_store.rs` - Add unit tests
- `src/hybrid_database.rs` - Add integration tests
- `src/crash_recovery.rs` - Add unit tests

### Files to Create
- `tests/integration_tests.rs` - E2E tests
- `benches/performance.rs` - Benchmarks (optional)

## Implementation Steps

### Step 1: InMemoryStore Unit Tests

```rust
// src/memory_store.rs - Add at bottom

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AttendanceData;

    fn make_attendance(user_id: &str, timestamp: &str) -> AttendanceData {
        AttendanceData {
            user_id: user_id.to_string(),
            timestamp: timestamp.to_string(),
            status: Some(1),
            punch: Some(0),
        }
    }

    #[test]
    fn test_new_store() {
        let store = InMemoryStore::new(1000);
        assert_eq!(store.count_total(), 0);
        assert_eq!(store.count_unsynced(), 0);
        assert!(!store.is_dirty());
    }

    #[test]
    fn test_insert_single() {
        let store = InMemoryStore::new(1000);
        let data = make_attendance("U001", "2026-01-29 10:00:00");

        let id = store.insert("DEV1".to_string(), data).unwrap();

        assert!(id > 0);
        assert_eq!(store.count_total(), 1);
        assert_eq!(store.count_unsynced(), 1);
        assert!(store.is_dirty());
    }

    #[test]
    fn test_insert_batch() {
        let store = InMemoryStore::new(1000);
        let batch: Vec<(String, AttendanceData)> = (1..=10)
            .map(|i| {
                (
                    "DEV1".to_string(),
                    make_attendance(&format!("U{:03}", i), "2026-01-29 10:00:00"),
                )
            })
            .collect();

        let count = store.insert_batch(batch).unwrap();

        assert_eq!(count, 10);
        assert_eq!(store.count_total(), 10);
        assert_eq!(store.count_unsynced(), 10);
    }

    #[test]
    fn test_capacity_limit() {
        let store = InMemoryStore::new(5); // Very small capacity
        let batch: Vec<(String, AttendanceData)> = (1..=10)
            .map(|i| {
                (
                    "DEV1".to_string(),
                    make_attendance(&format!("U{:03}", i), "2026-01-29 10:00:00"),
                )
            })
            .collect();

        let result = store.insert_batch(batch);

        // Should insert up to capacity
        assert!(result.is_ok());
        assert_eq!(store.count_total(), 5);
    }

    #[test]
    fn test_get_unsynced_fifo_order() {
        let store = InMemoryStore::new(1000);

        // Insert in order
        for i in 1..=5 {
            let data = make_attendance(&format!("U{:03}", i), "2026-01-29 10:00:00");
            store.insert("DEV1".to_string(), data).unwrap();
        }

        let logs = store.get_unsynced(10);

        // Should be in FIFO order (first inserted = first returned)
        assert_eq!(logs.len(), 5);
        assert!(logs[0].id < logs[1].id);
        assert!(logs[1].id < logs[2].id);
    }

    #[test]
    fn test_mark_in_flight() {
        let store = InMemoryStore::new(1000);
        let data = make_attendance("U001", "2026-01-29 10:00:00");
        let id = store.insert("DEV1".to_string(), data).unwrap();

        store.mark_in_flight(&[id]);

        assert_eq!(store.count_unsynced(), 0);
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 0);
    }

    #[test]
    fn test_mark_synced() {
        let store = InMemoryStore::new(1000);
        let data = make_attendance("U001", "2026-01-29 10:00:00");
        let id = store.insert("DEV1".to_string(), data).unwrap();

        store.mark_in_flight(&[id]);
        store.mark_synced(&[id]);

        assert_eq!(store.count_unsynced(), 0);
        assert_eq!(store.count_total(), 1);
    }

    #[test]
    fn test_reset_to_unsynced() {
        let store = InMemoryStore::new(1000);
        let data = make_attendance("U001", "2026-01-29 10:00:00");
        let id = store.insert("DEV1".to_string(), data).unwrap();

        store.mark_in_flight(&[id]);
        assert_eq!(store.count_unsynced(), 0);

        store.reset_to_unsynced(id);
        assert_eq!(store.count_unsynced(), 1);
    }

    #[test]
    fn test_get_unpersisted() {
        let store = InMemoryStore::new(1000);

        // Insert 5 logs (all unpersisted)
        for i in 1..=5 {
            let data = make_attendance(&format!("U{:03}", i), "2026-01-29 10:00:00");
            store.insert("DEV1".to_string(), data).unwrap();
        }

        let unpersisted = store.get_unpersisted();
        assert_eq!(unpersisted.len(), 5);

        // Mark some as persisted
        let ids: Vec<i64> = unpersisted[0..3].iter().map(|l| l.id).collect();
        store.mark_persisted(&ids);

        let unpersisted = store.get_unpersisted();
        assert_eq!(unpersisted.len(), 2);
    }

    #[test]
    fn test_concurrent_insert() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(InMemoryStore::new(10000));
        let mut handles = vec![];

        for t in 0..4 {
            let store = store.clone();
            let handle = thread::spawn(move || {
                for i in 0..100 {
                    let data = make_attendance(
                        &format!("T{}U{:03}", t, i),
                        "2026-01-29 10:00:00",
                    );
                    store.insert(format!("DEV{}", t), data).unwrap();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(store.count_total(), 400);
    }

    #[test]
    fn test_invalid_timestamp() {
        let store = InMemoryStore::new(1000);
        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "invalid-timestamp".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        let result = store.insert("DEV1".to_string(), data);

        assert!(matches!(result, Err(StoreError::InvalidTimestamp)));
    }
}
```

### Step 2: HybridDatabase Integration Tests

```rust
// src/hybrid_database.rs - Add at bottom

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AttendanceData;

    fn make_attendance(user_id: &str, timestamp: &str) -> AttendanceData {
        AttendanceData {
            user_id: user_id.to_string(),
            timestamp: timestamp.to_string(),
            status: Some(1),
            punch: Some(0),
        }
    }

    #[tokio::test]
    async fn test_hybrid_db_new() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000).await.unwrap();
        assert_eq!(hybrid.count_unsynced_logs(), 0);
    }

    #[tokio::test]
    async fn test_hybrid_save_and_checkpoint() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000).await.unwrap();

        // Save to memory
        for i in 1..=10 {
            let data = make_attendance(&format!("U{:03}", i), "2026-01-29 10:00:00");
            hybrid.save_raw_attendance("DEV1", data).unwrap();
        }

        assert_eq!(hybrid.count_unsynced_logs(), 10);
        assert!(hybrid.memory_store().is_dirty());

        // Checkpoint to disk
        let checkpointed = hybrid.checkpoint().await.unwrap();

        assert_eq!(checkpointed, 10);
        assert!(!hybrid.memory_store().is_dirty());
    }

    #[tokio::test]
    async fn test_hybrid_full_flow() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000).await.unwrap();

        // 1. Insert logs
        for i in 1..=5 {
            let data = make_attendance(&format!("U{:03}", i), "2026-01-29 10:00:00");
            hybrid.save_raw_attendance("DEV1", data).unwrap();
        }

        // 2. Get unsynced
        let logs = hybrid.get_unsynced_logs(10);
        assert_eq!(logs.len(), 5);

        // 3. Mark in-flight
        let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        hybrid.mark_in_flight(&ids);

        // 4. Mark synced
        hybrid.mark_synced(&ids[0..3]);
        assert_eq!(hybrid.count_unsynced_logs(), 0);

        // 5. Checkpoint
        let checkpointed = hybrid.checkpoint().await.unwrap();
        assert!(checkpointed > 0);
    }

    #[tokio::test]
    async fn test_hybrid_restart_recovery() {
        // Create and populate
        let db_url = "sqlite::memory:";
        let hybrid = HybridDatabase::new(db_url, 1000).await.unwrap();

        for i in 1..=5 {
            let data = make_attendance(&format!("U{:03}", i), "2026-01-29 10:00:00");
            hybrid.save_raw_attendance("DEV1", data).unwrap();
        }

        // Checkpoint to disk
        hybrid.checkpoint().await.unwrap();

        // Simulate restart: create new HybridDatabase
        let hybrid2 = HybridDatabase::new(db_url, 1000).await.unwrap();

        // Should recover logs from disk
        assert_eq!(hybrid2.count_unsynced_logs(), 5);
    }
}
```

### Step 3: Crash Recovery Tests

```rust
// src/crash_recovery.rs - Add at bottom

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_crash_marker_write_and_check() {
        let dir = tempdir().unwrap();
        let db_path = format!("sqlite:{}/test.db", dir.path().display());

        let marker = CrashMarker::new(&db_path);

        assert!(!marker.check_crash());

        marker.write().unwrap();
        assert!(marker.check_crash());

        marker.remove().unwrap();
        assert!(!marker.check_crash());
    }

    #[test]
    fn test_crash_marker_content() {
        let dir = tempdir().unwrap();
        let db_path = format!("sqlite:{}/test.db", dir.path().display());

        let marker = CrashMarker::new(&db_path);
        marker.write().unwrap();

        let content = marker.read_crash_info().unwrap();
        assert!(content.contains("pid="));
        assert!(content.contains("started="));

        marker.remove().unwrap();
    }

    #[tokio::test]
    async fn test_recover_from_crash() {
        let db = crate::database::Database::new("sqlite::memory:").await.unwrap();

        // Insert logs with in-flight status
        sqlx::query(
            "INSERT INTO zkt_raw_attendance (device_id, user_id, timestamp, is_synced) \
             VALUES ('DEV1', 'U001', '2026-01-29 10:00:00', 2)"
        )
        .execute(db.get_pool())
        .await
        .unwrap();

        // Verify in-flight
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 2"
        )
        .fetch_one(db.get_pool())
        .await
        .unwrap();
        assert_eq!(count, 1);

        // Run recovery
        let recovered = recover_from_crash(&db).await.unwrap();
        assert_eq!(recovered, 1);

        // Verify reset to unsynced
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 0"
        )
        .fetch_one(db.get_pool())
        .await
        .unwrap();
        assert_eq!(count, 1);
    }
}
```

### Step 4: Performance Benchmark Tests

```rust
// tests/integration_tests.rs

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

    // Assert sub-millisecond latency
    assert!(avg_latency_us < 1000, "Insert latency too high: {}us", avg_latency_us);
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

    // Assert sub-millisecond latency
    assert!(avg_latency_us < 1000, "Query latency too high: {}us", avg_latency_us);
}

#[test]
fn benchmark_memory_usage() {
    let store = InMemoryStore::new(100_000);

    // Insert 50K logs
    for i in 0..50_000 {
        let data = make_attendance(&format!("U{:05}", i), "2026-01-29 10:00:00");
        store.insert(format!("DEV{}", i % 100), data).unwrap();
    }

    // Memory estimation: ~464 bytes per log * 50K = ~23MB
    // This is a sanity check, actual memory tracked separately
    assert_eq!(store.count_total(), 50_000);
}
```

### Step 5: Verify Existing Tests Pass

```bash
# Run all existing tests
cargo test --all

# Run with verbose output
cargo test --all -- --nocapture

# Run specific test module
cargo test --lib database::tests
```

## Todo List

- [ ] Add InMemoryStore unit tests
- [ ] Add HybridDatabase integration tests
- [ ] Add CrashRecovery tests
- [ ] Add performance benchmark tests
- [ ] Run existing 100 tests, fix any regressions
- [ ] Verify sub-millisecond latency targets
- [ ] Test checkpoint behavior under load
- [ ] Test concurrent access patterns
- [ ] Document test coverage metrics

## Success Criteria

- [ ] All 100 existing tests pass
- [ ] All new tests pass (>30 new tests)
- [ ] Insert latency <1ms (benchmark verified)
- [ ] Query latency <1ms (benchmark verified)
- [ ] Memory usage within estimates
- [ ] No flaky tests

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Test flakiness | Use deterministic ordering, avoid time-based tests |
| Coverage gaps | Review each phase for test coverage |
| Performance regression | Add benchmark assertions |

## Next Steps

After all tests pass, the implementation is ready for deployment.
Final checklist:
1. Code review
2. Documentation update
3. Monitoring/alerting setup
4. Staged rollout plan
