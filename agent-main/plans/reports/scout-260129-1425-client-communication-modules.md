# Client Communication Modules Analysis Report

**Date:** 2026-01-29  
**Scope:** Three core communication clients  
**Total LOC:** 860 lines  

---

## Overview

The agent codebase implements a three-tier client communication architecture:
1. **Central Client** - WebSocket to central server (sync loop, heartbeat)
2. **Worker Client** - TCP JSON-RPC to worker sidecar (blocking I/O)
3. **ZK Client** - TCP JSON-RPC to ZKTeco device (async, timeout-aware)

All clients are message-driven, with error handling and reconnection logic integrated at the central client layer.

---

## src/central_client.rs (511 LOC)

### Purpose
Maintains persistent WebSocket connection to central server for syncing attendance logs, receiving config updates, and bidirectional communication. Core orchestrator for attendance data flow from agent to cloud.

### Protocol
- **Transport:** WebSocket (TLS-capable, timeout 30s)
- **Headers:** `X-Agent-Id`, `X-Token` for auth
- **Format:** JSON messages over text frames
- **Message Types:**
  - `Handshake` - Initial auth (agent_id, token, timestamp)
  - `Attendance` - Log sync (msg_id, device_id, user_id, timestamp, status, punch)
  - `Heartbeat` - Keep-alive every 30s
  - `ConfigPush` - Inbound device config updates
  - `Ack` - Inbound acknowledgment of synced logs (status: success/error)
  - `Error` - Server error notifications

### Key Functions/Methods
- `new()` - Constructor, returns (client, rx, reload_rx)
- `start()` - Launches three async tasks:
  1. **Sync Loop** (lines 56-156) - Polling sync of unsynced logs
  2. **Heartbeat Task** (lines 158-170) - 30s intervals
  3. **WS Connection Manager** (lines 172-245) - Connection retry/health

### Sync Loop Architecture
Adaptive sleep strategy based on queue backlog:
- **Idle** (0 logs): 500ms check interval
- **Moderate** (100-1000 unsynced): 100-200ms intervals
- **Backlogged** (>1000): 50ms aggressive drain

**Per-cycle operations:**
- Fetch up to 500 unsynced logs from memory
- Classify as new (updated_at NULL) vs retry (updated_at NOT NULL)
- Mark as in-flight in memory (reset every 60s for stale >5min)
- Send via MPSC channel (100 capacity)
- Latency target: <5ms

**Expiry Handling:**
- Reads `max_retry_count` and `retry_ttl_minutes` from agent config
- Moves expired logs to DLQ every cycle
- Default: 10 retries, 60-minute TTL

### Connection Management

**RetryManager State Machine:**
- **Exponential backoff:** 1s, 2s, 4s, 8s, 16s, 32s, 60s (capped)
- **Jitter:** ±25% to prevent thundering herd
- **Stability threshold:** 60s before resetting retry count
- **Max retries:** 10 before capping at 60s delay

**States:**
- `record_connection_established()` - Starts stability timer
- `record_connection_failed()` - Increments retry_count
- `record_dropped_prematurely()` - Same as failed
- `record_stable()` - Checks 60s threshold, resets count if passed

**Health Checks:**
- Handles ping/pong frames automatically
- Detects server close frames (logs warning)
- Catches WebSocket errors, I/O errors
- Re-initiates connection on any failure

### Integration Points
- **HybridDatabase:** Read/write unsynced logs, mark in-flight/synced
- **Memory Store:** Count unsynced, reset stale in-flight, mark synced
- **Disk DB:** UPS agent config, device CRUD, mark synced
- **Config Reload:** Sends via reload_tx when ConfigPush received
- **Message Channel:** Recv outbound messages from sync/heartbeat tasks

### Incoming Message Handlers

**ConfigPush:**
- Updates device list (upsert all)
- Deactivates stale devices not in push
- Updates `max_concurrent_workers` config
- Triggers reload signal

**Ack (success):**
- Marks log synced in memory AND disk
- Removes from in-flight state

**Ack (error):**
- Resets log back to unsynced state
- Retryable

**Error:**
- Logged at error level
- No state changes

---

## src/worker_client.rs (217 LOC)

### Purpose
Synchronous JSON-RPC client for communicating with worker sidecar process via TCP. Handles device control commands, configuration, and status queries over blocking socket I/O.

### Protocol
- **Transport:** TCP (blocking I/O)
- **Format:** JSON-RPC 2.0 (line-delimited)
- **Request:** `{"jsonrpc":"2.0", "method":"...", "params":{}, "id":N}\n`
- **Response:** `{"jsonrpc":"2.0", "result":{} | null, "error":{} | null, "id":N}\n`

### Timeout Configuration
- **Read timeout:** 300s (5 minutes, for slow device connections)
- **Write timeout:** 30s
- **Rationale:** Device commands may block waiting for I/O

### Key Methods
- `new(addr)` - Create client from TCP address string
- `from_stream(stream)` - Create from existing TcpStream (auto-converts non-blocking→blocking)
- `call(method, params)` - Synchronous RPC call (increments message ID)

