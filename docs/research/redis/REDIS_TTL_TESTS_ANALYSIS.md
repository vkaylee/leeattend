# Redis TTL Test Hanging Analysis

## Executive Summary

5 tests in `/home/elt1541/leeattend/apiserver/src/queue/report_queue.rs` are hanging for 60+ seconds because:

1. **Root Cause**: Tests directly call `RedisPool::new()` without checking if Redis is available
2. **Impact**: Attempts to establish connection to `redis://127.0.0.1:6379` which doesn't exist, causing 60s timeout per test
3. **Existing Pattern**: The codebase already has TWO proven patterns for handling Redis-unavailable scenarios
4. **Recommended Fix**: Implement the `setup_redis_or_skip!` macro pattern (proven in `report_race_condition_tests.rs`)

---

## Root Cause Analysis

### Why Tests Hang

**File**: `/home/elt1541/leeattend/apiserver/src/queue/report_queue.rs` (lines 688-1015)

All 5 failing tests follow this pattern:

```rust
#[tokio::test]
async fn test_update_job_metadata_preserves_existing_ttl() {
    // Line 691-692: BLOCKING - Attempts immediate connection
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Line 693-698: HANGS HERE - No timeout, no early exit
    let pool = match RedisPool::new(&redis_url).await {
        Ok(p) => p,
        Err(_) => {
            println!("SKIPPED: Redis not available");
            return;
        }
    };
}
```

**Problem**: `RedisPool::new()` calls `ConnectionManager::new()` which attempts actual TCP connection:

```rust
// From src/queue/redis_client.rs (lines 19-22)
pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
    let client = Client::open(redis_url)?;
    let conn_manager = ConnectionManager::new(client.clone()).await?;  // BLOCKS HERE
    Ok(Self { conn_manager, client })
}
```

When Redis isn't running, this waits for socket timeout (~60 seconds).

### Test Details

| Test Name | Lines | Issue |
|-----------|-------|-------|
| `test_multiple_updates_preserve_ttl` | 845-934 | No timeout wrapping, direct pool creation |
| `test_redis_compatibility_ttl_method` | 1015-1058 | No timeout wrapping, direct pool creation |
| `test_update_job_metadata_applies_default_ttl_when_missing` | 782-842 | No timeout wrapping, direct pool creation |
| `test_update_job_metadata_preserves_existing_ttl` | 689-779 | No timeout wrapping, direct pool creation |
| `test_update_job_status_preserves_ttl` | 937-1012 | No timeout wrapping, direct pool creation |

---

## Existing Mock/Skip Patterns in Codebase

### Pattern 1: Timeout + Health Check + Macro (RECOMMENDED)

**File**: `/home/elt1541/leeattend/apiserver/tests/report_race_condition_tests.rs` (lines 10-50)

This is the BEST pattern - used successfully in production tests:

```rust
// Helper to create test Redis pool or skip tests
async fn get_test_redis() -> Option<RedisPool> {
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Timeout 1: Protect connection attempt (5 seconds instead of 60)
    match tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        RedisPool::new(&redis_url),
    )
    .await
    {
        Ok(Ok(pool)) =>
            // Timeout 2: Verify Redis is working (5 seconds)
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                pool.health_check(),
            )
            .await
            {
                Ok(Ok(())) => Some(pool),
                _ => {
                    eprintln!("WARNING: Redis health check failed. Tests will skip.");
                    None
                }
            },
        _ => {
            eprintln!("WARNING: Redis is not available. Skipping Redis-dependent tests.");
            None
        }
    }
}

// Convenient macro for test usage
macro_rules! setup_redis_or_skip {
    () => {{
        match get_test_redis().await {
            Some(pool) => pool,
            None => {
                println!("SKIPPED: Redis not available");
                return;
            }
        }
    }};
}

// Usage in tests:
#[tokio::test]
async fn test_merge_flag_prevents_double_trigger() {
    let redis_pool = setup_redis_or_skip!();  // Skips if unavailable
    // ... rest of test
}
```

**Advantages**:
- Timeout prevents 60s hangs (reduced to 5s max wait)
- Health check verifies Redis actually works
- Early return without error (clean skip)
- Used in 6+ tests in `report_race_condition_tests.rs` successfully
- Can be reused across test files

---

### Pattern 2: #[ignore] + Expect (Alternative)

**File**: `/home/elt1541/leeattend/apiserver/tests/report_streaming_tests.rs` (multiple tests)

For tests that are explicitly integration tests requiring Redis:

```rust
#[tokio::test]
#[ignore]  // Requires Redis - run with: cargo test -- --ignored
async fn test_publisher_publish_chunk() {
    let pool = create_test_redis_pool().await;
    // Test body
}

// Helper:
async fn create_test_redis_pool() -> RedisPool {
    dotenvy::dotenv().ok();
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    RedisPool::new(&redis_url)
        .await
        .expect("Failed to connect to Redis for testing")
}
```

**Advantages**:
- Explicitly marks tests as requiring external service
- Won't be run by default CI
- Clear intent in code comments

**Disadvantages**:
- Doesn't prevent hangs if accidentally run
- Requires explicit `--ignored` flag

---

## Recommended Fix: Use Pattern 1

### Why Pattern 1 is Better

1. **Still runs in CI/CD** - doesn't require special flags
2. **Prevents hangs** - 5s timeout instead of 60s
3. **Already proven** - used in `report_race_condition_tests.rs`
4. **Reusable** - can apply to any test module
5. **Graceful degradation** - skips safely if Redis unavailable

