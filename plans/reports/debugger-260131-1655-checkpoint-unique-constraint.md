# Checkpoint UNIQUE Constraint Failure Analysis

**Date:** 2026-01-31 16:55
**Agent:** debugger
**Error:** `UNIQUE constraint failed: zkt_raw_attendance.id` (SQLite error 1555)

---

## Executive Summary

**Root Cause:** Checkpoint INSERT statements explicitly specify `id` values that already exist in database, causing PRIMARY KEY UNIQUE constraint violations before `ON CONFLICT(device_id, user_id, timestamp)` clause executes.

**Impact:** Checkpoint failures prevent memory-to-disk persistence, risking data loss on crash despite duplicate filtering working correctly during fetch.

**Priority:** CRITICAL - Data persistence mechanism broken

---

## Technical Analysis

### Error Timeline

1. Device fetch logs successfully (18996, 10979 duplicates filtered)
2. Logs saved to memory with auto-generated IDs
3. Checkpoint worker attempts disk persistence every 5s
4. INSERT fails with `UNIQUE constraint failed: zkt_raw_attendance.id`
5. Repeated failures logged despite duplicate filtering working

### Root Cause Chain

#### 1. ID Collision During Checkpoint

**File:** `/home/elt1541/leeattend/agent-main/src/database.rs:566-573`

```rust
INSERT INTO zkt_raw_attendance
    (id, device_id, user_id, timestamp, status, punch, is_synced, created_at, retry_count)
VALUES {}
ON CONFLICT(device_id, user_id, timestamp) DO NOTHING
```

**Problem:** Explicitly inserting `id` causes PRIMARY KEY violation:
- Memory assigns IDs using `AtomicI64` counter seeded from `disk.get_max_id()` + gap
- If disk already has logs with overlapping IDs, INSERT fails immediately
- `ON CONFLICT` clause only handles `(device_id, user_id, timestamp)` composite index
- PRIMARY KEY violations throw error **before** `ON CONFLICT` executes

#### 2. Schema Constraints

**File:** `/home/elt1541/leeattend/agent-main/migrations/20260108000000_init_schema.sql:2-10`

```sql
CREATE TABLE IF NOT EXISTS zkt_raw_attendance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,  -- UNIQUE by definition
    device_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    ...
);
```

**File:** `/home/elt1541/leeattend/agent-main/migrations/20260126000001_add_unique_index.sql:1`

```sql
CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_log
ON zkt_raw_attendance(device_id, user_id, timestamp);
```

**Dual Constraints:**
- PRIMARY KEY `id` - auto-increment, must be unique
- UNIQUE INDEX on `(device_id, user_id, timestamp)` - business logic dedup

#### 3. ID Seeding Gap Not Sufficient

**File:** `/home/elt1541/leeattend/agent-main/src/memory_store.rs:131-136`

```rust
pub fn seed_id(&self, next_id: i64) {
    // Add gap to prevent ID collision with any in-flight logs
    let safe_start = next_id + 1000; // 1000 ID gap
    self.next_id.store(safe_start, Ordering::SeqCst);
}
```

**File:** `/home/elt1541/leeattend/agent-main/src/hybrid_database.rs:21-23`

```rust
let max_id = disk.get_max_id().await?;
memory.seed_id(max_id + 1);
```

**Gap Issue:**
- Seed adds 1000 to max_id: `safe_start = (max_id + 1) + 1000`
- But checkpoint inserts unpersisted logs with IDs in range `[max_id+1001, current_counter]`
- If disk has logs in that range (from previous checkpoint), collision occurs
- Scenario: Crash before `mark_persisted` updates memory, restart loads stale max_id

### Data Flow Analysis

```
┌─────────────────────────────────────────────────────────────┐
│ Device Fetch → Memory Insert (ID: auto-increment)          │
│   ↓                                                         │
│ Memory Store (unpersisted: true, ID: 1001-2000)            │
│   ↓                                                         │
│ Checkpoint Worker (every 5s)                               │
│   ↓                                                         │
│ get_unpersisted() → logs with IDs 1001-2000                │
│   ↓                                                         │
│ disk.checkpoint_batch() → INSERT with explicit IDs         │
│   ↓                                                         │
│ ❌ UNIQUE constraint failed: id (if 1001-2000 exist)       │
│                                                             │
│ Expected: ON CONFLICT DO NOTHING skips duplicates          │
│ Actual: PRIMARY KEY violation aborts before ON CONFLICT    │
└─────────────────────────────────────────────────────────────┘
```

### Why Duplicate Filtering Works During Fetch

**File:** `/home/elt1541/leeattend/agent-main/src/main.rs:386-430`

Worker uses in-memory HashSet for dedup:
```rust
let mut seen_logs: HashSet<String> = HashSet::new();
// ... later
let key = format!("{}:{}", attendance.user_id, attendance.timestamp);
if seen_logs.insert(key) {
    // New log - send to queue
} else {
    // Duplicate - skip
}
```

This prevents duplicates **entering memory**, but doesn't solve checkpoint ID collision with existing disk rows.

---

## Evidence

### Logs Show Pattern

```
[DeviceA] Fetched 30000 logs (11004 new, 18996 duplicates filtered)
[DeviceB] Fetched 20000 logs (9021 new, 10979 duplicates filtered)
💾 Checkpoint: 0 logs persisted to disk
❌ Checkpoint failed: UNIQUE constraint failed: zkt_raw_attendance.id
```

