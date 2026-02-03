# Race Condition: ConfigPush vs Log Sync

**Investigation Date:** 2026-01-29
**Reporter:** debugger agent
**Severity:** Medium
**Impact:** Logs synced before config validated; no data loss but behavioral inconsistency

## Executive Summary

Agent receives ConfigPush with 0 devices, warns about no active devices, but immediately syncs 500 logs anyway. Root cause: **sync loop and device configuration check are independent async tasks with no synchronization**.

## Evidence from Logs

```
[2026-01-29T16:23:47Z INFO  agent::central_client] Received ConfigPush with 0 devices
[2026-01-29T16:23:47Z INFO  agent] 🔄 Configuration change detected! Restarting all workers...
[2026-01-29T16:23:49Z WARN  agent] No active devices found in database. Waiting for config push...
[2026-01-29T16:23:51Z INFO  agent::central_client] 📤 Syncing 500 logs to central server...
```

Timeline:
- T+0s: ConfigPush received with 0 devices
- T+0s: Reload triggered
- T+2s: Device check warns no active devices
- T+4s: Sync loop proceeds with 500 logs

## Root Cause Identification

### Critical Race Condition

**File:** `/home/elt1541/leeattend/agent-main/src/central_client.rs`
**Lines:** 61-163 (sync loop) vs 256-297 (ConfigPush handler)

#### Independent Task 1: Sync Loop (Line 61-163)
```rust
tokio::spawn(async move {
    let mut last_stale_check = Instant::now();
    loop {
        if !connected.load(Ordering::Relaxed) {
            warn!("🔴 Sync loop waiting - not connected to central server");
            sleep(Duration::from_secs(5)).await;
            continue;
        }

        // NO CHECK for valid device configuration!
        let logs = sync_hybrid.get_unsynced_logs(limit);
        let count = logs.len();

        if count > 0 {
            info!("📤 Syncing {} logs to central server...", count);
            // ... sends logs
        }
    }
});
```

**Issue:** Sync loop only checks `is_connected` flag but **never validates device configuration exists**.

#### Independent Task 2: Device Worker Loop (main.rs:249-476)
```rust
while !cancel_token.is_cancelled() {
    let db_devices = match hybrid_db.disk_db().get_active_devices().await {
        Ok(devices) => devices,
        Err(e) => { /* error */ }
    };

    if db_devices.is_empty() {
        warn!("No active devices found in database. Waiting for config push...");
    } else {
        info!("📡 Found {} devices, max {} concurrent workers",
              db_devices.len(), current_max_workers);
    }

    // Creates workers for polling devices
    // ...
}
```

**Issue:** Device check happens in **main orchestrator loop**, completely separate from sync loop.

#### Race Condition Flow

1. **Startup:** `/home/elt1541/leeattend/agent-main/src/main.rs:119-123`
   ```rust
   if let Err(e) = hybrid_db.disk_db().clear_startup_config().await {
       warn!("Failed to clear startup config: {}", e);
   } else {
       info!("🔄 Cleared local config on startup - waiting for ConfigPush from Server...");
   }
   ```
   - Clears all devices: `UPDATE devices SET is_active = 0`

2. **HybridDatabase Init:** `/home/elt1541/leeattend/agent-main/src/hybrid_database.rs:32-35`
   ```rust
   let pending = disk.get_all_logs().await?;
   let pending_count = pending.len();
   memory.load_from_disk(pending);
   info!("HybridDatabase: Loaded {} pending logs into memory", pending_count);
   ```
   - **Loads all unsynced logs into memory (500 logs in this case)**
   - These logs existed from previous sessions

3. **Sync Loop Starts:** `central_client.rs:61`
   ```rust
   tokio::spawn(async move {
       loop {
           if !connected.load(Ordering::Relaxed) {
               // Wait for connection
               continue;
           }
           // Immediately starts syncing whatever is in memory
           let logs = sync_hybrid.get_unsynced_logs(limit);
   ```
   - Starts checking for logs in memory immediately after connection
   - **No validation that device config is ready**

4. **WebSocket Connects:** `central_client.rs:189-193`
   ```rust
   match connect_central(&url, &agent_id, &token).await {
       Ok((ws_stream, _)) => {
           info!("✅ Connected to Central Server.");
           ws_connected.store(true, Ordering::Relaxed);  // <-- RACE STARTS HERE
   ```
   - Sets `is_connected = true`
   - Sync loop wakes up and starts sending logs

