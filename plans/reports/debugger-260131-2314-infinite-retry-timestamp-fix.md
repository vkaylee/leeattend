# Infinite Sync Retry Loop - Root Cause Analysis & Fix

**Date:** 2026-01-31
**Issue:** Agent keeps re-sending logs infinitely despite server ACK success
**Status:** ✅ **RESOLVED**

---

## Executive Summary

**Root Cause:** SQLite timestamp type mismatch in UPDATE query
**Impact:** 0 rows updated → logs never marked as synced → infinite retry loop
**Fix:** Convert NaiveDateTime to SQLite-compatible string format
**Result:** UPDATE queries now match correctly, retry loop eliminated

---

## Root Cause Deep Dive

### The Type Mismatch

1. **SQLite Storage Format**
   ```sql
   timestamp DATETIME NOT NULL  -- Stored as TEXT: "2026-01-31 10:00:00"
   ```

2. **Composite Key Transmission**
   ```rust
   // Sent to server as Unix timestamp
   format!("{}:{}:{}", device_id, user_id, timestamp.and_utc().timestamp())
   // Example: "Main Gate:101:1738314000"
   ```

3. **ACK Processing (BROKEN)**
   ```rust
   // Old code - returned NaiveDateTime
   fn parse_composite_key(key: &str) -> Option<(String, String, NaiveDateTime)> {
       let timestamp = chrono::DateTime::from_timestamp(timestamp_unix, 0)?.naive_utc();
       Some((device_id, user_id, timestamp))  // ❌ NaiveDateTime object
   }

   // UPDATE query
   sqlx::query("UPDATE ... WHERE timestamp = ?")
       .bind(timestamp)  // ❌ NaiveDateTime != "2026-01-31 10:00:00"
   ```

4. **The Silent Failure**
   ```rust
   let result = execute(...).await;
   // result.rows_affected() == 0  ❌ No match!
   // But code didn't check this, assumed success
   ```

### Why It Caused Infinite Retry

```
1. Log inserted: is_synced=0
2. Sent to server → marked in-flight: is_synced=2
3. Server ACKs → UPDATE WHERE timestamp=NaiveDateTime
4. 0 rows affected (no match) → is_synced stays 2
5. Watchdog resets in-flight logs → is_synced=0
6. Retry from step 2 → INFINITE LOOP
```

---

## The Fix

### 1. Changed parse_composite_key Return Type

```rust
// OLD (BROKEN)
fn parse_composite_key(key: &str) -> Option<(String, String, NaiveDateTime)>

// NEW (FIXED)
fn parse_composite_key(key: &str) -> Option<(String, String, String)>
```

### 2. Format Timestamp to SQLite Format

```rust
fn parse_composite_key(key: &str) -> Option<(String, String, String)> {
    let mut parts = key.rsplitn(3, ':');
    let timestamp_str = parts.next()?.trim();  // Trim whitespace
    let user_id = parts.next()?.trim();
    let device_id = parts.next()?.trim();

    // Parse Unix timestamp
    let timestamp_unix = timestamp_str.parse::<i64>().ok()?;
    let datetime = chrono::DateTime::from_timestamp(timestamp_unix, 0)?.naive_utc();

    // CRITICAL: Format to SQLite DATETIME format (matches DB storage)
    let timestamp_formatted = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

    Some((device_id.to_string(), user_id.to_string(), timestamp_formatted))
}
```

### 3. Added Comprehensive Logging

```rust
if let Some((device_id, user_id, timestamp_str)) = parse_composite_key(&msg_id) {
    log::info!("🔍 ACK Success: device='{}', user='{}', timestamp='{}'",
        device_id, user_id, timestamp_str);

    match sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 1
                       WHERE device_id = ? AND user_id = ? AND timestamp = ?")
        .bind(&device_id)
        .bind(&user_id)
        .bind(&timestamp_str)  // ✅ Now a String: "2026-01-31 10:00:00"
        .execute(...)
        .await
    {
        Ok(result) => {
            let rows = result.rows_affected();
            if rows == 0 {
                log::error!("❌ UPDATE affected 0 rows! device='{}', user='{}', timestamp='{}' - Log may not exist in DB or already synced",
                    device_id, user_id, timestamp_str);
            } else {
                log::debug!("✅ Marked {} row(s) as synced in DB", rows);
            }
        }
        Err(e) => {
            log::error!("❌ UPDATE query failed: {}", e);
        }
    }
} else {
    log::warn!("⚠️ Failed to parse composite key: {}", msg_id);
}
```

