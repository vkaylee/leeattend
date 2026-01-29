---
phase: 3
title: "Optimize Startup Load"
status: complete
effort: 1.5h
blocked_by: [phase-02]
completed: 2026-01-29
---

# Phase 3: Optimize load_from_disk()

## Context

**Related files:**
- `/home/elt1541/leeattend/agent-main/src/database.rs`
- `/home/elt1541/leeattend/agent-main/src/hybrid_database.rs`
- `/home/elt1541/leeattend/agent-main/src/memory_store.rs`

**Prerequisite:** Phase 2 complete (signatures index working)

## Overview

Tối ưu startup load bằng two-phase approach:
1. **Phase A:** Load signatures only (fast, low memory)
2. **Phase B:** Load unsynced logs only (needed for sync queue)

## Problem Statement

**Current `get_all_logs()` behavior:**
```sql
SELECT * FROM zkt_raw_attendance WHERE is_synced IN (0, 2)
```
- Loads full log data for all unsynced/in-flight
- Good for sync queue
- BUT: Doesn't load synced logs → no signatures for them → duplicates can re-enter

**Proposed two-phase:**
```sql
-- Phase A: Signatures only (all logs including synced)
SELECT device_id, user_id, timestamp FROM zkt_raw_attendance

-- Phase B: Unsynced logs only (for queue)
SELECT * FROM zkt_raw_attendance WHERE is_synced IN (0, 2)
```

## Key Insights

1. Phase A query returns ~25 bytes per row (vs ~200+ bytes for full log)
2. Can use streaming cursor for Phase A to avoid memory spike
3. Phase B same as current - no change needed

## Requirements

### Functional
- F1: All signatures loaded on startup (including synced logs)
- F2: Only unsynced/in-flight logs loaded into queue
- F3: Startup must be idempotent

### Non-functional
- NF1: 100K logs startup < 2 seconds
- NF2: Peak memory during load < 50MB

## Architecture

### New Database Methods

```rust
// database.rs
/// Load signatures only (device_id, user_id, timestamp) for dedup index
pub async fn get_all_signatures(&self) -> Result<Vec<SignatureRow>, Error>;

/// Minimal struct for signature loading
pub struct SignatureRow {
    pub device_id: String,
    pub user_id: String,
    pub timestamp: NaiveDateTime,
}
```

### Modified Startup Flow

```
HybridDatabase::new()
  │
  ├─ disk.get_max_id() → seed_id()
  │
  ├─ disk.get_all_signatures()  // NEW: Phase A
  │    └─ memory.load_signatures(signatures)
  │
  └─ disk.get_all_logs()        // Phase B: unchanged
       └─ memory.load_from_disk(logs)
```

## Related Code Files

### Files to Modify
| File | Changes |
|------|---------|
| `src/database.rs` | Add `get_all_signatures()`, `SignatureRow` |
| `src/hybrid_database.rs` | Call two-phase load |
| `src/memory_store.rs` | Add `load_signatures()` |

## Implementation Steps

### Step 1: Add SignatureRow struct (5 min)
```rust
// database.rs
#[derive(Debug, sqlx::FromRow)]
pub struct SignatureRow {
    pub device_id: String,
    pub user_id: String,
    pub timestamp: chrono::NaiveDateTime,
}
```

### Step 2: Add get_all_signatures() (15 min)
```rust
// database.rs
impl Database {
    /// Load all signatures for dedup index (startup only)
    /// Returns minimal data: device_id, user_id, timestamp
    pub async fn get_all_signatures(&self) -> Result<Vec<SignatureRow>, Error> {
        let rows = sqlx::query_as::<_, SignatureRow>(
            r#"
            SELECT device_id, user_id, timestamp
            FROM zkt_raw_attendance
            ORDER BY id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
```

### Step 3: Add load_signatures() to InMemoryStore (10 min)
```rust
// memory_store.rs
impl InMemoryStore {
    /// Load signatures only (Phase A of startup)
    /// Does NOT create MemoryLog entries - just populates signature index
    pub fn load_signatures(&self, rows: Vec<SignatureRow>) {
        for row in rows {
            let sig = Self::compute_signature(&row.device_id, &row.user_id, row.timestamp);
            self.signatures.insert(sig);
        }
    }
}
```

**Note:** Need to make `SignatureRow` accessible. Options:
1. Move to `models.rs` (preferred)
2. Re-export from `database.rs`