**Analysis:**
- Duplicate filtering prevents memory bloat
- Checkpoint fails despite filter working
- 0 logs persisted = all inserts rejected

### Database State

**Query:** `SELECT COUNT(*), MAX(id) FROM zkt_raw_attendance`

Likely shows:
- Large number of existing rows
- max_id in range that memory is trying to reuse

---

## Recommended Fixes

### Option 1: Remove Explicit ID from Checkpoint INSERT (Preferred)

**Change:** `/home/elt1541/leeattend/agent-main/src/database.rs:566-573`

```rust
// BEFORE
INSERT INTO zkt_raw_attendance
    (id, device_id, user_id, timestamp, status, punch, is_synced, created_at, retry_count)
VALUES {}
ON CONFLICT(device_id, user_id, timestamp) DO NOTHING

// AFTER
INSERT INTO zkt_raw_attendance
    (device_id, user_id, timestamp, status, punch, is_synced, created_at, retry_count)
VALUES {}
ON CONFLICT(device_id, user_id, timestamp) DO NOTHING
```

**Binding Changes:** Line 576-585 - Remove `log.id` binding

**Pros:**
- Let SQLite auto-generate IDs via AUTOINCREMENT
- ON CONFLICT clause handles business duplicates correctly
- Memory IDs become transient (only for in-flight tracking)

**Cons:**
- Memory and disk IDs no longer match
- Need to refactor sync tracking to use composite key instead of ID

### Option 2: Use INSERT OR REPLACE with ID Preservation

```rust
INSERT OR REPLACE INTO zkt_raw_attendance
    (id, device_id, user_id, timestamp, status, punch, is_synced, created_at, retry_count)
VALUES {}
```

**Pros:**
- Preserves memory-assigned IDs
- Handles both PRIMARY KEY and business duplicates

**Cons:**
- OR REPLACE deletes then inserts (may lose sync state updates)
- Violates least-surprise principle

### Option 3: Increase ID Gap to 1 Million

**Change:** `/home/elt1541/leeattend/agent-main/src/memory_store.rs:134`

```rust
let safe_start = next_id + 1_000_000; // Increase gap
```

**Pros:**
- Quick fix, minimal code change

**Cons:**
- Doesn't solve root cause (ID will still collide eventually)
- Wastes ID space (may hit INTEGER limit faster)

---

## Implementation Plan

### Phase 1: Immediate Mitigation

1. Add ON CONFLICT handling for PRIMARY KEY:
   ```rust
   INSERT INTO zkt_raw_attendance (...) VALUES (...)
   ON CONFLICT(id) DO NOTHING
   ```

2. Add checkpoint error recovery:
   ```rust
   match disk.checkpoint_batch(&unpersisted).await {
       Ok((inserted, skipped)) => info!("Checkpoint: {} inserted, {} skipped", inserted, skipped),
       Err(e) if e.to_string().contains("UNIQUE constraint") => {
           warn!("Checkpoint ID collision, retrying without explicit IDs...");
           // Fallback to auto-ID insert
       }
       Err(e) => return Err(e),
   }
   ```

### Phase 2: Root Fix

Refactor checkpoint to not use explicit IDs:

**Files to Modify:**
1. `/home/elt1541/leeattend/agent-main/src/database.rs:548-595`
   - Remove `id` from INSERT columns
   - Remove `log.id` from bindings
   - Return `Vec<i64>` of auto-generated IDs

2. `/home/elt1541/leeattend/agent-main/src/hybrid_database.rs:84-98`
   - Handle returned IDs from checkpoint
   - Map memory IDs to disk IDs for sync tracking

3. `/home/elt1541/leeattend/agent-main/src/memory_store.rs:42-55`
   - Add `disk_id: Option<i64>` field to MemoryLog
   - Use disk_id for mark_synced operations

### Phase 3: Testing

1. Unit test: Insert logs with explicit IDs that already exist
2. Integration test: Crash recovery with checkpoint ID gap
3. Load test: 100k logs checkpoint with existing duplicates

---

## Security Considerations

- No auth/data leakage issues
- Fix prevents data loss risk on crash
- Ensure checkpoint errors don't expose sensitive log data in error messages

---

## Unresolved Questions

1. Are there existing logs in production with ID collisions?
   - Query: `SELECT id, COUNT(*) FROM zkt_raw_attendance GROUP BY id HAVING COUNT(*) > 1`

2. What happens to in-flight logs during crash before checkpoint?
   - Risk: Memory IDs 1001-2000 lost, restart assigns 1001+ again → collision

3. Should we switch to UUID for log IDs to avoid collisions entirely?
   - Trade-off: UUID = no collisions, but larger index size and slower inserts

4. Is the 1000 ID gap comment accurate about "in-flight logs"?
   - Code comment says gap prevents collision with "in-flight logs"
   - But in-flight logs are already on disk (marked is_synced=2)
   - Gap should be for **unpersisted** logs, not in-flight

---

## Next Steps

1. Verify production database has ID collision using query above
2. Implement Phase 1 mitigation (ON CONFLICT handling)
3. Plan Phase 2 refactor for next maintenance window
4. Add monitoring for checkpoint failure rate
