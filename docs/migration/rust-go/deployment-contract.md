# Phase 1 — Deployment & Operational Contract

> Task ID: `P1-10`  
> Status: Completed  
> Reference Implementation: Rust `apiserver/` + Docker Compose + Caddy  
> Target Implementation: Go `apiserver/` + Parallel Multi-Service Orchestration  

---

## 1. Container Build Specifications

### 1.1 Existing Rust Dockerfile (`apiserver/Dockerfile`)
The Rust API server utilizes a 7-stage multi-stage build anchored on `cargo-chef` and `sccache` for compilation caching:

| Stage | Name | Base Image | Purpose | Command / Output |
|---|---|---|---|---|
| 1 | `chef` | `docker.io/blackdex/rust-musl:x86_64-musl-stable` | Installs `cargo-chef ^0.1.77`, `clippy`, `rustfmt`, build utilities | Toolchain base |
| 2 | `planner` | `chef` | Prepares dependency recipe | `cargo chef prepare --recipe-path recipe.json` |
| 3 | `cacher` | `chef` | Pre-compiles dependencies with sccache | `cargo chef cook --release --recipe-path recipe.json` |
| 4 | `dev` | `public.ecr.aws/docker/library/rust:1.93-bookworm` | Hot-reloading development container with `cargo-watch`, `diesel_cli`, debugging tools | `CMD ["cargo", "watch", "-x", "run"]` |
| 5 | `test` | `chef` | Test runner container with `cargo-nextest` 0.9.133, `cargo-llvm-cov`, `diesel_cli` | `CMD ["cargo", "nextest", "run"]` |
| 6 | `builder` | `chef` | Static release compilation against `x86_64-unknown-linux-musl` | Binary `/usr/local/bin/apiserver` |
| 7 | `runtime` | `gcr.io/distroless/static-debian12` | Hermetic non-root runtime container | User `65532`, port `3000`, volume `/app/storage` |

### 1.2 Go Toolchain & Test Container (`.compose/go/Dockerfile`)
The Go test container is pinned to reproducible patch versions:
- Base: `golang:1.24.13-bookworm` (`.compose/go/Dockerfile:5`)
- Build Gate Toolchain (Pinned):
  - `staticcheck` v0.6.1 (`honnef.co/go/tools/cmd/staticcheck`)
  - `govulncheck` v1.1.4 (`golang.org/x/vuln/cmd/govulncheck`)
  - `sqlc` v1.29.0 (`github.com/sqlc-dev/sqlc/cmd/sqlc`)
  - `oapi-codegen` v2.4.1 (`github.com/oapi-codegen/oapi-codegen/v2/cmd/oapi-codegen`)
- Image tag: `localhost/leeattend-test_go`

### 1.3 Target Go Production Dockerfile (`apiserver/Dockerfile.go` Specification)
The Go production deployment container must follow these exact constraints:
- **Stage 1 (Builder)**: `golang:1.24.13-bookworm`, `CGO_ENABLED=0`, compile flag `-ldflags="-s -w"`.
- **Stage 2 (Runtime)**: `gcr.io/distroless/static-debian12`.
- **User**: Non-root `65532:65532` (`nonroot:nonroot`).
- **Binary**: `/usr/local/bin/apiserver` (or `/app/apiserver`).
- **Volume Mount**: `/app/storage`.
- **Exposed Port**: `3001` (coexistence default) or `3000`.

---

## 2. Compose Services, Ports & Profiles

### 2.1 Service Topology Matrix

