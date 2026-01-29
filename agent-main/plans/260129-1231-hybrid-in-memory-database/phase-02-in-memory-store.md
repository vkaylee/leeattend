# Phase 2: In-Memory Store Implementation

## Context Links
- [Phase 1: Design](./phase-01-design-memory-model.md)
- [Current database.rs](/home/elt1541/leeattend/agent-main/src/database.rs)
- [models.rs](/home/elt1541/leeattend/agent-main/src/models.rs)

## Overview
- **Priority**: P1 (Critical)
- **Status**: pending
- **Effort**: 4h
- **Description**: Implement the core in-memory store with thread-safe operations

## Key Insights

### DashMap Choice Rationale
- Lock-free reads for sync loop queries
- Concurrent writes from DB writer without blocking reads
- Built-in sharding reduces contention
- No external dependency (already in cargo ecosystem)

### Alternative: Crossbeam SkipMap
- Pros: Better for ordered iteration
- Cons: Higher memory overhead, less battle-tested
- Decision: Use DashMap + BTreeSet index for ordered queries

## Requirements

### Functional Requirements
1. Thread-safe insert, update, delete operations
2. Fast query: get unsynced logs (FIFO order by ID)
3. Fast query: count unsynced logs
4. Batch operations for mark_synced, mark_in_flight
5. Deduplication check on insert

### Non-Functional Requirements
1. O(1) insert, O(1) lookup by ID
2. O(log n) for ordered iteration (BTreeSet)
3. Lock-free reads where possible
4. Atomic count tracking

## Architecture

### Crate Dependencies (add to Cargo.toml)
```toml
dashmap = "6"  # Concurrent HashMap
parking_lot = "0.12"  # Faster RwLock
```

### Module Structure
```
src/
  memory_store.rs    # InMemoryStore implementation
  hybrid_database.rs # HybridDatabase wrapper (Phase 4)
  database.rs        # Keep existing (disk-only operations)
```

## Related Code Files

### Files to Create
- `src/memory_store.rs` (~250 lines)

### Files to Modify
- `Cargo.toml` - Add dashmap, parking_lot dependencies

## Implementation Steps

### Step 1: Define Core Types

```rust
// src/memory_store.rs

use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::{AtomicI64, AtomicUsize, AtomicBool, Ordering};
use std::time::Instant;
use chrono::NaiveDateTime;

#[derive(Clone, Debug)]
pub struct MemoryLog {
    pub id: i64,
    pub device_id: String,
    pub user_id: String,
    pub timestamp: NaiveDateTime,
    pub status: Option<i32>,
    pub punch: Option<i32>,
    pub is_synced: i64,           // 0=unsynced, 1=synced, 2=in_flight
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub retry_count: i64,
    pub persisted: bool,          // true if already on disk
}

pub struct InMemoryStore {
    logs: DashMap<i64, MemoryLog>,
    unsynced_index: RwLock<BTreeSet<i64>>,
    inflight_index: RwLock<HashMap<i64, Instant>>,
    next_id: AtomicI64,
    dirty: AtomicBool,
    count: AtomicUsize,
    max_capacity: usize,
}
```

### Step 2: Implement Constructor & Load

```rust
impl InMemoryStore {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            logs: DashMap::with_capacity(max_capacity / 2),
            unsynced_index: RwLock::new(BTreeSet::new()),
            inflight_index: RwLock::new(HashMap::new()),
            next_id: AtomicI64::new(1),
            dirty: AtomicBool::new(false),
            count: AtomicUsize::new(0),
            max_capacity,
        }
    }

    /// Load existing logs from disk into memory (startup)
    pub fn load_from_disk(&self, logs: Vec<RawAttendanceLog>) {
        let mut max_id = 0i64;
        let mut unsynced = self.unsynced_index.write();
        let mut inflight = self.inflight_index.write();

        for log in logs {
            max_id = max_id.max(log.id);
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
                persisted: true, // Loaded from disk = already persisted
            };

            // Update indices
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
}
```

### Step 3: Implement Insert Operations

```rust
impl InMemoryStore {
    /// Insert a new attendance log. Returns assigned ID.
    pub fn insert(&self, device_id: String, data: AttendanceData) -> Result<i64, StoreError> {
        // Check capacity (backpressure)
        if self.count.load(Ordering::Relaxed) >= self.max_capacity {
            return Err(StoreError::CapacityExceeded);
        }

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let now = chrono::Utc::now().naive_utc();

        let timestamp = NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%dT%H:%M:%SZ"))
            .map_err(|_| StoreError::InvalidTimestamp)?;

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

    /// Batch insert with deduplication check
    pub fn insert_batch(&self, batch: Vec<(String, AttendanceData)>) -> Result<usize, StoreError> {
        let available = self.max_capacity.saturating_sub(self.count.load(Ordering::Relaxed));
        let to_insert = batch.len().min(available);

        if to_insert == 0 {
            return Err(StoreError::CapacityExceeded);
        }

        let mut inserted = 0;
        let mut unsynced = self.unsynced_index.write();

        for (device_id, data) in batch.into_iter().take(to_insert) {
            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            let now = chrono::Utc::now().naive_utc();

            if let Ok(timestamp) = NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%d %H:%M:%S")
                .or_else(|_| NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%dT%H:%M:%SZ"))
            {
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
        }

        self.count.fetch_add(inserted, Ordering::SeqCst);
        self.dirty.store(true, Ordering::SeqCst);
        Ok(inserted)
    }
}
```

