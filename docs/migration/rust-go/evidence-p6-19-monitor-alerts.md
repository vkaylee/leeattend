# P6-19 Monitor, Alerts, Jobs & Workspace Audit Parity Evidence

Task 6.3 Monitor, alerts, jobs, and workspace audit (12 routes) implemented in Go API, advancing contract route coverage from 370 to 382 (missing routes reduced from 33 to 21).

## Route Coverage Status

- Contract routes: **403**
- Go route registrations: **438** (+30 with aliases)
- Contract routes covered: **382** (+12)
- Contract routes missing: **21** (-12)

---

## 12 Newly Implemented & Wired Routes

All 12 routes wired in `apiserver/main.go`, implemented in `internal/api/monitor.go`, and queried in `internal/repository/monitor.go`:

### 1. System Monitor & Alerts (8 routes)
1. `GET /api/v1/monitor/overview` — 200 OK `MonitorOperationalSnapshot`. Real-time queue depths (high/normal/low), worker heartbeats, recent jobs, durable job overview, thresholds, dependencies (Redis, Postgres, DB pool), and quality metrics. Auth: `PlatformAdmin` OR `Support`.
2. `GET /api/v1/monitor/jobs` — 200 OK `MonitorJobsResponse`. Paginated legacy Redis jobs and durable PostgreSQL jobs with cursor-based pagination and status/state/search filters. Auth: `PlatformAdmin` OR `Support`.
3. `GET /api/v1/monitor/correlations` — 200 OK `MonitorCorrelationsResponse`. Correlated durable jobs and lifecycle events filtered by exact `request_id` or `trace_id`. Auth: `PlatformAdmin` OR `Support`.
4. `GET /api/v1/monitor/stream` — 200 OK (SSE). Real-time Server-Sent Events stream delivering `event: monitor-update` every 2s, client disconnect handling, and `event: monitor-error` fallback. Auth: `PlatformAdmin` OR `Support`.
5. `GET /api/v1/monitor/alerts` — 200 OK `MonitorAlertsResponse`. Paginated active and resolved system monitor alerts filtered by status and severity. Auth: `PlatformAdmin` OR `Support`.
6. `GET /api/v1/monitor/alerts/{alert_id}` — 200 OK `MonitorAlertDetailResponse`. Alert details and append-only lifecycle events (`monitor_alert_events`); 404 if missing. Auth: `PlatformAdmin` OR `Support`.
7. `POST /api/v1/monitor/alerts/{alert_id}/acknowledge` — 200 OK `MonitorAlertResponse`. Acknowledges active alert; idempotent 200 OK if already acknowledged; 409 Conflict if not active; 403 Forbidden for Support. Auth: `PlatformAdmin` ONLY.
8. `POST /api/v1/monitor/alerts/{alert_id}/resolve` — 200 OK `MonitorAlertResponse`. Resolves active alert; idempotent 200 OK if already resolved; 409 Conflict if not active; 403 Forbidden for Support. Auth: `PlatformAdmin` ONLY.

### 2. Workspace Jobs, Reports & Audit (4 routes)
9. `GET /api/v1/ws/reports/{job_id}` — 101 Switching Protocols (WebSocket). Pre-upgrade JWT authentication via `?token=`, `access_token` cookie, or Bearer header; tenant isolation verification against `metadata.workspace_id`; handles completed/failed race conditions prior to pubsub subscription; streams chunks and heartbeat pings. Auth: Member of `metadata.workspace_id`.
10. `GET /api/v1/workspaces/{ws_id}/audit-logs` — 200 OK `PaginatedAuditLogs`. Paginated audit logs from tenant database with action, start_date, and end_date filtering. Auth: Workspace Member + `audit_logs:read` (`PermissionAuditLogsRead`).
11. `GET /api/v1/workspaces/{ws_id}/jobs/{job_id}` — 200 OK `GenericJobStatusResponse`. Background report job progress and metadata from Redis with batch counts, percentage, and completion timestamps; tenant isolation check; 404 if missing. Auth: Workspace Member.
12. `GET /api/v1/workspaces/{ws_id}/monitor/jobs/{job_id}` — 200 OK `MonitorInfraJobRecord`. Durable infrastructure job record and associated lifecycle events; tenant isolation check; 404 if missing. Auth: Workspace Member.

---

## Complete Route Map (12/12)