| Service Name | Container Image | Internal Port | Host Port (Dev) | Network | Compose File | Healthcheck Contract |
|---|---|---|---|---|---|---|
| `db_system` | `postgres:18-alpine` | `5432` | `5434:5432` | `backend_net` | `docker-compose.yml:16`, `docker-compose.dev.yml:5` | `pg_isready -U ${DB_USER} -d ${DB_NAME}` (10s interval, 5s timeout) |
| `db_workspace` | `postgres:18-alpine` | `5432` | `5435:5432` | `backend_net` | `docker-compose.yml:31`, `docker-compose.dev.yml:12` | `pg_isready -U tenant_user -d tenant_db` |
| `db_legacy` | `postgres:18-alpine` | `5432` | `5433:5432` | `backend_net` | `docker-compose.yml:47` (profile `migration`) | `pg_isready -U postgres` |
| `redis` | `redis:7-alpine` | `6379` | None | `backend_net` | `docker-compose.yml:67` | `redis-cli -a ${REDIS_PASSWORD} ping` |
| `pgbouncer_tx` | `edoburu/pgbouncer:v1.25.2-p0` | `6543` | None | `backend_net` | `docker-compose.yml:77` | `pg_isready -h localhost -p 6543` (transaction mode) |
| `pgbouncer_sess`| `edoburu/pgbouncer:v1.25.2-p0` | `5452` | None | `backend_net` | `docker-compose.yml:105` | `pg_isready -h localhost -p 5452` (session mode) |
| `pgbouncer_tenant_tx` | `edoburu/pgbouncer:v1.25.2-p0` | `6543` | None | `backend_net` | `docker-compose.yml:132` | `pg_isready -h localhost -p 6543` |
| `pgbouncer_tenant_sess`| `edoburu/pgbouncer:v1.25.2-p0` | `5452` | None | `backend_net` | `docker-compose.yml:160` | `pg_isready -h localhost -p 5452` |
| `minio` | `quay.io/minio/minio:latest` | `9000`, `9001` | `9000:9000`, `9001:9001` | `backend_net` | `docker-compose.yml:246` | `mc ready local` |
| `apiserver` (Rust) | `localhost/leeattend_apiserver` | `3000` | None | `backend_net`, `frontend_net` | `docker-compose.yml:187` | `wget -qO- http://localhost:3000/api/v1/health` |
| `api-go` (Target) | `localhost/leeattend_apiserver_go` | `3001` | None | `backend_net`, `frontend_net` | `leedevkit.toml:29`, test compose | `wget -qO- http://localhost:3001/api/v1/health` |
| `webdashboard` | `localhost/leeattend_webdashboard` | `80` (prod), `5173` (dev) | None | `frontend_net` | `docker-compose.yml:264`, `docker-compose.dev.yml:116` | Caddy reverse proxy / Vite HMR |
| `caddy` (Dev) | `caddy:alpine` | `80` | `${DEV_PORT:-8080}:80` | `frontend_net` | `.compose/docker-compose.dev.yml:135` | HTTP 200 on reverse proxy root |
| `agent-main` | `localhost/leeattend_agent-main` | None | None | `backend_net`, `frontend_net` | `docker-compose.yml:277` | WebSocket client connection to API server |
| `cloudbeaver` | `dbeaver/cloudbeaver:latest` | `8978` | `8978:8978` | `backend_net` | `.compose/docker-compose.dev.yml:40` | Web GUI login ready |
| `redisinsight` | `redisinsight:latest` | `5540` | `5540:5540` | `backend_net` | `.compose/docker-compose.dev.yml:23` (profile `manual`) | Web GUI ready |

### 2.2 Test Profile Hierarchy (`.compose/docker-compose.test.yml`)
The test environment isolates containers by explicit execution profile:
- `infra-db`: Starts `db_system`, `db_workspace`, and runs `db_init` migration container.
- `infra-pooler`: Starts all 4 PgBouncer containers.
- `infra-redis`: Starts test Redis instance.
- `api` (`lint-api`, `unit-api`, `int-api`): Rust test container (`localhost/leeattend-test_apiserver`).
- `go` (`lint-go`, `unit-go`, `int-go`): Go test container (`localhost/leeattend-test_go`).
- `web` (`lint-web`, `unit-web`, `e2e-web`, `frontend`): Webdashboard Bun/Playwright test containers.

---

## 3. Health, Liveness & Readiness Response Contract

### 3.1 Liveness Probe (`GET /api/v1/health`)
- **Citation**: `apiserver/src/handlers/common.rs:515`
- **HTTP Status Code**: `200 OK`
- **Response Headers**: `Content-Type: application/json`
- **Response Payload**:
  ```json
  {
    "message": "API Server is running!"
  }
  ```
- **Operational Semantic**: In-memory liveness probe only. **Must never query PostgreSQL or Redis**. Returns HTTP 200 immediately if the HTTP listener is accepting connections.

### 3.2 Readiness Probe (`GET /api/v1/health/ready`)
- **Citation**: `apiserver/src/handlers/common.rs:522`
- **HTTP Status Code**:
  - `200 OK`: All mandatory dependencies healthy (`has_error = false`).
  - `500 Internal Server Error`: Any dependency failed (`has_error = true`).
- **Response Headers**: `Content-Type: application/json`
- **Response Payload Schema**:
  ```json
  {
    "status": "healthy",
    "version": "0.1.0",
    "uptime_seconds": 1842,
    "checks": {
      "database": "ok",
      "redis": "ok"
    }
  }
  ```
- **Failure Payload Schema**:
  ```json
  {
    "status": "unhealthy",
    "version": "0.1.0",
    "uptime_seconds": 1842,
    "checks": {
      "database": "error",
      "redis": "error"
    }
  }
  ```
