# System Architecture

## Runtime ownership

LeeAttend runs two application services:

- `apiserver/`: Go HTTP API, scheduled external attendance pull, durable import state, device CRUD, background jobs, and report-stream WebSockets.
- `webdashboard/`: React/Vite browser and Tauri client.

PostgreSQL stores system, tenant, attendance, and pull-state data. Redis provides rate limiting, queue coordination, and report Pub/Sub. MinIO stores uploaded files.

No edge process, ingestion WebSocket protocol, local device database, or device-only deployment is required.

## Attendance pull flow

1. `scheduler.Runner` enumerates enabled devices by workspace at the configured interval.
2. `external.AttendanceEventsClient` requests bounded pages from the configured provider over HTTPS.
3. The provider adapter sends account, workspace/device identity, cursor, and page-size parameters. Tokens stay in process memory and never enter logs.
4. `service.ExternalAttendancePullService` normalizes events and delegates persistence to the tenant pull repository.
5. A tenant transaction records processed event identity, routes attendance data through existing raw/staging/error semantics, and advances the cursor only after all rows commit.
6. Provider retries are limited to temporary transport, 429, and 5xx failures. Authentication, validation, and decode failures stop the page.

Processed event identity is scoped to `(workspace_id, device_id, external_event_id)`. Provider replay, process restart, and concurrent scheduler attempts therefore remain idempotent. Device credentials and attendance payloads are not logged.

## Migration modes

`LOG_PULL_ENABLED=false` disables the scheduler. When enabled:

- `LOG_PULL_MODE=shadow` fetches and normalizes events for reconciliation without attendance writes or write-cursor advancement.
- `LOG_PULL_MODE=write` enables transactional attendance import and cursor advancement.

Required provider configuration and bounded numeric settings are validated at startup. Shutdown cancels in-flight requests and stops the runner after the current bounded operation.

## HTTP API

The API server exposes authenticated REST resources for workspaces, employees, attendance, payroll, files, monitoring, and device configuration. Device CRUD remains available at:

- `GET /api/v1/devices`
- `POST /api/v1/devices`
- `PUT /api/v1/devices/{id}`
- `DELETE /api/v1/devices/{id}`

Workspace isolation is enforced through workspace context and tenant database predicates. Public routes are limited to health and authentication entrypoints.

## Report streaming

Report exports remain asynchronous. The API publishes progress chunks to Redis Pub/Sub and serves them through:

- `GET /api/v1/ws/reports/{job_id}`
- `GET /v1/ws/reports/{job_id}`
- `GET /ws/reports/{job_id}`

The report WebSocket is unrelated to attendance-device ingestion and remains part of the API contract.

## Data protection and observability

Logs and metrics contain request IDs, workspace/device IDs where permitted, counts, durations, statuses, and failure classes. They must not contain provider tokens, device passwords, raw attendance payloads, employee identifiers, or email addresses.

Metrics cover pull success/failure, provider latency, lag, fetched/imported/deduplicated/staged counts, cursor state, and consecutive failures. Readiness and alerts must expose provider dependency failure without exposing secret configuration.

## Deployment topology

```text
[Web Dashboard] --HTTP/WebSocket--> [Go apiserver]
                                  |--REST/SQL--> [PostgreSQL]
                                  |--queue/pubsub--> [Redis]
                                  |--object storage--> [MinIO]
                                  |--HTTPS pull--> [External attendance API]
                                  |--device configuration--> [Attendance devices through provider API]
```

Compose deploys `apiserver`, `webdashboard`, PostgreSQL, Redis, MinIO, and supporting poolers. No separate edge service or Rust application build belongs in the runtime topology.

## Verification boundaries

- API: `./leedevkit test api-go`
- Web: `./leedevkit test web`
- Compose: `./leedevkit validate-compose`

Tests must cover cursor commit ordering, duplicate provider events, concurrent lease attempts, provider timeout/rate-limit behavior, shadow-mode non-write behavior, and report WebSocket streaming.
