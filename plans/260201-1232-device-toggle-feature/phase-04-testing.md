# Phase 4: Testing

## Context Links
- [Plan Overview](./plan.md)
- [Phase 1: Backend](./phase-01-backend-configpush.md)
- [Phase 2: Agent](./phase-02-agent-configpush.md)
- [Phase 3: Frontend](./phase-03-frontend-toggle.md)

## Overview

| Property | Value |
|----------|-------|
| Priority | P1 |
| Status | Pending |
| Effort | 1 hour |
| Risk | Low |

Write tests to verify device disable functionality stops log sync.

## Test Categories

### 1. Backend Unit Tests (agent_manager.rs)

Verify `DeviceConfigPush` serialization includes `is_enabled`.

### 2. Agent Unit Tests (database.rs)

Verify `upsert_device` respects `is_enabled` param.

### 3. Agent Integration Tests

Verify disabled devices excluded from worker spawning.

### 4. E2E Test (Manual)

Verify full flow from UI toggle to agent worker stop.

## Implementation Steps

### Step 1: Backend Unit Test

**File:** `/home/elt1541/leeattend/apiserver/src/agent_manager.rs`

Add test after existing tests:

```rust
#[test]
fn test_device_config_push_serialization() {
    let device = DeviceConfigPush {
        device_id: "dev1".to_string(),
        name: "Test Device".to_string(),
        ip: "192.168.1.1".to_string(),
        port: 4370,
        brand: "zkteco".to_string(),
        is_enabled: false,
    };

    let json = serde_json::to_string(&device).unwrap();
    assert!(json.contains("\"is_enabled\":false"));
}

#[test]
fn test_config_push_with_disabled_device() {
    let config = ConfigData {
        devices: vec![
            DeviceConfigPush {
                device_id: "dev1".to_string(),
                name: "Enabled".to_string(),
                ip: "192.168.1.1".to_string(),
                port: 4370,
                brand: "zkteco".to_string(),
                is_enabled: true,
            },
            DeviceConfigPush {
                device_id: "dev2".to_string(),
                name: "Disabled".to_string(),
                ip: "192.168.1.2".to_string(),
                port: 4370,
                brand: "zkteco".to_string(),
                is_enabled: false,
            },
        ],
        max_concurrent_workers: Some(5),
    };

    let msg = CentralMessage::ConfigPush { data: config };
    let json = serde_json::to_string(&msg).unwrap();

    // Verify both devices included with correct is_enabled
    assert!(json.contains("\"is_enabled\":true"));
    assert!(json.contains("\"is_enabled\":false"));
}
```

### Step 2: Agent Unit Test (database.rs)

**File:** `/home/elt1541/leeattend/agent-main/src/database.rs`

Add test in `mod tests`:

```rust
#[tokio::test]
async fn test_upsert_device_disabled() {
    let db = setup_db().await;

    // Insert enabled device
    db.upsert_device("dev1", "Device 1", "192.168.1.1", 4370, "zkteco", "UTC", true).await.unwrap();

    let devices = db.get_active_devices().await.unwrap();
    assert_eq!(devices.len(), 1, "Should have 1 active device");

    // Update to disabled
    db.upsert_device("dev1", "Device 1", "192.168.1.1", 4370, "zkteco", "UTC", false).await.unwrap();

    let devices = db.get_active_devices().await.unwrap();
    assert_eq!(devices.len(), 0, "Disabled device should not appear in active devices");
}

#[tokio::test]
async fn test_upsert_device_re_enable() {
    let db = setup_db().await;

    // Insert disabled device
    db.upsert_device("dev1", "Device 1", "192.168.1.1", 4370, "zkteco", "UTC", false).await.unwrap();

    let devices = db.get_active_devices().await.unwrap();
    assert_eq!(devices.len(), 0, "Disabled device not in active list");

    // Re-enable
    db.upsert_device("dev1", "Device 1", "192.168.1.1", 4370, "zkteco", "UTC", true).await.unwrap();

    let devices = db.get_active_devices().await.unwrap();
    assert_eq!(devices.len(), 1, "Re-enabled device should appear");
}

#[tokio::test]
async fn test_mixed_enabled_disabled_devices() {
    let db = setup_db().await;

    db.upsert_device("dev1", "Device 1", "192.168.1.1", 4370, "zkteco", "UTC", true).await.unwrap();
    db.upsert_device("dev2", "Device 2", "192.168.1.2", 4370, "zkteco", "UTC", false).await.unwrap();
    db.upsert_device("dev3", "Device 3", "192.168.1.3", 4370, "zkteco", "UTC", true).await.unwrap();

    let devices = db.get_active_devices().await.unwrap();
    assert_eq!(devices.len(), 2, "Only enabled devices returned");

    let ids: Vec<&str> = devices.iter().map(|d| d.id.as_str()).collect();
    assert!(ids.contains(&"dev1"));
    assert!(ids.contains(&"dev3"));
    assert!(!ids.contains(&"dev2"));
}
```

