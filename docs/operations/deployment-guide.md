
LeeAttend runs as Go `apiserver`, React/Vite `webdashboard`, one PostgreSQL database, Redis, and MinIO.

## API server

Requirements: Go 1.26+, PostgreSQL 18+, Redis 7+, and an S3-compatible object store.

```bash
cd apiserver
go run .
```

The server listens on `APP_HOST` and `APP_PORT` (default `0.0.0.0:3000`). Production uses `docker-compose.yml` and `apiserver/Dockerfile`.


## Compose deployment

```bash
podman-compose -f docker-compose.yml up --build -d
podman-compose -f docker-compose.yml ps
podman-compose -f docker-compose.yml down
```

- `apiserver`: Go REST API and report WebSockets.
- `webdashboard`: React dashboard.
- `db`: single PostgreSQL database for all application tables.
- `redis`: rate limiting, queues, and report Pub/Sub.
- `minio`: S3-compatible file storage.
Report progress remains available through `/api/v1/ws/reports/{job_id}` and its compatibility aliases.

## Dashboard

```bash
cd webdashboard
bun install
bun run dev
```

Set `VITE_API_URL` to the apiserver URL only when deployment does not provide a reverse proxy. Build production assets with `bun run build`.

## Windows service

Run the Go apiserver binary under the selected Windows service manager. Apply the same environment variables as Linux.

- Use the apiserver readiness and health endpoints for service checks.

## Verification

./scripts/_compose-validate.sh
leedevkit test api
leedevkit test web
```

The API target runs Go formatting, vet, static analysis, vulnerability checks, module verification, and race-enabled tests in the pinned container. The web target runs frontend unit, type, lint, i18n, and E2E checks.