| # | Method | Path | Status | Handler | Auth Guard | DB / Redis Store |
|---|---|---|---|---|---|---|
| 1 | GET | `/api/v1/monitor/overview` | 200 OK | `MonitorHandler.GetOverview` | `PlatformAdmin` OR `Support` | Redis `worker:jobs:*`, `monitor:worker:*`; DB `infra_job_records` |
| 2 | GET | `/api/v1/monitor/jobs` | 200 OK | `MonitorHandler.GetMonitorJobs` | `PlatformAdmin` OR `Support` | Redis `report:job_list`, `report:meta:*`; DB `infra_job_records` |
| 3 | GET | `/api/v1/monitor/correlations` | 200 OK | `MonitorHandler.GetMonitorCorrelations` | `PlatformAdmin` OR `Support` | DB `infra_job_records`, `infra_job_events` |
| 4 | GET | `/api/v1/monitor/stream` | 200 OK (SSE) | `MonitorHandler.Stream` | `PlatformAdmin` OR `Support` | Redis + DB snapshots (interval: 2s) |
| 5 | GET | `/api/v1/monitor/alerts` | 200 OK | `MonitorHandler.ListAlerts` | `PlatformAdmin` OR `Support` | DB `monitor_alerts` |
| 6 | GET | `/api/v1/monitor/alerts/{alert_id}` | 200 OK | `MonitorHandler.GetAlert` | `PlatformAdmin` OR `Support` | DB `monitor_alerts`, `monitor_alert_events` |
| 7 | POST | `/api/v1/monitor/alerts/{alert_id}/acknowledge` | 200 OK | `MonitorHandler.AcknowledgeAlert` | `PlatformAdmin` ONLY (`Support` = 403) | DB `monitor_alerts`, `monitor_alert_events` |
| 8 | POST | `/api/v1/monitor/alerts/{alert_id}/resolve` | 200 OK | `MonitorHandler.ResolveAlert` | `PlatformAdmin` ONLY (`Support` = 403) | DB `monitor_alerts`, `monitor_alert_events` |
| 9 | GET | `/api/v1/ws/reports/{job_id}` | 101 Upgrade | `MonitorHandler.ReportStreamWS` | Workspace Member via JWT (`?token=`, cookie, header) | Redis PubSub `report:stream:{job_id}`, `report:meta:{job_id}` |
| 10 | GET | `/api/v1/workspaces/{ws_id}/audit-logs` | 200 OK | `MonitorHandler.GetAuditLogs` | Workspace Member + `audit_logs:read` | Tenant DB `audit_logs` |
| 11 | GET | `/api/v1/workspaces/{ws_id}/jobs/{job_id}` | 200 OK | `MonitorHandler.GetWorkspaceJobStatus` | Workspace Member + tenant match | Redis `report:meta:{job_id}` |
| 12 | GET | `/api/v1/workspaces/{ws_id}/monitor/jobs/{job_id}` | 200 OK | `MonitorHandler.GetWorkspaceMonitorJob` | Workspace Member + tenant match | DB `infra_job_records`, `infra_job_events` |

---

## Security & Implementation Details

1. **Role Separation for Alert Mutation vs Reading**:
   - `requirePlatformAdminOrSupport`: Allows `platform_admin` and `support` to read monitor overviews, jobs, correlations, stream, and alerts.
   - `requirePlatformAdminOnly`: Strictly restricts alert acknowledgment and resolution to `platform_admin`. Calls by `support` return `403 Forbidden`.
   - Reason sanitization: Whitespace-normalized, PII/email/phone redacting filter applied, bounded to 1..500 characters.

2. **Tenant Boundary & Permission Enforcement**:
   - `GetAuditLogs`: Checks workspace membership and permission `audit_logs:read` (`PermissionAuditLogsRead`).
   - `GetWorkspaceJobStatus`: Checks workspace membership and validates `meta.workspace_id == ws_id`.
   - `GetWorkspaceMonitorJob`: Checks workspace membership and validates `job.workspace_id == ws_id`.
   - `ReportStreamWS`: Validates token before upgrade; rejects non-members of `metadata.workspace_id` with 403 HTTP before WebSocket handshake.

3. **Real-time Protocols**:
   - SSE (`/monitor/stream`): Uses HTTP flusher with `text/event-stream`, `keep-alive`, initial snapshot delivery, and 2-second interval push; disconnects cleanly on context cancellation.
   - WebSocket (`/ws/reports/{job_id}`): Implements RFC 6455 handshake using Go standard library (`crypto/sha1`, `encoding/base64`, `net/http` hijacker); pre-upgrade race-check handles jobs that completed or failed before connection was established.