### Message Flow
1. Build JsonRpcRequest with auto-incrementing ID
2. Serialize to JSON + newline, write to stream
3. Flush socket
4. Read next line from BufReader
5. Deserialize JsonRpcResponse
6. Return error if response.error is set, else response.result

### Error Handling
- Detects closed sidecar (EOF) with "Sidecar closed connection"
- RPC errors propagated as io::Error::other()
- Invalid JSON treated as InvalidData error
- Write/read timeouts return timeout errors

### State Management
- `stream` - Owned TcpStream (blocking)
- `next_id` - Incremented counter for request/response correlation

---

## src/zk_client.rs (132 LOC)

### Purpose
Async JSON-RPC client for communicating with ZKTeco device wrapper. Supports command-response patterns and asynchronous message reading.

### Device Protocol
- **Transport:** TCP (async, tokio-based)
- **Format:** JSON-RPC 2.0 (line-delimited)
- **Default timeout:** 30s (user-configurable)

### Key Methods
- `new(stream)` - Constructor from TcpStream
- `set_timeout(seconds)` - Configure per-client timeout
- `call(method, params)` - Async RPC call with timeout wrapper
- `read_next_message()` - Async read-only (for push messages)

### Timeout Strategy
- Wraps read operation with `tokio::time::timeout()`
- Returns `TimedOut` error if exceeded
- Configurable per instance via `set_timeout()`

### Message Flow (call)
1. Build JsonRpcRequest with auto-incrementing ID
2. Serialize to JSON + newline, write to stream
3. Create BufReader over mutable stream reference
4. Apply timeout wrapper around `read_line()`
5. Deserialize JsonRpcResponse
6. Return error if response.error set, else response.result

### Connection Handling
- Detects stream closure (0 bytes read) → "Sidecar connection closed"
- Timeout errors return IoErrorKind::TimedOut
- No automatic reconnection (caller responsibility)

### Async Characteristics
- Uses `AsyncWriteExt`, `AsyncBufReadExt` traits
- Non-blocking I/O with tokio runtime
- Can be used in concurrent tasks

---

## Comparison Matrix

| Aspect | Central | Worker | ZK |
|--------|---------|--------|-----|
| **Transport** | WebSocket (TLS) | TCP (blocking) | TCP (async) |
| **I/O Model** | Async (tokio) | Sync (std) | Async (tokio) |
| **Timeout** | 30s connect | 300s read, 30s write | 30s (configurable) |
| **Reconnection** | Auto (retry manager) | Manual | Manual |
| **Message Format** | JSON text frames | JSON-RPC lines | JSON-RPC lines |
| **Bidirectional** | Yes (WebSocket) | Half-duplex RPC | Half-duplex RPC |
| **Authentication** | Headers (token) | None (IPC) | None (device) |
| **State** | Extensive (RetryManager, in-flight, sync loop) | Minimal (ID counter) | Minimal (ID counter) |

---

## External Dependencies

- `tokio` - Async runtime (central, zk)
- `tokio-tungstenite` - WebSocket client (central)
- `serde_json` - JSON (all)
- `log` - Logging (central)
- `rand` - Jitter generation (central)
- `futures-util` - Stream/Sink traits (central)
- `sqlx` - Database access (central)
- `chrono` - Timestamps (central)

---

## Performance Optimizations

### Central Client
1. **Adaptive sleep:** Scales polling interval based on queue depth (50-500ms)
2. **Batching:** Sends up to 500 logs per sync cycle
3. **In-flight tracking:** Prevents duplicate sends and enables retries
4. **Stale cleanup:** Periodic reset of logs stuck >5min in in-flight
5. **DLQ routing:** Moves expired logs away from retry loop
6. **Jittered backoff:** Prevents synchronized reconnect storms

### Worker Client
1. **Blocking I/O:** Simpler error handling vs async state machines
2. **Long read timeout:** Accommodates slow device I/O (5 min)
3. **BufReader:** Efficient line-oriented reading

### ZK Client
1. **Configurable timeout:** Allows tuning per device
2. **Async I/O:** Multiplexable with other tasks
3. **Separate read method:** Supports both RPC and message push patterns

---

## Security Considerations

- **Central Client:** Token passed in headers, TLS-capable URLs
- **Worker Client:** No auth (assumed local IPC, trusted sidecar)
- **ZK Client:** No auth (device communication, typically local network)
- **Message validation:** All clients validate JSON structure
- **Error logging:** No sensitive data logged (timestamps, IDs OK)

---

## Unresolved Questions

1. How is the `reload_tx` channel consumed? (likely in main loop to reinitialize worker clients)
2. What triggers worker sidecar shutdown? (error handling chain?)
3. Are ZK device commands queued or dropped if timeout exceeded?
4. How many concurrent message sends to central allowed (MPSC cap=100)?
5. Is connection stability timer reset on each message, or only after connect?
