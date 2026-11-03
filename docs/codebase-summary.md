# Codebase Summary

**Last Updated:** 2026-11-03

LeeAttend is an attendance and payroll system with a Go API server and React/Vite dashboard.

## Components

### `apiserver/`

Go `net/http` service using `pgx/v5` and PostgreSQL. Responsibilities:

- authenticated REST APIs for workspaces, employees, attendance, payroll, files, and monitoring;
- scheduled external attendance-event pull with HTTPS provider adapter;
- durable per-workspace/source cursors and idempotent processed-event metadata;
- existing raw, staging, and attendance import semantics;
- report export jobs streamed through Redis Pub/Sub and WebSocket endpoints.

`LOG_PULL_ENABLED` gates the scheduler. `LOG_PULL_MODE=shadow` normalizes provider events without attendance writes; `write` commits events and cursors transactionally.

### `webdashboard/`

React 19, Vite, TypeScript, Tailwind CSS 4, TanStack Query, and Tauri. The dashboard manages workspaces, employees, attendance, payroll, files, and monitoring. Playwright tests mock API calls through `page.route`.

### Supporting infrastructure

- PostgreSQL system and tenant databases.
- Redis for rate limiting, queue coordination, and report Pub/Sub.
- MinIO for S3-compatible object storage.
- Compose files for production, development, CI, and isolated test environments.

## Attendance-source flow

1. The scheduler enumerates enabled attendance sources within each workspace.
2. The configured provider adapter pulls bounded pages using a durable cursor.
3. A tenant transaction inserts processed-event identity, imports attendance data, stages unknown employees, records errors, and advances the cursor only after commit.
4. Provider retries and page replay are safe because event identity is unique per workspace/source.

No separate edge process, ingestion WebSocket protocol, or local attendance database remains in the runtime topology.

## Security and tenancy

- Workspace predicates scope every tenant query.
- JWT authentication and workspace RBAC protect API routes.
- Provider tokens stay out of logs, metrics, traces, and error payloads.
- Attendance payloads and employee identifiers are treated as sensitive data.

## Verification

```bash
./leedevkit validate-compose
./leedevkit test api-go
./leedevkit test web
```

The Go target runs formatting, vet, static analysis, vulnerability checks, module verification, and race-enabled tests. The web target runs unit, type, lint, i18n, and E2E checks.

## Directory structure

```text
.
├── apiserver/              # Go API, scheduler, repositories, services, migrations
├── webdashboard/           # React/Vite dashboard and Playwright tests
├── .compose/               # Compose test, CI, and Go gate definitions
├── docker-compose.yml      # Production services
├── scripts/                # Hermetic build, test, migration, and validation wrappers
└── docs/                   # Architecture, API, deployment, and operational docs
```
