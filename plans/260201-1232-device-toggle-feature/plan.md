---
title: "Device Enable/Disable Toggle Feature"
description: "Add is_enabled field to ConfigPush and implement inline toggle button in frontend"
status: completed
priority: P1
effort: 4h
branch: main
tags: [feature, backend, agent, frontend, device-management]
created: 2026-02-01
---

# Device Enable/Disable Toggle Feature

## Overview

Enable device enable/disable functionality across the full stack:
1. Backend sends `is_enabled` in ConfigPush to Agent
2. Agent respects `is_enabled` flag and stops workers for disabled devices
3. Frontend displays inline toggle button for quick enable/disable

## Current State

| Component | Status | Details |
|-----------|--------|---------|
| Backend DB | OK | `is_enabled` field exists in Device model and schema |
| Backend Handler | OK | `update_device` supports `is_enabled` updates |
| ConfigPush Struct | GAP | `DeviceConfigPush` missing `is_enabled` field |
| Agent Struct | GAP | `DeviceConfigPush` missing `is_enabled` field |
| Agent Worker | GAP | No `is_enabled` handling in ConfigPush processing |
| Frontend API | OK | `updateDevice` supports `is_enabled` |
| Frontend UI | GAP | No inline toggle, only edit modal |

## Implementation Phases

| Phase | Title | Effort | Status |
|-------|-------|--------|--------|
| 1 | [Backend ConfigPush Update](./phase-01-backend-configpush.md) | 30m | ✅ Complete |
| 2 | [Agent ConfigPush Handling](./phase-02-agent-configpush.md) | 1h | ✅ Complete |
| 3 | [Frontend Toggle Button](./phase-03-frontend-toggle.md) | 1.5h | ✅ Complete |
| 4 | [Testing](./phase-04-testing.md) | 1h | ✅ Complete |

## Architecture

```
Frontend (Toggle Button)
    |
    v PUT /workspaces/:ws_id/devices/:id {is_enabled: false}
    |
    v
API Server (update_device handler)
    |
    v notify_config_update(agent_id)
    |
    v WebSocket: ConfigPush { devices: [{..., is_enabled: false}] }
    |
    v
Agent (central_client.rs)
    |
    v upsert_device(..., is_enabled)
    |
    v reload_rx.send(()) - triggers worker restart
    |
    v get_active_devices() - returns only is_active=1 devices
    |
    v Workers for disabled devices are NOT spawned
```

## Design Decision: Option A (Recommended)

Send ALL devices with `is_enabled` flag rather than filtering disabled devices out.

**Rationale:**
- Agent can track all configured devices
- Cleaner separation of "exists" vs "active"
- More explicit behavior
- Supports future features like device statistics

## Key Files

### Backend (API Server)
- `/home/elt1541/leeattend/apiserver/src/agent_manager.rs` - DeviceConfigPush struct
- `/home/elt1541/leeattend/apiserver/src/services/agent_service.rs` - ConfigPush mapping

### Agent-Main
- `/home/elt1541/leeattend/agent-main/src/models.rs` - DeviceConfigPush struct
- `/home/elt1541/leeattend/agent-main/src/central_client.rs` - ConfigPush handler
- `/home/elt1541/leeattend/agent-main/src/database.rs` - upsert_device method
- `/home/elt1541/leeattend/agent-main/src/main.rs` - Worker spawning logic

### Frontend
- `/home/elt1541/leeattend/webdashboard/src/pages/DevicesPage.tsx` - Device list UI
- `/home/elt1541/leeattend/webdashboard/src/pages/AgentsPage.tsx` - Toggle pattern ref

## Success Criteria

- [x] Backend: `DeviceConfigPush` includes `is_enabled` field
- [x] Backend: `notify_config_update` maps `is_enabled` from Device model
- [x] Agent: `DeviceConfigPush` struct updated with `#[serde(default)]`
- [x] Agent: `upsert_device` receives and stores `is_enabled`
- [x] Agent: Disabled devices excluded from `get_active_devices()`
- [x] Agent: Workers restart when ConfigPush received
- [x] Frontend: Inline toggle button in DevicesPage
- [x] Frontend: Optimistic update with rollback on error
- [x] Frontend: Loading state during request
- [x] Test: All 215 tests pass