### Step 4: Update HybridDatabase::new() (15 min)
```rust
// hybrid_database.rs
impl HybridDatabase {
    pub async fn new(database_url: &str, max_memory_logs: usize) -> Result<Self, sqlx::Error> {
        let disk = Database::new(database_url).await?;
        let memory = Arc::new(InMemoryStore::new(max_memory_logs));

        // Seed ID counter from disk
        let max_id = disk.get_max_id().await?;
        memory.seed_id(max_id + 1);

        // Phase A: Load signatures (ALL logs including synced)
        let signatures = disk.get_all_signatures().await?;
        let sig_count = signatures.len();
        memory.load_signatures(signatures);
        info!("HybridDatabase: Loaded {} signatures for dedup index", sig_count);

        // Phase B: Load unsynced/in-flight logs into queue
        let pending = disk.get_all_logs().await?;
        let pending_count = pending.len();
        memory.load_from_disk(pending);
        info!("HybridDatabase: Loaded {} pending logs into memory", pending_count);

        Ok(Self { disk, memory })
    }
}
```

### Step 5: Optimize load_from_disk() (10 min)
```rust
// memory_store.rs
/// Load existing logs from disk into memory (startup recovery)
/// NOTE: Assumes load_signatures() was called first
pub fn load_from_disk(&self, logs: Vec<RawAttendanceLog>) {
    let mut max_id = 0i64;
    let mut unsynced = self.unsynced_index.write();
    let mut inflight = self.inflight_index.write();

    for log in logs {
        max_id = max_id.max(log.id);

        // Signature already loaded in Phase A - no need to compute again
        // But verify it exists (defensive)
        #[cfg(debug_assertions)]
        {
            let sig = Self::compute_signature(&log.device_id, &log.user_id, log.timestamp);
            debug_assert!(self.signatures.contains(&sig), "Signature should exist from Phase A");
        }

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

### Step 6: Add Unit Tests (20 min)
```rust
// hybrid_database.rs tests
#[tokio::test]
async fn test_startup_loads_signatures_for_synced_logs() {
    let hybrid = HybridDatabase::new("sqlite::memory:", 1000).await.unwrap();

    // Save logs
    for i in 1..=5 {
        let data = make_attendance(&format!("U{:03}", i), "2026-01-29 10:00:00");
        hybrid.save_raw_attendance("DEV1", data).unwrap();
    }

    // Mark all as synced
    let logs = hybrid.get_unsynced_logs(10);
    let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
    hybrid.mark_in_flight(&ids);
    hybrid.mark_synced(&ids);

    // Checkpoint to disk
    hybrid.checkpoint().await.unwrap();

    // Verify signature count
    assert!(hybrid.memory_store().count_signatures() >= 5);

    // Try to insert duplicate - should be rejected
    let data = make_attendance("U001", "2026-01-29 10:00:00");
    let result = hybrid.save_raw_attendance("DEV1", data);
    assert!(matches!(result, Err(StoreError::Duplicate)));
}
```

### Step 7: Add Performance Benchmark (Optional)
```rust
#[tokio::test]
#[ignore] // Run with: cargo test benchmark --ignored
async fn benchmark_startup_100k_logs() {
    use std::time::Instant;

    // Setup: Create temp DB with 100K logs
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_url = format!("sqlite:{}", temp_db.path().display());

    // ... insert 100K logs ...

    // Benchmark
    let start = Instant::now();
    let _hybrid = HybridDatabase::new(&db_url, 10_000).await.unwrap();
    let elapsed = start.elapsed();

    println!("Startup time for 100K logs: {:?}", elapsed);
    assert!(elapsed.as_secs() < 2, "Startup should be < 2s");
}
```

## Todo List

- [ ] Add `SignatureRow` struct to models.rs
- [ ] Add `get_all_signatures()` to Database
- [ ] Add `load_signatures()` to InMemoryStore
- [ ] Update `HybridDatabase::new()` for two-phase load
- [ ] Remove redundant signature computation from `load_from_disk()`
- [ ] Add test: `test_startup_loads_signatures_for_synced_logs`
- [ ] Run `cargo test` - verify all tests pass
- [ ] Optional: Add startup benchmark

## Success Criteria

1. All signatures (including synced) loaded on startup
2. Duplicates rejected even after cleanup
3. Startup time < 2s for 100K logs
4. All tests pass

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Two-phase load order wrong | Low | High | load_signatures() MUST run before load_from_disk() |
| Memory spike during load | Low | Medium | Streaming cursor for large DBs (future optimization) |

## Security Considerations

- Signature data is minimal (no sensitive fields)
- Read-only queries on startup

## Next Steps

After Phase 3 complete:
- Phase 4: Testing & benchmarks
