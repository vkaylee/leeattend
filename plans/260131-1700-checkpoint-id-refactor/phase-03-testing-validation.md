# Phase 3: Testing and Validation

## Context Links
- [Phase 1: Refactor Checkpoint](/home/elt1541/leeattend/plans/260131-1700-checkpoint-id-refactor/phase-01-refactor-checkpoint-insert.md)
- [Phase 2: Update ACK Correlation](/home/elt1541/leeattend/plans/260131-1700-checkpoint-id-refactor/phase-02-update-ack-correlation.md)

## Overview
- **Priority:** P1 Critical
- **Status:** Pending (blocked by Phase 2)
- **Effort:** 2h

Comprehensive testing to validate the refactored checkpoint and ACK correlation.

## Key Insights

1. Must test ID collision scenario that caused original bug
2. ACK flow end-to-end must work with composite keys
3. Performance must not regress

## Requirements

### Functional
- Regression test for ID collision scenario
- Unit tests for composite key methods
- Integration test for checkpoint + ACK flow

### Non-Functional
- Test execution < 2 min total
- 100% coverage of new code paths

## Related Code Files

### Modify
- `/home/elt1541/leeattend/agent-main/src/database.rs` (tests module)
- `/home/elt1541/leeattend/agent-main/src/memory_store.rs` (tests module)
- `/home/elt1541/leeattend/agent-main/src/hybrid_database.rs` (tests module)

## Implementation Steps

### Step 1: Add ID collision regression test (database.rs tests)

```rust
#[tokio::test]
async fn test_checkpoint_no_id_collision() {
    let db = setup_db().await;

    // Pre-populate disk with logs having IDs 1-100
    for i in 1..=100 {
        sqlx::query(
            "INSERT INTO zkt_raw_attendance
             (id, device_id, user_id, timestamp, status, punch, is_synced, created_at, retry_count)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(i)
        .bind("DEV1")
        .bind(format!("U{:03}", i))
        .bind(chrono::NaiveDateTime::parse_from_str("2026-01-31 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap())
        .bind(1)
        .bind(0)
        .bind(0)
        .bind(chrono::Utc::now().naive_utc())
        .bind(0i64)
        .execute(db.get_pool())
        .await
        .unwrap();
    }

    // Create memory logs with overlapping IDs (would have caused UNIQUE constraint failure before)
    let memory_logs: Vec<crate::memory_store::MemoryLog> = (50..=150)
        .map(|i| crate::memory_store::MemoryLog {
            id: i,  // IDs 50-100 overlap with disk!
            device_id: "DEV2".to_string(),
            user_id: format!("U{:03}", i + 1000),  // Different user_ids to avoid business duplicate
            timestamp: chrono::NaiveDateTime::parse_from_str("2026-01-31 11:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            status: Some(1),
            punch: Some(0),
            is_synced: 0,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: None,
            retry_count: 0,
            persisted: false,
        })
        .collect();

    // This should NOT fail with UNIQUE constraint error
    let result = db.checkpoint_batch(&memory_logs).await;
    assert!(result.is_ok(), "Checkpoint should not fail with ID collision");

    let (inserted, skipped, keys) = result.unwrap();
    assert_eq!(inserted, 101);  // All 101 logs should insert
    assert_eq!(skipped, 0);
    assert_eq!(keys.len(), 101);
}
```

### Step 2: Add composite key unit tests (memory_store.rs tests)

```rust
#[test]
fn test_composite_key_format() {
    let log = MemoryLog {
        id: 1,
        device_id: "DEV1".to_string(),
        user_id: "U001".to_string(),
        timestamp: NaiveDateTime::parse_from_str("2026-01-31 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        status: Some(1),
        punch: Some(0),
        is_synced: 0,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: None,
        retry_count: 0,
        persisted: false,
    };

    let key = log.composite_key();
    assert!(key.starts_with("DEV1:U001:"));
    assert!(key.ends_with(&log.timestamp.and_utc().timestamp().to_string()));
}

#[test]
fn test_composite_index_lookup() {
    let store = InMemoryStore::new(1000);

    let data = AttendanceData {
        user_id: "U001".to_string(),
        timestamp: "2026-01-31 10:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };

    let id = store.insert("DEV1".to_string(), data).unwrap();

    // Construct expected composite key
    let ts = NaiveDateTime::parse_from_str("2026-01-31 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let expected_key = format!("DEV1:U001:{}", ts.and_utc().timestamp());

    // Lookup should return the inserted ID
    assert_eq!(store.get_id_by_composite_key(&expected_key), Some(id));

    // Lookup non-existent key should return None
    assert_eq!(store.get_id_by_composite_key("INVALID:KEY:0"), None);
}

#[test]
fn test_mark_synced_by_key() {
    let store = InMemoryStore::new(1000);

    let data = AttendanceData {
        user_id: "U001".to_string(),
        timestamp: "2026-01-31 10:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };

    let id = store.insert("DEV1".to_string(), data).unwrap();

    // Mark in-flight first
    store.mark_in_flight(&[id]);

    // Construct composite key
    let ts = NaiveDateTime::parse_from_str("2026-01-31 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let key = format!("DEV1:U001:{}", ts.and_utc().timestamp());

    // Mark synced by key
    assert!(store.mark_synced_by_key(&key));

    // Verify synced
    let log = store.logs.get(&id).unwrap();
    assert_eq!(log.is_synced, 1);
}
```

