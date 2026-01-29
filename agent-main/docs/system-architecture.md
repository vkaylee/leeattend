# System Architecture Documentation

**Project:** Agent Attendance Sync System
**Version:** 1.0 (Hybrid In-Memory Database)
**Status:** Production Ready
**Last Updated:** 2026-01-29

---

## High-Level Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                  EXTERNAL INTERFACES                         │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐      ┌───────────────┐  ┌──────────────┐  │
│  │Time-Clock    │      │Central Server │  │Admin (Config)│  │
│  │Devices       │      │(sync target)  │  │Push Service  │  │
│  └──────┬───────┘      └───────▲───────┘  └────────┬─────┘  │
│         │ TCP/IP              │ WebSocket          │ HTTP   │
│         │ Polling(5s)         │ Batch Send         │ REST   │
│         └──────────┬──────────┘                    │        │
│                    │                              │        │
│         ┌──────────▼──────────────────────────────▼─────┐  │
│         │     AGENT SYSTEM (Rust Binary)                │  │
│         ├───────────────────────────────────────────────┤  │
│         │                                               │  │
│         │  ┌─────────────────────────────────────────┐ │  │
│         │  │ Worker Clients (Device Polling)         │ │  │
│         │  │ - Poll devices every 5s                 │ │  │
│         │  │ - Parse attendance records              │ │  │
│         │  │ - Send via mpsc channel                 │ │  │
│         │  │ - Handle device errors                  │ │  │
│         │  └────────────────┬──────────────────────┘ │  │
│         │                   │                        │  │
│         │  ┌────────────────▼──────────────────────┐ │  │
│         │  │ mpsc Channel (10K buffer)             │ │  │
│         │  │ - Backpressure on overflow            │ │  │
│         │  │ - Worker blocking if full             │ │  │
│         │  └────────────────┬──────────────────────┘ │  │
│         │                   │                        │  │
│         │  ┌────────────────▼──────────────────────┐ │  │
│         │  │ DB Writer Worker                      │ │  │
│         │  │ - Batch logs (100-200 logs/batch)     │ │  │
│         │  │ - Write to HybridDatabase             │ │  │
│         │  │ - Flush interval: 200ms               │ │  │
│         │  └────────────────┬──────────────────────┘ │  │
│         │                   │                        │  │
│         │  ┌────────────────▼──────────────────────┐ │  │
│         │  │ HYBRID DATABASE LAYER                 │ │  │
│         │  ├──────────────────────────────────────┤ │  │
│         │  │                                      │ │  │
│         │  │  ┌─────────────────────────────────┐│ │  │
│         │  │  │ InMemoryStore (DashMap)         ││ │  │
│         │  │  │ - Fast reads: <1ms              ││ │  │
│         │  │  │ - FIFO ordering (BTreeSet)      ││ │  │
│         │  │  │ - Backoff filtering             ││ │  │
│         │  │  │ - Max 100K logs (~50MB)         ││ │  │
│         │  │  │ - Capacity: bounded queue       ││ │  │
│         │  │  │ - Backpressure: blocking insert││ │  │
│         │  │  └────┬──────────────┬─────────┬──┘│ │  │
│         │  │       │              │         │   │ │  │
│         │  │  ┌────▼────────┐  ┌─▼──────┐  │  │ │  │
│         │  │  │Checkpoint   │  │Unsynced│  │  │ │  │
│         │  │  │Worker (5s)  │  │Index   │  │  │ │  │
│         │  │  └────┬────────┘  └────────┘  │  │ │  │
│         │  │       │                       │  │ │  │
│         │  │  ┌────▼──────────────────────┐│ │  │
│         │  │  │ SQLite Database (Disk)    ││ │  │
│         │  │  │ - Durability (backup)     ││ │  │
│         │  │  │ - Schema: normalized      ││ │  │
│         │  │  │ - Max 5s data loss        ││ │  │
│         │  │  │ - Migrations (8 total)    ││ │  │
│         │  │  └────┬─────────────────────┘│ │  │
│         │  │       │ Persisted Data       │ │  │
│         │  └───────┴──────────────────────┘ │  │
│         │                │                  │  │
│         │  ┌─────────────▼──────────────┐   │  │
│         │  │ Central Client              │   │  │
│         │  │ - WebSocket to Server       │   │  │
│         │  │ - 500ms sync cycle          │   │  │
│         │  │ - Adaptive sleep            │   │  │
│         │  │ - Backoff monitoring        │   │  │
│         │  │ - ACK handler               │   │  │
│         │  └─────────────┬───────────────┘   │  │
│         │                │                   │  │
│         │  ┌─────────────▼──────────────┐   │  │
│         │  │ Worker Client               │   │  │
│         │  │ - ConfigPush handler        │   │  │
│         │  │ - HTTP endpoints            │   │  │
│         │  │ - Command execution         │   │  │
│         │  └─────────────┬───────────────┘   │  │
│         │                │                   │  │
│         │  ┌─────────────▼──────────────┐   │  │
│         │  │ Graceful Shutdown           │   │  │
│         │  │ - Full checkpoint flush     │   │  │
│         │  │ - Wait all workers          │   │  │
│         │  │ - Remove marker file        │   │  │
│         │  └─────────────────────────────┘   │  │
│         │                                   │  │
│         └───────────────────────────────────┘  │
│                                               │
└──────────────────────────────────────────────┘
```

---

## Component Details

### 1. Device Polling Workers

**Modules:** `zk_client.rs`, `main.rs`

**Responsibility:**
- Connect to time-clock devices (ZKTeco protocol)
- Poll for new attendance records every 5 seconds
- Parse device response into AttendanceData
- Handle connection failures with backoff

**Data Flow:**
```
Device (TCP) → zk_client (parse) → AttendanceData → mpsc::channel
```

**Key Operations:**
```rust
let data = AttendanceData {
    user_id: "0026",           // Employee ID
    timestamp: "2026-01-29 10:30:00",
    status: Some(1),           // Check-in=1, Check-out=2
    punch: Some(1),            // Punch type
};
```

**Error Handling:**
- Device unreachable: Retry with exponential backoff
- Parse error: Log to `parsing_errors` table
- Connection timeout: Skip this poll cycle

---

### 2. mpsc Channel & Buffer

**Purpose:** Decouple polling from storage

**Configuration:**
- Buffer size: 10K records
- Blocking behavior: Workers block if full
- Monitors: Backpressure alerts if >8K

**Data Structure:**
```
Device 1 ─┐
Device 2 ─┼──> mpsc::channel(10K) ──> DB Writer Worker
Device N ─┘
```

**Backpressure:**
- If channel full: Device polling blocks
- Prevents memory explosion
- Natural throttle (slow DB = slow polling)

---

### 3. DB Writer Worker

**Module:** `main.rs`

**Responsibility:**
- Consume from mpsc channel
- Batch records (100-200 per flush)
- Write batches to HybridDatabase
- Handle write errors

**Batching Strategy:**
```rust
loop {
    // Collect batch with timeout
    let mut batch = Vec::new();
    let timeout = Duration::from_millis(200);

    while batch.len() < 100 && elapsed < timeout {
        if let Ok(record) = channel.recv_timeout(timeout) {
            batch.push(record);
        }
    }

    // Write batch
    if !batch.is_empty() {
        hybrid_db.save_logs(&batch)?;
    }
}
```

**Performance Impact:**
- Batching: 100 records per 200ms = 500 records/sec
- Reduces write frequency
- Improves I/O efficiency

---

### 4. HybridDatabase Layer

**Module:** `hybrid_database.rs`

**Architecture:**
```
    Client Code (main, central_client)
           │
           ├──> save_logs()
           │    ├─> InMemoryStore::insert() [<1ms]
           │    └─> Async checkpoint scheduled
           │
           └──> get_unsynced_logs()
                └─> InMemoryStore::get_unsynced() [<1ms]