- **Dependency Check Logic**:
  1. `database`: Obtains a connection from API pool (`state.infra().diesel().api.get()`) and executes `SELECT 1` (`common.rs:534`). Evaluates to `"ok"` or `"error"`.
  2. `redis`: If Redis is configured, executes `redis.health_check()` (`PING`, `common.rs:549`). Evaluates to `"ok"`, `"error"`, or `"disabled"`.
  3. `status`: `"healthy"` if both checks succeed; `"unhealthy"` if either check fails.

### 3.3 Infrastructure Monitoring Health (`GET /api/v1/system/health`)
- **Citation**: `apiserver/src/handlers/system_admin.rs:395`
- **Access Control**: Protected by `SystemContext` checking `PlatformAdmin` or `Support` role.
- **Response Payload**:
  ```json
  {
    "status": "ok",
    "viewer_user_id": "123e4567-e89b-12d3-a456-426614174000",
    "viewer_roles": ["PlatformAdmin"]
  }
  ```

---

## 4. Resource Limits & Sizing Specifications

### 4.1 Production Limits & Allocation

| Resource | Service | Specification | Implementation / Citation |
|---|---|---|---|
| **Database Shared Memory** | `db_system`, `db_workspace` | `542MB` (`shm_size: 512mb`) | `.compose/docker-compose.test.yml:47, 69` |
| **PostgreSQL Max Connections** | Database Engine | `1000` (Test), `200` (Dev) | `docker-compose.test.yml:41`, `docker-compose.dev.yml:6` |
| **PgBouncer Client Connections**| `pgbouncer_tx` / `sess` | `2000` (`MAX_CLIENT_CONN`) | `docker-compose.yml:93, 121, 148, 176` |
| **PgBouncer Default Pool Size**| `pgbouncer_tx` / `sess` | `50` (`DEFAULT_POOL_SIZE`) | `docker-compose.yml:94, 122, 149, 177` |
| **PgBouncer Min Pool Size** | `pgbouncer_tx` / `sess` | `10` (`MIN_POOL_SIZE`) | `docker-compose.yml:95, 123, 150, 178` |
| **PgBouncer Reserve Pool Size**| `pgbouncer_tx` / `sess` | `5` (`RESERVE_POOL_SIZE`) | `docker-compose.yml:96, 124, 151, 179` |
| **PgBouncer Max Prepared Stmts**| `pgbouncer_tx` | `250` (`MAX_PREPARED_STATEMENTS`)| `docker-compose.yml:98, 153` |
| **Global Request Body Cap** | API Server | `2MB` (`MAX_REQUEST_BODY_BYTES`) | `apiserver/src/lib.rs:347-353` |
| **File Upload Body Cap** | API Server | `20MB` (`MAX_FILE_UPLOAD_BODY_BYTES`)| `apiserver/src/routes/files.rs:12` |
| **Bulk Import Body Cap** | API Server | `10MB` (`MAX_BULK_UPLOAD_BODY_BYTES`)| `apiserver/src/routes/attendance.rs:14` |
| **Worker Concurrency** | API Background Worker | `5` jobs (`WORKER_MAX_CONCURRENT_JOBS`)| `apiserver/src/worker/runner.rs:134` |
| **Recommended Container Limits**| Go API Server | `1-2 vCPU`, `512MB-1GB RAM` | `GOMEMLIMIT=768MiB` recommended |

---

## 5. Certificates & TLS Termination

### 5.1 Development Architecture (`Caddyfile-dev`)
- **Port**: Listens on `:80` inside container, mapped to host port `${DEV_PORT:-8080}` (`docker-compose.dev.yml:147`).
- **Proxy Rules**:
  - `/*` $\to$ reverse proxy to `webdashboard:5173` (Vite dev server with HMR).
  - `/api/*` $\to$ reverse proxy to `apiserver:3000` (Rust API).
  - Error pages: 50x rewritten to `/50x.html` (`Caddyfile-dev:13-19`).

### 5.2 Production Architecture (`webdashboard/Caddyfile`)
- **Edge Termination**: TLS is terminated upstream at Cloudflare Edge (`.env.production:1-5`).
- **Container Caddy**: Serves static React assets from `/srv` and proxies `/api/*` to `apiserver:3000`.
- **Security Headers Enforced** (`webdashboard/Caddyfile:3-10`):
  - `Strict-Transport-Security`: `max-age=31536000; includeSubDomains; preload`
  - `X-Frame-Options`: `SAMEORIGIN`
  - `X-Content-Type-Options`: `nosniff`
  - `X-XSS-Protection`: `0`
  - `Referrer-Policy`: `strict-origin-when-cross-origin`
  - `Content-Security-Policy`: restrictive policy permitting websocket connections (`connect-src 'self' ws: wss:`).

