# Phase 4: Database Logic Changes

## Context Links
- Plan: [plan.md](./plan.md)
- Database: `agent-main/src/database.rs`
- Memory Store: `agent-main/src/memory_store.rs`

## Overview
- **Priority**: P1
- **Status**: pending
- **Effort**: 1.5h
- **Depends on**: Phase 3

Update database operations to handle timezone-aware timestamp conversion.

## Key Insights

1. **Timezone Lookup Flow**: Before saving attendance, query device timezone from DB
2. **Conversion Chain**: Local String -> Parse with Timezone -> Convert to UTC -> Extract Unix timestamp
3. **Fallback Strategy**: Default to UTC if device not found or timezone invalid
4. **Memory Store**: Also needs timestamp conversion during insert

## Requirements

### Functional
- `save_raw_attendance`: Accept device_id, lookup timezone, convert local time to UTC i64
- `upsert_device`: Accept and save timezone column
- `get_device_timezone`: New function to query device timezone
- Memory store insert functions: Accept device timezone parameter

### Non-functional
- Timezone lookup should be cached or batched for performance
- Graceful fallback to UTC on any parsing error

## Architecture

### Conversion Flow
```
save_raw_attendance(device_id, AttendanceData)
    |
    +-> Query devices table for timezone
    |   (SELECT timezone FROM devices WHERE id = ?)
    |
    +-> Parse timezone string to Tz
    |   ("Asia/Ho_Chi_Minh".parse::<Tz>().unwrap_or(Tz::UTC))
    |
    +-> Parse local time string
    |   (NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%d %H:%M:%S"))
    |
    +-> Convert to timezone-aware datetime
    |   (tz.from_local_datetime(&naive).single().unwrap_or(...))
    |
    +-> Convert to UTC and extract Unix timestamp
    |   (dt.with_timezone(&Utc).timestamp())
    |
    +-> INSERT INTO zkt_raw_attendance (..., timestamp INTEGER)
```

## Related Code Files

### Modify
- `agent-main/src/database.rs`
- `agent-main/src/memory_store.rs`
- `agent-main/src/hybrid_database.rs`

## Implementation Steps

### Step 1: Add chrono-tz import to database.rs

At top of file, add:
```rust
use chrono_tz::Tz;
```

### Step 2: Add get_device_timezone() function

Add new function in Database impl:
```rust
/// Get timezone for a device, defaults to UTC if not found
pub async fn get_device_timezone(&self, device_id: &str) -> String {
    let result: Option<(String,)> = sqlx::query_as(
        "SELECT timezone FROM devices WHERE id = ? AND is_active = 1"
    )
    .bind(device_id)
    .fetch_optional(&self.pool)
    .await
    .ok()
    .flatten();

    result.map(|r| r.0).unwrap_or_else(|| "UTC".to_string())
}
```

### Step 3: Create timestamp conversion helper

Add helper function in database.rs:
```rust
/// Convert local time string to UTC Unix timestamp using device timezone
fn convert_local_to_unix_timestamp(
    timestamp_str: &str,
    timezone_str: &str,
) -> Result<i64, String> {
    // Parse timezone (default to UTC if invalid)
    let tz: Tz = timezone_str.parse().unwrap_or(Tz::UTC);

    // Parse local time string
    let naive = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%SZ"))
        .map_err(|e| format!("Invalid timestamp format: {}", e))?;

    // Convert to timezone-aware, handling ambiguous times
    let local_dt = tz.from_local_datetime(&naive)
        .single()  // Rejects ambiguous DST transitions
        .or_else(|| tz.from_local_datetime(&naive).earliest())  // Fallback to earliest
        .ok_or_else(|| "Ambiguous or invalid local time".to_string())?;

    // Convert to UTC and get Unix timestamp
    Ok(local_dt.with_timezone(&chrono::Utc).timestamp())
}
```

### Step 4: Update save_raw_attendance()

Replace current implementation:
```rust
pub async fn save_raw_attendance(
    &self,
    device_id: &str,
    data: AttendanceData,
) -> Result<i64, Error> {
    // Get device timezone
    let timezone = self.get_device_timezone(device_id).await;

    // Convert local time to Unix timestamp
    let timestamp = match convert_local_to_unix_timestamp(&data.timestamp, &timezone) {
        Ok(ts) => ts,
        Err(e) => {
            let raw_json = serde_json::to_string(&data).unwrap_or_default();
            let error_msg = format!("Timestamp conversion error (tz={}): {}", timezone, e);

            sqlx::query(
                r#"
                INSERT INTO parsing_errors (device_id, raw_data, error_msg, created_at)
                VALUES (?, ?, ?, CURRENT_TIMESTAMP)
                "#
            )
            .bind(device_id)
            .bind(raw_json)
            .bind(error_msg)
            .execute(&self.pool)
            .await?;

            return Ok(0);
        }
    };

    // Write to disk first
    let res = sqlx::query(
        r#"
        INSERT INTO zkt_raw_attendance (device_id, user_id, timestamp, status, punch)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(device_id, user_id, timestamp) DO NOTHING
        "#,
    )
    .bind(device_id)
    .bind(&data.user_id)
    .bind(timestamp)  // Now i64
    .bind(data.status)
    .bind(data.punch)
    .execute(&self.pool)
    .await?;

    let id = res.last_insert_rowid();

    // If inserted (not duplicate), also insert to memory store
    if res.rows_affected() > 0 {
        let _ = self.memory_store.insert_with_timestamp(
            device_id.to_string(),
            data,
            timestamp,  // Pass pre-computed timestamp
        );
    }

    Ok(id)
}
```

### Step 5: Update upsert_device() to accept timezone

