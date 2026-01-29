# Phase 3: Sync Loop Optimization

## Context Links
- [Phase 2: In-Memory Store](./phase-02-in-memory-store.md)
- [Current central_client.rs](/home/elt1541/leeattend/agent-main/src/central_client.rs)

## Overview
- **Priority**: P1 (Critical)
- **Status**: pending
- **Effort**: 3h
- **Description**: Modify sync loop to read from in-memory store with 500ms latency target

## Key Insights

### Current Sync Loop Analysis (central_client.rs:61-139)
```rust
// Current behavior:
loop {
    // 1. DLQ cleanup (disk query)
    sync_db.move_expired_logs_to_dlq(...)

    // 2. Fetch unsynced from DISK (SLOW: 5-20ms)
    let logs = sync_db.get_unsynced_logs(500).await;

    // 3. Mark in-flight on DISK (SLOW: 10-30ms)
    sync_db.mark_in_flight(&log_ids).await;

    // 4. Send to server via WebSocket
    for log in logs { sync_tx.send(msg).await; }

    // 5. Sleep: 10s if caught up, 500ms if backlogged
    if count < limit { sleep(10s) } else { sleep(500ms) }
}
```

### Performance Bottleneck
- Disk read: `get_unsynced_logs()` = 5-20ms per query
- Disk write: `mark_in_flight()` = 10-30ms per batch
- Total: 15-50ms per sync cycle (+ network)
- **With in-memory: <2ms per sync cycle**

## Requirements

### Functional Requirements
1. Sync loop reads from InMemoryStore (not disk)
2. mark_in_flight updates memory immediately
3. ACK handler updates memory on success/failure
4. DLQ logic remains (checks retry_count in memory)
5. Maintain FIFO order for sync

### Non-Functional Requirements
1. Sync cycle latency <5ms (excluding network)
2. No blocking on disk I/O during sync
3. Graceful degradation if memory store unavailable

## Architecture

### Modified Data Flow
```
     Sync Loop (500ms)
            |
            v
    +---------------+
    | InMemoryStore |  <-- Fast read (<1ms)
    | get_unsynced()|
    +-------+-------+
            |
            v
    mark_in_flight()  <-- Memory update (<1ms)
            |
            v
    Send to WebSocket
            |
            v
    ACK Handler
            |
    +-------+-------+
    | mark_synced() |  <-- Memory update
    | or reset()    |
    +---------------+
```

### Timing Strategy

| Condition | Sleep Duration | Rationale |
|-----------|----------------|-----------|
| Queue empty | 500ms | Target latency |
| Queue has data | 100ms | Drain faster |
| Backlogged (>1000) | 50ms | Catch up mode |
| Network error | 1000ms | Back off |

## Related Code Files

### Files to Modify
- `src/central_client.rs` - Sync loop to use memory store
- `src/main.rs` - Initialize hybrid store, pass to CentralClient

### Files to Reference
- `src/memory_store.rs` - Created in Phase 2

## Implementation Steps

### Step 1: Update CentralClient Constructor

```rust
// src/central_client.rs

use crate::memory_store::InMemoryStore;
use std::sync::Arc;

pub struct CentralClient {
    url: String,
    agent_id: String,
    token: String,
    db: Database,                         // Keep for DLQ and config
    store: Arc<InMemoryStore>,            // NEW: In-memory store
    tx: mpsc::Sender<CentralMessage>,
    reload_tx: mpsc::Sender<()>,
    is_connected: Arc<AtomicBool>,
}

impl CentralClient {
    pub fn new(
        url: String,
        agent_id: String,
        token: String,
        db: Database,
        store: Arc<InMemoryStore>,        // NEW: Accept store
    ) -> (Self, mpsc::Receiver<CentralMessage>, mpsc::Receiver<()>) {
        let (tx, rx) = mpsc::channel(100);
        let (reload_tx, reload_rx) = mpsc::channel(1);
        let is_connected = Arc::new(AtomicBool::new(false));
        (
            Self { url, agent_id, token, db, store, tx, reload_tx, is_connected },
            rx,
            reload_rx
        )
    }
}
```

### Step 2: Modify Sync Loop

