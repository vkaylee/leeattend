# Implementation Plan: Exponential Backoff for Sync Retries

**Status:** Ready for Implementation
**Effort:** 2-3 hours
**Risk:** Low (single function change, no schema changes)

---

## Overview

Implement stateless time-based exponential backoff in `memory_store.rs::get_unsynced()` to prevent rapid-fire retries of failing logs while allowing new logs to sync immediately.

**Formula:** `delay = min(2^retry_count, 60) seconds`

| retry_count | delay |
|------------|-------|
| 0 | 0s (new logs - immediate) |
| 1 | 2s |
| 2 | 4s |
| 3 | 8s |
| 4 | 16s |
| 5 | 32s |
| 6+ | 60s (capped) |

---

## Implementation Steps

### Step 1: Modify `get_unsynced()` in `memory_store.rs`

**File:** `/home/elt1541/leeattend/agent-main/src/memory_store.rs`
**Lines:** 198-210

**Current Code:**
```rust
pub fn get_unsynced(&self, limit: usize) -> Vec<MemoryLog> {
    let unsynced = self.unsynced_index.read();
    let mut result = Vec::with_capacity(limit);

    for id in unsynced.iter().take(limit) {
        if let Some(log) = self.logs.get(id) {
            result.push(log.clone());
        }
    }

    result
}
```

**New Code:**
```rust
pub fn get_unsynced(&self, limit: usize) -> Vec<MemoryLog> {
    let unsynced = self.unsynced_index.read();
    let mut result = Vec::with_capacity(limit);
    let now = chrono::Utc::now().naive_utc();

    for id in unsynced.iter() {
        if result.len() >= limit {
            break;
        }
        if let Some(log) = self.logs.get(id) {
            // Skip logs still in backoff period
            if let Some(updated_at) = log.updated_at {
                let delay_secs = (1i64 << log.retry_count.min(6)).min(60);
                let eligible_at = updated_at + chrono::Duration::seconds(delay_secs);
                if now < eligible_at {
                    continue; // Still in backoff
                }
            }
            result.push(log.clone());
        }
    }

    result
}
```

**Changes:**
- Add `now` timestamp at start
- Continue iterating past `limit` to find eligible logs
- Check backoff: `now < updated_at + 2^retry_count` (capped at 60s)
- New logs (updated_at=None) pass through immediately

**Acceptance Criteria:**
- [ ] New logs (retry_count=0, updated_at=None) sync immediately
- [ ] Logs with retry_count=3 wait 8 seconds before retry
- [ ] Logs with retry_count >= 6 wait max 60 seconds
- [ ] FIFO order maintained for eligible logs

---

### Step 2: Add Metrics for Backoff Monitoring

**File:** `/home/elt1541/leeattend/agent-main/src/memory_store.rs`

Add helper method to count logs in backoff:

```rust
/// Count logs currently in backoff (not eligible for sync)
pub fn count_in_backoff(&self) -> usize {
    let unsynced = self.unsynced_index.read();
    let now = chrono::Utc::now().naive_utc();
    let mut count = 0;

    for id in unsynced.iter() {
        if let Some(log) = self.logs.get(id) {
            if let Some(updated_at) = log.updated_at {
                let delay_secs = (1i64 << log.retry_count.min(6)).min(60);
                let eligible_at = updated_at + chrono::Duration::seconds(delay_secs);
                if now < eligible_at {
                    count += 1;
                }
            }
        }
    }

    count
}
```

**File:** `/home/elt1541/leeattend/agent-main/src/central_client.rs`
**Line ~143** (after adaptive sleep logic)

Add logging:
```rust
let backoff_count = sync_hybrid.memory_store().count_in_backoff();
if backoff_count > 0 {
    debug!("⏳ {} logs in backoff, {} eligible for sync", backoff_count, unsynced_count - backoff_count);
}
```

**Acceptance Criteria:**
- [ ] `count_in_backoff()` returns accurate count
- [ ] Debug log shows backoff status when logs are waiting

---

### Step 3: Add Unit Tests

**File:** `/home/elt1541/leeattend/agent-main/src/memory_store.rs`
**Add at end of `mod tests`**