```rust
pub async fn upsert_device(
    &self,
    id: &str,
    name: &str,
    ip: &str,
    port: i32,
    brand: &str,
    timezone: Option<&str>,  // New parameter
) -> Result<(), Error> {
    let tz = timezone.unwrap_or("UTC");

    sqlx::query(
        r#"
        INSERT INTO devices (id, name, ip, port, brand, timezone, last_seen, is_active)
        VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, 1)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            ip = excluded.ip,
            port = excluded.port,
            brand = excluded.brand,
            timezone = excluded.timezone,
            last_seen = CURRENT_TIMESTAMP,
            is_active = 1
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(ip)
    .bind(port)
    .bind(brand)
    .bind(tz)
    .execute(&self.pool)
    .await?;
    Ok(())
}
```

### Step 6: Update save_raw_attendance_batch()

Similar changes for batch insert:
```rust
pub async fn save_raw_attendance_batch(
    &self,
    batch: Vec<(String, AttendanceData)>,
) -> Result<usize, Error> {
    if batch.is_empty() {
        return Ok(0);
    }

    // Group by device_id to minimize timezone lookups
    let mut device_timezones: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    // Pre-fetch timezones for all unique devices
    for (device_id, _) in &batch {
        if !device_timezones.contains_key(device_id) {
            let tz = self.get_device_timezone(device_id).await;
            device_timezones.insert(device_id.clone(), tz);
        }
    }

    // Parse and prepare all records
    type BatchRecord = (String, String, i64, Option<i32>, Option<i32>);
    let mut records: Vec<BatchRecord> = Vec::new();

    for (device_id, data) in &batch {
        let timezone = device_timezones.get(device_id).map(|s| s.as_str()).unwrap_or("UTC");

        if let Ok(timestamp) = convert_local_to_unix_timestamp(&data.timestamp, timezone) {
            records.push((
                device_id.clone(),
                data.user_id.clone(),
                timestamp,
                data.status,
                data.punch,
            ));
        }
    }

    if records.is_empty() {
        return Ok(0);
    }

    // ... rest of batch insert logic with timestamp as i64
}
```

### Step 7: Update memory_store.rs insert functions

Add new insert function that accepts pre-computed timestamp:
```rust
/// Insert a new attendance log with pre-computed Unix timestamp
pub fn insert_with_timestamp(
    &self,
    device_id: String,
    data: AttendanceData,
    timestamp: i64,
) -> Result<i64, StoreError> {
    // Compute signature with i64 timestamp
    let signature = Self::compute_signature(&device_id, &data.user_id, timestamp, data.punch, data.status);
    if self.signatures.contains(&signature) {
        return Err(StoreError::Duplicate);
    }

    // Check capacity
    if self.count.load(Ordering::Relaxed) >= self.max_capacity {
        return Err(StoreError::CapacityExceeded);
    }

    // Add signature
    self.signatures.insert(signature);

    let id = self.next_id.fetch_add(1, Ordering::SeqCst);
    let now = chrono::Utc::now().naive_utc();

    let log = MemoryLog {
        id,
        device_id: device_id.clone(),
        user_id: data.user_id.clone(),
        timestamp,  // i64 directly
        status: data.status,
        punch: data.punch,
        is_synced: 0,
        created_at: now,
        updated_at: None,
        retry_count: 0,
        persisted: false,
    };

    let composite_hash = MemoryLog::compute_composite_hash(&device_id, &data.user_id, timestamp);

    self.logs.insert(id, log);
    self.composite_index.insert(composite_hash, id);
    self.unsynced_index.write().insert(id);
    self.count.fetch_add(1, Ordering::SeqCst);
    self.dirty.store(true, Ordering::SeqCst);

    Ok(id)
}
```

### Step 8: Update get_active_devices() query

Update to include timezone:
```rust
pub async fn get_active_devices(&self) -> Result<Vec<DeviceConfig>, Error> {
    sqlx::query_as::<_, DeviceConfig>(
        r#"
        SELECT id, name, ip, port, brand, timezone, last_seen, is_active
        FROM devices
        WHERE is_active = 1
        "#
    )
    .fetch_all(&self.pool)
    .await
}
```

### Step 9: Update checkpoint_batch() for i64 timestamp

```rust
pub async fn checkpoint_batch(&self, logs: &[crate::memory_store::MemoryLog]) -> Result<(usize, usize, Vec<(String, String, i64)>), Error> {
    // ...
    let composite_keys: Vec<(String, String, i64)> = logs.iter()
        .map(|l| (l.device_id.clone(), l.user_id.clone(), l.timestamp))  // timestamp is now i64
        .collect();
    // ...
}
```

## Todo List

- [ ] Add chrono-tz import
- [ ] Implement get_device_timezone()
- [ ] Implement convert_local_to_unix_timestamp()
- [ ] Update save_raw_attendance() with timezone lookup
- [ ] Update upsert_device() to accept timezone
- [ ] Update save_raw_attendance_batch() with timezone lookup
- [ ] Add insert_with_timestamp() to memory_store
- [ ] Update get_active_devices() query
- [ ] Update checkpoint_batch() return type
- [ ] Update all callers of upsert_device() in central_client.rs
- [ ] Run cargo check

## Success Criteria

1. `cargo check` passes
2. Timestamp conversion uses device timezone
3. Fallback to UTC works gracefully
4. Batch operations pre-fetch timezones efficiently

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| N+1 timezone queries | Medium | Medium | Batch pre-fetch timezones |
| DST ambiguity | Low | Low | Use earliest() fallback |
| Invalid timezone string | Low | Low | Default to UTC |

## Security Considerations
- No security impact

## Next Steps
After this phase, proceed to [Phase 5: Sync Logic](./phase-05-sync-logic.md)