### Implementation Steps

**Step 1**: Add helper function and macro to test module (or common.rs)

Create at top of `/home/elt1541/leeattend/apiserver/src/queue/report_queue.rs` in test section (before line 551):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{DateRange, ReportConfig, ReportType, TaskType, JobStatus, RedisPool};
    use chrono::NaiveDate;

    // ========== Redis Connection Helper ==========
    /// Helper to create test Redis pool with timeout and health check
    /// Returns None if Redis is unavailable (allowing tests to skip gracefully)
    async fn get_test_redis() -> Option<RedisPool> {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        match tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            RedisPool::new(&redis_url),
        )
        .await
        {
            Ok(Ok(pool)) => match tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                pool.health_check(),
            )
            .await
            {
                Ok(Ok(())) => Some(pool),
                _ => {
                    eprintln!("WARNING: Redis health check failed. Tests will skip.");
                    None
                }
            },
            _ => {
                eprintln!("WARNING: Redis is not available. Skipping Redis-dependent tests.");
                None
            }
        }
    }

    /// Macro for convenient test setup with auto-skip on Redis unavailable
    macro_rules! setup_redis_or_skip {
        () => {{
            match get_test_redis().await {
                Some(pool) => pool,
                None => {
                    println!("SKIPPED: Redis not available");
                    return;
                }
            }
        }};
    }

    // ========== Existing Unit Tests (unchanged) ==========
    #[test]
    fn test_queue_key_constants() {
        // ...existing code...
    }
```

**Step 2**: Replace all 5 failing tests to use the macro

Replace lines 691-699 in each test with:

```rust
// OLD (causes hanging):
let redis_url =
    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
let pool = match RedisPool::new(&redis_url).await {
    Ok(p) => p,
    Err(_) => {
        println!("SKIPPED: Redis not available");
        return;
    }
};

// NEW (safe timeout + skip):
let pool = setup_redis_or_skip!();
```

**Example of updated test**:

```rust
#[tokio::test]
async fn test_update_job_metadata_preserves_existing_ttl() {
    // Use setup_redis_or_skip! instead of direct connection
    let pool = setup_redis_or_skip!();

    let queue = ReportQueue::new(pool.clone());
    let job_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();

    // ... rest of test unchanged ...
}
```

**Apply same fix to all 5 tests**:
- `test_multiple_updates_preserve_ttl` (line 845)
- `test_redis_compatibility_ttl_method` (line 1015)
- `test_update_job_metadata_applies_default_ttl_when_missing` (line 782)
- `test_update_job_metadata_preserves_existing_ttl` (line 689)
- `test_update_job_status_preserves_ttl` (line 937)

---

## Alternative Option: Pattern 2 (If Tests Should Be Integration-Only)

If these tests are specifically integration tests and should ONLY run when Redis is available:

```rust
#[tokio::test]
#[ignore]  // Requires Redis - run with: cargo test -- --ignored
async fn test_update_job_metadata_preserves_existing_ttl() {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let pool = RedisPool::new(&redis_url)
        .await
        .expect("Failed to connect to Redis for testing");

    // ... rest of test ...
}
```

**This approach**:
- Requires: `cargo test -- --ignored` to run these tests
- Skips during normal CI/CD
- Suitable if Redis is considered an external integration dependency

**Trade-off**: These tests won't run in normal test suite, only in dedicated integration test runs.

---

## Implementation Checklist

Choose ONE approach:

### Option A: Pattern 1 (Recommended - Graceful Skip)
- [ ] Add `get_test_redis()` helper function
- [ ] Add `setup_redis_or_skip!` macro
- [ ] Replace redis connection in 5 tests with macro call
- [ ] Run tests to verify skip behavior: `cargo test -p apiserver test_update_job_metadata`
- [ ] Verify no 60s hangs occur

### Option B: Pattern 2 (Integration-Only)
- [ ] Add `#[ignore]` to 5 tests
- [ ] Keep existing redis connection code
- [ ] Run tests to verify they skip: `cargo test -p apiserver`
- [ ] Verify they run: `cargo test -p apiserver -- --ignored`

---

## Verification Commands

```bash
# Test Pattern 1 implementation (should skip cleanly):
cargo test -p apiserver test_update_job_metadata --lib

# Test Pattern 2 implementation (should skip without --ignored):
cargo test -p apiserver test_update_job_metadata

# Run integration tests if using Pattern 2:
cargo test -p apiserver test_update_job_metadata -- --ignored
```

---

## Related Files

- **Test file**: `/home/elt1541/leeattend/apiserver/src/queue/report_queue.rs` (lines 688-1059)
- **Redis client**: `/home/elt1541/leeattend/apiserver/src/queue/redis_client.rs`
- **Reference implementation**: `/home/elt1541/leeattend/apiserver/tests/report_race_condition_tests.rs` (lines 10-50)
- **Alternative pattern**: `/home/elt1541/leeattend/apiserver/tests/report_streaming_tests.rs` (multiple tests)

---

## Summary

**Root Cause**: Tests directly call `RedisPool::new()` without timeouts or health checks, causing 60s hangs when Redis unavailable.

**Solution**: Use proven `setup_redis_or_skip!` macro pattern from `report_race_condition_tests.rs` to:
1. Wrap connection in 5s timeout
2. Verify Redis with health check
3. Skip tests gracefully if unavailable

**Impact**: Reduces test suite hang time from 300s (5 × 60s) to <25s (5 × 5s timeout).