---

## 6. Service Identities & Authentication Boundaries

1. **System Database Principal**:
   - Username: `${DB_USER}`
   - Authentication: SCRAM-SHA-256 (`docker-compose.yml:100`).
   - Scope: System tables (`users`, `workspaces`, `system_settings`, `authorized_agents`, `audit_logs`).
2. **Tenant Database Principal**:
   - Username: `tenant_user` (`docker-compose.yml:40`).
   - Authentication: SCRAM-SHA-256.
   - Scope: Workspace tenant databases (`employees`, `attendances`, `shifts`, `leaves`, `payrolls`).
3. **Agent Terminal Identity**:
   - Identifier: `agent_id` (UUID format, `docker-compose.yml:293`).
   - Secret: `AGENT_TOKEN` passed via `x-token` header, verified against database `authorized_agents.token_hash` via SHA-256 (`apiserver/src/crypto.rs:151`).
4. **Container Process Identity**:
   - Runtime UID: `65532:65532` (`nonroot:nonroot` in distroless, `apiserver/Dockerfile:79`).

---

## 7. Parallel Rust + Go Coexistence & Collision Avoidance

To enable seamless canary testing, differential verification, and zero-downtime rollback, Rust and Go services run concurrently without port or resource contention.

### 7.1 Port & Service Collision Risks & Mitigations

| Risk Vector | Description | Severity | Concrete Mitigation |
|---|---|---|---|
| **Host / Internal Port Collision** | Both processes attempting to bind `0.0.0.0:3000` | **High** | • Rust binds port `3000` (`APP_PORT=3000`).<br>• Go binds port `3001` (`APP_PORT=3001`, defined in `leedevkit.toml:33`).<br>• Both use dynamic port binding fallback: inspects preferred port, tests `port..port+10`, writes actual port to `.api_port` (Rust) or `.api_go_port` (Go) (`main.rs:219-238`). |
| **Compose Service Collision** | Conflicting container names in Docker/Podman network | **High** | • Rust container service name: `apiserver` (image `localhost/leeattend_apiserver`).<br>• Go container service name: `api-go` or `apiserver_go` (image `localhost/leeattend_apiserver_go`). |
| **PostgreSQL Pool Starvation** | Running both servers doubles backend connections, exhausting Postgres `max_connections` | **High** | • Both services connect strictly via **PgBouncer** (`pgbouncer_tx:6543`, `pgbouncer_tenant_tx:6543`).<br>• PgBouncer pools transactions into max 50 backend connections, insulating PostgreSQL from client pool counts. |
| **Redis Job Queue Double-Claim** | Both Rust and Go background workers dequeuing `queue:jobs` simultaneously | **Critical** | • During Phase 4-5 API migration, background worker in Go is disabled via `WORKER_MAX_CONCURRENT_JOBS=0`.<br>• When testing Go worker parity, Rust worker is stopped, or separate queue namespaces are used (`queue:jobs:go`). |
| **Filesystem Storage Overwrite** | Both writing temporary files into `/app/storage/imports/` | **Low** | • File generator uses UUID prefixes (`{uuid}_{sanitized_filename}`, `key_generator.rs:37`), guaranteeing unique file paths. |
| **Agent WebSocket Stealing** | Agent connects to Rust, but command sent from Go | **Medium** | • Agent connection routes to single endpoint via Caddy until cutover. In canary mode, Caddy sticky routes `/api/v1/ws` to Rust until agent protocol is certified on Go. |

### 7.2 Incremental Route Cutover via Caddy
Caddy supports granular path routing for domain-by-domain cutover:

```caddy
# Caddyfile cutover definition (Phase 6 canary deployment)
:80 {
    # 1. Fully migrated domain routes cut over to Go API Server (Port 3001)
    handle /api/v1/workspaces/* {
        reverse_proxy api-go:3001
    }
    handle /api/v1/system/* {
        reverse_proxy api-go:3001
    }

    # 2. Unmigrated domain routes retain baseline Rust API Server (Port 3000)
    handle /api/* {
        reverse_proxy apiserver:3000
    }

    # 3. Frontend SPA static files & Vite proxy
    handle /* {
        reverse_proxy webdashboard:5173
    }
}
```

### 7.3 Immediate Rollback Protocol
- **Trigger**: Any 5xx anomaly, p99 latency spike, or database integrity mismatch in Go service.
- **Action**: Update Caddy proxy target from `api-go:3001` back to `apiserver:3000` and execute `caddy reload`.
- **Zero Schema Rollback**: Because Go runs strictly on top of existing PostgreSQL tables and PgBouncer poolers, rollback is instantaneous with zero data migration or schema alteration.