```

**Key Methods:**

| Method | Latency | Target |
|--------|---------|--------|
| `save_logs()` | <1ms | Dual-write (memory first) |
| `get_unsynced_logs()` | <1ms | Memory reads only |
| `checkpoint()` | 50-500ms | Async, non-blocking |
| `reset_in_flight()` | <1ms | Crash recovery |

**Dual-Write Strategy:**
```rust
pub fn save_logs(&self, logs: &[MemoryLog]) -> Result<usize, Error> {
    // 1. Write to fast path (CRITICAL)
    for log in logs {
        self.memory.insert(log.id, log.clone())?;
    }

    // 2. Write to durable path (background)
    tokio::spawn(async move {
        self.database.save_logs(logs).await
            .or_insert_in_backoff(); // Retry if fails
    });

    Ok(logs.len())
}
```

---

### 5. InMemoryStore

**Module:** `memory_store.rs`
**Size:** 583 lines | **Tests:** 11 | **Coverage:** 98%

**Data Structure:**
```rust
pub struct InMemoryStore {
    logs: DashMap<i64, MemoryLog>,           // Primary storage
    unsynced_index: RwLock<BTreeSet<i64>>,   // Fast unsynced queries
    inflight_index: RwLock<HashMap<i64, Instant>>,  // Timeout tracking
    next_id: AtomicI64,                       // ID generator
    dirty: AtomicBool,                        // Checkpoint optimization
    count: AtomicUsize,                       // Backpressure tracking
}
```

**Operation Latencies:**
- Insert: <1ms (DashMap sharded write)
- Query unsynced: <1ms (BTreeSet iteration)
- Mark in-flight: <1ms (index update)
- Count: <1ms (atomic read)

**Backoff Algorithm:**
```
For each unsynced log:
    if log.updated_at == None:
        // New log (never synced)
        return immediately
    else:
        // Retry attempt
        delay_secs = min(2^retry_count, 60)
        eligible_at = log.updated_at + delay_secs
        if now >= eligible_at:
            return  // Ready for retry
        else:
            skip    // Still in backoff
