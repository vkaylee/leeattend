# Redis TTL Tests: Exact Code Changes

## File: `/home/elt1541/leeattend/apiserver/src/queue/report_queue.rs`

### Change 1: Add Helper + Macro (After line 555)

**Location**: After `mod tests {` and imports (line 555)

**Add this code block**:

```rust
    // ========== Helper for Redis Connection with Timeout ==========
    /// Helper to create test Redis pool or skip if unavailable
    /// Uses timeout to prevent 60+ second hangs when Redis is down
    async fn get_test_redis() -> Option<RedisPool> {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        // Timeout protection: 5 seconds to connect instead of 60+
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            RedisPool::new(&redis_url),
        )
        .await
        {
            Ok(Ok(pool)) => {
                // Verify Redis is actually working with health check
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
                }
            }
            _ => {
                eprintln!("WARNING: Redis is not available. Skipping Redis-dependent tests.");
                None
            }
        }
    }

    /// Convenience macro for test setup: returns pool or skips test
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

    // ========== Unit Tests (Key Format, Serialization, etc.) ==========
```

---

### Change 2: Update `test_update_job_metadata_preserves_existing_ttl` (Line 689)

**Before** (Lines 691-699):
```rust
    #[tokio::test]
    async fn test_update_job_metadata_preserves_existing_ttl() {
        // Test: When updating metadata with existing TTL, preserve that TTL
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let pool = match RedisPool::new(&redis_url).await {
            Ok(p) => p,
            Err(_) => {
                println!("SKIPPED: Redis not available");
                return;
            }
        };
```

**After**:
```rust
    #[tokio::test]
    async fn test_update_job_metadata_preserves_existing_ttl() {
        // Test: When updating metadata with existing TTL, preserve that TTL
        let pool = setup_redis_or_skip!();
```

---

### Change 3: Update `test_update_job_metadata_applies_default_ttl_when_missing` (Line 782)

**Before** (Lines 784-792):
```rust
    #[tokio::test]
    async fn test_update_job_metadata_applies_default_ttl_when_missing() {
        // Test: When updating metadata without TTL (new key), apply default 24h TTL
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let pool = match RedisPool::new(&redis_url).await {
            Ok(p) => p,
            Err(_) => {
                println!("SKIPPED: Redis not available");
                return;
            }
        };
```

**After**:
```rust
    #[tokio::test]
    async fn test_update_job_metadata_applies_default_ttl_when_missing() {
        // Test: When updating metadata without TTL (new key), apply default 24h TTL
        let pool = setup_redis_or_skip!();
```

---

### Change 4: Update `test_multiple_updates_preserve_ttl` (Line 845)

**Before** (Lines 847-855):
```rust
    #[tokio::test]
    async fn test_multiple_updates_preserve_ttl() {
        // Test: Multiple updates should preserve and not reset TTL
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let pool = match RedisPool::new(&redis_url).await {
            Ok(p) => p,
            Err(_) => {
                println!("SKIPPED: Redis not available");
                return;
            }
        };
```

**After**:
```rust
    #[tokio::test]
    async fn test_multiple_updates_preserve_ttl() {
        // Test: Multiple updates should preserve and not reset TTL
        let pool = setup_redis_or_skip!();
```

---

### Change 5: Update `test_update_job_status_preserves_ttl` (Line 937)

**Before** (Lines 939-947):
```rust
    #[tokio::test]
    async fn test_update_job_status_preserves_ttl() {
        // Test: update_job_status (which calls update_job_metadata) preserves TTL
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let pool = match RedisPool::new(&redis_url).await {
            Ok(p) => p,
            Err(_) => {
                println!("SKIPPED: Redis not available");
                return;
            }
        };
```

**After**:
```rust
    #[tokio::test]
    async fn test_update_job_status_preserves_ttl() {
        // Test: update_job_status (which calls update_job_metadata) preserves TTL
        let pool = setup_redis_or_skip!();
```

---

### Change 6: Update `test_redis_compatibility_ttl_method` (Line 1015)

**Before** (Lines 1017-1025):
```rust
    #[tokio::test]
    async fn test_redis_compatibility_ttl_method() {
        // Test: Verify TTL command works (Redis 3.0+ compatible)
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let pool = match RedisPool::new(&redis_url).await {
            Ok(p) => p,
            Err(_) => {
                println!("SKIPPED: Redis not available");
                return;
            }
        };
```

**After**:
```rust
    #[tokio::test]
    async fn test_redis_compatibility_ttl_method() {
        // Test: Verify TTL command works (Redis 3.0+ compatible)
        let pool = setup_redis_or_skip!();
```

---

## Summary of Changes

| Test | Lines | Change Type | Details |
|------|-------|------------|---------|
| test_update_job_metadata_preserves_existing_ttl | 691-699 | Replace | ~9 lines → 1 line |
| test_update_job_metadata_applies_default_ttl_when_missing | 784-792 | Replace | ~9 lines → 1 line |
| test_multiple_updates_preserve_ttl | 847-855 | Replace | ~9 lines → 1 line |
| test_update_job_status_preserves_ttl | 939-947 | Replace | ~9 lines → 1 line |
| test_redis_compatibility_ttl_method | 1017-1025 | Replace | ~9 lines → 1 line |

## Total Impact

- **Lines Added**: ~50 (helper + macro, added once)
- **Lines Removed**: ~45 (9 lines × 5 tests)
- **Net Change**: +5 lines (one-time overhead)
- **Tests Affected**: 5
- **Time Saved**: 275+ seconds per test run (60s → 5s × 5 tests)

## Verification Script

```bash
#!/bin/bash
echo "Testing Redis TTL fixes..."

# Run just these 5 tests to verify fix
cargo test -p apiserver \
  test_update_job_metadata_preserves_existing_ttl \
  test_update_job_metadata_applies_default_ttl_when_missing \
  test_multiple_updates_preserve_ttl \
  test_update_job_status_preserves_ttl \
  test_redis_compatibility_ttl_method \
  --lib

echo ""
echo "Expected behavior:"
echo "✅ If Redis running: All 5 tests PASS"
echo "✅ If Redis down: All 5 tests SKIP with message 'SKIPPED: Redis not available'"
echo "❌ No 60-second hangs!"
```

## Files Referenced

- **Main test file**: `/home/elt1541/leeattend/apiserver/src/queue/report_queue.rs`
- **Reference implementation**: `/home/elt1541/leeattend/apiserver/tests/report_race_condition_tests.rs` (lines 10-50)
- **Redis client**: `/home/elt1541/leeattend/apiserver/src/queue/redis_client.rs`

## Why This Fix Works

1. **Timeout Wrapper**: Prevents infinite wait (60s → 5s)
2. **Health Check**: Verifies Redis is actually working
3. **Early Return**: Tests skip gracefully, no error
4. **Proven Pattern**: Already used successfully in 6+ tests
5. **Reusable**: Macro can be applied to other test modules

## Next Steps

1. Apply all 6 changes to `/home/elt1541/leeattend/apiserver/src/queue/report_queue.rs`
2. Run tests: `cargo test -p apiserver --lib report_queue`
3. Verify no hangs: Tests complete in < 30 seconds
4. Commit with message: "fix: add timeout protection for Redis tests to prevent 60s hangs"
