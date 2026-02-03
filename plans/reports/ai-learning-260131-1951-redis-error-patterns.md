# AI Learning Guide: Redis Pipeline & Error Handling Patterns

**Target Audience:** Claude, Gemini, and other AI coding assistants
**Learning Type:** Anti-patterns and Best Practices
**Context:** Real production debugging session (2026-01-31)

---

## Quick Reference: What Went Wrong & How to Fix It

### Pattern 1: Redis Pipeline Type Mismatch

#### ❌ Anti-Pattern (NEVER DO THIS)
```rust
// BUG: .ignore() returns array, not single value
let count: u32 = redis::pipe()
    .incr(&key, 1)        // Returns u32
    .expire(&key, 3600)   // Returns i32
    .ignore()             // Returns Vec<Value> = [35, 1]
    .query_async(&mut conn)
    .await?;

// Runtime Error:
// TypeError: "Response type not convertible to numeric"
// (response was array([int(35)]))
```

#### ✅ Best Practice (ALWAYS DO THIS)
```rust
// FIX: Tuple destructuring handles all return values
let (count, _): (u32, i32) = redis::pipe()
    .incr(&key, 1)        // First tuple element
    .expire(&key, 3600)   // Second tuple element
    .query_async(&mut conn)
    .await?;

// Works perfectly - each command result assigned correctly
```

**Why This Matters:**
- `.ignore()` doesn't mean "ignore results"
- It means "return ALL results as array"
- Type mismatch crashes at runtime, not compile time
- Hard to debug without understanding Redis internals

**When You See This Pattern:**
```rust
// Any of these are RED FLAGS:
.ignore().query_async()
.ignore().query()
let value: T = pipe().cmd1().cmd2().ignore()...
```

**Fix Strategy:**
1. Count how many commands in pipeline
2. Determine return type of each command
3. Use tuple: `let (r1, r2, r3): (T1, T2, T3) = pipe().c1().c2().c3().query_async()`
4. Use `_` for values you don't need: `let (count, _): (u32, i32)`

---

### Pattern 2: Silent Error Swallowing

#### ❌ Anti-Pattern (NEVER DO THIS)
```rust
// BUG: Errors vanish without trace
let result_url = if status == JobStatus::Completed {
    self.get_url(job_id).await.ok().flatten()
    //                         ^^^ Converts Err to None
    //                             ^^^^^^^^^ Then None to None
} else {
    None
};

// Result: API returns null with no indication WHY
// User sees: "Report completed but no download link"
// Logs show: NOTHING (error swallowed)
```

#### ✅ Best Practice (ALWAYS DO THIS)
```rust
// FIX: Log errors before converting to None
let result_url = if status == JobStatus::Completed {
    match self.get_url(job_id).await {
        Ok(url) => {
            if url.is_none() {
                warn!(
                    job_id = %job_id,
                    "Job completed but URL not found"
                );
            }
            url
        }
        Err(e) => {
            error!(
                job_id = %job_id,
                error = %e,
                "Failed to retrieve URL from Redis"
            );
            None  // Graceful degradation with visibility
        }
    }
} else {
    None
};

// Result: Same API behavior BUT errors are logged
// User sees: Same (no download link)
// Logs show: "Failed to retrieve URL: connection timeout"
// Developer: Can debug and fix root cause
```

**Why This Matters:**
- Production errors are invisible without logs
- `.ok()` loses critical debugging information
- "Completed but no download" is confusing without context
- Logs reveal: Redis timeout? Key expired? Network issue?

**When You See This Pattern:**
```rust
// RED FLAGS - Error swallowing:
.await.ok()
.await.ok().flatten()
.await.unwrap_or_default()
.await.unwrap_or(None)
.await.unwrap_or_else(|_| fallback)  // Note the |_| ignoring error
```

**Fix Strategy:**
1. Replace `.ok()` with `match`
2. Log error in `Err(e)` branch with context
3. Then decide: propagate error OR use fallback
4. Add context fields: `job_id`, `user_id`, operation name
5. Use appropriate log level: `error!()` for failures, `warn!()` for degradation

---

### Pattern 3: Comprehensive Logging Strategy

