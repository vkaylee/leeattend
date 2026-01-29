# Phase 5: Crash Recovery

## Context Links
- [Phase 4: Disk Persistence](./phase-04-disk-persistence.md)
- [Database migrations](/home/elt1541/leeattend/agent-main/migrations/)

## Overview
- **Priority**: P2 (Important)
- **Status**: complete
- **Effort**: 2h
- **Description**: Handle unexpected crashes with acceptable 5-10s data loss window

## Key Insights

### Crash Scenarios

| Scenario | Data at Risk | Recovery Strategy |
|----------|--------------|-------------------|
| Process kill (SIGKILL) | Unpersisted logs (5-10s) | Accept loss, reload from disk |
| Power failure | Unpersisted logs + SQLite WAL | WAL recovery + accept memory loss |
| OOM kill | All in-memory | Reload from disk checkpoint |
| Disk full during checkpoint | Current batch | Retry on restart |

### Acceptable Data Loss
- **Requirement**: Max 5-10 seconds of data loss
- **With 5s checkpoint**: Worst case = 5s of logs lost
- **Mitigation**: Device re-polls, server reconciliation

## Requirements

### Functional Requirements
1. On startup, detect incomplete shutdown (crash marker)
2. Reset in-flight logs (is_synced=2) to unsynced (is_synced=0)
3. Reconcile ID counter with disk state
4. Log recovery actions for audit

### Non-Functional Requirements
1. Recovery completes in <5 seconds
2. No orphaned in-flight states
3. No ID collisions after crash

## Architecture

### Startup Recovery Sequence
```
1. Open Database (SQLite)
        |
        v
2. Check crash marker file
        |
   +----+----+
   |         |
Clean    Crashed
   |         |
   v         v
3a. Normal  3b. Reset in-flight
   start        logs to unsynced
        |
        v
4. Get max_id from disk
        |
        v
5. Create InMemoryStore(max_id + 1)
        |
        v
6. Load pending logs (is_synced=0,2)
        |
        v
7. Write crash marker file
        |
        v
8. Start normal operation
        |
        v
9. Delete crash marker on clean shutdown
```

### Crash Marker File
```
Location: {database_dir}/.agent_running
Content: PID and timestamp
```

## Related Code Files

### Files to Modify
- `src/main.rs` - Startup recovery logic
- `src/database.rs` - Reset in-flight method

### Files to Create
- `src/crash_recovery.rs` (~80 lines)

## Implementation Steps

### Step 1: Create Crash Marker Module

```rust
// src/crash_recovery.rs

use std::fs;
use std::path::Path;
use log::{info, warn};

const CRASH_MARKER_FILE: &str = ".agent_running";

pub struct CrashMarker {
    path: String,
}

impl CrashMarker {
    pub fn new(db_path: &str) -> Self {
        // Extract directory from database URL
        // "sqlite:foo.db?mode=rwc" -> "."
        // "sqlite:/path/to/foo.db" -> "/path/to"
        let dir = if db_path.starts_with("sqlite:") {
            let path = db_path.strip_prefix("sqlite:").unwrap_or(db_path);
            let path = path.split('?').next().unwrap_or(path);
            Path::new(path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string())
        } else {
            ".".to_string()
        };

        let path = format!("{}/{}", dir, CRASH_MARKER_FILE);
        Self { path }
    }

    /// Check if crash marker exists (indicates previous crash)
    pub fn check_crash(&self) -> bool {
        Path::new(&self.path).exists()
    }

    /// Read crash marker content (for logging)
    pub fn read_crash_info(&self) -> Option<String> {
        fs::read_to_string(&self.path).ok()
    }

    /// Write crash marker (called on startup)
    pub fn write(&self) -> std::io::Result<()> {
        let content = format!(
            "pid={}\nstarted={}\n",
            std::process::id(),
            chrono::Utc::now().to_rfc3339()
        );
        fs::write(&self.path, content)
    }

    /// Remove crash marker (called on clean shutdown)
    pub fn remove(&self) -> std::io::Result<()> {
        if Path::new(&self.path).exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

pub async fn recover_from_crash(db: &crate::database::Database) -> Result<u64, sqlx::Error> {
    info!("Crash recovery: Resetting in-flight logs...");

    // Reset is_synced=2 to is_synced=0
    let reset_count = db.reset_syncing_status().await?;

    // Also reset any logs stuck in stale syncing state
    let stale_count = db.reset_stale_syncing(300).await?; // 5 minute timeout

    let total = reset_count + stale_count;
    if total > 0 {
        warn!("Crash recovery: Reset {} in-flight logs to pending", total);
    }

    Ok(total)
}
```

