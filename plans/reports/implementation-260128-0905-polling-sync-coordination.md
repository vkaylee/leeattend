# Polling-Sync Coordination Implementation

**Date:** 2026-01-28 09:05
**Status:** ✅ COMPLETE - Ready for Deployment

## Problem

Device polling loop fetched logs repeatedly without waiting for sync:

```
01:59:12 - Fetched 22,839 logs (1st fetch)
01:59:18 - 10 batches synced (5,000 logs)
02:01:05 - Fetched 22,839 logs AGAIN (all duplicates!)
```

**Issue:** No coordination between polling and sync loops.

## Solution: Wait for Sync Before Next Fetch

Added coordination mechanism:
1. After fetching logs from device → Send to DB writer
2. If large fetch (>500 logs) → Wait for sync to catch up
3. Poll database every 10s: How many unsynced?
4. When unsynced < 1,000 → Resume device polling
5. Only then fetch from device again

### Implementation

#### 1. Added `count_unsynced_logs()` Method

**File:** `src/database.rs:162-170`

```rust
/// Count total unsynced logs for sync coordination
pub async fn count_unsynced_logs(&self) -> Result<i64, Error> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 0"
    )
    .fetch_one(&self.pool)
    .await?;

    Ok(count)
}
```

#### 2. Updated Polling Loop with Coordination

**File:** `src/main.rs:252-303`

```rust
if let Some(logs) = res.as_array().filter(|l| !l.is_empty()) {
    let fetched_count = logs.len();
    info!("[{}] Fetched {} logs", worker.name, fetched_count);

    // Send logs to DB writer
    for log in logs {
        // ... send to channel
    }

    // Wait for sync to catch up before fetching again (coordination)
    if fetched_count > 500 {
        info!("[{}] Large fetch detected ({}), waiting for sync...",
              worker.name, fetched_count);

        const SYNC_THRESHOLD: i64 = 1000;
        loop {
            sleep(Duration::from_secs(10)).await;

            match worker_db.count_unsynced_logs().await {
                Ok(unsynced) => {
                    if unsynced < SYNC_THRESHOLD {
                        info!("[{}] Sync caught up ({} unsynced), resuming",
                              worker.name, unsynced);
                        break;
                    } else {
                        info!("[{}] Waiting for sync ({} unsynced)",
                              worker.name, unsynced);
                    }
                }
                Err(e) => {
                    error!("[{}] Failed to check unsynced: {}", worker.name, e);
                    break; // Continue polling on error
                }
            }
        }
    }
}
```

#### 3. Pass Database to Worker

**File:** `src/main.rs:148-154`

```rust
for worker in workers {
    let worker_db = db.clone();  // Clone for worker coordination
    // ... other clones

    tokio::spawn(async move {
        // Worker can now query unsynced count
    });
}
```

## Behavior Changes

### Before (No Coordination)
```
01:59:12 - Fetch 22,839 logs
01:59:13 - Send to DB writer
01:59:18 - Sync 5,000 logs (still 17,839 pending)
02:01:05 - Fetch 22,839 logs AGAIN (all duplicates!)
```

### After (With Coordination)
```
01:59:12 - Fetch 22,839 logs
01:59:13 - Send to DB writer
01:59:13 - Large fetch detected, waiting for sync...
01:59:18 - Sync 5,000 logs
01:59:23 - Check unsynced: 17,839 (> 1,000 threshold)
01:59:33 - Check unsynced: 12,839 (> 1,000 threshold)
01:59:43 - Check unsynced: 7,839 (> 1,000 threshold)
01:59:53 - Check unsynced: 2,839 (> 1,000 threshold)
02:00:03 - Check unsynced: 839 (< 1,000 threshold)
02:00:03 - Sync caught up, resuming device polling
02:00:08 - Fetch from device (5s later)
```

**Key difference:** Device fetch only happens when sync queue mostly drained.

## Configuration

**Thresholds:**
- **Large fetch:** 500 logs (triggers coordination)
- **Sync threshold:** 1,000 unsynced logs (resume polling when below)
- **Check interval:** 10 seconds (how often to poll unsynced count)
- **Normal polling:** 5 seconds (when not waiting)

