# Scout Report: agent-main Memory Analysis

## Overview
Analyzed agent-main (27 src files, ~4686 LOC) for memory issues using patterns from [rust-agentic-skills](https://github.com/udapy/rust-agentic-skills).

## Memory Issues Found

### Issue 1: CRITICAL — Unbounded `seen_logs` HashSet per Worker
**File:** `agent-main/src/main.rs:407`
```rust
let mut seen_logs: HashSet<String> = HashSet::new();
```
- Per-device HashSet grows indefinitely during device connection lifetime
- Key format: `"user_id:timestamp"` — String allocations accumulate
- 10K+ unique logs per device = significant memory pressure
- **No eviction policy** — only clears on device reconnect/restart
- **Fix:** Use bounded LRU cache or periodic eviction (e.g., keep last 10K entries)

### Issue 2: HIGH — Clone on Hot Path in `get_unsynced()`
**File:** `agent-main/src/memory_store/store.rs:286`
```rust
result.push(log.clone()); // clones MemoryLog with String fields
```
- Called every sync cycle (50-200ms intervals)
- Clones full `MemoryLog` including owned `device_id: String` + `user_id: String`
- 500 logs/cycle × 2 String clones = 1000 allocations per cycle
- **Fix:** Use `Arc<MemoryLog>` in DashMap, or return references/IDs instead of clones

### Issue 3: HIGH — Clone on Hot Path in `get_unpersisted()`
**File:** `agent-main/src/memory_store/store.rs:460-463`
```rust
self.logs.iter()
    .filter(|entry| !entry.persisted)
    .map(|entry| entry.clone())  // clones ALL unpersisted logs
    .collect()
```
- Called every 5s by checkpoint worker
- Full scan of ALL logs in DashMap + clone matching entries
- Same issue: String fields cloned unnecessarily
- **Fix:** Track unpersisted IDs in a separate index (like unsynced_index), avoid full scan

### Issue 4: MEDIUM — `get_all()` Full Clone
**File:** `agent-main/src/memory_store/store.rs:476-478`
```rust
self.logs.iter().map(|entry| entry.clone()).collect()
```
- Clones entire store content
- Used for full checkpoint — allocates O(n) memory for 100K logs
- **Fix:** Not called frequently, but could use streaming/batched approach

### Issue 5: MEDIUM — Signature Set Never Shrinks
**File:** `agent-main/src/memory_store/store.rs:17`
```rust
signatures: DashSet<u64>,  // Hash signatures for deduplication
```
- Signatures persist even after `mark_synced()` removes logs (line 388)
- Design choice for dedup protection, but grows monotonically
- 100K signatures × 8 bytes = ~800KB (acceptable but worth monitoring)
- Only shrinks via `unload_device_memory()` — not called for always-active devices
- **Fix:** Add age-based cleanup for signatures older than N days

### Issue 6: MEDIUM — Unnecessary String Allocation in Polling Loop
**File:** `agent-main/src/main.rs:433-435`
```rust
if let Ok(attendance) = serde_json::from_value::<models::AttendanceData>(log.clone()) {
    let key = format!("{}:{}", attendance.user_id, attendance.timestamp);
```
- `log.clone()` — clones the serde_json::Value before deserializing
- `format!()` — allocates new String for every log checked (even duplicates)
- **Fix:** Use `serde_json::from_value(log)` (take ownership), compute key hash instead of String

### Issue 7: LOW — Redundant `.clone()` on Insert
**File:** `agent-main/src/memory_store/store.rs:185-186`
```rust
device_id: device_id.clone(),
user_id: data.user_id.clone(),
```
- `device_id` is passed as `String` (owned) but still cloned because used after for composite hash (line 198)
- **Fix:** Compute composite hash before constructing MemoryLog, then move strings

### Issue 8: LOW — `serde_json::Value` Cloned Before Deser
**File:** `agent-main/src/main.rs:433`
```rust
serde_json::from_value::<models::AttendanceData>(log.clone())
```
- `from_value` takes ownership; `.clone()` is unnecessary if we can consume the value
- Current loop uses `for log in logs` where logs is `&[Value]` — clone needed for borrow
- **Fix:** Change to owned iteration: `for log in logs.into_iter().cloned()` or restructure

## Rust Agentic Skills Patterns Applied

### From Rust Core (idiomatic_rust.md)
- **Iterator combinators** over manual loops: `get_unpersisted()` already uses iterators ✓
- **Expression-oriented**: Code follows this pattern ✓
- **Type-driven design**: `StoreError` enum is well-designed ✓
- **Visibility**: Fields properly scoped with `pub(crate)` ✓

### From Lint Hunter (dictionary_of_pain.md)
- **E0502 pattern** (borrow + clone): `get_unsynced()` clones inside DashMap borrow — this is the root cause of Issue #2. The `log.clone()` exists because DashMap's `.get()` returns a `Ref` guard that borrows the map.
- **Fix strategy**: Wrap values in `Arc` so clone is cheap (pointer copy, not deep copy)

### From Security Specialist
- No `unsafe` blocks found ✓
- No hardcoded secrets ✓
- FxHasher used for performance (not cryptographic — appropriate for dedup) ✓

## Recommended Priority

| # | Issue | Impact | Effort | Priority |
|---|-------|--------|--------|----------|
| 1 | Unbounded seen_logs | Memory leak | Low | **P0** |
| 2 | get_unsynced() clone | Allocation churn | Medium | **P1** |
| 3 | get_unpersisted() full scan | CPU + alloc | Medium | **P1** |
| 6 | Polling loop allocations | Allocation churn | Low | **P2** |
| 7 | Redundant clone on insert | Minor alloc | Low | **P2** |
| 5 | Signature set growth | Memory growth | Low | **P3** |
| 4 | get_all() full clone | Rare path | Low | **P3** |
| 8 | Value clone before deser | Minor alloc | Low | **P3** |

## Key Files
- `agent-main/src/main.rs` (519 lines) — worker loops, seen_logs
- `agent-main/src/memory_store/store.rs` (528+ lines) — core store, hot paths
- `agent-main/src/memory_store/types.rs` (70 lines) — MemoryLog struct
- `agent-main/src/central_client/connection.rs` (341 lines) — WS handling
- `agent-main/src/hybrid_database.rs` — orchestration layer

## Unresolved Questions
1. What's the typical `seen_logs` size in production? (determines urgency of P0)
2. Should `Arc<MemoryLog>` change be propagated through HybridDatabase API?
3. Is `get_all()` actually used in production path or only tests?
