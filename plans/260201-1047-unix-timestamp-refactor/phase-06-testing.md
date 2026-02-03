# Phase 6: Testing & Validation

## Context Links
- Plan: [plan.md](./plan.md)
- Tests: `agent-main/tests/*.rs`
- Database tests: `agent-main/src/database.rs` (mod tests)

## Overview
- **Priority**: P1
- **Status**: pending
- **Effort**: 1h
- **Depends on**: Phase 5

Update all tests to use i64 timestamps and validate timezone conversion.

## Key Insights

1. **Test Fixtures**: All timestamp strings need conversion to i64
2. **Timezone Tests**: Add specific tests for timezone conversion
3. **Edge Cases**: DST transitions, invalid timezones, missing devices
4. **WSL Environment**: Use `./run-tests.sh --api` for all test runs

## Requirements

### Functional
- All existing tests pass with i64 timestamps
- New tests for timezone conversion
- New tests for fallback to UTC

### Non-functional
- Tests run under 60 seconds total
- No flaky tests

## Test Categories

### 1. Unit Tests (database.rs mod tests)
- `test_save_and_retrieve_attendance` - Update to use i64
- `test_save_attendance_batch` - Update to use i64
- New: `test_timezone_conversion_vietnam`
- New: `test_timezone_fallback_to_utc`

### 2. Integration Tests (tests/*.rs)
- `integration_test.rs` - Update fixtures
- `sync_log_classification_test.rs` - Update fixtures
- `hybrid_database_integration_test.rs` - Update fixtures

### 3. Central Client Tests
- `test_parse_composite_key_*` - Return i64
- `test_handle_incoming_ack_*` - Use i64 binding

## Implementation Steps

### Step 1: Update database.rs test fixtures

```rust
#[tokio::test]
async fn test_save_and_retrieve_attendance() {
    let db = setup_db().await;

    // First, add a device with timezone
    db.upsert_device("D1", "Device 1", "192.168.1.1", 4370, "zkteco", Some("Asia/Ho_Chi_Minh")).await.unwrap();

    let data = AttendanceData {
        user_id: "U1".into(),
        timestamp: "2026-01-01 08:00:00".into(),  // Local time (Vietnam)
        status: Some(1),
        punch: Some(1),
    };
    db.save_raw_attendance("D1", data).await.unwrap();

    let logs = db.get_unsynced_logs(10).await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].user_id, "U1");

    // Verify timestamp is Unix epoch (2026-01-01 08:00:00 Asia/Ho_Chi_Minh = 2026-01-01 01:00:00 UTC)
    // = 1735693200 (approximate)
    assert!(logs[0].timestamp > 0, "Timestamp should be positive Unix epoch");
}
```

### Step 2: Add timezone conversion tests

```rust
#[tokio::test]
async fn test_timezone_conversion_vietnam() {
    let db = setup_db().await;

    // Add device with Vietnam timezone (UTC+7)
    db.upsert_device("DEV_VN", "Vietnam Office", "192.168.1.1", 4370, "zkteco", Some("Asia/Ho_Chi_Minh")).await.unwrap();

    let data = AttendanceData {
        user_id: "U001".into(),
        timestamp: "2026-01-01 14:00:00".into(),  // 2 PM Vietnam time
        status: Some(1),
        punch: Some(0),
    };
    db.save_raw_attendance("DEV_VN", data).await.unwrap();

    let logs = db.get_unsynced_logs(10).await.unwrap();
    assert_eq!(logs.len(), 1);

    // 2026-01-01 14:00:00 Vietnam (UTC+7) = 2026-01-01 07:00:00 UTC
    // Expected Unix: 1735714800
    let expected_utc = chrono::DateTime::parse_from_rfc3339("2026-01-01T07:00:00Z")
        .unwrap()
        .timestamp();

    assert_eq!(logs[0].timestamp, expected_utc, "Timestamp should be converted to UTC");
}

#[tokio::test]
async fn test_timezone_fallback_to_utc() {
    let db = setup_db().await;

    // Device with invalid timezone
    db.upsert_device("DEV_BAD", "Bad TZ", "192.168.1.1", 4370, "zkteco", Some("Invalid/Timezone")).await.unwrap();

    let data = AttendanceData {
        user_id: "U001".into(),
        timestamp: "2026-01-01 10:00:00".into(),
        status: Some(1),
        punch: Some(0),
    };
    db.save_raw_attendance("DEV_BAD", data).await.unwrap();

    let logs = db.get_unsynced_logs(10).await.unwrap();
    assert_eq!(logs.len(), 1);

    // Should fall back to UTC
    // 2026-01-01 10:00:00 UTC = 1735725600
    let expected_utc = chrono::DateTime::parse_from_rfc3339("2026-01-01T10:00:00Z")
        .unwrap()
        .timestamp();

    assert_eq!(logs[0].timestamp, expected_utc, "Should fall back to UTC");
}

#[tokio::test]
async fn test_device_without_timezone_uses_utc() {
    let db = setup_db().await;

    // Device without explicit timezone (None)
    db.upsert_device("DEV_NONE", "No TZ", "192.168.1.1", 4370, "zkteco", None).await.unwrap();

    let data = AttendanceData {
        user_id: "U001".into(),
        timestamp: "2026-01-01 10:00:00".into(),
        status: Some(1),
        punch: Some(0),
    };
    db.save_raw_attendance("DEV_NONE", data).await.unwrap();

    let logs = db.get_unsynced_logs(10).await.unwrap();
    assert_eq!(logs.len(), 1);

    // Should use default UTC
    let expected_utc = chrono::DateTime::parse_from_rfc3339("2026-01-01T10:00:00Z")
        .unwrap()
        .timestamp();

    assert_eq!(logs[0].timestamp, expected_utc);
}
```

