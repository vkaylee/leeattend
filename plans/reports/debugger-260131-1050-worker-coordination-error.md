# Worker Coordination Error Analysis

**Error**: `QueueError("Response was of incompatible type - TypeError: \"Response type not convertible to numeric.\" (response was array([int(35)]))")`

**Date**: 2026-01-31T03:49:22Z
**Component**: apiserver::worker::runner
**Parent Job**: c4ab3752-af99-442e-b2b2-804a1db518d5

## Root Cause

Redis pipe query returning array instead of single value when calling `increment_completed()` or `increment_failed()`.

**Location**: `apiserver/src/queue/report_queue.rs:165-170`

```rust
let count: u32 = redis::pipe()
    .incr(&counter_key, 1)
    .expire(&counter_key, JOB_TTL_SECS)
    .ignore()  // <-- PROBLEM: .ignore() not working as expected
    .query_async(&mut conn)
    .await?;
```

## Type Mismatch Details

**Expected**: `u32` (single value from INCR command)
**Actual**: `array([int(35)])` - array with both INCR and EXPIRE results

The `.ignore()` modifier should suppress the EXPIRE result, but redis crate v0.27 is returning both command results in an array format.

## Affected Code Sections

### 1. Report Coordinator (`apiserver/src/services/report_coordinator.rs:318-322`)

```rust
let completed_count = if success {
    self.queue
        .increment_completed(parent_job_id)  // <-- FAILS HERE
        .await
        .map_err(|e| ReportError::QueueError(e.to_string()))?
```

### 2. Report Queue (`apiserver/src/queue/report_queue.rs`)

**Lines 160-173**: `increment_completed()` - INCR + EXPIRE pipe
**Lines 183-194**: `increment_failed()` - INCR + EXPIRE pipe
**Lines 351-363**: `increment_daily_completed()` - INCR + EXPIRE pipe
**Lines 368-380**: `increment_daily_failed()` - INCR + EXPIRE pipe

All use same pattern with `.ignore()`.

### 3. Worker Runner (`apiserver/src/worker/runner.rs:372-384`)

```rust
if let Err(e) = self
    .coordinator
    .on_batch_complete(parent_job_id, batch_index, success)  // <-- Calls increment
    .await
{
    error!(
        parent_id = %parent_id,
        error = ?e,
        "Failed to report completion to coordinator"  // <-- ERROR LOGGED HERE
    );
}
```

## Expected vs Actual Behavior

**Expected Flow**:
1. Worker completes batch
2. Calls `coordinator.on_batch_complete()`
3. Coordinator calls `queue.increment_completed()`
4. Redis pipe: INCR returns new count (e.g., 35), EXPIRE is ignored
5. Function returns `Ok(35u32)`
6. Coordinator checks if all batches done

**Actual Flow**:
1. Worker completes batch
2. Calls `coordinator.on_batch_complete()`
3. Coordinator calls `queue.increment_completed()`
4. Redis pipe: Returns `array([int(35), int(1)])` or similar
5. Type conversion fails: cannot convert array to u32
6. Error propagates to worker runner
7. Logged as QueueError

## Impact Assessment

**Severity**: HIGH - Breaks distributed report processing

**Symptoms**:
- Worker completes batch successfully
- Cannot report completion to coordinator
- Coordinator doesn't know batch is done
- Merge job never triggers
- Report stuck in "processing" state
- S3 upload never happens

**Affected Operations**:
- All distributed attendance reports (7000+ employees)
- Batch completion tracking
- Job progress monitoring
- Merge triggering logic

**Data Integrity**: PARTIAL
- Batch processing completes successfully
- Batch results stored in Redis
- Only coordination/tracking broken
- Manual recovery possible via Redis inspection

## Fix Recommendations

### Option 1: Extract first element from array (Quick Fix)

