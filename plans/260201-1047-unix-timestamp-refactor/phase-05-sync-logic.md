# Phase 5: Sync & Composite Key Refactor

## Context Links
- Plan: [plan.md](./plan.md)
- Central Client: `agent-main/src/central_client.rs`
- Memory Store: `agent-main/src/memory_store.rs`

## Overview
- **Priority**: P1
- **Status**: pending
- **Effort**: 1.5h
- **Depends on**: Phase 4

Update sync logic to use i64 timestamps in composite keys and ACK handling.

## Key Insights

1. **Composite Key Format**: `device_id:user_id:timestamp_unix` (timestamp already i64)
2. **parse_composite_key()**: Simplified - no DateTime conversion needed for DB query
3. **ACK Handling**: Use `WHERE timestamp = ?` with i64 binding directly
4. **AttendanceData.timestamp**: Still sent as String to server (server handles formatting)

## Requirements

### Functional
- Composite key uses i64 timestamp directly
- ACK parsing extracts i64 timestamp for DB query
- ConfigPush passes timezone to upsert_device()

### Non-functional
- Reduce conversion overhead during sync
- Maintain backward compatibility with server (server expects String timestamp in Attendance message)

## Architecture

### Composite Key Flow
```
Memory:
    MemoryLog { device_id, user_id, timestamp: i64 }
        |
        v
    composite_key() = "DEV1:U001:1738314000"  (i64 directly)
        |
        v
ACK:
    parse_composite_key("DEV1:U001:1738314000")
        |
        v
    (device_id, user_id, timestamp: i64)
        |
        v
    UPDATE ... WHERE timestamp = 1738314000  (i64 binding)
```

### Attendance Message to Server
```rust
CentralMessage::Attendance {
    msg_id: log.composite_key(),  // "DEV1:U001:1738314000"
    data: AttendanceData {
        timestamp: format_for_server(log.timestamp),  // "2025-01-31 09:00:00" (UTC)
    },
    ...
}
```

## Related Code Files

### Modify
- `agent-main/src/central_client.rs`
- `agent-main/src/memory_store.rs`

## Implementation Steps

### Step 1: Update parse_composite_key() in central_client.rs

Simplify to return i64 timestamp instead of String:
```rust
/// Parse composite key "device_id:user_id:timestamp_unix" back to components
/// Returns timestamp as i64 for direct DB binding
fn parse_composite_key(key: &str) -> Option<(String, String, i64)> {
    // Split from right: timestamp (last), user_id (second-to-last), device_id (rest)
    let mut parts = key.rsplitn(3, ':');

    // Extract in reverse order: timestamp, user_id, device_id
    let timestamp_str = parts.next()?.trim();
    let user_id = parts.next()?.trim();
    let device_id = parts.next()?.trim();

    // Parse Unix timestamp directly
    let timestamp: i64 = timestamp_str.parse().ok()?;

    Some((device_id.to_string(), user_id.to_string(), timestamp))
}
```

### Step 2: Update ACK handling for successful sync

```rust
CentralMessage::Ack { msg_id, status, .. } => {
    if status.as_deref() == Some("success") {
        // Update memory
        if !hybrid_db.mark_synced_by_key(&msg_id) {
            log::debug!("ACK for already-synced log (disk update OK): {}", msg_id);
        }

        // Update disk using composite key with i64 timestamp
        if let Some((device_id, user_id, timestamp)) = parse_composite_key(&msg_id) {
            log::info!("ACK Success: device='{}', user='{}', timestamp={}",
                device_id, user_id, timestamp);

            match sqlx::query(
                "UPDATE zkt_raw_attendance SET is_synced = 1
                 WHERE device_id = ? AND user_id = ? AND timestamp = ?"
            )
            .bind(&device_id)
            .bind(&user_id)
            .bind(timestamp)  // i64 binding directly
            .execute(hybrid_db.disk_db().get_pool())
            .await
            {
                Ok(result) => {
                    let rows = result.rows_affected();
                    if rows == 0 {
                        log::error!("UPDATE affected 0 rows! device='{}', user='{}', timestamp={} - Log may not exist",
                            device_id, user_id, timestamp);
                    } else {
                        log::debug!("Marked {} row(s) as synced in DB", rows);
                    }
                }
                Err(e) => {
                    log::error!("UPDATE query failed: {}", e);
                }
            }
        } else {
            log::warn!("Failed to parse composite key: {}", msg_id);
        }
    } else {
        // Failed ACK - reset to unsynced
        // ... similar changes with i64 binding
    }
}
```

### Step 3: Update failed ACK handling

```rust
} else {
    // Failed ACK - reset to unsynced for retry
    if !hybrid_db.reset_to_unsynced_by_key(&msg_id) {
        log::debug!("Failed ACK for already-synced log: {}", msg_id);
    }

    // Reset disk using composite key with i64 timestamp
    if let Some((device_id, user_id, timestamp)) = parse_composite_key(&msg_id) {
        log::warn!("ACK Failed: Resetting device='{}', user='{}', timestamp={}",
            device_id, user_id, timestamp);

        let _ = sqlx::query(
            "UPDATE zkt_raw_attendance SET is_synced = 0
             WHERE device_id = ? AND user_id = ? AND timestamp = ?"
        )
        .bind(&device_id)
        .bind(&user_id)
        .bind(timestamp)  // i64 binding directly
        .execute(hybrid_db.disk_db().get_pool())
        .await;
    }
}
```

