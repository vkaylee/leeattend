# Phase 1: Backend ConfigPush Update

## Context Links
- [Scout Report](/home/elt1541/leeattend/plans/reports/scout-260201-1225-device-toggle-codebase.md)
- [Plan Overview](./plan.md)

## Overview

| Property | Value |
|----------|-------|
| Priority | P1 |
| Status | Pending |
| Effort | 30 minutes |
| Risk | Low |

Add `is_enabled` field to `DeviceConfigPush` struct and update the mapping in `notify_config_update`.

## Key Insight

The `Device` model already has `is_enabled: bool` field. The gap is only in the `DeviceConfigPush` DTO used for WebSocket messages.

## Requirements

### Functional
- DeviceConfigPush must include `is_enabled: bool` field
- ConfigPush must send `is_enabled` value from Device model
- No breaking changes to existing WebSocket protocol

### Non-Functional
- Backward compatible (Agent should handle missing field gracefully)

## Related Code Files

### Modify
1. `/home/elt1541/leeattend/apiserver/src/agent_manager.rs`
   - Add `is_enabled: bool` to `DeviceConfigPush` struct (line 10-16)

2. `/home/elt1541/leeattend/apiserver/src/services/agent_service.rs`
   - Update mapping in `notify_config_update` (line 22-29)

## Implementation Steps

### Step 1: Update DeviceConfigPush Struct (agent_manager.rs)

**Before (line 9-16):**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfigPush {
    pub device_id: String,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub brand: String,
}
```

**After:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfigPush {
    pub device_id: String,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub brand: String,
    pub is_enabled: bool,
}
```

### Step 2: Update ConfigPush Mapping (agent_service.rs)

**Before (line 21-30):**
```rust
let config_data = ConfigData {
    devices: devices.into_iter().map(|d| {
        DeviceConfigPush {
            device_id: d.id.to_string(),
            name: d.name,
            ip: d.ip,
            port: d.port,
            brand: d.brand
        }
    }).collect(),
    max_concurrent_workers: max_workers
};
```

**After:**
```rust
let config_data = ConfigData {
    devices: devices.into_iter().map(|d| {
        DeviceConfigPush {
            device_id: d.id.to_string(),
            name: d.name,
            ip: d.ip,
            port: d.port,
            brand: d.brand,
            is_enabled: d.is_enabled,
        }
    }).collect(),
    max_concurrent_workers: max_workers
};
```

## Todo List

- [ ] Update `DeviceConfigPush` struct in `agent_manager.rs`
- [ ] Update mapping in `notify_config_update` in `agent_service.rs`
- [ ] Run `cargo build` to verify compilation
- [ ] Run `cargo test` on agent_service tests

## Success Criteria

- [ ] `DeviceConfigPush` struct has `is_enabled: bool` field
- [ ] ConfigPush WebSocket message includes `is_enabled` for each device
- [ ] All existing tests pass
- [ ] No breaking changes to API

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Agent breaks on new field | Low | Medium | Agent uses `#[serde(default)]` for backward compat |
| Missing import | Low | Low | Compile-time check |

## Security Considerations

- No new security concerns
- Workspace validation unchanged
- WebSocket auth unchanged

## Next Steps

After completion, proceed to [Phase 2: Agent ConfigPush Handling](./phase-02-agent-configpush.md)