```rust
#[test]
fn test_backoff_new_logs_immediate() {
    let store = InMemoryStore::new(1000);

    // Insert new log (retry_count=0, updated_at=None)
    let data = AttendanceData {
        user_id: "U1".to_string(),
        timestamp: "2026-01-29 10:00:00".to_string(),
        status: Some(1),
        punch: Some(1),
    };
    store.insert("D1".to_string(), data).unwrap();

    // New log should be returned immediately
    let logs = store.get_unsynced(10);
    assert_eq!(logs.len(), 1);
}

#[test]
fn test_backoff_skips_recent_retry() {
    let store = InMemoryStore::new(1000);

    // Insert and simulate first retry
    let data = AttendanceData {
        user_id: "U1".to_string(),
        timestamp: "2026-01-29 10:00:00".to_string(),
        status: Some(1),
        punch: Some(1),
    };
    let id = store.insert("D1".to_string(), data).unwrap();

    // Mark in-flight (sets retry_count=1, updated_at=now)
    store.mark_in_flight(&[id]);
    store.reset_to_unsynced(id);

    // Should NOT be returned (within 2s backoff)
    let logs = store.get_unsynced(10);
    assert_eq!(logs.len(), 0);
    assert_eq!(store.count_in_backoff(), 1);
}

#[test]
fn test_backoff_allows_after_delay() {
    let store = InMemoryStore::new(1000);

    // Insert log
    let data = AttendanceData {
        user_id: "U1".to_string(),
        timestamp: "2026-01-29 10:00:00".to_string(),
        status: Some(1),
        punch: Some(1),
    };
    let id = store.insert("D1".to_string(), data).unwrap();

    // Simulate retry with old updated_at (3 seconds ago)
    if let Some(mut log) = store.logs.get_mut(&id) {
        log.retry_count = 1;
        log.updated_at = Some(chrono::Utc::now().naive_utc() - chrono::Duration::seconds(3));
    }

    // Should be returned (past 2s backoff for retry_count=1)
    let logs = store.get_unsynced(10);
    assert_eq!(logs.len(), 1);
}

#[test]
fn test_backoff_cap_at_60_seconds() {
    let store = InMemoryStore::new(1000);

    // Insert log with high retry count
    let data = AttendanceData {
        user_id: "U1".to_string(),
        timestamp: "2026-01-29 10:00:00".to_string(),
        status: Some(1),
        punch: Some(1),
    };
    let id = store.insert("D1".to_string(), data).unwrap();

    // Simulate many retries (retry_count=10)
    if let Some(mut log) = store.logs.get_mut(&id) {
        log.retry_count = 10;
        log.updated_at = Some(chrono::Utc::now().naive_utc() - chrono::Duration::seconds(59));
    }

    // Should NOT be returned (still within 60s cap)
    let logs = store.get_unsynced(10);
    assert_eq!(logs.len(), 0);

    // After 61 seconds, should be eligible
    if let Some(mut log) = store.logs.get_mut(&id) {
        log.updated_at = Some(chrono::Utc::now().naive_utc() - chrono::Duration::seconds(61));
    }
    let logs = store.get_unsynced(10);
    assert_eq!(logs.len(), 1);
}

#[test]
fn test_backoff_new_logs_prioritized() {
    let store = InMemoryStore::new(1000);

    // Insert retry log (in backoff)
    let data1 = AttendanceData {
        user_id: "RETRY".to_string(),
        timestamp: "2026-01-29 10:00:00".to_string(),
        status: Some(1),
        punch: Some(1),
    };
    let id1 = store.insert("D1".to_string(), data1).unwrap();
    store.mark_in_flight(&[id1]);
    store.reset_to_unsynced(id1);

    // Insert new log
    let data2 = AttendanceData {
        user_id: "NEW".to_string(),
        timestamp: "2026-01-29 10:00:01".to_string(),
        status: Some(1),
        punch: Some(1),
    };
    store.insert("D1".to_string(), data2).unwrap();

    // Only NEW should be returned (RETRY is in backoff)
    let logs = store.get_unsynced(10);
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].user_id, "NEW");
}
```

**Acceptance Criteria:**
- [ ] All 5 tests pass
- [ ] Tests cover: new logs, recent retry, past backoff, cap at 60s, priority

---

### Step 4: Update HybridDatabase (passthrough)

**File:** `/home/elt1541/leeattend/agent-main/src/hybrid_database.rs`

No changes needed - `get_unsynced_logs()` already delegates to `memory.get_unsynced()`.

Optionally add passthrough for metrics:
```rust
/// Count logs currently in backoff
pub fn count_in_backoff(&self) -> usize {
    self.memory.count_in_backoff()
}
```

---

## Test Strategy

1. **Unit Tests:** Run `cargo test memory_store` - verify backoff logic
2. **Integration Test:** Add test in `tests/retry_limit_test.rs`:
   - Insert log, mark in-flight, reset, verify backoff applies
3. **Manual Test:**
   - Start agent, inject failing log
   - Observe logs show "X logs in backoff" message
   - Verify retry spacing increases

**Commands:**
```bash
cargo test memory_store
cargo test retry
cargo run  # manual observation
```

---

## Rollback Plan

If issues arise:
1. Revert `get_unsynced()` to original (remove backoff filter)
2. Remove `count_in_backoff()` method
3. Remove debug logging in central_client.rs

Single commit, easy revert: `git revert HEAD`

---

## Effort Breakdown

| Step | Effort | Risk |
|------|--------|------|
| Step 1: Modify get_unsynced() | 30 min | Low |
| Step 2: Add metrics | 20 min | Low |
| Step 3: Unit tests | 45 min | Low |
| Step 4: Integration test | 30 min | Low |
| Manual testing | 20 min | Low |
| **Total** | **~2.5 hours** | **Low** |

---

## Key Design Decisions

1. **Stateless:** No new fields - uses existing `retry_count` + `updated_at`
2. **In-memory only:** Backoff computed at read time, not stored
3. **New logs prioritized:** Only retries have backoff (updated_at != None)
4. **Cap at 60s:** Prevents infinite backoff while giving server time to recover
5. **FIFO maintained:** Iterates all unsynced, skips ineligible

---

## Unresolved Questions

None - implementation path clear.

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/memory_store.rs` | `get_unsynced()` + `count_in_backoff()` + tests |
| `src/hybrid_database.rs` | Optional passthrough for metrics |
| `src/central_client.rs` | Debug logging for backoff count |

**Total LOC:** ~80 (including tests)
