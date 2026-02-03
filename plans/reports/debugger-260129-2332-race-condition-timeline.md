# Race Condition Timeline Diagram

```
STARTUP SEQUENCE
================

Time    Main Thread              Sync Loop Thread         WebSocket Thread        Database State
----    -----------              ----------------         ----------------        --------------
T+0s    Agent starts
        └─> clear_startup_config()                                               devices: [] (all deactivated)
        └─> HybridDatabase::new()                                                logs: [500 unsynced logs loaded to memory]
        └─> CentralClient::new()
                                 Spawned (waiting)
        └─> spawn sync loop ────────────┘
        └─> spawn WS manager ──────────────────────────────> Spawned
                                                             └─> Connecting...

T+1s                                                         Connected!
                                                             ws_connected = true ──> is_connected: true

T+1s                             Wakes up!
                                 Check: is_connected? ✓
                                 Check: device config? ✗ (MISSING!)
                                 get_unsynced_logs(500)
                                 Found: 500 logs
                                 └─> Start syncing...

T+2s                             📤 Syncing log 1/500...
                                 📤 Syncing log 2/500...
                                                             Handshake sent
                                                             Waiting for ConfigPush...

T+3s                             📤 Syncing log 100/500...
                                                             ConfigPush received:
                                                             { devices: [] }
                                                             └─> Process empty list  devices: [] (still empty)
                                                             └─> reload_tx.send()

T+3s    reload_rx received!
        "🔄 Config change detected"
        └─> Cancel workers
        └─> sleep(2s)...
                                 📤 Syncing log 200/500...

T+4s                             📤 Syncing log 300/500...

T+5s    Sleep done
        get_active_devices()                                                     devices: []
        └─> Found: 0 devices
        ⚠️ "No active devices"
                                 📤 Syncing log 500/500
                                 ✅ Sync complete

T+6s    Start worker loop
        (No workers to spawn)
        └─> Wait for next reload...


THE RACE CONDITION
==================

Critical Gap: Lines 65-69 in central_client.rs

    if !connected.load(Ordering::Relaxed) {
        warn!("Sync loop waiting - not connected");
        sleep(5s);
        continue;
    }

    // ⚠️ MISSING CHECK HERE:
    // if !has_valid_device_config() {
    //     warn!("Sync loop waiting - no device config");
    //     sleep(5s);
    //     continue;
    // }

    let logs = sync_hybrid.get_unsynced_logs(limit);  // Proceeds anyway!


INDEPENDENT ASYNC TASKS
========================

Task 1: Sync Loop (central_client.rs:61)
│
├─> Checks: is_connected flag only
├─> Polls: Memory for unsynced logs
└─> Sends: Logs to server
    └─> NO DEPENDENCY on device config

Task 2: Device Worker Loop (main.rs:249)
│
├─> Checks: get_active_devices() from database
├─> Warns: If empty
└─> Spawns: Device polling workers
    └─> NO DEPENDENCY on sync state

Task 3: ConfigPush Handler (central_client.rs:259)
│
├─> Updates: Devices in database
├─> Signals: reload_tx (triggers Task 2)
└─> NO SYNC with Task 1


MEMORY STATE vs DEVICE STATE
=============================

Memory State (HybridDatabase)           Device State (Database)
─────────────────────────               ──────────────────────

On Startup:                             On Startup:
┌──────────────────────┐               ┌──────────────────────┐
│ Load from disk:      │               │ clear_startup_config │
│ - 500 unsynced logs  │               │ UPDATE devices       │
│   (from prev session)│               │ SET is_active = 0    │
│                      │               │                      │
│ Ready to sync ✓      │               │ No devices ✗         │
└──────────────────────┘               └──────────────────────┘

Sync Loop reads from:                  Device Workers read from:
      ↓                                        ↓
    MEMORY                                  DATABASE
    (has 500 logs)                          (0 devices)
      ↓                                        ↓
   ✅ SYNC!                                  ⚠️ WAIT!


DATA FLOW
=========

Old Session (yesterday):
  Device "Office" → Polls 500 logs → Saves to DB → Agent crashes before sync

Current Session (today):
  1. Agent starts
  2. Clears device config (Office deactivated)
  3. Loads 500 old logs into memory
  4. Connects to server
  5. Syncs 500 logs (success!)
  6. ConfigPush arrives with 0 devices
  7. Main loop warns "No active devices"

Result: Logs synced successfully, but sequence appears broken


ROOT CAUSE SUMMARY
==================

┌─────────────────────────────────────────────────────────┐
│ Sync loop and device config are ORTHOGONAL concerns:   │
│                                                         │
│ Sync Loop:     "Send whatever is in memory"            │
│                ↓                                        │
│                Checks: is_connected                     │
│                Does NOT check: device_config_valid      │
│                                                         │
│ Device Loop:   "Poll new logs from configured devices" │
│                ↓                                        │
│                Checks: get_active_devices()             │
│                Does NOT affect: sync loop               │
│                                                         │
│ Result: Race condition when device config updates      │
│         during active sync operations                  │
└─────────────────────────────────────────────────────────┘
```