### Step 4: Implement Query Operations

```rust
impl InMemoryStore {
    /// Get unsynced logs (FIFO order by ID)
    pub fn get_unsynced(&self, limit: usize) -> Vec<MemoryLog> {
        let unsynced = self.unsynced_index.read();
        let mut result = Vec::with_capacity(limit);

        for id in unsynced.iter().take(limit) {
            if let Some(log) = self.logs.get(id) {
                result.push(log.clone());
            }
        }

        result
    }

    /// Count unsynced logs
    pub fn count_unsynced(&self) -> usize {
        self.unsynced_index.read().len()
    }

    /// Count total logs
    pub fn count_total(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    /// Check if store is dirty (has unpersisted changes)
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }
}
```

### Step 5: Implement Update Operations

```rust
impl InMemoryStore {
    /// Mark logs as in-flight (is_synced=2)
    pub fn mark_in_flight(&self, ids: &[i64]) {
        let now = Instant::now();
        let mut unsynced = self.unsynced_index.write();
        let mut inflight = self.inflight_index.write();

        for &id in ids {
            if let Some(mut log) = self.logs.get_mut(&id) {
                log.is_synced = 2;
                log.retry_count += 1;
                log.updated_at = Some(chrono::Utc::now().naive_utc());
            }
            unsynced.remove(&id);
            inflight.insert(id, now);
        }

        self.dirty.store(true, Ordering::SeqCst);
    }

    /// Mark logs as synced (is_synced=1)
    pub fn mark_synced(&self, ids: &[i64]) {
        let mut inflight = self.inflight_index.write();

        for &id in ids {
            if let Some(mut log) = self.logs.get_mut(&id) {
                log.is_synced = 1;
            }
            inflight.remove(&id);
        }

        self.dirty.store(true, Ordering::SeqCst);
    }

    /// Reset failed sync (is_synced=2 -> 0)
    pub fn reset_to_unsynced(&self, id: i64) {
        let mut unsynced = self.unsynced_index.write();
        let mut inflight = self.inflight_index.write();

        if let Some(mut log) = self.logs.get_mut(&id) {
            log.is_synced = 0;
        }
        unsynced.insert(id);
        inflight.remove(&id);

        self.dirty.store(true, Ordering::SeqCst);
    }
}
```

### Step 6: Implement Checkpoint Support

```rust
impl InMemoryStore {
    /// Get all unpersisted logs for checkpointing
    pub fn get_unpersisted(&self) -> Vec<MemoryLog> {
        self.logs.iter()
            .filter(|entry| !entry.persisted)
            .map(|entry| entry.clone())
            .collect()
    }

    /// Mark logs as persisted after successful checkpoint
    pub fn mark_persisted(&self, ids: &[i64]) {
        for &id in ids {
            if let Some(mut log) = self.logs.get_mut(&id) {
                log.persisted = true;
            }
        }
    }

    /// Get all logs for full checkpoint
    pub fn get_all(&self) -> Vec<MemoryLog> {
        self.logs.iter().map(|entry| entry.clone()).collect()
    }

    /// Clear dirty flag after checkpoint
    pub fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::SeqCst);
    }

    /// Remove synced logs older than threshold (memory cleanup)
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
        }
        self.count.fetch_sub(count, Ordering::SeqCst);

        count
    }
}
```

### Step 7: Error Types

```rust
#[derive(Debug)]
pub enum StoreError {
    CapacityExceeded,
    InvalidTimestamp,
    NotFound,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::CapacityExceeded => write!(f, "Store capacity exceeded"),
            StoreError::InvalidTimestamp => write!(f, "Invalid timestamp format"),
            StoreError::NotFound => write!(f, "Log not found"),
        }
    }
}

impl std::error::Error for StoreError {}
```

## Todo List

- [ ] Add dashmap and parking_lot to Cargo.toml
- [ ] Create src/memory_store.rs with core types
- [ ] Implement InMemoryStore::new() and load_from_disk()
- [ ] Implement insert() and insert_batch()
- [ ] Implement get_unsynced() and count_unsynced()
- [ ] Implement mark_in_flight() and mark_synced()
- [ ] Implement checkpoint helpers (get_unpersisted, mark_persisted)
- [ ] Implement cleanup_synced() for memory management
- [ ] Add unit tests for each operation
- [ ] Benchmark insert/query performance

## Success Criteria

- [ ] All operations thread-safe (no data races)
- [ ] Insert latency <1ms for single log
- [ ] Query latency <1ms for 500 unsynced logs
- [ ] Memory usage matches estimates from Phase 1
- [ ] Unit tests pass

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| DashMap version incompatibility | Pin to dashmap v6 |
| Index inconsistency | Single writer for index updates |
| Memory leak on cleanup | Explicit cleanup_synced() calls |

## Next Steps

Proceed to Phase 3: Sync Loop Optimization after implementing and testing InMemoryStore.