```

**Backoff Timeline:**
| Attempt | delay | Total |
|---------|-------|-------|
| New (0) | 0s | immediate |
| 1st retry (1) | 2s | 2s |
| 2nd retry (2) | 4s | 6s |
| 3rd retry (3) | 8s | 14s |
| 4th retry (4) | 16s | 30s |
| 5th retry (5) | 32s | 62s |
| 6+ retries (6+) | 60s | 120s+ |

---

### 6. Checkpoint Worker

**Module:** `main.rs`
**Interval:** 5 seconds (configurable)

**Workflow:**
```
Every 5 seconds:
    1. Query dirty flag from InMemoryStore
    2. If dirty:
        a. Get all persisted=false logs from memory
        b. Batch INSERT to SQLite (ON CONFLICT DO NOTHING)
        c. Mark as persisted in memory
        d. Clear dirty flag
    3. If error:
        - Log warning
        - Retry on next cycle
        - Logs remain in memory
```

**Persistence Guarantee:**
- Graceful shutdown: 100% (full flush)
- Normal operation: All logs synced eventually
- Crash: Last ~5s of logs may be lost

**Non-Blocking Design:**
```rust
tokio::spawn({
    let db = hybrid_db.clone();
    async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            if let Err(e) = db.checkpoint().await {
                error!("Checkpoint failed: {}", e);
                // Continue, retry next cycle
            }
        }
    }
});
```

---

### 7. Crash Recovery System

**Module:** `crash_recovery.rs`
**Size:** 119 lines | **Tests:** 6 | **Coverage:** 94%

**Mechanism:**

```
Startup:
    1. Check if .agent_running exists
        YES → Crash detected!
             - Load unsynced logs from disk into memory
             - Find all is_synced=2 (in-flight) logs
             - Reset them to is_synced=0 (unsynced)
             - Update disk
             - Clear marker file
             - Resume normal operation
        NO  → Normal startup
             - Load disk data into memory
             - Create marker file
             - Begin polling
```

**Key Design Decisions:**

1. **Marker File** (not database flag)
   - Simpler implementation
   - No DB query on startup
   - Survives partial crashes

2. **ID Gap Management** (1000 ID buffer)
   - Prevents collision on restart
   - Max gap: 50 restarts (50K IDs)
   - Collision detection in save_logs

3. **Stale In-Flight Timeout** (5 minutes)
   - Auto-reset logs stuck in-flight
   - Prevents indefinite blocking
   - Rare in practice

---

### 8. Central Client (Sync Loop)

**Module:** `central_client.rs`
**Size:** 489 lines | **Tests:** Multiple integration tests

**Sync Cycle** (500ms target):

```
┌────────────────────────────────────┐
│ Start Sync Cycle (every 500ms)     │
└────────┬─────────────────────────┘
         │
         ├─> Read unsynced logs [<1ms]
         │   └─> From InMemoryStore (memory)
         │
         ├─> Filter by backoff [<1ms]
         │   └─> Skip recent retries
         │
         ├─> Serialize to JSON [1-5ms]
         │   └─> Batch: 100 logs
         │
         ├─> Send via WebSocket [200-1000ms]
         │   └─> To central server
         │
         ├─> Wait for ACK [200-1000ms]
         │   └─> Server confirms receipt
         │
         ├─> Mark synced [<1ms]
         │   └─> Update is_synced=1 in memory
         │
         └─> Adaptive sleep [remaining time]
             ├─> If backlog: 50ms
             ├─> If normal: 500ms
             └─> If ahead: 500ms
