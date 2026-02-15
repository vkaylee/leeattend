# Redis TTL Tests: Quick Fix Guide

## Problem (60-second hangs)

```
5 tests hang waiting for Redis connection:
❌ test_multiple_updates_preserve_ttl        (60s+)
❌ test_redis_compatibility_ttl_method       (60s+)
❌ test_update_job_metadata_applies_default_ttl_when_missing (60s+)
❌ test_update_job_metadata_preserves_existing_ttl (60s+)
❌ test_update_job_status_preserves_ttl      (60s+)

Total suite timeout: 300+ seconds
```

## Root Cause

```rust
// Current code (BLOCKS FOR 60 SECONDS):
let redis_url = std::env::var("REDIS_URL")
    .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
let pool = match RedisPool::new(&redis_url).await {  // <- HANGS HERE (60s)
    Ok(p) => p,
    Err(_) => {
        println!("SKIPPED: Redis not available");
        return;
    }
};
```

No timeout protection = TCP socket timeout (~60 seconds) when Redis unavailable.

## Solution: Copy Proven Pattern

Proven pattern exists in `/home/elt1541/leeattend/apiserver/tests/report_race_condition_tests.rs`:

### Step 1: Add Helper + Macro

Add this to test module (top of `#[cfg(test)]` section in report_queue.rs):

```rust
#[cfg(test)]
mod tests {
    // ... existing imports ...

    // ADD THIS SECTION:
    async fn get_test_redis() -> Option<RedisPool> {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        match tokio::time::timeout(
            tokio::time::Duration::from_secs(5),  // 5 sec timeout
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
                _ => None,
            },
            _ => None,
        }
    }

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

    // ... rest of existing tests ...
}
```

### Step 2: Replace in All 5 Tests

Change from:
```rust
#[tokio::test]
async fn test_update_job_metadata_preserves_existing_ttl() {
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let pool = match RedisPool::new(&redis_url).await {
        Ok(p) => p,
        Err(_) => {
            println!("SKIPPED: Redis not available");
            return;
        }
    };
```

To:
```rust
#[tokio::test]
async fn test_update_job_metadata_preserves_existing_ttl() {
    let pool = setup_redis_or_skip!();
```

**Apply same change to**:
- test_multiple_updates_preserve_ttl
- test_redis_compatibility_ttl_method
- test_update_job_metadata_applies_default_ttl_when_missing
- test_update_job_status_preserves_ttl

## Result

```
✅ test_multiple_updates_preserve_ttl        (5s timeout or skip)
✅ test_redis_compatibility_ttl_method       (5s timeout or skip)
✅ test_update_job_metadata_applies_default_ttl_when_missing (5s timeout or skip)
✅ test_update_job_metadata_preserves_existing_ttl (5s timeout or skip)
✅ test_update_job_status_preserves_ttl      (5s timeout or skip)

Total suite: ~25 seconds max (instead of 300+)
```

## Why This Works

| Issue | Solution |
|-------|----------|
| 60s socket timeout | Wrap in 5s tokio::time::timeout |
| Hangs in CI/CD | Early return (skip) if unavailable |
| No error messages | Clear "SKIPPED" output |
| Tests break CI | Tests now skip gracefully |

## Benefits

1. **Proven**: Used successfully in report_race_condition_tests.rs (6+ tests)
2. **Fast**: 5s timeout vs 60s
3. **Safe**: Skips gracefully, no error
4. **Reusable**: Can apply to other modules
5. **CI-friendly**: Works in unit test suite

## Time Saved

- Before: 300+ seconds (5 tests × 60s hangs)
- After: ~25 seconds (5 tests × 5s timeout or skip)
- **Improvement: 92% faster** ⚡

---

For detailed analysis, see: `/home/elt1541/leeattend/REDIS_TTL_TESTS_ANALYSIS.md`
