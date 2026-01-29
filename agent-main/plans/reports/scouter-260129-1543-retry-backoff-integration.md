# Scout Report: Retry Mechanism for Stateless Exponential Backoff

## Exploration Scope
Target: Retry logic for implementing stateless exponential backoff
Boundaries: src/memory_store.rs, src/central_client.rs, src/database.rs, tests/retry_limit_test.rs

## Current Retry Infrastructure

### 1. MemoryLog Structure (memory_store.rs:11-24)
```rust
pub struct MemoryLog {
    pub id: i64,
    pub device_id: String,
    pub user_id: String,
    pub timestamp: NaiveDateTime,
    pub status: Option<i32>,
    pub punch: Option<i32>,
    pub is_synced: i64,           // 0=unsynced, 1=synced, 2=in_flight
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,  // Set on mark_in_flight
    pub retry_count: i64,                   // Incremented on mark_in_flight
    pub persisted: bool,
}
```

### 2. Current get_unsynced() (memory_store.rs:198-210)
**Location:** memory_store.rs:198-210
**Current Logic:**
- Returns logs where is_synced == 0 (unsynced)
- No time-based filtering
- FIFO order by ID (insertion order)
- Returns Vec<MemoryLog>

**Key Code:**
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

### 3. Retry Count Tracking

**Incremented at:** memory_store.rs:236
```rust
pub fn mark_in_flight(&self, ids: &[i64]) {
    for &id in ids {
        if let Some(mut log) = self.logs.get_mut(&id) {
            log.is_synced = 2;
            log.retry_count += 1;                         // ← Incremented here
            log.updated_at = Some(chrono::Utc::now().naive_utc());  // ← Set here
        }
        // ...
    }
}
```

**Persisted:** Yes, in database.rs:294
```rust
pub async fn mark_in_flight(&self, ids: &[i64]) -> Result<(), Error> {
    // Update memory store first
    self.memory_store.mark_in_flight(ids);
    
    // Also update disk
    let query = format!(
        "UPDATE zkt_raw_attendance 
        SET is_synced = 2, 
            updated_at = CURRENT_TIMESTAMP, 
            retry_count = retry_count + 1    // ← Persisted to disk
        WHERE id IN ({})",
        placeholders.join(",")
    );
}
```

**Current Max:** 10 (configurable via agent_config)

### 4. Updated_at Tracking

**Set at:** 
- memory_store.rs:237 (on mark_in_flight)
- database.rs:294 (persisted on mark_in_flight)

**Persisted:** Yes, CURRENT_TIMESTAMP written to disk

**Type:** NaiveDateTime (UTC)

**Current State:**
- Set to current time when log marked in-flight
- Never updated again until next retry
- Used for TTL expiration check in DLQ cleanup

### 5. Sync Loop (central_client.rs:64-155)

**Polling Interval:** Adaptive (50-500ms)
```rust
let sleep_duration = if count == 0 {
    Duration::from_millis(500)  // Idle: check every 500ms
} else if unsynced_count > 1000 {
    Duration::from_millis(50)   // Backlogged: drain fast
} else if unsynced_count > 100 {
    Duration::from_millis(100)  // Moderate
} else {
    Duration::from_millis(200)  // Normal
};
```

**Calls get_unsynced():** Every cycle (line 99)
```rust
let logs = sync_hybrid.get_unsynced_logs(limit);  // ← Calls memory store
```

**Marks in-flight:** Line 116
```rust
let log_ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
sync_hybrid.mark_in_flight(&log_ids);  // ← Increments retry_count, sets updated_at
```

**Key Behavior:**
- Continuous polling loop
- No sleep before retry attempts
- All unsynced logs eligible for immediate retry

### 6. ACK Handler (central_client.rs:268-282)

**Success Path:** Line 271-273
```rust
if status.as_deref() == Some("success") {
    hybrid_db.mark_synced(&[id]);                    // ← Memory: is_synced = 1
    let _ = hybrid_db.disk_db().mark_synced(&[id]).await;  // ← Disk: is_synced = 1
}
```

**Failure Path:** Line 275-280
```rust
else {
    hybrid_db.reset_to_unsynced(id);  // ← Memory: is_synced = 0, back to unsynced_index
    let _ = sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE id = ?")
        .bind(id)
        .execute(hybrid_db.disk_db().get_pool())
        .await;
}
```

**Critical:** reset_to_unsynced() does NOT increment retry_count or update updated_at
**Result:** retry_count and updated_at remain from last mark_in_flight() call

### 7. DLQ Cleanup (central_client.rs:71-85)

**Runs:** Every sync loop cycle
**Criteria:** 
- retry_count >= max_retries (default: 10)
- OR (updated_at NOT NULL AND age > ttl_minutes)

**Code:**
```rust
let max_retries = match sync_hybrid.disk_db().get_agent_config("max_retry_count").await {
    Ok(Some(val)) => val.parse::<i64>().unwrap_or(10),
    _ => 10,
};
let ttl_minutes = match sync_hybrid.disk_db().get_agent_config("retry_ttl_minutes").await {
    Ok(Some(val)) => val.parse::<i64>().unwrap_or(60),
    _ => 60,
};

if let Ok(moved) = sync_hybrid.disk_db().move_expired_logs_to_dlq(max_retries, ttl_minutes).await {
    if moved > 0 {
        warn!("⚠️ Moved {} expired logs to dead letter queue", moved);
    }
}
```

## Integration Points for Exponential Backoff

### PRIMARY: Modify get_unsynced() (memory_store.rs:198-210)

**Current:** Returns all unsynced (is_synced = 0) without time filtering