```

**Adaptive Sleep Logic:**
```rust
let unsynced_count = store.get_unsynced_count();

let sleep_ms = if unsynced_count > 500 {
    50    // Backlog: poll fast
} else if unsynced_count > 100 {
    100   // Moderate: poll medium
} else {
    500   // Caught up: poll slow
};

tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
```

**WebSocket Message Format:**
```json
{
    "logs": [
        {
            "id": 1001,
            "device_id": "D001",
            "user_id": "0026",
            "timestamp": "2026-01-29 10:30:00",
            "status": 1,
            "punch": 1
        }
    ]
}
```

**ACK Handler:**
```rust
match server_response {
    AckMessage { synced_ids } => {
        for id in synced_ids {
            memory_store.mark_synced(id)?;
        }
    }
    ErrorMessage { reason } => {
        error!("Sync failed: {}", reason);
        // Retry with backoff
    }
}
```

---

### 9. Worker Client (Configuration)

**Module:** `worker_client.rs`
**Size:** 200 lines

**HTTP Endpoints:**
- `POST /worker/config-push` - Receive config updates
- `POST /worker/log-push` - Receive direct logs
- `GET /worker/ping` - Health check

**ConfigPush Handler:**
```
Receive ConfigPush
    ├─> Parse new configuration
    ├─> Update agent_config table
    ├─> Reload config in memory
    └─> No restart needed
```

**LogPush Handler:**
```
Receive LogPush (direct upload from server)
    ├─> Parse attendance records
    ├─> Save to database
    └─> Return ACK
```

---

### 10. Database Layer

**Module:** `database.rs`
**Size:** 973 lines | **Tests:** 37

**Schema:**

| Table | Purpose | Columns |
|-------|---------|---------|
| `zkt_raw_attendance` | Primary logs | id, device_id, user_id, timestamp, status, punch, is_synced, retry_count, updated_at |
| `devices` | Device config cache | id, name, ip, port, brand, last_seen |
| `agent_config` | Local settings | key, value |
| `zkt_failed_logs` | Dead letter queue | id, device_id, user_id, timestamp, failure_reason, retry_attempts |
| `parsing_errors` | Device parse errors | id, device_id, raw_data, error_msg |

**SQLite Configuration:**
```sql
PRAGMA journal_mode=WAL;      -- Write-ahead logging (durability)
PRAGMA synchronous=NORMAL;    -- Balance speed/safety
PRAGMA busy_timeout=30000;    -- Wait for locks
PRAGMA cache_size=-64000;     -- 64MB cache
PRAGMA temp_store=MEMORY;     -- Temp tables in memory
```

**Key Operations:**

1. **Batch Insert:**
   ```sql
   INSERT OR REPLACE INTO zkt_raw_attendance (...)
   VALUES (...), (...), ...
   ```

2. **Query Unsynced:**
   ```sql
   SELECT * FROM zkt_raw_attendance
   WHERE is_synced = 0
   ORDER BY id ASC
   LIMIT ?
   ```

3. **Mark Synced:**
   ```sql
   UPDATE zkt_raw_attendance
   SET is_synced = 1, updated_at = NOW()
   WHERE id IN (?, ?, ...)
   ```

---

## Data Flow Diagrams

### Complete Sync Workflow

```
Timeline: Device → Memory → Disk → Server (End-to-End)

T=0:00 ┌─────────────────────────────────────────────┐
       │ Device generates attendance record          │
       │ User ID: 0026, Time: 10:30:00              │
       └─────────────────┬───────────────────────────┘
                         │ TCP/IP
T=0:05 ┌─────────────────▼───────────────────────────┐
       │ Device Polling Worker (zk_client)          │
       │ - Connect to device                         │
       │ - Retrieve records                          │
       │ - Parse to AttendanceData                   │
       └─────────────────┬───────────────────────────┘
                         │
T=0:05 ┌─────────────────▼───────────────────────────┐
       │ Send to mpsc channel                        │
       │ - Enqueue for DB writer                    │
       └─────────────────┬───────────────────────────┘
                         │