5. **ConfigPush Arrives:** `central_client.rs:259-274`
   ```rust
   CentralMessage::ConfigPush { data } => {
       info!("Received ConfigPush with {} devices", data.devices.len());
       // ... updates devices table
       let _ = reload_tx.try_send(());  // Triggers device worker reload
   }
   ```
   - Updates device config in database
   - Triggers reload_rx in main.rs:455

6. **Main Loop Reloads:** `main.rs:455-469`
   ```rust
   _ = reload_rx.recv() => {
       info!("🔄 Configuration change detected! Restarting all workers...");
       // ... cancels workers, sleeps 2 seconds
   }
   // Then loops back to line 252
   let db_devices = match hybrid_db.disk_db().get_active_devices().await {
   ```
   - Takes 2+ seconds to restart worker loop
   - Meanwhile sync loop already sent logs

### Why 500 Logs Exist

**File:** `/home/elt1541/leeattend/agent-main/src/database.rs:194-209`
```rust
pub async fn get_all_logs(&self) -> Result<Vec<RawAttendanceLog>, Error> {
    let logs = sqlx::query_as::<_, RawAttendanceLog>(
        r#"
        SELECT id, device_id, user_id, timestamp, status, punch,
               is_synced, created_at, updated_at, retry_count
        FROM zkt_raw_attendance
        WHERE is_synced IN (0, 2)  -- 0=unsynced, 2=in-flight
        ORDER BY id ASC
        "#
    )
    .fetch_all(&self.pool)
    .await?;
```

- Loads ALL unsynced/in-flight logs from disk into memory on startup
- These are logs from previous agent sessions
- `clear_startup_config()` only clears devices, **NOT attendance logs**
- Logs persist across restarts for crash recovery

## Technical Analysis

### Synchronization Gap

**No mutex/barrier between:**
1. Device configuration state (checked in main.rs:252)
2. Sync loop execution (central_client.rs:61)

### Timing Dependencies

```
T+0s:  Agent starts, clears device config
T+0s:  HybridDatabase loads 500 logs into memory
T+0s:  Sync loop spawned (waiting for connection)
T+0s:  Main loop spawned (checking devices)
T+Xs:  WebSocket connects -> is_connected = true
T+Xs:  Sync loop starts processing immediately
T+Xs:  ConfigPush arrives, updates devices
T+X+2s: Main loop detects 0 devices (ConfigPush had 0)
T+X+4s: Sync loop already sent 500 logs
```

### Why This Happens

1. **Sync loop has no concept of device config validity**
   - Only checks: is_connected + has_logs_in_memory
   - Assumption: if logs exist, they're valid to sync

2. **Device config and log sync are orthogonal**
   - Device config controls POLLING (fetching new logs from devices)
   - Log sync controls UPLOADING (sending existing logs to server)
   - No dependency enforced between them

3. **Memory is pre-seeded with logs**
   - HybridDatabase loads logs before any config validation
   - Sync loop sees logs and immediately starts syncing

## System Behavior Patterns

### Current Behavior
- Sync loop: "I have logs → I'm connected → SEND"
- Main loop: "No devices → Can't poll new logs"
- **Disconnect:** Logs from previous sessions get synced regardless of current device config

### Expected Behavior (unclear from requirements)
Two possible interpretations:

**Option A:** Logs should NOT sync without active devices
- Prevents orphaned logs from stale device configs
- Requires device config as prerequisite for sync

**Option B:** Logs SHOULD sync regardless of device state
- Crash recovery: flush old logs even if devices removed
- Logs are already in database with valid device_id
- Current behavior is actually correct

## File Paths and Line Numbers

### ConfigPush Handling
**File:** `/home/elt1541/leeattend/agent-main/src/central_client.rs`
- **Line 259-274:** ConfigPush message handling
- **Line 260:** Log "Received ConfigPush with N devices"
- **Line 264-272:** Device upsert and deactivation logic
- **Line 273:** Triggers reload via `reload_tx.try_send(())`

### Sync Loop Execution
**File:** `/home/elt1541/leeattend/agent-main/src/central_client.rs`
- **Line 61-163:** Sync loop spawn
- **Line 65-69:** Connection check (NO device config check)
- **Line 97-112:** Get logs from memory and sync
- **Line 102-112:** Log "📤 Syncing N logs to central server..."

