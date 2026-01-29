# Phase 4: Disk Persistence Layer

## Context Links
- [Phase 3: Sync Loop](./phase-03-sync-loop-optimization.md)
- [Current database.rs](/home/elt1541/leeattend/agent-main/src/database.rs)
- [Migrations](/home/elt1541/leeattend/agent-main/migrations/)

## Overview
- **Priority**: P1 (Critical)
- **Status**: ✅ Complete (2026-01-29)
- **Effort**: 3h
- **Description**: Implement periodic disk checkpointing from in-memory store

## Key Insights

### Checkpoint Strategy Options

| Strategy | Pros | Cons | Data Loss Window |
|----------|------|------|------------------|
| Full dump every N seconds | Simple | Slow for large stores | N seconds |
| Incremental (dirty only) | Fast | Complex tracking | N seconds |
| WAL-style append | Minimal I/O | Recovery complexity | ~0 |

**Decision**: Incremental checkpoint (dirty-only) with full dump fallback

### Checkpoint Timing
- Normal: Every 5 seconds
- Heavy load (>1000 dirty): Every 2 seconds
- Shutdown: Immediate full flush

## Requirements

### Functional Requirements
1. Periodic flush of unpersisted logs to SQLite
2. Mark logs as persisted after successful write
3. Full flush on graceful shutdown
4. Startup recovery from disk
5. Maintain unique constraint (device_id, user_id, timestamp)

### Non-Functional Requirements
1. Checkpoint latency <500ms for 1000 logs
2. No blocking of sync loop during checkpoint
3. SQLite WAL mode for concurrent reads

## Architecture

### Checkpoint Worker
```
+-------------------+
| Checkpoint Worker |  (Periodic: 5s)
+--------+----------+
         |
         v
+--------+----------+
|  InMemoryStore    |
|  get_unpersisted()|
+--------+----------+
         |
         v
+--------+----------+
|  Database         |  (Batch INSERT)
|  save_batch()     |
+--------+----------+
         |
         v
+--------+----------+
|  InMemoryStore    |
|  mark_persisted() |
+-------------------+
```

### Data Flow
1. Checkpoint worker wakes every 5s
2. Get all logs where `persisted = false`
3. Batch INSERT to SQLite (ON CONFLICT DO NOTHING)
4. Mark successfully inserted logs as `persisted = true`
5. Clear dirty flag

## Related Code Files

### Files to Create
- `src/hybrid_database.rs` (~150 lines) - Orchestrates memory + disk

### Files to Modify
- `src/main.rs` - Add checkpoint worker
- `src/database.rs` - Add checkpoint batch methods

## Implementation Steps

### Step 1: Add Checkpoint Methods to Database

```rust
// src/database.rs

impl Database {
    /// Batch checkpoint: Insert unpersisted logs from memory to disk
    /// Returns: (inserted_count, skipped_duplicates)
    pub async fn checkpoint_batch(&self, logs: &[MemoryLog]) -> Result<(usize, usize), Error> {
        if logs.is_empty() {
            return Ok((0, 0));
        }

        let mut tx = self.pool.begin().await?;
        let mut inserted = 0;
        let mut skipped = 0;

        // Batch insert with ON CONFLICT DO NOTHING
        // SQLite limit: ~500 values per statement
        for chunk in logs.chunks(100) {
            let placeholders: Vec<&str> = chunk.iter()
                .map(|_| "(?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .collect();
            let query = format!(
                r#"
                INSERT INTO zkt_raw_attendance
                    (id, device_id, user_id, timestamp, status, punch, is_synced, created_at, retry_count)
                VALUES {}
                ON CONFLICT(device_id, user_id, timestamp) DO NOTHING
                "#,
                placeholders.join(",")
            );

            let mut q = sqlx::query(&query);
            for log in chunk {
                q = q.bind(log.id)
                    .bind(&log.device_id)
                    .bind(&log.user_id)
                    .bind(log.timestamp)
                    .bind(log.status)
                    .bind(log.punch)
                    .bind(log.is_synced)
                    .bind(log.created_at)
                    .bind(log.retry_count);
            }

            let result = q.execute(&mut *tx).await?;
            inserted += result.rows_affected() as usize;
            skipped += chunk.len() - result.rows_affected() as usize;
        }

        tx.commit().await?;
        Ok((inserted, skipped))
    }

    /// Update sync status for checkpointed logs
    pub async fn update_sync_status_batch(&self, updates: &[(i64, i64)]) -> Result<(), Error> {
        if updates.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for chunk in updates.chunks(100) {
            for &(id, is_synced) in chunk {
                sqlx::query("UPDATE zkt_raw_attendance SET is_synced = ? WHERE id = ?")
                    .bind(is_synced)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /// Get max ID from disk (for memory store ID seeding)
    pub async fn get_max_id(&self) -> Result<i64, Error> {
        let max_id: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(id) FROM zkt_raw_attendance"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(max_id.unwrap_or(0))
    }
}
```

### Step 2: Create HybridDatabase Wrapper