### Step 3: Agent ConfigPush Test (central_client.rs)

**File:** `/home/elt1541/leeattend/agent-main/src/central_client.rs`

Add test in `mod tests`:

```rust
#[tokio::test]
async fn test_config_push_disabled_device() {
    let hybrid_db = HybridDatabase::new(":memory:", 1000).await.unwrap();
    let (reload_tx, mut reload_rx) = mpsc::channel(1);

    // Simulate ConfigPush with disabled device
    let config_push = r#"{
        "type": "config_push",
        "data": {
            "devices": [
                {
                    "device_id": "dev1",
                    "name": "Enabled Device",
                    "ip": "192.168.1.1",
                    "port": 4370,
                    "brand": "zkteco",
                    "is_enabled": true
                },
                {
                    "device_id": "dev2",
                    "name": "Disabled Device",
                    "ip": "192.168.1.2",
                    "port": 4370,
                    "brand": "zkteco",
                    "is_enabled": false
                }
            ],
            "max_concurrent_workers": 5
        }
    }"#;

    handle_incoming_text(config_push, &hybrid_db, &reload_tx).await;

    // Verify reload signal sent
    let reload_result = reload_rx.try_recv();
    assert!(reload_result.is_ok(), "ConfigPush should trigger reload");

    // Verify only enabled device is active
    let devices = hybrid_db.disk_db().get_active_devices().await.unwrap();
    assert_eq!(devices.len(), 1, "Only enabled device should be active");
    assert_eq!(devices[0].id, "dev1");
}

#[tokio::test]
async fn test_config_push_default_enabled() {
    let hybrid_db = HybridDatabase::new(":memory:", 1000).await.unwrap();
    let (reload_tx, _reload_rx) = mpsc::channel(1);

    // Simulate ConfigPush without is_enabled field (backward compat)
    let config_push = r#"{
        "type": "config_push",
        "data": {
            "devices": [
                {
                    "device_id": "dev1",
                    "name": "Device",
                    "ip": "192.168.1.1",
                    "port": 4370,
                    "brand": "zkteco"
                }
            ],
            "max_concurrent_workers": 5
        }
    }"#;

    handle_incoming_text(config_push, &hybrid_db, &reload_tx).await;

    // Verify device defaults to enabled
    let devices = hybrid_db.disk_db().get_active_devices().await.unwrap();
    assert_eq!(devices.len(), 1, "Device should default to enabled");
}
```

### Step 4: E2E Test (Manual Checklist)

1. **Start agent:** `cargo run` in agent-main
2. **Open browser:** Navigate to DevicesPage
3. **Find enabled device:** Should show green "ACTIVE" badge
4. **Click Disable button:** Confirm dialog appears
5. **Confirm:** Button shows loading spinner
6. **Verify toast:** Success message appears
7. **Verify badge:** Changes to "DISABLED"
8. **Check agent logs:** Should show "Configuration change detected! Restarting all workers..."
9. **Verify workers:** Disabled device worker should NOT restart
10. **Re-enable device:** Click Enable button
11. **Verify worker:** Device worker should start polling

## Todo List

- [ ] Add serialization tests in agent_manager.rs
- [ ] Add upsert_device tests in database.rs
- [ ] Add ConfigPush handling tests in central_client.rs
- [ ] Run all backend tests: `./run-tests.sh --api`
- [ ] Run all agent tests: `cd agent-main && cargo test`
- [ ] Perform manual E2E test
- [ ] Document test results

## Success Criteria

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] E2E flow works end-to-end
- [ ] Disabled device worker stops
- [ ] Re-enabled device worker starts
- [ ] No regression in existing tests

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Flaky tests | Low | Medium | Use in-memory DB |
| Missing test coverage | Medium | Low | E2E test as backup |
| Timing issues | Low | Medium | Add appropriate waits |

## Test Files Summary

| File | Tests Added |
|------|------------|
| `apiserver/src/agent_manager.rs` | 2 |
| `agent-main/src/database.rs` | 3 |
| `agent-main/src/central_client.rs` | 2 |

## Next Steps

After all tests pass, the feature is ready for code review and merge.
