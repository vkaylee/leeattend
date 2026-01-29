---
phase: 2
title: "Refactor Insert with Dedup"
status: complete
effort: 1h
blocked_by: [phase-01]
completed: 2026-01-29
---

# Phase 2: Refactor insert() với Hash Check

## Context

**Related files:**
- `/home/elt1541/leeattend/agent-main/src/memory_store.rs`
- `/home/elt1541/leeattend/agent-main/src/hybrid_database.rs`

**Prerequisite:** Phase 1 complete (signatures field exists)

## Overview

Sửa logic `insert()` và `cleanup_synced()`:
1. `insert()`: Check hash trước, reject nếu duplicate
2. `cleanup_synced()`: Xóa log nhưng GIỮ LẠI signature

## Key Insights

1. Current `insert()` không check duplicate - chỉ có DB constraint
2. `cleanup_synced()` xóa log hoàn toàn - mất dedup protection
3. Need new error type: `StoreError::Duplicate`

## Requirements

### Functional
- F1: `insert()` must check signature before inserting
- F2: `insert()` must return `StoreError::Duplicate` if signature exists
- F3: `cleanup_synced()` must NOT remove signatures
- F4: `load_from_disk()` must populate signatures index

### Non-functional
- NF1: Insert performance < 1μs per log (hash + set check)
- NF2: Duplicate rejection must be atomic

## Architecture

### New Error Type
```rust
#[derive(Debug)]
pub enum StoreError {
    CapacityExceeded,
    InvalidTimestamp,
    NotFound,
    Duplicate,  // NEW
}
```

### Modified insert() Flow
```
insert(device_id, data)
  │
  ├─ Parse timestamp
  │
  ├─ Compute signature
  │
  ├─ signatures.contains(sig)?
  │    ├─ YES → return Err(Duplicate)
  │    └─ NO → continue
  │
  ├─ Check capacity
  │
  ├─ signatures.insert(sig)  // Add signature FIRST
  │
  └─ logs.insert(id, log)    // Then add log
```

**Why signature first?**
- If insert fails after signature added → harmless (extra dedup protection)
- If log added before signature → race condition window for duplicates

## Related Code Files

### Files to Modify
| File | Changes |
|------|---------|
| `src/memory_store.rs` | Modify insert(), insert_batch(), load_from_disk(), cleanup_synced() |

## Implementation Steps

### Step 1: Add Duplicate Error (5 min)
```rust
#[derive(Debug)]
pub enum StoreError {
    CapacityExceeded,
    InvalidTimestamp,
    NotFound,
    Duplicate,  // NEW
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::CapacityExceeded => write!(f, "Store capacity exceeded"),
            StoreError::InvalidTimestamp => write!(f, "Invalid timestamp format"),
            StoreError::NotFound => write!(f, "Log not found"),
            StoreError::Duplicate => write!(f, "Duplicate log rejected"),  // NEW
        }
    }
}
```

### Step 2: Modify insert() (15 min)
```rust
pub fn insert(&self, device_id: String, data: AttendanceData) -> Result<i64, StoreError> {
    // Parse timestamp first (needed for signature)
    let timestamp = NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%dT%H:%M:%SZ"))
        .map_err(|_| StoreError::InvalidTimestamp)?;

    // Compute signature and check for duplicate
    let signature = Self::compute_signature(&device_id, &data.user_id, timestamp);
    if self.signatures.contains(&signature) {
        return Err(StoreError::Duplicate);
    }

    // Check capacity (backpressure)
    if self.count.load(Ordering::Relaxed) >= self.max_capacity {
        return Err(StoreError::CapacityExceeded);
    }

    // Add signature FIRST (before log)
    self.signatures.insert(signature);

    let id = self.next_id.fetch_add(1, Ordering::SeqCst);
    let now = chrono::Utc::now().naive_utc();

    let log = MemoryLog {
        id,
        device_id,
        user_id: data.user_id,
        timestamp,
        status: data.status,
        punch: data.punch,
        is_synced: 0,
        created_at: now,
        updated_at: None,
        retry_count: 0,
        persisted: false,
    };

    self.logs.insert(id, log);
    self.unsynced_index.write().insert(id);
    self.count.fetch_add(1, Ordering::SeqCst);
    self.dirty.store(true, Ordering::SeqCst);

    Ok(id)
}
```