### Step 2: Update Database for Recovery

```rust
// src/database.rs - Add/modify methods

impl Database {
    /// Reset all in-flight logs to unsynced (crash recovery)
    pub async fn reset_syncing_status(&self) -> Result<u64, Error> {
        let result = sqlx::query(
            "UPDATE zkt_raw_attendance SET is_synced = 0 WHERE is_synced = 2"
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Reset stale in-flight logs (timeout recovery)
    /// Already exists, but ensure it returns count
    pub async fn reset_stale_syncing(&self, timeout_secs: i64) -> Result<u64, Error> {
        let result = sqlx::query(
            "UPDATE zkt_raw_attendance SET is_synced = 0 \
             WHERE is_synced = 2 \
             AND (strftime('%s', 'now') - strftime('%s', updated_at)) > ?"
        )
        .bind(timeout_secs)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
```

### Step 3: Integrate Recovery into main.rs

```rust
// src/main.rs - Startup sequence

use agent::crash_recovery::{CrashMarker, recover_from_crash};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... existing setup ...

    let config = agent::config::Config::load();

    // === CRASH RECOVERY ===
    let crash_marker = CrashMarker::new(&config.database_url);

    if crash_marker.check_crash() {
        warn!("Detected previous crash or unclean shutdown");
        if let Some(info) = crash_marker.read_crash_info() {
            warn!("Previous session info:\n{}", info);
        }
    }

    let db = Database::new(&config.database_url).await?;

    // Perform crash recovery if needed
    if crash_marker.check_crash() {
        let recovered = recover_from_crash(&db).await?;
        info!("Crash recovery complete: {} logs reset", recovered);
    }

    // Write new crash marker
    if let Err(e) = crash_marker.write() {
        warn!("Failed to write crash marker: {}", e);
    }

    // ... rest of initialization ...

    // === CLEAN SHUTDOWN ===
    // In shutdown handler:
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Shutdown signal received (Ctrl+C).");
            cloned_token.cancel();
        }
    });

    // At very end of main:
    // Remove crash marker on clean exit
    if let Err(e) = crash_marker.remove() {
        warn!("Failed to remove crash marker: {}", e);
    }
    info!("Clean shutdown complete.");

    Ok(())
}
```

### Step 4: ID Gap Strategy

```rust
// src/memory_store.rs - Add ID gap for safety

impl InMemoryStore {
    pub fn seed_id(&self, starting_id: i64) {
        // Add gap to prevent ID collision with any in-flight logs
        // that might have been assigned but not persisted before crash
        let safe_start = starting_id + 1000; // 1000 ID gap
        self.next_id.store(safe_start, Ordering::SeqCst);
    }
}
```

### Step 5: In-Flight Timeout Handler

```rust
// src/memory_store.rs - Add in-flight timeout

impl InMemoryStore {
    /// Reset in-flight logs that have been stuck for too long
    pub fn reset_stale_inflight(&self, timeout: Duration) -> usize {
        let now = Instant::now();
        let mut to_reset = Vec::new();

        {
            let inflight = self.inflight_index.read();
            for (&id, &started_at) in inflight.iter() {
                if now.duration_since(started_at) > timeout {
                    to_reset.push(id);
                }
            }
        }

        for id in &to_reset {
            self.reset_to_unsynced(*id);
        }

        to_reset.len()
    }
}

// Add to sync loop (central_client.rs):
// Every 60 seconds, reset stale in-flight logs
let reset_count = sync_store.reset_stale_inflight(Duration::from_secs(300));
if reset_count > 0 {
    warn!("Reset {} stale in-flight logs (>5min)", reset_count);
}
```

## Todo List

- [x] Create crash_recovery.rs module
- [x] Implement CrashMarker struct
- [x] Add recover_from_crash function
- [x] Integrate recovery into main.rs startup
- [x] Add crash marker cleanup on shutdown
- [x] Implement ID gap strategy
- [x] Add in-flight timeout handler
- [x] Test crash scenarios (kill -9, OOM)
- [x] Verify no duplicate logs after recovery

## Success Criteria

- [x] Recovery completes in <5 seconds
- [x] No orphaned in-flight logs after restart
- [x] ID counter correctly seeded (no collisions)
- [x] Clean shutdown removes crash marker
- [x] Crash leaves marker intact

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Crash marker on NFS | Low | Use local filesystem |
| ID collision | High | 1000 ID gap on restart |
| Lost checkpoint | Medium | Accept 5-10s loss window |

## Security Considerations

- Crash marker contains no sensitive data (PID, timestamp only)
- Recovery does not expose additional attack surface

## Next Steps

Proceed to Phase 6: Testing & Validation after implementing crash recovery.