```rust
// src/hybrid_database.rs

use crate::database::Database;
use crate::memory_store::{InMemoryStore, MemoryLog, StoreError};
use crate::models::{AttendanceData, RawAttendanceLog};
use std::sync::Arc;
use log::{info, warn, error};

pub struct HybridDatabase {
    disk: Database,
    memory: Arc<InMemoryStore>,
}

impl HybridDatabase {
    pub async fn new(database_url: &str, max_memory_logs: usize) -> Result<Self, sqlx::Error> {
        let disk = Database::new(database_url).await?;
        let memory = Arc::new(InMemoryStore::new(max_memory_logs));

        // Seed ID counter from disk
        let max_id = disk.get_max_id().await?;
        memory.seed_id(max_id + 1);

        // Load pending logs into memory
        let pending = disk.get_all_logs().await?;
        let count = pending.len();
        memory.load_from_disk(pending);
        info!("HybridDatabase: Loaded {} pending logs into memory", count);

        Ok(Self { disk, memory })
    }

    pub fn memory_store(&self) -> Arc<InMemoryStore> {
        self.memory.clone()
    }

    pub fn disk_db(&self) -> &Database {
        &self.disk
    }

    /// Save attendance - goes to memory first
    pub fn save_raw_attendance(&self, device_id: &str, data: AttendanceData) -> Result<i64, StoreError> {
        self.memory.insert(device_id.to_string(), data)
    }

    /// Batch save - goes to memory first
    pub fn save_raw_attendance_batch(&self, batch: Vec<(String, AttendanceData)>) -> Result<usize, StoreError> {
        self.memory.insert_batch(batch)
    }

    /// Get unsynced logs from memory
    pub fn get_unsynced_logs(&self, limit: usize) -> Vec<MemoryLog> {
        self.memory.get_unsynced(limit)
    }

    /// Count unsynced logs
    pub fn count_unsynced_logs(&self) -> usize {
        self.memory.count_unsynced()
    }

    /// Mark logs as in-flight
    pub fn mark_in_flight(&self, ids: &[i64]) {
        self.memory.mark_in_flight(ids)
    }

    /// Mark logs as synced
    pub fn mark_synced(&self, ids: &[i64]) {
        self.memory.mark_synced(ids)
    }

    /// Reset log to unsynced (failed ACK)
    pub fn reset_to_unsynced(&self, id: i64) {
        self.memory.reset_to_unsynced(id)
    }

    /// Checkpoint unpersisted logs to disk
    pub async fn checkpoint(&self) -> Result<usize, sqlx::Error> {
        let unpersisted = self.memory.get_unpersisted();
        if unpersisted.is_empty() {
            return Ok(0);
        }

        let (inserted, _skipped) = self.disk.checkpoint_batch(&unpersisted).await?;

        // Mark as persisted in memory
        let ids: Vec<i64> = unpersisted.iter().map(|l| l.id).collect();
        self.memory.mark_persisted(&ids);
        self.memory.clear_dirty();

        Ok(inserted)
    }

    /// Full flush on shutdown
    pub async fn flush_all(&self) -> Result<usize, sqlx::Error> {
        // First checkpoint any unpersisted
        self.checkpoint().await?;

        // Then update disk with final sync states
        let all_logs = self.memory.get_all();
        let updates: Vec<(i64, i64)> = all_logs.iter()
            .filter(|l| l.persisted)  // Only update already-persisted logs
            .map(|l| (l.id, l.is_synced))
            .collect();

        self.disk.update_sync_status_batch(&updates).await?;

        Ok(updates.len())
    }
}
```

### Step 3: Add Checkpoint Worker to main.rs

```rust
// src/main.rs - Add checkpoint worker

// After initializing HybridDatabase
let checkpoint_hybrid = hybrid_db.clone();
let checkpoint_cancel = cancel_token.clone();

tokio::spawn(async move {
    info!("Checkpoint Worker started (5s interval)");

    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if checkpoint_hybrid.memory_store().is_dirty() {
                    match checkpoint_hybrid.checkpoint().await {
                        Ok(count) => {
                            if count > 0 {
                                info!("Checkpoint: {} logs persisted to disk", count);
                            }
                        }
                        Err(e) => {
                            error!("Checkpoint failed: {}", e);
                        }
                    }
                }
            }
            _ = checkpoint_cancel.cancelled() => {
                info!("Checkpoint Worker: Final flush...");
                match checkpoint_hybrid.flush_all().await {
                    Ok(count) => info!("Final flush: {} logs written", count),
                    Err(e) => error!("Final flush failed: {}", e),
                }
                info!("Checkpoint Worker stopped");
                break;
            }
        }
    }
});
```

### Step 4: Graceful Shutdown Handling

```rust
// src/main.rs - Update shutdown handler

tokio::spawn(async move {
    if tokio::signal::ctrl_c().await.is_ok() {
        info!("Shutdown signal received (Ctrl+C).");
        info!("Flushing in-memory data to disk...");

        // Cancel token will trigger checkpoint worker's final flush
        cloned_token.cancel();
    }
});

// At end of main:
// Wait for checkpoint worker to complete final flush
sleep(Duration::from_secs(2)).await;
info!("Shutdown complete.");
```

## Todo List

- [x] Add checkpoint_batch() to Database
- [x] Add update_sync_status_batch() to Database
- [x] Add get_max_id() to Database
- [x] Create HybridDatabase wrapper
- [x] Add checkpoint worker to main.rs
- [x] Update shutdown handler for graceful flush
- [x] Test checkpoint with various batch sizes
- [x] Test shutdown flush behavior
- [x] Verify no data loss on restart

## Success Criteria

- [x] Checkpoint completes in <500ms for 1000 logs
- [x] No duplicate inserts (ON CONFLICT works)
- [x] Graceful shutdown persists all in-memory data
- [x] Restart recovers all pending logs
- [x] WAL mode prevents read blocking

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Disk full | High | Monitor disk space, alert |
| Checkpoint timeout | Medium | Async with timeout, retry |
| ID collision | High | Seed from disk max_id + gap |

## Next Steps

Proceed to Phase 5: Crash Recovery for handling unexpected termination.