### Step 3: Modify insert_batch() (15 min)
```rust
pub fn insert_batch(&self, batch: Vec<(String, AttendanceData)>) -> Result<usize, StoreError> {
    let available = self.max_capacity.saturating_sub(self.count.load(Ordering::Relaxed));
    if available == 0 {
        return Err(StoreError::CapacityExceeded);
    }

    let mut inserted = 0;
    let mut unsynced = self.unsynced_index.write();

    for (device_id, data) in batch.into_iter() {
        if inserted >= available {
            break; // Capacity limit reached
        }

        // Parse timestamp
        let timestamp = match NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%dT%H:%M:%SZ"))
        {
            Ok(ts) => ts,
            Err(_) => continue, // Skip invalid timestamp
        };

        // Check for duplicate
        let signature = Self::compute_signature(&device_id, &data.user_id, timestamp);
        if self.signatures.contains(&signature) {
            continue; // Skip duplicate
        }

        // Add signature FIRST
        self.signatures.insert(signature);

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let now = chrono::Utc::now().naive_utc();

        let log = MemoryLog {
            id,
            device_id,
            user_id: data.user_id,
            timestamp,
            status: data.status,
            punch: data.punch,
            is_synced: 0,
            created_at: now,
            updated_at: None,
            retry_count: 0,
            persisted: false,
        };

        self.logs.insert(id, log);
        unsynced.insert(id);
        inserted += 1;
    }

    self.count.fetch_add(inserted, Ordering::SeqCst);
    if inserted > 0 {
        self.dirty.store(true, Ordering::SeqCst);
    }
    Ok(inserted)
}
```

### Step 4: Modify load_from_disk() (10 min)
```rust
pub fn load_from_disk(&self, logs: Vec<RawAttendanceLog>) {
    let mut max_id = 0i64;
    let mut unsynced = self.unsynced_index.write();
    let mut inflight = self.inflight_index.write();

    for log in logs {
        max_id = max_id.max(log.id);

        // Add signature for ALL loaded logs (including synced ones in future)
        let signature = Self::compute_signature(&log.device_id, &log.user_id, log.timestamp);
        self.signatures.insert(signature);

        let memory_log = MemoryLog {
            id: log.id,
            device_id: log.device_id,
            user_id: log.user_id,
            timestamp: log.timestamp,
            status: log.status,
            punch: log.punch,
            is_synced: log.is_synced,
            created_at: log.created_at.unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            updated_at: log.updated_at,
            retry_count: log.retry_count,
            persisted: true,
        };

        if log.is_synced == 0 {
            unsynced.insert(log.id);
        } else if log.is_synced == 2 {
            inflight.insert(log.id, Instant::now());
        }

        self.logs.insert(log.id, memory_log);
    }

    self.next_id.store(max_id + 1, Ordering::SeqCst);
    self.count.store(self.logs.len(), Ordering::SeqCst);
}
```

### Step 5: Modify cleanup_synced() (10 min)
```rust
/// Remove synced logs older than threshold (memory cleanup)
/// IMPORTANT: Signatures are NOT removed - they provide ongoing dedup protection
pub fn cleanup_synced(&self, max_age_secs: u64) -> usize {
    let threshold = chrono::Utc::now().naive_utc() - chrono::Duration::seconds(max_age_secs as i64);
    let mut to_remove = Vec::new();

    for entry in self.logs.iter() {
        if entry.is_synced == 1 && entry.created_at < threshold {
            to_remove.push(*entry.key());
        }
    }

    let count = to_remove.len();
    for id in to_remove {
        self.logs.remove(&id);
        // NOTE: DO NOT remove signature - keep for dedup protection
    }
    self.count.fetch_sub(count, Ordering::SeqCst);

    count
}
```

