# FIFO Order Implementation for Sync Queue

**Date:** 2026-01-28 09:47
**Status:** ✅ COMPLETE

## Problem

`get_unsynced_logs()` query had no ORDER BY clause, causing undefined order when syncing to API server. Logs might not be processed FIFO.

## Solution

Added `ORDER BY id ASC` to ensure FIFO (First In, First Out) order.

### Code Change

**File:** `src/database.rs:138-153`

```rust
pub async fn get_unsynced_logs(&self, limit: i64) -> Result<Vec<RawAttendanceLog>, Error> {
    let logs = sqlx::query_as::<_, RawAttendanceLog>(
        r#"
        SELECT id, device_id, user_id, timestamp, status, punch, is_synced, created_at
        FROM zkt_raw_attendance
        WHERE is_synced = 0
        ORDER BY id ASC  ← ADDED
        LIMIT ?
        "#,
    )
    .bind(limit)
    .fetch_all(&self.pool)
    .await?;

    Ok(logs)
}
```

## Why `id ASC`?

- **id** is auto-increment PRIMARY KEY
- Represents insertion order (first inserted = lowest id)
- Already indexed (PRIMARY KEY), no performance penalty
- Deterministic and reliable

## Test Coverage

### New Test: `test_fifo_order_get_unsynced_logs`

**File:** `src/database.rs:614-648`

**Validates:**
1. Insert logs with specific user_ids: "FIRST", "SECOND", "THIRD", "FOURTH", "FIFTH"
2. Fetch logs with `get_unsynced_logs()`
3. Assert returned in same order
4. Assert IDs are ascending

**Result:** ✅ Test passes

## Behavior Guarantee

**Before fix:**
```
Insert: A, B, C
Fetch:  B, A, C (undefined order)
Sync:   B → A → C (wrong order!)
```

**After fix:**
```
Insert: A, B, C
Fetch:  A, B, C (FIFO guaranteed)
Sync:   A → B → C (correct order!)
```

## Performance

**No performance impact:**
- `id` is PRIMARY KEY (auto-indexed)
- ORDER BY on indexed column is fast
- SQLite optimizer handles efficiently

## Test Results

✅ **All 47 tests passing** (was 46, added 1 FIFO test)
- Database: 13 tests (+1 for FIFO)
- Central client: 9 tests
- Worker client: 3 tests
- Retry logic: 7 tests
- Integration: 8 tests
- DB writer singleton: 3 tests
- Device clear safety: 4 tests

✅ **Release build succeeds**

## Related Guarantees

**Entire pipeline now FIFO:**

1. **Device → Queue:** Channel maintains order
2. **Queue → DB:** Batch writer inserts in order (id auto-increment)
3. **DB → Sync:** `ORDER BY id ASC` ensures FIFO ← THIS FIX
4. **Sync → Central:** Sent in fetched order

**End-to-end FIFO guarantee!**

## Files Modified

1. `src/database.rs` - Added ORDER BY id ASC to query
2. `src/database.rs` - Added test_fifo_order_get_unsynced_logs

## Deployment

✅ Ready for production
✅ Backward compatible
✅ No schema changes needed
✅ Performance unaffected
