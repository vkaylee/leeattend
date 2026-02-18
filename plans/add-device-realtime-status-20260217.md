# Implementation Plan: Enhanced Device Status & Real-time Inspection

## Overview
Improve the connectivity settings page by providing real-time device metadata (time, user/log counts, configuration) using a hybrid approach of enhanced background updates and on-demand commands.

## Approach
1.  **Extend Models**: Add an `info` field to `DeviceStatus` to store arbitrary device metadata.
2.  **Enhance Worker**: Update the Python sidecar to fetch metadata from the physical device.
3.  **Real-time Trigger**: Add a UI button to trigger an immediate status refresh.

## Steps

### 1. Backend Model Update
- Modify `apiserver/src/models.rs` to add `info: Option<serde_json::Value>` to `DeviceStatus`.
- Update `AgentStatus` if necessary to propagate this.

### 2. Python Sidecar Enhancement
- File: `agent-zk-worker/zk_worker.py`
- Add `get_device_info` to `ZKService` using `zk.get_users_count()`, `zk.get_attendance_count()`, and `zk.get_time()`.
- Update `ZKWorkerClient` to include this info in the periodic `status_update` payload.

### 3. Agent Orchestrator Update
- File: `agent-main/src/main.rs`
- Ensure the JSON-RPC response from the worker is correctly mapped and sent to the API Server.

### 4. Frontend Implementation
- File: `webdashboard/src/pages/workspace/settings/connectivity.tsx` (Target path based on standard patterns)
- Add a "Details" or "Refresh" button.
- Display metadata:
  - **Device Time**: Compare with browser time to show offset.
  - **Records**: Show "X/Y" (Attendance/Users).
  - **Firmware**: If available.

## Testing
- Verify that clicking "Refresh" updates the metadata.
- Verify that metadata is populated automatically every heartbeat cycle.
- Verify that offline devices show the "Last Known" info with a warning.

## Rollback Plan
- Revert to previous Git commit.
- The JSON-RPC communication is backward compatible (extra fields are ignored).