T=0:05 ┌─────────────────▼───────────────────────────┐
       │ InMemoryStore (immediate)                   │
       │ - Assign ID: 1001                          │
       │ - Add to DashMap                           │
       │ - Add to unsynced_index                    │
       │ Latency: <1ms                              │
       └─────────────────┬───────────────────────────┘
                         │
T=0:20 ┌─────────────────▼───────────────────────────┐
       │ DB Writer batch (200ms window)              │
       │ - Collect 100 records or timeout           │
       │ - Write batch to disk                      │
       │ Latency: 10-50ms                           │
       └─────────────────┬───────────────────────────┘
                         │
T=0:20 ┌─────────────────▼───────────────────────────┐
       │ SQLite (durable backup)                     │
       │ - Batch INSERT with WAL journaling         │
       │ - Max 5s loss window if crash             │
       └─────────────────┬───────────────────────────┘
                         │
T=0.5  ┌─────────────────▼───────────────────────────┐
       │ Central Client Sync Cycle (500ms)           │
       │ - Query unsynced from memory (<1ms)         │
       │ - Filter by exponential backoff (<1ms)      │
       │ - Serialize to JSON (1-5ms)                │
       │ - Send to server (batch)                   │
       │ Latency: 50-500ms                          │
       └─────────────────┬───────────────────────────┘
                         │ WebSocket
T=0.7  ┌─────────────────▼───────────────────────────┐
       │ Server (Central System)                     │
       │ - Receive logs                             │
       │ - Validate                                 │
       │ - Process (HR/Payroll)                     │
       └─────────────────┬───────────────────────────┘
                         │ ACK message
T=0.9  ┌─────────────────▼───────────────────────────┐
       │ InMemoryStore (mark synced)                 │
       │ - Update is_synced=1                       │
       │ - Remove from unsynced_index              │
       │ Latency: <1ms                              │
       └─────────────────┬───────────────────────────┘
                         │
T=5.0  ┌─────────────────▼───────────────────────────┐
       │ Checkpoint Worker (periodic)                │
       │ - Mark log as persisted=1                  │
       │ - Update disk (for crash recovery)         │
       │ Latency: 50-100ms                          │
       └─────────────────────────────────────────────┘

TOTAL END-TO-END: ~1-2 seconds
(Device → Sync → Server → Confirmed)
```

### Crash Recovery Workflow

```
T=crash ┌──────────────────────────────────┐
        │ Unexpected shutdown               │
        │ - In-flight logs: is_synced=2    │
        │ - Memory lost                    │
        │ - Disk: Last checkpoint (~5s ago)│
        └────────────┬─────────────────────┘
                     │
T=crash+0.1 ┌────────▼─────────────────────┐
        │ Agent restart                    │
        │ Check .agent_running marker      │
        └────────┬─────────────────────────┘
                     │
T=crash+0.2 ┌────────▼─────────────────────┐
        │ Detect crash (marker exists)     │
        │ - Read unsynced from disk        │
        │ - Load into memory               │
        │ - Find is_synced=2 (in-flight)  │
        └────────┬─────────────────────────┘
                     │
T=crash+0.3 ┌────────▼─────────────────────┐
        │ Reset in-flight logs             │
        │ - Update: is_synced=2 → 0        │
        │ - Mark for retry                 │
        │ - Update disk                    │
        └────────┬─────────────────────────┘
                     │
T=crash+0.5 ┌────────▼─────────────────────┐
        │ Create new marker                │
        │ Resume normal operation          │
        │ RECOVERY COMPLETE                │
        │ Max data loss: 5 seconds         │
        └──────────────────────────────────┘