```rust
pub async fn increment_completed(&self, parent_job_id: Uuid) -> Result<u32, redis::RedisError> {
    let mut conn = self.pool.conn();
    let counter_key = format!("{}{}", COUNTER_PREFIX, parent_job_id);

    // Pipe returns array even with .ignore(), extract first element
    let results: Vec<u32> = redis::pipe()
        .incr(&counter_key, 1)
        .expire(&counter_key, JOB_TTL_SECS)
        .ignore()
        .query_async(&mut conn)
        .await?;

    let count = results.first()
        .copied()
        .ok_or_else(|| redis::RedisError::from((
            redis::ErrorKind::TypeError,
            "Empty response from pipe"
        )))?;

    Ok(count)
}
```

### Option 2: Separate commands (Reliable)

```rust
pub async fn increment_completed(&self, parent_job_id: Uuid) -> Result<u32, redis::RedisError> {
    let mut conn = self.pool.conn();
    let counter_key = format!("{}{}", COUNTER_PREFIX, parent_job_id);

    // Use atomic_incr with separate EXPIRE
    let count: u32 = conn.incr(&counter_key, 1).await?;
    let _: () = conn.expire(&counter_key, JOB_TTL_SECS).await?;

    Ok(count)
}
```

**Tradeoff**: Two round-trips vs one pipe, but guaranteed correctness.

### Option 3: Use tuple destructuring (Explicit)

```rust
pub async fn increment_completed(&self, parent_job_id: Uuid) -> Result<u32, redis::RedisError> {
    let mut conn = self.pool.conn();
    let counter_key = format!("{}{}", COUNTER_PREFIX, parent_job_id);

    // Explicitly expect both results
    let (count, _expire_ok): (u32, i32) = redis::pipe()
        .incr(&counter_key, 1)
        .expire(&counter_key, JOB_TTL_SECS)
        .query_async(&mut conn)
        .await?;

    Ok(count)
}
```

**Note**: Remove `.ignore()` and destructure tuple.

## Recommended Solution

**Use Option 3** (tuple destructuring) for all 4 affected functions:
- Most explicit and type-safe
- No array parsing needed
- Single pipeline (efficient)
- Clear intent

Apply to:
- `increment_completed()` (L160-173)
- `increment_failed()` (L183-194)
- `increment_daily_completed()` (L351-363)
- `increment_daily_failed()` (L368-380)

## Side Effects of Fix

**Positive**:
- Worker coordination restored
- Report jobs complete properly
- Merge triggers correctly
- S3 uploads work

**Neutral**:
- No performance impact (same pipeline)
- No breaking changes to API

**Testing Needed**:
- Unit tests for queue methods
- Integration test: full report workflow
- Verify existing jobs recover
- Monitor Redis response format

## Recent Changes

Introduced in commit `9caa956` (2026-01-30):
- "feat(reports): add distributed report processing system"
- Original implementation used `.ignore()` assuming single value return
- Likely worked in development/testing with older redis-rs version
- Production environment may have different redis server behavior

## Verification Steps

1. Check Redis server version on production
2. Test redis-rs v0.27 pipe behavior with current Redis server
3. Apply fix to all 4 increment methods
4. Run integration tests with real Redis instance
5. Deploy and monitor error logs
6. Verify stuck jobs can recover

## Unresolved Questions

1. Why does `.ignore()` return array in production but not dev?
   - Redis server version difference?
   - redis-rs crate behavior change?
   - Connection manager vs direct connection?

2. Are there other redis pipes with `.ignore()` elsewhere?
   - Searched codebase: only in report_queue.rs
   - No other affected code

3. Can existing stuck jobs recover automatically?
   - Need manual Redis inspection
   - May need to re-enqueue or mark complete manually
   - Or implement recovery endpoint

## Related Files

- `apiserver/src/queue/report_queue.rs` - Queue implementation (FIX HERE)
- `apiserver/src/services/report_coordinator.rs` - Coordinator calling increment
- `apiserver/src/worker/runner.rs` - Worker logging error
- `apiserver/Cargo.toml` - redis = "0.27" dependency

## Next Steps

1. Apply Option 3 fix to all 4 increment methods
2. Add unit test covering pipe return values
3. Run `./run-tests.sh --api queue` to verify
4. Check for stuck jobs in production Redis
5. Deploy fix and monitor logs for 24h