### Step 3: Add unknown device test

```rust
#[tokio::test]
async fn test_unknown_device_uses_utc() {
    let db = setup_db().await;

    // Save attendance for unknown device (not in devices table)
    let data = AttendanceData {
        user_id: "U001".into(),
        timestamp: "2026-01-01 10:00:00".into(),
        status: Some(1),
        punch: Some(0),
    };
    db.save_raw_attendance("UNKNOWN_DEV", data).await.unwrap();

    let logs = db.get_unsynced_logs(10).await.unwrap();
    assert_eq!(logs.len(), 1);

    // Should fall back to UTC (device not found)
    let expected_utc = chrono::DateTime::parse_from_rfc3339("2026-01-01T10:00:00Z")
        .unwrap()
        .timestamp();

    assert_eq!(logs[0].timestamp, expected_utc);
}
```

### Step 4: Update integration test fixtures

**File**: `agent-main/tests/integration_test.rs`

Update all test data that inserts attendance records.

### Step 5: Update memory_store tests

```rust
#[test]
fn test_composite_key_with_i64_timestamp() {
    let log = MemoryLog {
        id: 1,
        device_id: "DEV1".to_string(),
        user_id: "U001".to_string(),
        timestamp: 1738314000,  // i64 Unix timestamp
        status: Some(1),
        punch: Some(0),
        is_synced: 0,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: None,
        retry_count: 0,
        persisted: false,
    };

    let key = log.composite_key();
    assert_eq!(key, "DEV1:U001:1738314000");
}
```

### Step 6: Update central_client tests

```rust
#[test]
fn test_parse_composite_key_returns_i64() {
    let key = "DEV1:U001:1738314000";
    let result = parse_composite_key(key);
    assert!(result.is_some());

    let (device_id, user_id, timestamp) = result.unwrap();
    assert_eq!(device_id, "DEV1");
    assert_eq!(user_id, "U001");
    assert_eq!(timestamp, 1738314000i64);

    // Verify it's actually i64
    let _: i64 = timestamp;
}
```

### Step 7: Update hybrid_database tests

```rust
#[tokio::test]
async fn test_full_checkpoint_ack_flow_with_i64() {
    let hybrid = HybridDatabase::new("sqlite::memory:", 1000).await.unwrap();

    // Add device with timezone
    hybrid.disk_db().upsert_device("DEV1", "Test", "192.168.1.1", 4370, "zkteco", Some("UTC")).await.unwrap();

    // Insert logs
    for i in 0..10 {
        let data = AttendanceData {
            user_id: format!("U{:03}", i),
            timestamp: format!("2026-01-31 10:{:02}:00", i),
            status: Some(1),
            punch: Some(0),
        };
        hybrid.save_raw_attendance("DEV1", data).unwrap();
    }

    // Checkpoint
    let inserted = hybrid.checkpoint().await.unwrap();
    assert_eq!(inserted, 10);

    // Get logs and verify composite key format
    let logs = hybrid.get_unsynced_logs(20);
    for log in &logs {
        let key = log.composite_key();
        // Key should end with a number (Unix timestamp)
        let parts: Vec<&str> = key.rsplitn(2, ':').collect();
        let timestamp: i64 = parts[0].parse().expect("Should be valid i64");
        assert!(timestamp > 0);
    }
}
```

### Step 8: Run all tests

```bash
# WSL environment - MUST use run-tests.sh
./run-tests.sh --api agent
```

## Todo List

- [ ] Update test_save_and_retrieve_attendance
- [ ] Add test_timezone_conversion_vietnam
- [ ] Add test_timezone_fallback_to_utc
- [ ] Add test_device_without_timezone_uses_utc
- [ ] Add test_unknown_device_uses_utc
- [ ] Update integration_test.rs fixtures
- [ ] Update memory_store tests
- [ ] Update central_client tests
- [ ] Update hybrid_database tests
- [ ] Run ./run-tests.sh --api agent
- [ ] Fix any failing tests
- [ ] Verify no regressions

## Success Criteria

1. All tests pass: `./run-tests.sh --api agent`
2. No flaky tests
3. Code coverage for timezone conversion
4. Edge cases covered (invalid TZ, missing device, DST)

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Many test failures | High | Medium | Systematic fixture updates |
| Flaky tests | Low | Low | Use deterministic timestamps |
| WSL I/O issues | Medium | Medium | Always use run-tests.sh |

## Security Considerations
- No security impact

## Validation Checklist

After all tests pass:

- [ ] Manual test: Insert log with Vietnam timezone, verify UTC conversion
- [ ] Manual test: ConfigPush with timezone, verify device updated
- [ ] Manual test: Sync flow end-to-end with server
- [ ] Check SQLite: `SELECT timestamp FROM zkt_raw_attendance` shows INTEGER values
- [ ] Check SQLite: `SELECT timezone FROM devices` shows timezone values