### Device Configuration Check
**File:** `/home/elt1541/leeattend/agent-main/src/main.rs`
- **Line 252-259:** Query active devices from database
- **Line 261-265:** Warn if empty or log device count
- **Line 455-469:** Reload handler (triggered by ConfigPush)

### Startup Config Clear
**File:** `/home/elt1541/leeattend/agent-main/src/main.rs`
- **Line 119-123:** Clear startup config (removes all active devices)

**File:** `/home/elt1541/leeattend/agent-main/src/database.rs`
- **Line 473-483:** `clear_startup_config()` implementation
- **Line 480:** `UPDATE devices SET is_active = 0` (clears all devices)

### Memory Initialization
**File:** `/home/elt1541/leeattend/agent-main/src/hybrid_database.rs`
- **Line 32-35:** Load all unsynced logs into memory on startup

**File:** `/home/elt1541/leeattend/agent-main/src/database.rs`
- **Line 194-209:** `get_all_logs()` - fetches all unsynced/in-flight logs

## Actionable Recommendations

### Immediate Fix (High Priority)
**Add device validation gate to sync loop**

**File:** `/home/elt1541/leeattend/agent-main/src/central_client.rs:65-69`

Current code:
```rust
if !connected.load(Ordering::Relaxed) {
    warn!("🔴 Sync loop waiting - not connected to central server");
    sleep(Duration::from_secs(5)).await;
    continue;
}
```

Proposed fix:
```rust
if !connected.load(Ordering::Relaxed) {
    warn!("🔴 Sync loop waiting - not connected to central server");
    sleep(Duration::from_secs(5)).await;
    continue;
}

// NEW: Check device config is ready
let device_count = match sync_hybrid.disk_db().get_active_devices().await {
    Ok(devices) => devices.len(),
    Err(_) => 0,
};

if device_count == 0 {
    warn!("🔴 Sync loop waiting - no active devices configured");
    sleep(Duration::from_secs(5)).await;
    continue;
}
```

**Trade-off:** Adds database query overhead to sync loop (runs every 50-500ms depending on queue size)

### Long-term Improvement (Medium Priority)
**Use shared atomic state for config readiness**

Add to CentralClient:
```rust
struct CentralClient {
    // ... existing fields
    is_connected: Arc<AtomicBool>,
    config_ready: Arc<AtomicBool>,  // NEW
}
```

Update ConfigPush handler:
```rust
CentralMessage::ConfigPush { data } => {
    // ... existing logic
    if data.devices.len() > 0 {
        config_ready.store(true, Ordering::Relaxed);
    } else {
        config_ready.store(false, Ordering::Relaxed);
    }
}
```

Update sync loop:
```rust
if !connected.load(Ordering::Relaxed) || !config_ready.load(Ordering::Relaxed) {
    sleep(Duration::from_secs(5)).await;
    continue;
}
```

**Benefit:** No database query overhead, simple atomic check

### Alternative: Clarify Requirements
**If current behavior is acceptable:**
- Document that sync operates independently of device config
- Explain this enables crash recovery log flushing
- Update warning message to be less alarming:
  ```rust
  warn!("No active devices configured - waiting for ConfigPush (existing logs will still sync)");
  ```

## Risk Assessment

### Current Impact
- **Data Integrity:** No data loss or corruption
- **Functional Impact:** Logs sync successfully despite warnings
- **User Experience:** Confusing log sequence suggests malfunction
- **System Health:** No actual malfunction, logs sync correctly

### If ConfigPush with 0 Devices is Valid
- System correctly flushes old logs even when devices removed
- Warnings are accurate (no devices to poll) but misleading

### If ConfigPush with 0 Devices is Invalid
- Indicates server-side issue (why send empty config?)
- Agent behaves reasonably by continuing to sync existing logs

## Unresolved Questions

1. **Is ConfigPush with 0 devices expected?**
   - If server removes all devices, should agent stop syncing?
   - Or should agent flush existing logs before going idle?

2. **Should device config gate log syncing?**
   - Prevents syncing orphaned logs from deleted devices
   - But breaks crash recovery if devices were removed while agent down

3. **What's the desired startup sequence?**
   - Wait for ConfigPush before any sync?
   - Or sync existing logs immediately on connection?

4. **Why did server send ConfigPush with 0 devices?**
   - Initial handshake before real config?
   - Bug in server-side device query?
   - Intentional "clear all" command?