```

---

## Performance Characteristics

### Latency Breakdown

| Component | Operation | Latency | Bottleneck |
|-----------|-----------|---------|-----------|
| Device Polling | TCP connect | 10-100ms | Network |
| Polling | Parse response | 1-5ms | CPU |
| Channel | Enqueue | <1ms | DashMap |
| DB Writer | Batch write | 10-50ms | Disk I/O |
| Memory | Insert | <1ms | Lock-free |
| Memory | Query unsynced | <1ms | BTreeSet |
| Memory | Mark in-flight | <1ms | DashMap |
| Checkpoint | Persist | 50-500ms | Disk I/O |
| Sync | Read + serialize | 5-10ms | JSON encode |
| Sync | Network send | 100-500ms | Network |
| Sync | Wait ACK | 100-500ms | Server |
| Sync | Update synced | <1ms | DashMap |
| **TOTAL** | **Full cycle** | **~500ms-1.5s** | **Network** |

### Memory Usage

| Scenario | In-Memory | On-Disk | Total |
|----------|-----------|---------|-------|
| 100 logs | 50KB | 20KB | 70KB |
| 1K logs | 500KB | 200KB | 700KB |
| 10K logs | 5MB | 2MB | 7MB |
| 50K logs | 25MB | 10MB | 35MB |
| 100K logs | 50MB | 20MB | 70MB |

**Memory per Log:**
- MemoryLog struct: ~256 bytes
- DashMap overhead: ~64 bytes
- Total: ~320 bytes per log
- 100K logs: 32MB + overhead = ~50MB

### Throughput

| Operation | Rate | Time Budget |
|-----------|------|-------------|
| Device polling | 5s interval | Limited by devices |
| Log ingestion | 500 logs/sec | Via mpsc channel |
| Sync cycle | 2 cycles/sec | 500ms interval |
| Sync throughput | 200 logs/sec | 100 logs per cycle |
| Checkpoint | Every 5s | Async, non-blocking |

---

## Security Architecture

### Data Protection Layers

```
┌─ InMemoryStore (Runtime)
│  - Data in RAM
│  - Lost on crash (acceptable 5s window)
│  - Protected by system memory isolation
│
├─ SQLite Database (Persistence)
│  - File-based storage
│  - WAL journaling for durability
│  - Default file permissions (0644)
│  - No encryption (PII not stored)
│
└─ WebSocket Connection (Network)
   - TLS enforced by server
   - Certificate validation
   - Token-based authentication
```

### Input Validation Pipeline

```
Device → Parse → Validate → Memory → Serialize → Network
         ↓          ↓
      Format    Length
      checks    checks
                Type
                checks
```

### Access Control

| Component | Access Scope | Isolation |
|-----------|--------------|-----------|
| Device polling | Local network | Firewall rules |
| Central client | Server only | API key in env |
| Worker service | Config push only | ConfigPush auth |
| Database | Local file | Unix permissions |
| Logs | Internal only | No external API |

---

## Scalability & Limits

### Bounded Capacity

```
mpsc channel:     10K records max
InMemoryStore:   100K logs max (~50MB)
SQLite:          Unlimited (disk bound)
Device polling:  N devices (each 5s cycle)
```

### Backpressure Mechanism

```
Device Polling (5s)
    │
    ├─> mpsc channel full (10K)?
    │   YES: Block polling → Slow to pull from device
    │
    ├─> InMemoryStore full (100K)?
    │   YES: Block write → Blocking in save_logs()
    │
    └─> Sync loop slow?
        YES: Checkpoint backlog → Sleep reduced to 50ms
```

### Typical Deployment

| Device Count | Polling Rate | Expected Logs/Day | Memory |
|--------------|--------------|------------------|--------|
| 1 device | 5s | 5K | 2MB |
| 5 devices | 5s | 25K | 10MB |
| 10 devices | 5s | 50K | 25MB |
| 20 devices | 5s | 100K | 50MB |

---

## Integration Points

### External Systems

| System | Protocol | Authentication | Purpose |
|--------|----------|-----------------|---------|
| Time-Clock Device | TCP/IP | None | Attendance records |
| Central Server | WebSocket/TLS | API Token | Log sync |
| Config Service | HTTP REST | API Token | Configuration push |

### Internal Modules

| Module | Purpose | Dependency |
|--------|---------|-----------|
| main.rs | Orchestrator | All modules |
| hybrid_database.rs | Facade | InMemoryStore + Database |
| central_client.rs | Sync | InMemoryStore |
| worker_client.rs | Config | Database |
| crash_recovery.rs | Recovery | Database |

---

## Deployment Architecture

### Single Agent Deployment

```
Device → Agent (Docker) → Server
         └─ SQLite DB
            Log rotate (365 days)
            Max 5s data loss
```

### Multiple Agents (Fleet)

```
Device Group 1 → Agent 1 → Server
Device Group 2 → Agent 2 → Server  (Independent)
Device Group N → Agent N → Server
```

**No Coordination:** Each agent is independent

---

**Document Version:** 1.0
**Status:** PRODUCTION READY
**Last Updated:** 2026-01-29