### Step 4: Update ConfigPush to pass timezone

```rust
CentralMessage::ConfigPush { data } => {
    info!("Received ConfigPush with {} devices", data.devices.len());

    // ... existing max_concurrent_workers handling ...

    // Deactivate devices not in this push
    let active_ids: Vec<String> = data.devices.iter().map(|d| d.device_id.clone()).collect();
    if let Ok(deactivated) = hybrid_db.disk_db().deactivate_devices_not_in(&active_ids).await
        && deactivated > 0 {
            info!("Deactivated {} stale devices", deactivated);
        }

    for d in data.devices {
        // Pass timezone from ConfigPush
        let _ = hybrid_db.disk_db().upsert_device(
            &d.device_id,
            &d.name,
            &d.ip,
            d.port,
            &d.brand,
            d.timezone.as_deref(),  // Pass optional timezone
        ).await;
    }

    let _ = reload_tx.try_send(());
}
```

### Step 5: Update Attendance message formatting

In sync loop, format timestamp as String for server:
```rust
for log in logs {
    // Convert i64 Unix timestamp to UTC datetime string for server
    let timestamp_str = chrono::DateTime::from_timestamp(log.timestamp, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "1970-01-01 00:00:00".to_string());

    let msg = CentralMessage::Attendance {
        msg_id: log.composite_key(),  // Uses i64 directly
        agent_id: sync_agent_id.clone(),
        device_id: log.device_id.clone(),
        data: AttendanceData {
            user_id: crate::utils::normalize_employee_id(&log.user_id),
            timestamp: timestamp_str,  // String for server
            status: log.status,
            punch: log.punch,
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    // ...
}
```

### Step 6: Update memory_store get_id_by_composite_key()

```rust
/// Get memory ID by composite key string (parses and computes hash)
/// Format: "device_id:user_id:timestamp_unix"
pub fn get_id_by_composite_key(&self, composite_key: &str) -> Option<i64> {
    // Split from right: timestamp (last), user_id (second-to-last), device_id (rest)
    let mut parts = composite_key.rsplitn(3, ':');

    let timestamp_str = parts.next()?;
    let user_id = parts.next()?;
    let device_id = parts.next()?;

    // Parse timestamp directly as i64
    let timestamp: i64 = timestamp_str.parse().ok()?;

    let hash = MemoryLog::compute_composite_hash(device_id, user_id, timestamp);
    self.composite_index.get(&hash).map(|entry| *entry)
}
```

### Step 7: Update load_from_disk() to handle i64 timestamp

```rust
pub fn load_from_disk(&self, logs: Vec<RawAttendanceLog>) {
    let mut max_id = 0i64;
    let mut unsynced = self.unsynced_index.write();
    let mut inflight = self.inflight_index.write();

    for log in logs {
        max_id = max_id.max(log.id);

        // Signature now uses i64 timestamp directly
        let signature = Self::compute_signature(&log.device_id, &log.user_id, log.timestamp, log.punch, log.status);
        self.signatures.insert(signature);

        let memory_log = MemoryLog {
            id: log.id,
            device_id: log.device_id.clone(),
            user_id: log.user_id.clone(),
            timestamp: log.timestamp,  // Already i64
            status: log.status,
            punch: log.punch,
            is_synced: log.is_synced,
            created_at: log.created_at.unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            updated_at: log.updated_at,
            retry_count: log.retry_count,
            persisted: true,
        };

        let composite_hash = MemoryLog::compute_composite_hash(&log.device_id, &log.user_id, log.timestamp);
        // ...
    }
}
```

### Step 8: Update test functions in central_client.rs

```rust
#[test]
fn test_parse_composite_key_normal() {
    let key = "DEV1:U001:1738314000";
    let result = parse_composite_key(key);
    assert!(result.is_some());

    let (device_id, user_id, timestamp) = result.unwrap();
    assert_eq!(device_id, "DEV1");
    assert_eq!(user_id, "U001");
    assert_eq!(timestamp, 1738314000i64);  // i64 directly
}

#[test]
fn test_parse_composite_key_with_colon_in_device_id() {
    let key = "Office:Main:Gate:U001:1738314000";
    let result = parse_composite_key(key);
    assert!(result.is_some());

    let (device_id, user_id, timestamp) = result.unwrap();
    assert_eq!(device_id, "Office:Main:Gate");
    assert_eq!(user_id, "U001");
    assert_eq!(timestamp, 1738314000i64);
}
```

## Todo List

- [ ] Update parse_composite_key() to return i64 timestamp
- [ ] Update ACK success handling with i64 binding
- [ ] Update ACK failure handling with i64 binding
- [ ] Update ConfigPush to pass timezone
- [ ] Update Attendance message formatting (i64 -> String for server)
- [ ] Update get_id_by_composite_key() to use i64
- [ ] Update load_from_disk() for i64 timestamp
- [ ] Update compute_composite_hash() calls
- [ ] Update test functions
- [ ] Run cargo check

## Success Criteria

1. `cargo check` passes
2. Composite keys use i64 directly in hash computation
3. DB queries bind i64 timestamp correctly
4. Server receives String-formatted timestamp in Attendance messages

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Server incompatibility | Low | Medium | Server expects String, we format before sending |
| Hash collision | Very Low | Low | Same hash function, different input type |
| Test failures | High | Medium | Update all test fixtures |

## Security Considerations
- No security impact

## Next Steps
After this phase, proceed to [Phase 6: Testing](./phase-06-testing.md)
