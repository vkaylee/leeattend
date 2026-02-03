# Phase 2: Agent ConfigPush Handling

## Context Links
- [Scout Report](/home/elt1541/leeattend/plans/reports/scout-260201-1225-device-toggle-codebase.md)
- [Plan Overview](./plan.md)
- [Phase 1: Backend](./phase-01-backend-configpush.md)

## Overview

| Property | Value |
|----------|-------|
| Priority | P1 |
| Status | Pending |
| Effort | 1 hour |
| Risk | Medium |

Update Agent to receive and handle `is_enabled` field in ConfigPush. Disabled devices should not spawn workers.

## Key Insights

1. Agent already has `is_active` field in `DeviceConfig` struct (database.rs line 56)
2. `get_active_devices()` filters by `is_active = 1` (database.rs line 550-559)
3. `upsert_device()` always sets `is_active = 1` (database.rs line 488-497)
4. ConfigPush handler calls `upsert_device()` for each device (central_client.rs line 310-312)
5. Worker loop uses `get_active_devices()` to spawn workers (main.rs line 259)

**Gap:** `upsert_device()` doesn't accept `is_enabled` param - it hardcodes `is_active = 1`

## Requirements

### Functional
- DeviceConfigPush struct must include `is_enabled: bool`
- `upsert_device()` must accept `is_enabled` parameter
- Disabled devices should have `is_active = 0` in database
- Workers should not spawn for disabled devices (handled by `get_active_devices` filter)

### Non-Functional
- Backward compatible with missing field (default to true)
- No performance impact

## Related Code Files

### Modify
1. `/home/elt1541/leeattend/agent-main/src/models.rs`
   - Add `is_enabled: bool` to `DeviceConfigPush` struct (line 60-68)

2. `/home/elt1541/leeattend/agent-main/src/database.rs`
   - Update `upsert_device()` signature to accept `is_enabled` param (line 485)
   - Update SQL to use param instead of hardcoded `1`

3. `/home/elt1541/leeattend/agent-main/src/central_client.rs`
   - Update ConfigPush handler to pass `is_enabled` to `upsert_device` (line 310-312)

## Implementation Steps

### Step 1: Update DeviceConfigPush Struct (models.rs)

**Before (line 60-68):**
```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceConfigPush {
    pub device_id: String,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub brand: String,
    pub timezone: Option<String>,
}
```

**After:**
```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceConfigPush {
    pub device_id: String,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub brand: String,
    pub timezone: Option<String>,
    #[serde(default = "default_enabled")]  // Backward compat
    pub is_enabled: bool,
}

fn default_enabled() -> bool { true }
```

### Step 2: Update upsert_device Signature (database.rs)

**Before (line 485-508):**
```rust
pub async fn upsert_device(&self, id: &str, name: &str, ip: &str, port: i32, brand: &str, timezone: &str) -> Result<(), Error> {
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
    .bind(timezone)
    .execute(&self.pool)
    .await?;
    Ok(())
}
```

**After:**
```rust
pub async fn upsert_device(&self, id: &str, name: &str, ip: &str, port: i32, brand: &str, timezone: &str, is_enabled: bool) -> Result<(), Error> {
    sqlx::query(
        r#"
        INSERT INTO devices (id, name, ip, port, brand, timezone, last_seen, is_active)
        VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, ?)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            ip = excluded.ip,
            port = excluded.port,
            brand = excluded.brand,
            timezone = excluded.timezone,
            last_seen = CURRENT_TIMESTAMP,
            is_active = excluded.is_active
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(ip)
    .bind(port)
    .bind(brand)
    .bind(timezone)
    .bind(is_enabled)
    .execute(&self.pool)
    .await?;
    Ok(())
}
```

### Step 3: Update ConfigPush Handler (central_client.rs)

**Before (line 310-312):**
```rust
for d in data.devices {
    let timezone = d.timezone.as_deref().unwrap_or("UTC");
    let _ = hybrid_db.disk_db().upsert_device(&d.device_id, &d.name, &d.ip, d.port, &d.brand, timezone).await;
}
```

**After:**
```rust
for d in data.devices {
    let timezone = d.timezone.as_deref().unwrap_or("UTC");
    let _ = hybrid_db.disk_db().upsert_device(&d.device_id, &d.name, &d.ip, d.port, &d.brand, timezone, d.is_enabled).await;
}
```

### Step 4: Update Test Calls

Update all test calls to `upsert_device` to include `is_enabled` param:

**Files to update:**
- `database.rs` tests (multiple occurrences)
- `central_client.rs` tests (if any)
- `hybrid_database_integration_test.rs`

**Pattern:**
```rust
// Before:
db.upsert_device("dev1", "Device 1", "192.168.1.1", 4370, "zkteco", "UTC").await.unwrap();

// After:
db.upsert_device("dev1", "Device 1", "192.168.1.1", 4370, "zkteco", "UTC", true).await.unwrap();
```

## Todo List

- [ ] Add `is_enabled` field to `DeviceConfigPush` in models.rs
- [ ] Add `default_enabled()` function for serde default
- [ ] Update `upsert_device()` signature in database.rs
- [ ] Update SQL query to use `is_enabled` param
- [ ] Update ConfigPush handler in central_client.rs
- [ ] Update all test calls to `upsert_device`
- [ ] Run `cargo build` to verify compilation
- [ ] Run `cargo test` to verify all tests pass

## Success Criteria

- [ ] Agent compiles without errors
- [ ] ConfigPush with `is_enabled: false` sets device `is_active = 0`
- [ ] Disabled devices excluded from `get_active_devices()`
- [ ] Workers not spawned for disabled devices
- [ ] All tests pass

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Breaking test calls | High | Low | Grep and update all occurrences |
| Backward compat | Low | Medium | Use `#[serde(default)]` for is_enabled |
| Worker still runs | Low | Medium | Verify get_active_devices filter |

## Security Considerations

- No new security concerns
- Database access unchanged
- WebSocket auth handled upstream

## Next Steps

After completion, proceed to [Phase 3: Frontend Toggle Button](./phase-03-frontend-toggle.md)