**Rationale:**
- 500 log threshold catches bulk fetches (22k) but not incremental (50-100)
- 1,000 unsynced allows some buffer (2 batches worth)
- 10s check interval balances responsiveness vs DB load

## Test Coverage

### New Test: `test_count_unsynced_logs`

**File:** `src/database.rs:571-611`

**Validates:**
- ✅ Initially 0 unsynced
- ✅ After insert: 150 unsynced
- ✅ After mark in-flight: 100 unsynced (50 at is_synced=2)
- ✅ After mark synced: 100 unsynced (only is_synced=0 counted)

### Test Results

**All 46 tests passing** (was 45, added 1):
- Database: 12 tests (+1 for count)
- Central client: 9 tests
- Worker client: 3 tests
- Retry logic: 7 tests
- Integration: 8 tests
- DB writer singleton: 3 tests
- Device clear safety: 4 tests

## Expected Logs

**Large fetch scenario:**
```
[XUONG 1] Fetched 22839 logs
[XUONG 1] Large fetch detected (22839), waiting for sync to catch up...
[XUONG 1] Waiting for sync (22839 unsynced, threshold 1000)
[XUONG 1] Waiting for sync (17839 unsynced, threshold 1000)
...
[XUONG 1] Sync caught up (839 unsynced), resuming device polling
```

**Small fetch scenario (< 500):**
```
[OFFICE] Fetched 150 logs
[Normal 5s polling continues]
```

## Benefits

### Eliminated Duplicate Processing
**Before:**
- Fetch 22,839 logs every ~2 min
- Process 22,839 duplicates each time
- 2.13s to check 100 duplicates × 228 batches = ~487s wasted

**After:**
- Fetch 22,839 logs once
- Wait for sync to complete
- Next fetch: mostly duplicates but MUCH less frequent

### Reduced Resource Usage
- ✅ Less CPU (fewer duplicate checks)
- ✅ Less network (fewer device fetches)
- ✅ Less DB load (fewer conflict checks)
- ✅ Cleaner logs (fewer slow query warnings)

### Better System Behavior
- ✅ Polling aligned with sync capacity
- ✅ No overwhelming DB with duplicates
- ✅ Natural backpressure mechanism
- ✅ Graceful handling of large backlogs

## Trade-offs

**Pro:**
- ✅ Dramatic reduction in duplicate processing
- ✅ Better resource utilization
- ✅ Self-throttling behavior

**Con:**
- ⚠️ Slightly more complex logic
- ⚠️ Additional database query every 10s (lightweight SELECT COUNT)
- ⚠️ Longer wait after large fetch (by design)

## Files Modified

1. `src/database.rs` - Added `count_unsynced_logs()` method
2. `src/main.rs` - Added coordination logic to polling loop
3. Database tests - Added `test_count_unsynced_logs`

## Deployment

✅ All 46 tests passing
✅ Release build succeeds
✅ Ready for production

**Expected behavior after deployment:**
1. First device fetch: 22,839 logs (or whatever device has)
2. Agent waits for sync to catch up
3. Periodic status updates every 10s
4. When < 1,000 unsynced: Resume polling
5. Next fetch: Mostly duplicates but MUCH less frequent

## Future Improvements

If needed, these thresholds can be made configurable:
- `LARGE_FETCH_THRESHOLD` (currently 500)
- `SYNC_THRESHOLD` (currently 1,000)
- `CHECK_INTERVAL` (currently 10s)

Add to `.env`:
```bash
LARGE_FETCH_THRESHOLD=500
SYNC_THRESHOLD=1000
SYNC_CHECK_INTERVAL_SECS=10
```

## Related Documentation

- Database lock fixes: `plans/reports/completion-260128-0833-database-lock-fixes.md`
- Device clear safety: `plans/reports/implementation-260128-0836-device-clear-safety-tests.md`
- Slow query analysis: `plans/reports/analysis-260128-0851-slow-query-warning.md`
