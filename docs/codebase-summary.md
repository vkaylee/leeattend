# Codebase Summary

**Last Updated:** 2026-11-08

LeeAttend is an attendance and payroll system with a Go API server and React/Vite dashboard.

## Components

### `apiserver/`

Go `net/http` service using `pgx/v5` and PostgreSQL. Responsibilities:

- authenticated REST APIs for organization-wide employees, attendance, payroll, files, and monitoring;
- transactional bootstrap of one organization and its first administrator.

All system and business tables share one PostgreSQL database. The final schema uses singleton `organization_settings`, direct user roles, and no workspace tables or `workspace_id` columns.

### `webdashboard/`

React 19, Vite, TypeScript, Tailwind CSS 4, and TanStack Query. The dashboard manages organization settings, employees, attendance, payroll, files, and monitoring.


### Supporting infrastructure

- PostgreSQL unified database for all system and application tables.
- Redis for rate limiting, queue coordination, and report Pub/Sub.
- MinIO for S3-compatible object storage.
- Compose files for production, development, CI, and isolated test environments.

## Attendance-source flow

1. The scheduler enumerates enabled attendance sources within the organization.
2. The configured provider adapter pulls bounded pages using a durable cursor.
3. A database transaction inserts processed-event identity, imports attendance data, stages unknown employees, records errors, and advances the cursor only after commit.
4. Provider retries and page replay are safe because event identity is unique per organization/source.

No separate edge process, ingestion WebSocket protocol, or local attendance database remains in the runtime topology.

## Security and tenancy

- One implicit organization; no client-selectable tenant scope.
- JWT authentication and direct user-role authorization protect API routes.
- Provider tokens stay out of logs, metrics, traces, and error payloads.
- Attendance payloads and employee identifiers are treated as sensitive data.

## Verification

./leedevkit validate-compose
./leedevkit test api
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