```rust
// Inside CentralClient::start()

let sync_store = self.store.clone();
let sync_db = db.clone();
let sync_tx = tx.clone();
let sync_agent_id = agent_id.clone();
let connected = self.is_connected.clone();

tokio::spawn(async move {
    loop {
        if !connected.load(Ordering::Relaxed) {
            warn!("Sync loop waiting - not connected");
            sleep(Duration::from_secs(5)).await;
            continue;
        }

        // DLQ cleanup still uses disk DB (low frequency)
        let max_retries = match sync_db.get_agent_config("max_retry_count").await {
            Ok(Some(val)) => val.parse::<i64>().unwrap_or(10),
            _ => 10,
        };
        let ttl_minutes = match sync_db.get_agent_config("retry_ttl_minutes").await {
            Ok(Some(val)) => val.parse::<i64>().unwrap_or(60),
            _ => 60,
        };

        // Check in-memory for expired logs and move to DLQ
        // (Implementation in Phase 5)

        // === CHANGED: Read from memory instead of disk ===
        let limit = 500;
        let logs = sync_store.get_unsynced(limit);
        let count = logs.len();

        if count > 0 {
            let new_count = logs.iter().filter(|l| l.updated_at.is_none()).count();
            let retry_count = count - new_count;

            if retry_count > 0 {
                info!("Syncing {} logs ({} new, {} retries)", count, new_count, retry_count);
            } else {
                info!("Syncing {} logs", count);
            }

            // === CHANGED: Mark in-flight in MEMORY ===
            let log_ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
            sync_store.mark_in_flight(&log_ids);

            // Send to server (unchanged)
            for log in logs {
                let msg = CentralMessage::Attendance {
                    msg_id: log.id.to_string(),
                    agent_id: sync_agent_id.clone(),
                    device_id: log.device_id.clone(),
                    data: AttendanceData {
                        user_id: crate::utils::normalize_employee_id(&log.user_id),
                        timestamp: log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                        status: log.status,
                        punch: log.punch,
                    },
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                if sync_tx.send(msg).await.is_err() {
                    error!("Failed to send log to sync channel");
                    break;
                }
            }
        }

        // === CHANGED: Adaptive sleep based on queue size ===
        let unsynced_count = sync_store.count_unsynced();
        let sleep_duration = if count == 0 {
            Duration::from_millis(500)  // Idle: check every 500ms
        } else if unsynced_count > 1000 {
            Duration::from_millis(50)   // Backlogged: drain fast
        } else if unsynced_count > 100 {
            Duration::from_millis(100)  // Moderate: drain
        } else {
            Duration::from_millis(200)  // Normal: balanced
        };

        sleep(sleep_duration).await;
    }
});
```

### Step 3: Modify ACK Handler

```rust
// Inside handle_incoming_text()

pub async fn handle_incoming_text(
    text: &str,
    db: &Database,
    store: &Arc<InMemoryStore>,    // NEW: Add store parameter
    reload_tx: &mpsc::Sender<()>
) {
    if let Ok(msg) = serde_json::from_str::<CentralMessage>(text) {
        match msg {
            CentralMessage::Ack { msg_id, status, .. } => {
                if let Ok(id) = msg_id.parse::<i64>() {
                    if status.as_deref() == Some("success") {
                        // === CHANGED: Update memory store ===
                        store.mark_synced(&[id]);
                    } else {
                        // === CHANGED: Reset in memory ===
                        store.reset_to_unsynced(id);
                    }
                }
            }
            // ... other handlers unchanged
        }
    }
}
```

### Step 4: Update main.rs Initialization

```rust
// src/main.rs

use agent::memory_store::InMemoryStore;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... existing setup ...

    let db = Database::new(&config.database_url).await?;

    // === NEW: Initialize in-memory store ===
    let store = Arc::new(InMemoryStore::new(100_000)); // 100K max logs

    // Load existing unsynced logs from disk into memory
    info!("Loading unsynced logs into memory...");
    let existing_logs = db.get_all_logs().await?; // Need to add this method
    store.load_from_disk(existing_logs);
    info!("Loaded {} logs into memory", store.count_total());

    // Pass store to CentralClient
    let (central_client, central_rx, mut reload_rx) = CentralClient::new(
        config.central_server_url.clone(),
        config.agent_id.clone(),
        config.agent_token.clone(),
        db.clone(),
        store.clone(),                    // NEW: Pass store
    );
    central_client.start(central_rx).await;

    // ... rest unchanged ...
}
```

### Step 5: Add get_all_logs to Database

```rust
// src/database.rs

impl Database {
    /// Get all logs for memory store initialization (startup only)
    pub async fn get_all_logs(&self) -> Result<Vec<RawAttendanceLog>, Error> {
        let logs = sqlx::query_as::<_, RawAttendanceLog>(
            r#"
            SELECT id, device_id, user_id, timestamp, status, punch,
                   is_synced, created_at, updated_at, retry_count
            FROM zkt_raw_attendance
            WHERE is_synced IN (0, 2)  -- Only load pending/in-flight
            ORDER BY id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }
}
```

## Todo List

- [ ] Update CentralClient constructor to accept Arc<InMemoryStore>
- [ ] Modify sync loop to read from memory store
- [ ] Modify ACK handler to update memory store
- [ ] Update main.rs to initialize and pass memory store
- [ ] Add get_all_logs() to Database for startup load
- [ ] Implement adaptive sleep based on queue size
- [ ] Add metrics logging for sync latency
- [ ] Test with mock WebSocket server

## Success Criteria

- [ ] Sync cycle completes in <5ms (excluding network)
- [ ] 500ms end-to-end latency achievable
- [ ] No disk I/O during normal sync operations
- [ ] ACK updates propagate to memory immediately
- [ ] FIFO order maintained

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Memory/disk inconsistency | High | Periodic checkpoint (Phase 4) |
| Lost ACKs | Medium | In-flight timeout reset |
| Thundering herd on reconnect | Medium | Staggered sync start |

## Next Steps

Proceed to Phase 4: Disk Persistence Layer for checkpoint implementation.
