---
date: "2026-02-17"
problem_type: "feature_enhancement"
severity: "medium"
symptoms:
  - "Users cannot see real-time device status (time, log counts) in the dashboard"
  - "No way to force refresh device status from the UI"
root_cause: "Incomplete status reporting mechanism and missing UI integration"
tags:
  - connectivity
  - devices
  - realtime
  - websocket
---

# Enhanced Device Status & Real-time Inspection

## Overview
Implemented a full-stack enhancement to provide real-time device metadata (time, users/logs counts, firmware) and an on-demand refresh capability in the connectivity settings page.

## Investigation Steps

| Attempt | Hypothesis | Result |
|---------|------------|--------|
| 1. Check existing status logic | Agent might already send metadata | ❌ Agent only sent "online/offline" status without metadata. |
| 2. Check ZK library capabilities | `pyzk2` library might not support counts | ✅ Library supports `get_users_count` and `get_attendance_count`. |
| 3. Check UI data flow | UI might not be fetching dynamic status | ✅ UI was only fetching static config, not dynamic status from `/agents`. |

## Implementation Details

### 1. Backend (Rust - apiserver)
- Extended `DeviceStatus` struct in `models.rs` with an `info: Option<serde_json::Value>` field.
- Updated `AgentManager` unit tests to handle the new field.
- Ensured `AgentStatus` propagates device metadata to the frontend.

### 2. Worker (Python - agent-zk-worker)
- Added `get_info()` method to `ZKService` to fetch:
  - Users count
  - Logs count
  - Device time
  - Firmware version
- Integrated `device.get_info` RPC method into `ZKWorkerClient`.

### 3. Agent Orchestrator (Rust - agent-main)
- Added a `Status Reporting Task` that collects device statuses and sends a batch update to the central server every 60 seconds.
- Integrated `device.get_info` calls into the polling loop (every 10 cycles or on connect).
- Implemented immediate "offline" status reporting upon connection failure.

### 4. Frontend (React - webdashboard)
- Updated `DeviceStatus` type definition.
- Integrated `useAgentsStatus` hook into `ConnectivityTopology`.
- Added "Refresh Status" action to `DeviceListItem` that sends a direct command to the agent.
- Improved UI with status pills and loading indicators for metadata.

## Lessons Learned

### Mistakes Made
- **Type Errors**: Added fields to Rust models but forgot to update Unit Test initializers, causing build failures.
- **Ownership/Moves**: Attempted to start the `CentralClient` after calling a method that moved `self`, causing a compiler error.
- **I18n Keys**: Added new UI strings without updating locale files, failing the translation usage test.

### Key Breakthrough
- **FxHasher**: Used `FxHasher` for high-performance deduplication in the worker loop.
- **Semaphore Integration**: Correctly positioned metadata fetching within the semaphore-controlled loop to prevent resource spikes.

## Results
- **Success Criteria**: 
  - [x] UI displays device time, users, and logs.
  - [x] "Refresh" button successfully triggers a hardware check.
  - [x] All backend and frontend tests passed.
  - [x] No regression in existing device CRUD flows.
