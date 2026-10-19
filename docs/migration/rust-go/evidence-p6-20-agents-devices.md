# P6-20 Devices, Authorized Agents, Command Dispatch, WebSocket Hub & Gap Closure Evidence

Task 6.4: Devices, Authorized Agents, Command Dispatch, WebSocket Hub (14 routes) and remaining 7 attendance/workflow gap routes (Total 21 routes) implemented in Go API server.
Contract route coverage reaches 403/403 (100% complete, 0 missing).

## Route Coverage Status

- Contract routes: **403**
- Go route registrations: **430** (+ aliases)
- Contract routes covered: **403** (+21)
- Contract routes missing: **0** (-21)

---

## 21 Implemented & Wired Routes

All 21 routes wired in `apiserver/main.go`, implemented in `internal/api/` handlers, backed by `internal/repository/` queries, and covered by unit tests:

### 1. Device Management (5 routes)
1. `GET /api/v1/devices` — 200 OK `[]Device`. Lists devices in workspace from tenant DB `devices`. Auth: Workspace context.
2. `POST /api/v1/devices` — 201 Created `{"id": "<uuid>"}`. Registers device in tenant DB; notifies connected agent via WebSocket `config_push`.
3. `PUT /api/v1/devices/{id}` — 200 OK `Device`. Updates device attributes; preserves password if omitted; notifies agent via `config_push`.
4. `DELETE /api/v1/devices/{id}` — 204 No Content. Deletes device from tenant DB; notifies agent; 404 if missing.
5. `GET /api/v1/devices/{id}/info` — 200 OK JSON object. Dispatches on-demand `get_device_info` command over WebSocket hub with 10s timeout; returns real-time hardware telemetry.

### 2. Authorized Agents (6 routes)
6. `GET /api/v1/authorized-agents` — 200 OK `[]AuthorizedAgent`. Queries workspace authorized agents from System DB with masked tokens (`••••••••`); supports `?include_inactive=true`.
7. `POST /api/v1/authorized-agents` — 201 Created `{"id": "<id>"}`. Registers edge agent with SHA-256 hashed token, `is_active=true`, `is_trusted=false`.
8. `GET /api/v1/authorized-agents/{id}` — 200 OK `AuthorizedAgent`. Fetches authorized agent by ID with token masked.
9. `PATCH /api/v1/authorized-agents/{id}` — 200 OK `AuthorizedAgent`. Partial update; validates non-empty update; updates SHA-256 token if provided; disconnects agent if untrusted or inactive; otherwise pushes config update.
10. `DELETE /api/v1/authorized-agents/{id}` — 204 No Content. Soft-deletes agent (`is_active = false`) and terminates WebSocket connection.
11. `PUT /api/v1/authorized-agents/{id}/trust` — 200 OK. Updates `is_trusted` flag; disconnects agent if untrusted.

### 3. Agent Status & Command Dispatch (2 routes)
12. `GET /api/v1/agents` — 200 OK `[]AgentStatus`. Returns real-time connectivity status and per-device state combining WebSocket hub memory, Redis presence keys (`agent:presence:{agent_id}`, TTL 60s), and database configurations.
13. `POST /api/v1/agent/{agent_id}/command` — 200 OK `{"message": "Command sent"}`. Verifies workspace ownership of agent and dispatches `AgentCommand` over WebSocket hub.

### 4. WebSocket Hub Connection (1 route + alias)
14. `GET /api/v1/ws` and `GET /ws` — 101 Switching Protocols. RFC 6455 WebSocket hub connection endpoint.
    - Pre-auth via `X-Agent-Id` and `X-Token` headers (and query parameters fallback).
    - Validates active status and SHA-256 token hash against System DB `authorized_agents`.
    - Enforces `is_trusted == true`.
    - Updates `last_ip` and `last_seen_at`.
    - Pushes initial `config_push` frame with device configurations and worker settings.
    - Message loop handles `heartbeat` (updates Redis presence, replies `heartbeat_ack`), `attendance` (replies `ack`), `command_response` (resolves pending command channels), and ping/pong.

### 5. Attendance & Workflow Gap Closure (7 routes)
15. `GET /api/v1/workspaces/{ws_id}/attendance/summaries` — 200 OK `PaginatedSummaries`. Paginated daily attendance summaries. RBAC: `attendance:summaries:read` allows querying any/all employees; without permission, enforces self-scope to caller's own employee ID (403 on mismatch).
16. `POST /api/v1/workspaces/{ws_id}/attendance/reprocess-all` — 200 OK `{"message": "Reprocess jobs queued successfully", "job_id": "<uuid>"}`. Bulk attendance reprocessing with 0..90 day range validation. Permission: `attendance:reprocess`.
17. `POST /api/v1/workspaces/{ws_id}/attendance/reprocess/{employee_id}` — 202 Accepted `{"message": "Reprocess job queued successfully", "job_id": "<uuid>"}`. Single-employee attendance reprocessing with 0..90 day range validation. Permission: `attendance:reprocess`.
18. `POST /api/v1/workspaces/{ws_id}/attendance/adjustments/{id}/approve` — 200 OK. POST route alias for adjustment approval/rejection. Permission: `attendance:adjustments:approve`.
19. `PUT /api/v1/workspaces/{ws_id}/time/entries/{id}` — 200 OK `TimeEntryRecord`. Updates punch times, entry type, note, or segment; recalculates duration minutes; 404 if missing. Permission: `attendance:logs:import`.
20. `DELETE /api/v1/workspaces/{ws_id}/time/entries/{id}` — 204 No Content. Deletes time entry from tenant DB; 404 if missing. Permission: `attendance:logs:import`.
21. `GET /api/v1/workspaces/{ws_id}/approval-workflows/pending-tasks` — 200 OK `PaginatedTasksResponse`. Paginated pending workflow tasks resolved across user, roles, permissions, and department manager assignments, excluding self-approvals. Permission: `approval_workflows:read`.

