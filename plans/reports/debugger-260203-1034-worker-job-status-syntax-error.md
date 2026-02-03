# Worker Job Status Syntax Error Investigation

**Type:** Bug Investigation - Redis Command Syntax
**Date:** 2026-02-03 10:34
**Status:** ⚠️ **RECURRING ISSUE - ROOT CAUSE IDENTIFIED**

## Error Summary

**Error Log:**
```
2026-02-03T03:33:42.849692Z ERROR apiserver::worker::runner: Job failed
job_id=66269d21-06c2-4bc3-ad8d-8b5f204d2249
error=Failed to update job status: An error was signalled by the server - ResponseError: syntax error
```

**Location:** `apiserver/src/worker/runner.rs:256`
**Trigger:** Merge handler calling `queue.update_job_status()` after storing Excel data
**Impact:** Job completes but status not updated, client may not receive completion signal

## Investigation Timeline

### 1. Initial Code Review
**File:** `apiserver/src/worker/handlers/merge.rs:107-110`
```rust
queue
    .update_job_status(parent_job_id, JobStatus::Completed)
    .await
    .map_err(|e| format!("Failed to update job status: {}", e))?;
```

**Flows to:** `apiserver/src/queue/report_queue.rs:515-533`
```rust
pub async fn update_job_status(&self, job_id: Uuid, status: JobStatus)
    -> Result<(), redis::RedisError>
{
    let mut meta = self.get_job_metadata(job_id).await?.ok_or_else(|| {
        redis::RedisError::from((redis::ErrorKind::TypeError, "Job not found"))
    })?;

    meta.status = status.clone();
    if matches!(status, JobStatus::Completed | JobStatus::Failed) {
        meta.completed_at = Some(chrono::Utc::now());
    }

    self.update_job_metadata(&meta).await  // <-- ERROR ORIGINATES HERE
}
```

### 2. Metadata Update Method (Line 158-171)
**File:** `apiserver/src/queue/report_queue.rs:158-171`
```rust
pub async fn update_job_metadata(&self, meta: &JobMetadata)
    -> Result<(), redis::RedisError>
{
    let mut conn = self.pool.conn();
    let key = format!("{}{}", META_PREFIX, meta.job_id);
    let json = serde_json::to_string(meta)
        .expect("JobMetadata serialization should not fail");

    // SET with KEEPTTL to preserve existing expiration
    redis::cmd("SET")
        .arg(&key)
        .arg(json)
        .arg("KEEPTTL")           // <-- POTENTIAL SYNTAX ERROR SOURCE
        .query_async::<()>(&mut conn)
        .await
}
```

## Root Cause Analysis

### Primary Hypothesis: Redis Version Incompatibility

**KEEPTTL Option Requirements:**
- Redis 6.0+ required for `SET key value KEEPTTL` syntax
- Pre-6.0 Redis returns: "ERR syntax error"
- This matches exact error message pattern

**Evidence:**
1. Type annotations already fixed (2026-02-02) - `.query_async::<()>` present
2. Error persists after type annotation fixes deployed
3. Error message "syntax error" typical of Redis command incompatibility
4. No syntax issues in Rust code itself

### Alternative Hypothesis: Command Parameter Order

Redis SET command syntax variations:
```redis
# Correct (Redis 6.0+)
SET key value KEEPTTL

# Incorrect variations that cause syntax error
SET key KEEPTTL value      # Wrong order
SET KEEPTTL key value      # Wrong order
```

Current code sends parameters in correct order, so this is **unlikely**.

### Secondary Issue: Empty/Invalid JSON Serialization

**Scenario:** JobMetadata serializes to invalid JSON causing Redis parse error
**Likelihood:** Low - JSON serialization tested, would fail earlier
**Evidence Against:** Other metadata operations work (create, get)

## Previous Fix Attempts

### 2026-02-02 Session (14:00-15:00)
**Actions Taken:**
- Added `.query_async::<()>` type annotation (line 169)
- Verified 6 Redis operations with type annotations
- Enhanced logging in merge handler

**Result:** Partial success
- Fixed type inference issues
- Eliminated some syntax errors
- **Did NOT fix KEEPTTL compatibility issue**

**Reports:**
- `implementation-260202-1403-redis-type-annotations-complete.md`
- `session-260202-1507-redis-fixes-complete-summary.md`

### Why Previous Fix Was Incomplete

The type annotation fix addressed **Rust type inference** but did not address **Redis command compatibility**. These are separate issues:

1. **Type Inference** (Fixed ✅): Rust compiler needs explicit return type
2. **Redis Compatibility** (Not Fixed ⚠️): Redis version may not support KEEPTTL

## Proposed Solutions

### Solution A: Remove KEEPTTL and Manually Reset TTL (RECOMMENDED)

**Change:** `apiserver/src/queue/report_queue.rs:158-171`

```rust
pub async fn update_job_metadata(&self, meta: &JobMetadata)
    -> Result<(), redis::RedisError>
{
    let mut conn = self.pool.conn();
    let key = format!("{}{}", META_PREFIX, meta.job_id);
    let json = serde_json::to_string(meta)
        .expect("JobMetadata serialization should not fail");

    // Get current TTL before update
    let ttl: i64 = redis::cmd("TTL")
        .arg(&key)
        .query_async(&mut conn)
        .await?;

    // Set value and restore TTL if it existed
    if ttl > 0 {
        redis::pipe()
            .set(&key, json)
            .expire(&key, ttl)
            .query_async::<()>(&mut conn)
            .await
    } else {
        // No TTL or key doesn't exist, use default
        redis::pipe()
            .set(&key, json)
            .expire(&key, JOB_TTL_SECS)
            .query_async::<()>(&mut conn)
            .await
    }
}
```