---

## How to Monitor the Fix

### Success Indicators

Watch for these log patterns:

```
✅ Normal operation:
🔍 ACK Success: device='Main Gate', user='101', timestamp='2026-01-31 10:00:00'
✅ Marked 1 row(s) as synced in DB

✅ Already synced (expected):
🔍 ACK Success: device='Main Gate', user='101', timestamp='2026-01-31 10:00:00'
❌ UPDATE affected 0 rows! ... - Log may not exist in DB or already synced
```

### Error Patterns to Watch

```
⚠️ Parse failure (should never happen):
⚠️ Failed to parse composite key: Main Gate:101:1738314000

❌ Database error:
❌ UPDATE query failed: database locked

❌ Persistent 0 rows (indicates OTHER issue):
❌ UPDATE affected 0 rows! device='Main Gate', user='101', timestamp='2026-01-31 10:00:00'
(If this repeats for same log, investigate DB schema or data corruption)
```

---

## Verification Steps

### 1. Check Production Logs

```bash
# Look for successful updates
grep "✅ Marked.*row.*synced" /var/log/agent.log

# Check for 0-row failures
grep "❌ UPDATE affected 0 rows" /var/log/agent.log

# Verify no infinite retries
grep "🔍 ACK Success.*Main Gate:101" /var/log/agent.log | uniq -c
# Should see each log ACKed once, not repeatedly
```

### 2. Query SQLite Directly

```sql
-- Check is_synced status distribution
SELECT is_synced, COUNT(*) FROM zkt_raw_attendance GROUP BY is_synced;
-- Expected: is_synced=0 (unsynced), is_synced=1 (synced)
-- NOT expected: is_synced=2 (stuck in-flight)

-- Find logs stuck in-flight for >5 minutes
SELECT * FROM zkt_raw_attendance
WHERE is_synced = 2
AND created_at < datetime('now', '-5 minutes');
-- Should return 0 rows (or very few during active sync)
```

### 3. Server-Side Verification

```sql
-- Postgres: Check for duplicate inserts (indicates retries)
SELECT device_id, user_id, timestamp, COUNT(*)
FROM attendance_logs
GROUP BY device_id, user_id, timestamp
HAVING COUNT(*) > 1;
-- Should return 0 rows (ON CONFLICT DO NOTHING prevents duplicates)
```

---

## Additional Fixes Applied

### Whitespace Handling

Added `.trim()` to all parsed components to handle hidden whitespace:

```rust
let timestamp_str = parts.next()?.trim();
let user_id = parts.next()?.trim();
let device_id = parts.next()?.trim();
```

This prevents issues like:
- `" 101"` vs `"101"` (leading space)
- `"Main Gate "` vs `"Main Gate"` (trailing space)

---

## Test Coverage

All 5 composite key parsing tests passing:

1. ✅ `test_parse_composite_key_normal` - Standard format
2. ✅ `test_parse_composite_key_with_colon_in_device_id` - "Office:Main:Gate"
3. ✅ `test_parse_composite_key_single_colon_device` - "Office:Gate"
4. ✅ `test_parse_composite_key_invalid_timestamp` - Error handling
5. ✅ `test_parse_composite_key_missing_parts` - Malformed input

Timestamp format verified:
```rust
assert_eq!(timestamp_str, "2025-01-31 09:00:00");  // SQLite format
```

---

## Performance Impact

**Before Fix:**
- Infinite retry loop
- Server receives same log hundreds of times
- Network bandwidth wasted
- Database write pressure (ON CONFLICT prevents dupes)

**After Fix:**
- Single transmission per log
- Immediate ACK processing
- Clean `is_synced=1` state
- No retry overhead

---

## Conclusion

The infinite retry loop was caused by a subtle type mismatch between how timestamps are stored in SQLite (TEXT: "2026-01-31 10:00:00") and how they were being queried (NaiveDateTime object). SQLx serialized the NaiveDateTime differently, causing the WHERE clause to never match, resulting in 0 rows updated and the log remaining unsynced.

The fix ensures the timestamp is formatted to the exact string format SQLite uses, guaranteeing successful UPDATE queries and eliminating the retry loop.

**Status:** ✅ Production-ready, all tests passing

---

## Commit Reference

```
commit 6208b78
fix(agent): resolve infinite retry loop caused by timestamp type mismatch
```