---

## Complete Route Map (21/21)

| # | Method | Path | Status | Handler | Auth Guard / Permission | Storage |
|---|---|---|---|---|---|---|
| 1 | GET | `/api/v1/devices` | 200 OK | `AgentDeviceHandler.ListDevices` | Workspace Context | Tenant DB `devices` |
| 2 | POST | `/api/v1/devices` | 201 Created | `AgentDeviceHandler.CreateDevice` | Workspace Context | Tenant DB `devices` |
| 3 | PUT | `/api/v1/devices/{id}` | 200 OK | `AgentDeviceHandler.UpdateDevice` | Workspace Context | Tenant DB `devices` |
| 4 | DELETE | `/api/v1/devices/{id}` | 204 No Content | `AgentDeviceHandler.DeleteDevice` | Workspace Context | Tenant DB `devices` |
| 5 | GET | `/api/v1/devices/{id}/info` | 200 OK | `AgentDeviceHandler.GetDeviceInfo` | Workspace Context | WebSocket Hub (10s timeout) |
| 6 | GET | `/api/v1/authorized-agents` | 200 OK | `AgentDeviceHandler.ListAuthorizedAgents` | Workspace Context | System DB `authorized_agents` |
| 7 | POST | `/api/v1/authorized-agents` | 201 Created | `AgentDeviceHandler.CreateAuthorizedAgent` | Workspace Context | System DB `authorized_agents` |
| 8 | GET | `/api/v1/authorized-agents/{id}` | 200 OK | `AgentDeviceHandler.GetAuthorizedAgent` | Workspace Context | System DB `authorized_agents` |
| 9 | PATCH | `/api/v1/authorized-agents/{id}` | 200 OK | `AgentDeviceHandler.UpdateAuthorizedAgent` | Workspace Context | System DB `authorized_agents` |
| 10 | DELETE | `/api/v1/authorized-agents/{id}` | 204 No Content | `AgentDeviceHandler.DeleteAuthorizedAgent` | Workspace Context | System DB `authorized_agents` |
| 11 | PUT | `/api/v1/authorized-agents/{id}/trust` | 200 OK | `AgentDeviceHandler.UpdateAgentTrust` | Workspace Context | System DB `authorized_agents` |
| 12 | GET | `/api/v1/agents` | 200 OK | `AgentDeviceHandler.ListAgents` | Workspace Context | System DB, Tenant DB, Redis |
| 13 | POST | `/api/v1/agent/{agent_id}/command` | 200 OK | `AgentDeviceHandler.SendCommand` | Workspace Context | WebSocket Hub |
| 14 | GET | `/api/v1/ws` (and `/ws`) | 101 Upgrade | `AgentDeviceHandler.WebSocket` | `X-Agent-Id` + `X-Token` (pre-auth) | WebSocket Hub, System DB, Redis |
| 15 | GET | `/api/v1/workspaces/{ws_id}/attendance/summaries` | 200 OK | `AttendanceHandler.Summaries` | `attendance:summaries:read` (conditional) | Tenant DB `attendance_daily_summaries` |
| 16 | POST | `/api/v1/workspaces/{ws_id}/attendance/reprocess-all` | 200 OK | `AttendanceHandler.ReprocessAll` | `attendance:reprocess` | Job Queue / Redis |
| 17 | POST | `/api/v1/workspaces/{ws_id}/attendance/reprocess/{employee_id}` | 202 Accepted | `AttendanceHandler.ReprocessEmployee` | `attendance:reprocess` | Job Queue / Redis |
| 18 | POST | `/api/v1/workspaces/{ws_id}/attendance/adjustments/{id}/approve` | 200 OK | `AttendanceHandler.ApproveAdjustment` | `attendance:adjustments:approve` | Tenant DB |
| 19 | PUT | `/api/v1/workspaces/{ws_id}/time/entries/{id}` | 200 OK | `AttendanceHandler.UpdateTimeEntry` | `attendance:logs:import` | Tenant DB `time_entries` |
| 20 | DELETE | `/api/v1/workspaces/{ws_id}/time/entries/{id}` | 204 No Content | `AttendanceHandler.DeleteTimeEntry` | `attendance:logs:import` | Tenant DB `time_entries` |
| 21 | GET | `/api/v1/workspaces/{ws_id}/approval-workflows/pending-tasks` | 200 OK | `ApprovalWorkflowHandler.PendingTasks` | `approval_workflows:read` | Tenant DB `approval_workflow_tasks` |
