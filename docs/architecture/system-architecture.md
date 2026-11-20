# System Architecture

## Runtime ownership

LeeAttend runs one application service pair:

- `apiserver/`: Go HTTP API, background jobs, and report-stream WebSockets.
- `webdashboard/`: React/Vite browser dashboard.

PostgreSQL stores organization business, attendance, and payroll data in one database. The final schema uses `organization_settings`, `roles`, `role_permissions`, and `user_roles` with no workspace columns or tables. Redis provides rate limiting, queue coordination, and report Pub/Sub. MinIO stores uploaded files.

No edge process, tenant database, external attendance provider, or online activation service is required.


## HTTP API

The API server exposes authenticated REST resources for the singleton organization, employees, attendance, payroll, files, monitoring, and attendance-source operations.

Organization scope is resolved from trusted server bootstrap context. Public API routes omit organization and workspace identifiers, and legacy `/workspaces/{id}` paths are rejected before routing.

## Report streaming

Report exports remain asynchronous. The API publishes progress chunks to Redis Pub/Sub and serves them through:

- `GET /api/v1/ws/reports/{job_id}`
- `GET /v1/ws/reports/{job_id}`
- `GET /ws/reports/{job_id}`

The report WebSocket is unrelated to attendance-source ingestion and remains part of the API contract.

## Data protection and observability

Logs and metrics contain request IDs, source IDs where permitted, counts, durations, statuses, and failure classes. They must not contain provider tokens, raw attendance payloads, employee identifiers, or email addresses.

Metrics cover pull success/failure, provider latency, lag, fetched/imported/deduplicated/staged counts, cursor state, and consecutive failures. Readiness and alerts must expose provider dependency failure without exposing secret configuration.

## Deployment topology

```text
[Web Dashboard] --HTTP/WebSocket--> [Go apiserver]
                                  |--REST/SQL--> [PostgreSQL organization database]
                                  |--queue/pubsub--> [Redis]
                                  |--object storage--> [MinIO]
```

Compose deploys `apiserver`, `webdashboard`, one PostgreSQL database, Redis, and MinIO. No tenant pooler or desktop application build belongs in the runtime topology.

## Verification boundaries

- API: `leedevkit test api`
- Web: `leedevkit test web`
- CLI discovery: `leedevkit --help`