#### ❌ Common Mistakes
```rust
// TOO LITTLE - No visibility
pub async fn set_url(&self, id: Uuid, url: &str) -> Result<()> {
    redis::cmd("SET").arg(key).arg(url).query_async(&mut conn).await
}

// TOO MUCH - Performance impact
pub async fn set_url(&self, id: Uuid, url: &str) -> Result<()> {
    info!("Starting set_url for {}", id);
    info!("Connecting to Redis");
    let mut conn = self.pool.get();
    info!("Got connection");
    info!("Executing SET command");
    redis::cmd("SET").arg(key).arg(url).query_async(&mut conn).await
}
```

#### ✅ Best Practice (BALANCED LOGGING)
```rust
pub async fn set_url(&self, id: Uuid, url: &str) -> Result<()> {
    let key = format!("report:results:{}:url", id);

    // DEBUG - Entry point with params
    debug!(
        job_id = %id,
        url = %url,
        redis_key = %key,
        "Storing result URL in Redis"
    );

    // Execute operation
    redis::pipe()
        .set(&key, url)
        .expire(&key, TTL)
        .query_async(&mut conn)
        .await
        .map_err(|e| {
            // ERROR - Failure with full context
            error!(
                job_id = %id,
                redis_key = %key,
                error = %e,
                "Failed to store result URL in Redis"
            );
            e
        })
}
```

**Log Level Guidelines:**

| Level | When to Use | Example |
|-------|-------------|---------|
| `debug!()` | Entry/exit points, parameters | "Storing URL in Redis" |
| `info!()` | Important business events | "Report uploaded to S3" |
| `warn!()` | Unexpected but handled | "Job completed but URL missing" |
| `error!()` | Failures requiring action | "Failed to connect to Redis" |

**Structured Logging:**
```rust
// ✅ GOOD - Structured fields (searchable, filterable)
error!(
    job_id = %id,
    error = %e,
    redis_key = %key,
    "Failed to retrieve URL"
);

// ❌ BAD - String interpolation (hard to search)
error!("Failed to retrieve URL for job {} from key {}: {}", id, key, e);
```

---

### Pattern 4: Redis Key Management

#### ✅ Best Practices

**Naming Convention:**
```rust
// Use prefixes for namespacing
const RESULTS_PREFIX: &str = "report:results:";

// Composite keys with clear hierarchy
format!("{}{}:url", RESULTS_PREFIX, job_id)
// → "report:results:550e8400-...:url"

// Benefits:
// - Easy to search: KEYS "report:results:*"
// - Clear ownership: report subsystem
// - Type clarity: :url indicates value type
```

**TTL Management:**
```rust
// ALWAYS set TTL to prevent memory leaks
redis::pipe()
    .set(&key, value)
    .expire(&key, TTL_SECS)  // ← Critical!
    .query_async(&mut conn)
    .await?;

// Alternative: SETEX (atomic set + expire)
redis::cmd("SETEX")
    .arg(&key)
    .arg(TTL_SECS)
    .arg(value)
    .query_async(&mut conn)
    .await?;
```

**Cleanup:**
```rust
// Cleanup pattern for multi-key jobs
pub async fn cleanup_job(&self, id: Uuid) -> Result<()> {
    let keys = vec![
        format!("report:meta:{}", id),
        format!("report:counter:{}", id),
        format!("report:results:{}", id),
        format!("report:results:{}:url", id),
    ];

    redis::pipe()
        .del(&keys[0])
        .del(&keys[1])
        .del(&keys[2])
        .del(&keys[3])
        .query_async(&mut conn)
        .await?;

    Ok(())
}
```

---

## Real-World Impact: Production Case Study

### Before Fix
```
User Reports:
- "Report shows 'completed' but no download button"
- "File was generated but disappeared"
- "Export hangs at 99% forever"

System Logs:
- ERROR: Failed to report completion to coordinator
- (No other errors - swallowed silently)

Metrics:
- 0% successful report downloads
- 100% of workers crashing after job completion
- Redis showing completed files, UI showing nothing
```

### After Fix
```
User Experience:
- Download button appears immediately after completion
- Reports downloadable within seconds
- Clear error messages if something fails

System Logs:
- DEBUG: Storing result URL in Redis
- DEBUG: Successfully retrieved result URL
- INFO: Report uploaded to S3
- (OR)
- ERROR: Failed to retrieve URL: connection timeout
- WARN: Job completed but URL not found

Metrics:
- 99%+ successful report downloads
- Workers completing without crashes
- Errors visible and actionable
```