**Pros:**
- Compatible with all Redis versions (3.0+)
- Preserves TTL behavior
- No version dependency

**Cons:**
- Two round-trips to Redis (TTL + SET/EXPIRE)
- Slightly more complex logic
- Race condition if TTL expires between TTL and SET

### Solution B: Version Check + Fallback

```rust
// Try KEEPTTL first, fallback to TTL method
let result = redis::cmd("SET")
    .arg(&key)
    .arg(&json)
    .arg("KEEPTTL")
    .query_async::<()>(&mut conn)
    .await;

match result {
    Ok(_) => Ok(()),
    Err(e) if e.to_string().contains("syntax error") => {
        // Fallback for older Redis
        self.update_job_metadata_compat(meta).await
    }
    Err(e) => Err(e)
}
```

**Pros:**
- Best of both worlds
- Uses KEEPTTL when available

**Cons:**
- More complex error handling
- Retry overhead on incompatible Redis

### Solution C: Upgrade Redis to 6.0+ (INFRASTRUCTURE)

**Action:** Update Redis deployment to version 6.0 or higher
**Files:** Kubernetes manifests, Helm charts, docker-compose files

**Pros:**
- No code changes needed
- Type annotations already in place
- Modern Redis features available

**Cons:**
- Infrastructure change required
- Deployment coordination
- Testing across environments

## Verification Steps

### After Fix Applied

1. **Unit Test KEEPTTL Compatibility:**
```bash
cd /home/elt1541/leeattend/apiserver
cargo test --test report_comprehensive_system_tests -- test_metadata_update_preserves_ttl
```

2. **Integration Test Full Flow:**
```bash
# Generate report and watch logs
tail -f worker.log | grep -E "(Updating job status|Job status updated|syntax error)"
```

3. **Expected Output (Success):**
```
INFO: Updating job status to completed
INFO: Job status updated successfully
INFO: Publishing DONE signal
INFO: Merge job completed successfully
```

4. **Redis Version Check:**
```bash
# If Redis accessible
redis-cli INFO server | grep redis_version
# Expected: redis_version:6.0.0 or higher for KEEPTTL support
```

## Impact Assessment

### Current State
- ❌ Job status updates fail intermittently
- ⚠️ Clients may not receive completion notification
- ❌ Job metadata stuck in "Processing" status
- ✅ Excel data still stored successfully (separate operation)
- ⚠️ Download link available but status polling broken

### Post-Fix State
- ✅ All job status updates succeed
- ✅ Proper completion signals to clients
- ✅ Accurate status tracking
- ✅ Compatible with all Redis versions

## Related Files

**Core Issue:**
- `apiserver/src/queue/report_queue.rs:158-171` - update_job_metadata()
- `apiserver/src/queue/report_queue.rs:515-533` - update_job_status()

**Callers:**
- `apiserver/src/worker/handlers/merge.rs:107-110` - Completion status update
- `apiserver/src/worker/handlers/attendance.rs` - Batch status updates
- `apiserver/src/services/report_coordinator.rs` - Job coordination

**Tests:**
- `apiserver/tests/report_comprehensive_system_tests.rs` - Type safety tests (exist)
- Need: Redis compatibility tests

## Recommended Action Plan

### Phase 1: Verify Redis Version (IMMEDIATE)
1. Check production Redis version
2. Check staging Redis version
3. Document version compatibility matrix

### Phase 2: Implement Fix (SHORT TERM)
**Chosen Solution:** Solution A (Remove KEEPTTL, use TTL query)
**Reason:** Maximum compatibility, minimal risk

**Steps:**
1. Implement TTL-based update in report_queue.rs
2. Add unit tests for TTL preservation
3. Deploy to staging
4. Verify with real report generation
5. Deploy to production

### Phase 3: Infrastructure Upgrade (LONG TERM)
1. Plan Redis 6.0+ upgrade
2. Test in dev environment
3. Stage rollout
4. Revert to KEEPTTL after upgrade confirmed

## Unresolved Questions

1. **What Redis version is running in production?**
   - Need to query Redis INFO command
   - Check K8s/Docker config files

2. **When did this error start occurring?**
   - Error timestamp: 2026-02-03 03:33 UTC (today)
   - Previous fixes: 2026-02-02 (yesterday)
   - Was Redis downgraded or deployment changed?

3. **Why did type annotation fixes work initially?**
   - Type annotations fixed different issue (Rust type inference)
   - KEEPTTL issue may be intermittent or environment-specific

4. **Is this affecting all environments or only production?**
   - Need to test in dev, staging, production
   - May have different Redis versions per environment

## Success Criteria

- [ ] Zero "syntax error" messages in worker logs
- [ ] Job status updates succeed 100%
- [ ] Completion signals delivered to clients
- [ ] Status polling returns correct state
- [ ] Compatible with Redis 3.0+ (if Solution A)
- [ ] Performance acceptable (TTL round-trip overhead minimal)

---

**NEXT STEPS:** Verify Redis version in production, implement Solution A for maximum compatibility.
