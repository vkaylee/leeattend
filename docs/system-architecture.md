# System Architecture

## Runtime ownership

LeeAttend runs one application service pair:

- `apiserver/`: Go HTTP API, background jobs, and report-stream WebSockets.
- `webdashboard/`: React/Vite browser and Tauri client.

PostgreSQL stores system, workspace, attendance, and payroll data in one database. Redis provides rate limiting, queue coordination, and report Pub/Sub. MinIO stores uploaded files.

No edge process, tenant database, external attendance provider, or online activation service is required.


## HTTP API

The API server exposes authenticated REST resources for workspaces, employees, attendance, payroll, files, monitoring, and attendance-source operations.

Singleton organization binding is enforced through configuration and shared database context. Public routes are limited to health and authentication entrypoints.

## Report streaming

Report exports remain asynchronous. The API publishes progress chunks to Redis Pub/Sub and serves them through:

- `GET /api/v1/ws/reports/{job_id}`
- `GET /v1/ws/reports/{job_id}`
- `GET /ws/reports/{job_id}`

The report WebSocket is unrelated to attendance-source ingestion and remains part of the API contract.

## Data protection and observability

Logs and metrics contain request IDs, workspace/source IDs where permitted, counts, durations, statuses, and failure classes. They must not contain provider tokens, raw attendance payloads, employee identifiers, or email addresses.

Metrics cover pull success/failure, provider latency, lag, fetched/imported/deduplicated/staged counts, cursor state, and consecutive failures. Readiness and alerts must expose provider dependency failure without exposing secret configuration.

## Deployment topology

```text
[Web Dashboard] --HTTP/WebSocket--> [Go apiserver]
                                  |--REST/SQL--> [PostgreSQL system + workspace]
                                  |--queue/pubsub--> [Redis]
                                  |--object storage--> [MinIO]
```

Compose deploys `apiserver`, `webdashboard`, one PostgreSQL database, Redis, and MinIO. No tenant pooler, external attendance provider, or Rust application build belongs in the runtime topology.

## Verification boundaries

- API: `./leedevkit test api`
- Web: `./leedevkit test web`
- Compose: `./leedevkit validate-compose`