### Step 3: Add integration test for full flow (hybrid_database.rs tests)

```rust
#[tokio::test]
async fn test_full_checkpoint_ack_flow_with_composite_key() {
    let hybrid = HybridDatabase::new("sqlite::memory:", 1000).await.unwrap();

    // Insert logs to memory
    for i in 0..10 {
        let data = AttendanceData {
            user_id: format!("U{:03}", i),
            timestamp: format!("2026-01-31 10:{:02}:00", i),
            status: Some(1),
            punch: Some(0),
        };
        hybrid.save_raw_attendance("DEV1", data).unwrap();
    }

    // Checkpoint to disk
    let inserted = hybrid.checkpoint().await.unwrap();
    assert_eq!(inserted, 10);

    // Simulate ACK flow: get unsynced logs, extract composite keys, mark synced
    let logs = hybrid.get_unsynced_logs(10);
    for log in &logs {
        let composite_key = log.composite_key();

        // Mark in-flight
        hybrid.mark_in_flight(&[log.id]);

        // Simulate successful ACK
        assert!(hybrid.mark_synced_by_key(&composite_key));
    }

    // Verify all synced
    assert_eq!(hybrid.count_unsynced_logs(), 0);
}

#[tokio::test]
async fn test_ack_failure_resets_to_unsynced() {
    let hybrid = HybridDatabase::new("sqlite::memory:", 1000).await.unwrap();

    let data = AttendanceData {
        user_id: "U001".to_string(),
        timestamp: "2026-01-31 10:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };

    let id = hybrid.save_raw_attendance("DEV1", data).unwrap();

    // Mark in-flight
    hybrid.mark_in_flight(&[id]);
    assert_eq!(hybrid.count_unsynced_logs(), 0);  // In-flight, not unsynced

    // Construct composite key
    let ts = NaiveDateTime::parse_from_str("2026-01-31 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let key = format!("DEV1:U001:{}", ts.and_utc().timestamp());

    // Simulate failed ACK - reset to unsynced
    assert!(hybrid.reset_to_unsynced_by_key(&key));

    // Should be back in unsynced queue
    assert_eq!(hybrid.count_unsynced_logs(), 1);
}
```

### Step 4: Add performance benchmark test

```rust
#[tokio::test]
async fn test_checkpoint_performance_1000_logs() {
    let hybrid = HybridDatabase::new("sqlite::memory:", 10000).await.unwrap();

    // Insert 1000 logs
    for i in 0..1000 {
        let data = AttendanceData {
            user_id: format!("U{:06}", i),
            timestamp: format!("2026-01-31 {:02}:{:02}:{:02}", i / 3600, (i % 3600) / 60, i % 60),
            status: Some(1),
            punch: Some(0),
        };
        hybrid.save_raw_attendance("DEV1", data).unwrap();
    }

    // Measure checkpoint time
    let start = std::time::Instant::now();
    let inserted = hybrid.checkpoint().await.unwrap();
    let elapsed = start.elapsed();

    assert_eq!(inserted, 1000);
    assert!(
        elapsed.as_millis() < 500,  // Should be well under 500ms
        "Checkpoint took {:?}, expected < 500ms", elapsed
    );
}
```

### Step 5: Update existing tests for new return types

Search and update all calls to `checkpoint_batch` in tests:

```bash
grep -n "checkpoint_batch" agent-main/src/*.rs
```

Update each test to handle `(usize, usize, Vec<CompositeKey>)` return type.

### Step 6: Run full test suite

```bash
./run-tests.sh --api
```

## Todo List

- [ ] Add ID collision regression test
- [ ] Add composite key format test
- [ ] Add composite index lookup test
- [ ] Add mark_synced_by_key test
- [ ] Add full checkpoint+ACK flow integration test
- [ ] Add ACK failure reset test
- [ ] Add performance benchmark test
- [ ] Update existing tests for new return types
- [ ] Run full test suite
- [ ] Verify 0 test failures

## Success Criteria

- [ ] All new tests pass
- [ ] All existing tests pass (may need updates for return type change)
- [ ] ID collision scenario no longer causes UNIQUE constraint error
- [ ] Checkpoint performance < 500ms for 1000 logs
- [ ] ACK correlation works with composite keys

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Test flakiness with timestamps | Use fixed timestamps in tests |
| Performance test too strict | Allow 500ms margin (current baseline ~50ms) |

## Security Considerations

- Tests use in-memory SQLite, no persistent data exposure
- No credentials or sensitive data in test fixtures

## Next Steps

After Phase 3 complete:
- Merge to main branch
- Monitor production for checkpoint errors
- Close debugger issue
