# Phase 3: Update Data Models

## Context Links
- Plan: [plan.md](./plan.md)
- Models: `agent-main/src/models.rs`
- Memory Store: `agent-main/src/memory_store.rs`

## Overview
- **Priority**: P1
- **Status**: pending
- **Effort**: 30m
- **Depends on**: Phase 2

Update Rust structs to use i64 for timestamps and add timezone to device config.

## Key Insights

1. **RawAttendanceLog.timestamp**: Change from `NaiveDateTime` to `i64`
2. **SignatureRow.timestamp**: Also change to `i64`
3. **MemoryLog.timestamp**: Change to `i64`
4. **DeviceConfig**: Add `timezone: String` field
5. **DeviceConfigPush**: Add `timezone: Option<String>` for server push
6. **AttendanceData**: Keep `timestamp: String` (input from sidecar unchanged)

## Requirements

### Functional
- All timestamp fields use i64 (Unix seconds)
- Device has timezone configuration
- ConfigPush can optionally update timezone

### Non-functional
- Maintain sqlx::FromRow derive compatibility
- Keep serde (de)serialization working

## Architecture

### Before
```rust
pub struct RawAttendanceLog {
    pub timestamp: chrono::NaiveDateTime,
    // ...
}
```

### After
```rust
pub struct RawAttendanceLog {
    pub timestamp: i64,  // Unix timestamp (UTC seconds)
    // ...
}
```

## Related Code Files

### Modify
- `agent-main/src/models.rs`
- `agent-main/src/memory_store.rs`

### Affected (update in later phases)
- `agent-main/src/database.rs`
- `agent-main/src/central_client.rs`

## Implementation Steps

### Step 1: Update models.rs - RawAttendanceLog

**File**: `agent-main/src/models.rs`

Change line 11:
```rust
// From:
pub timestamp: chrono::NaiveDateTime,

// To:
pub timestamp: i64,  // Unix timestamp (UTC seconds)
```

### Step 2: Update models.rs - SignatureRow

Change line 25:
```rust
// From:
pub timestamp: chrono::NaiveDateTime,

// To:
pub timestamp: i64,  // Unix timestamp (UTC seconds)
```

### Step 3: Update models.rs - FailedLog

Change line 36:
```rust
// From:
pub timestamp: chrono::NaiveDateTime,

// To:
pub timestamp: i64,  // Unix timestamp (UTC seconds)
```

### Step 4: Update models.rs - DeviceConfig

Add timezone field after line 53 (after `brand`):
```rust
pub struct DeviceConfig {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub port: i64,
    pub brand: String,
    pub timezone: String,  // ADD: IANA timezone (e.g., 'Asia/Ho_Chi_Minh')
    pub last_seen: Option<chrono::NaiveDateTime>,
    pub is_active: Option<bool>,
}
```

### Step 5: Update models.rs - DeviceConfigPush

Add timezone field after line 65:
```rust
pub struct DeviceConfigPush {
    pub device_id: String,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub brand: String,
    pub timezone: Option<String>,  // ADD: Optional timezone from server
}
```

### Step 6: Update memory_store.rs - MemoryLog

**File**: `agent-main/src/memory_store.rs`

Change line 48:
```rust
// From:
pub timestamp: NaiveDateTime,

// To:
pub timestamp: i64,  // Unix timestamp (UTC seconds)
```

### Step 7: Update MemoryLog::composite_key()

Change lines 61-67:
```rust
// From:
pub fn composite_key(&self) -> String {
    format!(
        "{}:{}:{}",
        self.device_id,
        self.user_id,
        self.timestamp.and_utc().timestamp()
    )
}

// To:
pub fn composite_key(&self) -> String {
    format!(
        "{}:{}:{}",
        self.device_id,
        self.user_id,
        self.timestamp  // Already i64, no conversion needed
    )
}
```

### Step 8: Update MemoryLog::compute_composite_hash()

Change line 77:
```rust
// From:
fn compute_composite_hash(device_id: &str, user_id: &str, timestamp: NaiveDateTime) -> u64 {
    let mut hasher = FxHasher::default();
    device_id.hash(&mut hasher);
    user_id.hash(&mut hasher);
    timestamp.and_utc().timestamp().hash(&mut hasher);
    hasher.finish()
}

// To:
fn compute_composite_hash(device_id: &str, user_id: &str, timestamp: i64) -> u64 {
    let mut hasher = FxHasher::default();
    device_id.hash(&mut hasher);
    user_id.hash(&mut hasher);
    timestamp.hash(&mut hasher);  // Already i64
    hasher.finish()
}
```

### Step 9: Update compute_signature()

Change line 139:
```rust
// From:
fn compute_signature(device_id: &str, user_id: &str, timestamp: NaiveDateTime, ...) -> u64 {
    // ...
    timestamp.and_utc().timestamp().hash(&mut hasher);
    // ...
}

// To:
fn compute_signature(device_id: &str, user_id: &str, timestamp: i64, ...) -> u64 {
    // ...
    timestamp.hash(&mut hasher);  // Already i64
    // ...
}
```

## Todo List

- [ ] Update RawAttendanceLog.timestamp to i64
- [ ] Update SignatureRow.timestamp to i64
- [ ] Update FailedLog.timestamp to i64
- [ ] Add DeviceConfig.timezone field
- [ ] Add DeviceConfigPush.timezone field
- [ ] Update MemoryLog.timestamp to i64
- [ ] Update MemoryLog::composite_key() to use i64 directly
- [ ] Update compute_composite_hash() signature
- [ ] Update compute_signature() signature
- [ ] Run cargo check to verify compilation

## Success Criteria

1. `cargo check` passes (even with temporary errors in database.rs/central_client.rs)
2. All model structs have correct field types
3. Hash functions accept i64 directly

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Breaking existing code | High | Expected | Complete database.rs in Phase 4 |
| sqlx::FromRow mismatch | Medium | Medium | Verify INTEGER binding works |

## Security Considerations
- No security impact (type changes only)

## Next Steps
After this phase, proceed to [Phase 4: Database Logic](./phase-04-database-logic.md)
