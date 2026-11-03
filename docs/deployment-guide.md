# Deployment Guide

LeeAttend runs as Go `apiserver`, React/Vite `webdashboard`, PostgreSQL, Redis, and MinIO. The apiserver pulls attendance logs from a configured external API; no edge process is deployed.

## API server

Requirements: Go 1.26+, PostgreSQL 18+, Redis 7+, and an S3-compatible object store.

```bash
cd apiserver
go run .
```

The server listens on `APP_HOST` and `APP_PORT` (default `0.0.0.0:3000`). Production uses `docker-compose.yml` and `apiserver/Dockerfile`.

## Scheduled log pull configuration

The scheduler is disabled by default. Configure these variables in the apiserver environment:

```env
LOG_PULL_ENABLED=false
LOG_PULL_BASE_URL=https://provider.example.invalid
# LOG_PULL_TOKEN=
LOG_PULL_ACCOUNT_ID=account-id
LOG_PULL_MODE=shadow
LOG_PULL_INTERVAL_SECS=60
LOG_PULL_TIMEOUT_SECS=30
LOG_PULL_PAGE_SIZE=100
LOG_PULL_MAX_CONCURRENT_REQUESTS=4
```

When enabled, URL, token, and account ID are required. `shadow` fetches and normalizes events without attendance writes. `write` commits events and the next cursor transactionally after reconciliation. Cursor state and processed event identity are tenant-scoped per workspace and attendance source.

Never log provider tokens, raw attendance payloads, or employee identifiers.

## Compose deployment

```bash
podman-compose -f docker-compose.yml up --build -d
podman-compose -f docker-compose.yml ps
podman-compose -f docker-compose.yml down
```

Services:

- `apiserver`: Go REST API, scheduled external attendance pull, and report WebSockets.
- `webdashboard`: React dashboard.
- `db_system`, `db_workspace`: PostgreSQL system and tenant databases.
- `redis`: rate limiting, queues, and report Pub/Sub.
- `minio`: S3-compatible file storage.

Report progress remains available through `/api/v1/ws/reports/{job_id}` and its compatibility aliases.

## Dashboard

```bash
cd webdashboard
bun install
bun run dev
```

Set `VITE_API_URL` to the apiserver URL. Build production assets with `bun run build`; Tauri packaging remains optional.

## Windows service

Run the Go apiserver binary under the selected Windows service manager. Apply the same environment variables as Linux.

## Operations

- Keep `LOG_PULL_MODE=shadow` through provider contract, pagination, identity, deduplication, and restart reconciliation.
- Monitor pull success/failure, provider latency, lag, fetched/imported/deduplicated/staged counts, and consecutive failures.
- Stop pull writes with `LOG_PULL_ENABLED=false` or `LOG_PULL_MODE=shadow`; the last committed cursor remains durable.
- Use the apiserver readiness and health endpoints for service checks.

## Verification

./leedevkit validate-compose
./leedevkit test api
./leedevkit test web
```

The API target runs Go formatting, vet, static analysis, vulnerability checks, module verification, and race-enabled tests in the pinned container. The web target runs frontend unit, type, lint, i18n, and E2E checks.