**Required Change:** Add time-based filter using retry_count and updated_at

**Pseudo-code:**
```rust
pub fn get_unsynced(&self, limit: usize) -> Vec<MemoryLog> {
    let unsynced = self.unsynced_index.read();
    let mut result = Vec::with_capacity(limit);
    let now = chrono::Utc::now().naive_utc();

    for id in unsynced.iter() {
        if let Some(log) = self.logs.get(id) {
            // New logic: Check if enough time has passed since last attempt
            if let Some(last_attempt) = log.updated_at {
                let retry_delay = calculate_exponential_backoff(log.retry_count);
                let next_retry_time = last_attempt + retry_delay;
                
                if now < next_retry_time {
                    continue;  // Skip this log (backoff not elapsed)
                }
            }
            // First attempt (updated_at is None) or backoff elapsed
            result.push(log.clone());
            if result.len() >= limit {
                break;
            }
        }
    }
    result
}
```

**Backoff Formula (stateless):**
```rust
fn calculate_exponential_backoff(retry_count: i64) -> chrono::Duration {
    // Base: 1s, 2s, 4s, 8s, 16s, 32s, 60s (max)
    let base_secs = std::cmp::min(2u64.pow(retry_count as u32), 60);
    chrono::Duration::seconds(base_secs as i64)
}
```

### SECONDARY: Verify mark_in_flight() (memory_store.rs:228-244)

**Current State:** CORRECT
- Sets updated_at = current time
- Increments retry_count
- Both persisted to disk

**No changes needed** - existing implementation already provides required fields

### TERTIARY: Test Coverage (tests/retry_limit_test.rs)

**Existing Tests:**
1. test_retry_count_increments (line 6): Verifies retry_count increments
2. test_max_retry_limit_moves_to_dlq (line 46): Verifies DLQ on max retries
3. test_ttl_expiration_moves_to_dlq (line 102): Verifies TTL expiration
4. test_valid_retries_not_moved_to_dlq (line 159): Verifies logs within limits stay
5. test_batch_dlq_move (line 203): Verifies batch DLQ
6. test_dlq_preserves_metadata (line 255): Verifies metadata preservation
7. test_configurable_retry_limits (line 304): Verifies config

**New Test Needed:**
```rust
#[tokio::test]
async fn test_exponential_backoff_timing() {
    // Insert log
    // Mark in-flight (retry_count = 1, updated_at = now)
    // Reset to unsynced
    // Verify get_unsynced() returns empty (backoff not elapsed)
    // Sleep for backoff duration
    // Verify get_unsynced() returns log (backoff elapsed)
}
```

## Performance Analysis

### Current get_unsynced() Performance
- O(n) iteration over unsynced_index (BTreeSet)
- Memory latency target: <5ms (line 119-121)
- No additional queries or disk I/O

### Exponential Backoff Impact
**Added Operations per log:**
1. Check if updated_at exists (branch)
2. Calculate backoff duration (arithmetic: 1 subtraction, 1 min, 1 pow)
3. Add duration to NaiveDateTime (chrono library call)
4. Compare now < next_retry_time (comparison)

**Estimated overhead:** <1µs per log (arithmetic + branch)
**Risk:** NEGLIGIBLE - still well within 5ms latency target

### Backpressure Considerations
**Scenario:** 10,000 logs with high retry_count all backoff
- Sync loop calls get_unsynced(500) every 50-500ms
- All logs skipped due to backoff
- Returns empty Vec
- Sync loop sleeps for 500ms (idle mode)

**Result:** Graceful degradation, no CPU thrashing

## Warnings

⚠️ reset_to_unsynced() (memory_store.rs:261-272) does NOT update updated_at
- This is CORRECT behavior for backoff
- updated_at represents "last attempt time"
- Reset only changes is_synced flag

⚠️ Backoff bypassed on Agent restart
- Memory store reloaded from disk (memory_store.rs:80-113)
- All logs with is_synced = 0 added to unsynced_index
- get_unsynced() will respect updated_at from disk
- Backoff timing preserved across restarts

⚠️ Clock skew risk
- All timestamps use Utc::now().naive_utc()
- Stateless calculation immune to clock adjustments during runtime
- Risk limited to Agent restart with system time change

⚠️ DLQ interaction
- Backoff delays retry attempts
- TTL expiration still applies (based on updated_at age)
- Logs may hit TTL before exhausting max_retries if backoff too long
- Current config: max_retries=10, ttl=60min
- Max backoff delay: 60s → 10 retries = ~10 minutes
- Safe: Well below 60min TTL

## Existing Test Files
- tests/retry_limit_test.rs (346 lines, comprehensive retry/DLQ tests)
- tests/sync_log_classification_test.rs (retry classification tests)
- tests/startup_config_clear_test.rs (config tests)
- tests/push_logs_integration_test.rs (integration tests)

## Summary

**Ready for Implementation:**
- Retry tracking infrastructure complete (retry_count, updated_at)
- Persistence layer working (memory + disk)
- DLQ cleanup handles edge cases
- Test coverage comprehensive

**Single Integration Point:**
- Modify memory_store.rs::get_unsynced() lines 198-210
- Add time-based filtering using existing fields
- No changes needed to mark_in_flight() or reset_to_unsynced()

**Implementation Complexity:** LOW
**Estimated LOC:** +15 lines (backoff calculation + filtering logic)
**Risk:** MINIMAL (no breaking changes, graceful degradation)

**Next Steps:**
1. Implement calculate_exponential_backoff() helper function
2. Add time-based filtering to get_unsynced()
3. Add unit test for backoff timing
4. Verify memory latency remains <5ms
5. Test cross-restart backoff preservation