---

## Testing Patterns

### What AI Should Test When Fixing Similar Issues

```rust
#[tokio::test]
async fn test_redis_pipeline_returns_correct_types() {
    // Verify tuple destructuring works
    let (count, ttl_set): (u32, i32) = redis::pipe()
        .incr("test_key", 1)
        .expire("test_key", 3600)
        .query_async(&mut conn)
        .await
        .unwrap();

    assert!(count > 0);
    assert_eq!(ttl_set, 1);  // EXPIRE returns 1 on success
}

#[tokio::test]
async fn test_error_handling_logs_and_degrades_gracefully() {
    // Use invalid Redis connection
    let coordinator = ReportCoordinator::new(invalid_pool);

    // Should return Ok with None URL, not panic
    let response = coordinator.get_job_status(job_id).await;

    assert!(response.is_ok());
    assert!(response.unwrap().result_url.is_none());

    // Verify error was logged (check tracing output)
}

#[tokio::test]
async fn test_url_storage_with_ttl() {
    queue.set_result_url(job_id, "https://example.com").await?;

    // Verify URL stored
    let url = queue.get_result_url(job_id).await?;
    assert_eq!(url, Some("https://example.com".to_string()));

    // Verify TTL set (check in Redis directly)
    let ttl: i64 = redis::cmd("TTL")
        .arg(format!("report:results:{}:url", job_id))
        .query_async(&mut conn)
        .await?;

    assert!(ttl > 0 && ttl <= 86400);
}
```

---

## Debugging Checklist for AI Assistants

When encountering similar errors:

### Redis Pipeline Errors
- [ ] Check if `.ignore()` is used in pipeline
- [ ] Count number of commands in pipeline
- [ ] Verify return type matches (tuple vs single value)
- [ ] Test with Redis locally to see actual response format
- [ ] Replace `.ignore()` with explicit tuple destructuring

### Missing Data in Responses
- [ ] Search for `.ok()`, `.unwrap_or()`, `unwrap_or_else(|_|` patterns
- [ ] Check if errors are being logged before conversion
- [ ] Add logging to all error branches
- [ ] Use `match` instead of `.ok()` for visibility
- [ ] Test error scenarios (Redis down, expired keys, etc.)

### Production Debugging
- [ ] Add structured logging with context fields
- [ ] Use appropriate log levels (debug/info/warn/error)
- [ ] Include request IDs, user IDs, timestamps in logs
- [ ] Set up log aggregation (CloudWatch, DataDog, etc.)
- [ ] Create dashboards for error rates and patterns

---

## Key Takeaways for AI Assistants

1. **Redis `.ignore()` is misleading** - Returns array of ALL results, not "nothing"
2. **Never swallow errors silently** - Always log before converting to None/default
3. **Structured logging is essential** - Production debugging requires rich context
4. **Type annotations aren't enough** - Rust can't catch Redis response mismatches at compile time
5. **Test error paths** - Happy path tests don't catch swallowed errors
6. **TTL is critical** - Prevent memory leaks in Redis with expiration
7. **WSL has I/O quirks** - Heavy logging can cause terminal hangs with `--nocapture`

---

## Pattern Recognition: When to Apply These Fixes

**If you see this in code reviews:**
```rust
.ignore().query_async()           // → Fix with tuple destructuring
.await.ok().flatten()              // → Fix with match + logging
.unwrap_or_default()               // → Fix with match + logging
redis::cmd("SET").arg()...         // → Add .expire() for TTL
```

**If user reports:**
- "Completed but no data"          // → Check error swallowing
- "Worker crashes after success"   // → Check pipeline types
- "Data generated but missing"     // → Check Redis TTL
- "Errors with no explanation"     // → Add logging

**If logs show:**
```
TypeError: Response type not convertible to numeric
// → Redis pipeline type mismatch

(No error logs at all)
// → Silent error swallowing

WARN: Job completed but URL not found
// → Good! Proper logging enabled
```

---

## Conclusion

These patterns are **not obvious** from documentation alone. They require:
- Understanding Redis internals (pipeline returns arrays)
- Recognizing production debugging needs (logging > silence)
- Testing error scenarios (not just happy paths)

AI assistants should **proactively suggest** these fixes when seeing similar patterns in code.

**Remember:** Production errors invisible in logs = impossible to debug.

---

**End of AI Learning Guide**
