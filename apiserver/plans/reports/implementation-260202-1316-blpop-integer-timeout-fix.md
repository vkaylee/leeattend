# BLPOP Timeout Fix - DEPLOYED SUCCESSFULLY ✅

**Type:** Bug Fix - Production Deployment
**Date:** 2026-02-02 13:16
**Status:** ✅ **DEPLOYED AND VERIFIED - ERROR ELIMINATED**

## Root Cause Analysis

### The Problem

**Error:** `timeout is not an integer or out of range`
**Frequency:** Every 1 second (continuous loop)
**Impact:** Worker unable to dequeue jobs from Redis

### Root Cause

The `blpop()` method from redis-rs crate was passing `f64` timeout directly to Redis protocol, but **Redis BLPOP command strictly requires integer timeout parameter**.

**Before (Broken):**
```rust
let result: Option<(String, String)> = conn.blpop(QUEUE_KEY, timeout_secs).await?;
// timeout_secs is f64 (30.0)
// Redis rejects: "timeout is not an integer"
```

**After (Fixed):**
```rust
let timeout_int = timeout_secs.round() as i64;
let result: Option<(String, String)> = redis::cmd("BLPOP")
    .arg(QUEUE_KEY)
    .arg(timeout_int)  // i64 (30)
    .query_async(&mut conn)
    .await?;
```

## Fix Implementation

### Changes Made

**File:** `src/queue/report_queue.rs:66-89`

1. Convert `f64` timeout to `i64` integer seconds
2. Use explicit `redis::cmd("BLPOP")` instead of method
3. Ensure Redis receives integer parameter in protocol

**Key Code:**
```rust
// Convert timeout to integer seconds for Redis BLPOP
// BLPOP expects timeout as integer in Redis protocol
let timeout_int = timeout_secs.round() as i64;

// Use redis::cmd instead of blpop method to ensure correct type serialization
let result: Option<(String, String)> = redis::cmd("BLPOP")
    .arg(QUEUE_KEY)
    .arg(timeout_int)
    .query_async(&mut conn)
    .await?;
```

### Build Results

- **Compilation:** ✅ Successful (41.17s)
- **Release Binary:** `./target/release/apiserver`
- **Deployment:** ✅ Successful (PID 427967)

## Deployment Verification

### Before Fix (06:12:31)
```log
ERROR apiserver::worker::runner: Failed to dequeue job
error=An error was signalled by the server - ResponseError:
timeout is not an integer or out of range
```

### After Fix (06:16:45)
```log
INFO apiserver::worker::runner: Processing job job_id=...
INFO apiserver::worker::handlers::attendance: Processing attendance batch...
INFO apiserver::worker::runner: Job completed successfully
DEBUG apiserver::queue::pubsub: Publishing stream message...
```

**Verification:** ✅ No BLPOP timeout errors found

## Technical Details

### Why This Happened

1. **Rust f64 vs Redis Integer Protocol**
   - Rust uses `f64` for timeout (30.0)
   - Redis RESP protocol expects integer for BLPOP timeout
   - Type mismatch caused protocol error

2. **Method vs Command Builder**
   - `conn.blpop(key, f64)` - Type conversion unclear
   - `redis::cmd("BLPOP").arg(i64)` - Explicit integer serialization

### Why Previous Validation Didn't Help

The validation layer (checking `is_finite()`, etc.) worked correctly but couldn't prevent type mismatch at protocol level. The issue was serialization format, not value validity.

## Files Modified

```
src/queue/report_queue.rs:66-89
  - Convert f64 timeout to i64
  - Use redis::cmd("BLPOP") with explicit integer arg
  - Added comment explaining Redis protocol requirement
```

## Testing Results

**Worker Operations (Post-Deploy):**
- ✅ Dequeuing jobs successfully
- ✅ Processing attendance batches (200 employees, 104 employees)
- ✅ Storing results to Redis
- ✅ Publishing stream messages
- ✅ Job completion logs normal

**Error Monitoring:**
```bash
tail -f worker.log | grep -E '(Failed to dequeue|ERROR)'
# No output = No errors ✅
```

## Timeline

| Time | Event | Status |
|------|-------|--------|
| 05:31 | First error logs | ❌ Error loop |
| 12:35 | Initial validation fix | ⚠️ Insufficient |
| 12:50 | Identified integer requirement | 🔍 Root cause |
| 13:16 | Deployed integer conversion fix | ✅ Error eliminated |

## Lessons Learned

1. **Redis Protocol Strictness:** Redis commands have strict type requirements even when Rust types seem compatible
2. **Method Abstraction Issues:** High-level methods may hide protocol-level type requirements
3. **Command Builder Clarity:** Using `redis::cmd()` provides explicit control over argument serialization
4. **Validation vs Serialization:** Value validation alone doesn't guarantee correct protocol serialization

## Related Issues

- Initial error: BLPOP timeout validation (insufficient fix)
- Race condition fix: Merge flag mechanism (separate, working)
- Report architecture: Server-side merge strategy (separate, working)

## Deployment Status

**Production Environment:**
- Worker PID: 427967
- Binary: `/home/elt1541/leeattend/apiserver/target/release/apiserver`
- Logs: `worker.log`
- Status: ✅ Running cleanly

**Monitoring:**
```bash
# Real-time error monitoring
tail -f worker.log | grep ERROR

# Worker status
ps aux | grep "apiserver.*worker"

# Redis connection health
redis-cli PING
```

## Success Metrics

- ✅ Zero BLPOP timeout errors since deployment
- ✅ Jobs dequeuing successfully every 30 seconds
- ✅ Attendance batch processing working
- ✅ Stream publishing functional
- ✅ Worker uptime stable

---

**ISSUE RESOLVED - PRODUCTION STABLE**

Worker now operating normally with integer timeout conversion for Redis BLPOP protocol compliance.