### Step 6: Add Unit Tests (20 min)
```rust
#[test]
fn test_insert_rejects_duplicate() {
    let store = InMemoryStore::new(1000);
    let data1 = AttendanceData {
        user_id: "U1".to_string(),
        timestamp: "2026-01-29 10:00:00".to_string(),
        status: Some(1),
        punch: Some(1),
    };
    let data2 = data1.clone();

    // First insert succeeds
    let id1 = store.insert("D1".to_string(), data1).unwrap();
    assert!(id1 > 0);

    // Second insert with same data should fail
    let result = store.insert("D1".to_string(), data2);
    assert!(matches!(result, Err(StoreError::Duplicate)));

    // Store should have only 1 log
    assert_eq!(store.count_total(), 1);
}

#[test]
fn test_cleanup_preserves_signatures() {
    let store = InMemoryStore::new(1000);

    // Insert and sync a log
    let data = AttendanceData {
        user_id: "U1".to_string(),
        timestamp: "2026-01-29 10:00:00".to_string(),
        status: Some(1),
        punch: Some(1),
    };
    let id = store.insert("D1".to_string(), data.clone()).unwrap();

    // Mark as synced
    store.mark_in_flight(&[id]);
    store.mark_synced(&[id]);

    // Manually set old created_at
    if let Some(mut log) = store.logs.get_mut(&id) {
        log.created_at = chrono::Utc::now().naive_utc() - chrono::Duration::seconds(7200);
    }

    // Cleanup
    let removed = store.cleanup_synced(3600);
    assert_eq!(removed, 1);
    assert_eq!(store.count_total(), 0);

    // Try to insert same log again - should be rejected
    let result = store.insert("D1".to_string(), data);
    assert!(matches!(result, Err(StoreError::Duplicate)));

    // Signature count should still be 1
    assert_eq!(store.count_signatures(), 1);
}

#[test]
fn test_batch_insert_skips_duplicates() {
    let store = InMemoryStore::new(1000);

    let batch = vec![
        ("D1".to_string(), AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        }),
        ("D1".to_string(), AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(), // Duplicate
            status: Some(1),
            punch: Some(1),
        }),
        ("D1".to_string(), AttendanceData {
            user_id: "U2".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(), // Different user
            status: Some(1),
            punch: Some(1),
        }),
    ];

    let inserted = store.insert_batch(batch).unwrap();
    assert_eq!(inserted, 2, "Should insert 2, skip 1 duplicate");
    assert_eq!(store.count_total(), 2);
    assert_eq!(store.count_signatures(), 2);
}

#[test]
fn test_load_from_disk_populates_signatures() {
    let store = InMemoryStore::new(1000);
    let ts = NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

    let logs = vec![
        RawAttendanceLog {
            id: 1,
            device_id: "D1".to_string(),
            user_id: "U1".to_string(),
            timestamp: ts,
            status: Some(1),
            punch: Some(1),
            is_synced: 0,
            created_at: Some(chrono::Utc::now().naive_utc()),
            updated_at: None,
            retry_count: 0,
        },
    ];

    store.load_from_disk(logs);

    // Signature should be populated
    assert_eq!(store.count_signatures(), 1);

    // Insert same log should be rejected
    let data = AttendanceData {
        user_id: "U1".to_string(),
        timestamp: "2026-01-29 10:00:00".to_string(),
        status: Some(1),
        punch: Some(1),
    };
    let result = store.insert("D1".to_string(), data);
    assert!(matches!(result, Err(StoreError::Duplicate)));
}
```

## Todo List

- [ ] Add `StoreError::Duplicate` variant
- [ ] Update Display impl for StoreError
- [ ] Modify `insert()` to check signature before inserting
- [ ] Modify `insert_batch()` to skip duplicates
- [ ] Modify `load_from_disk()` to populate signatures
- [ ] Modify `cleanup_synced()` to preserve signatures
- [ ] Add test: `test_insert_rejects_duplicate`
- [ ] Add test: `test_cleanup_preserves_signatures`
- [ ] Add test: `test_batch_insert_skips_duplicates`
- [ ] Add test: `test_load_from_disk_populates_signatures`
- [ ] Run `cargo test` - verify all tests pass
- [ ] Verify existing tests still pass

## Success Criteria

1. Duplicate logs are rejected with `StoreError::Duplicate`
2. `cleanup_synced()` removes logs but keeps signatures
3. `load_from_disk()` populates signatures index
4. All 161+ existing tests pass
5. New dedup tests pass

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking existing tests | Medium | Medium | Some tests may expect duplicates to succeed |
| Race condition | Low | Low | Signature insert before log insert |

## Security Considerations

- Duplicate rejection is security feature (prevents replay attacks)
- No sensitive data exposure

## Next Steps

After Phase 2 complete:
- Phase 3: Optimize startup load from disk
