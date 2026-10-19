# Rust apiserver Decommission & Retirement Record

**Date:** 2026-10-20  
**Phase:** Phase P9 (P9-05, P9-07, P9-08)  
**Status:** **RETIRED & DECOMMISSIONED**  
**Successor Service:** Enterprise Go API Server (`github.com/vkaylee/leeattend/apiserver`)  

---

## 1. Decommission Summary

The reference Rust `apiserver` (Axum/Tokio/Diesel) has reached end-of-life and is decommissioned from production service following the completion and verification of the enterprise Go API server.

| Dimension | Legacy Rust apiserver | Successor Go apiserver | Retirement Status |
|---|---|---|---|
| **Primary Binary** | `target/release/apiserver` | `bin/apiserver` | Binary generation retired |
| **Container Image** | `localhost/leeattend_apiserver:staging` (Rust runtime, 54.2MB) | `localhost/leeattend_apiserver` (distroless Go static, 38.4MB) | Rust image retired |
| **Ingress Port** | `:3000` | `:3000` | Traffic cut over 100% to Go |
| **Worker Queue** | Consumed `worker:jobs:*` | Consumes `worker:jobs:*` (`WORKER_MAX_CONCURRENT_JOBS=5`) | Background worker queue transferred |
| **Database Migrations** | `apiserver/migrations/` | `apiserver/migrations/` | Shared migrations retained in-tree |

---

## 2. Ingress Traffic Cutover

- **Reverse Proxy:** Edge gateway (`staging/caddy/Caddyfile`) directs 100% of `/api/*` traffic to Go runtime (`api-go:3001` in staging, `apiserver:3000` in production).
- **Canary Weights:** `staging/caddy/canary_weights.conf` set to `100% Go`.
- **Domain Cutover:** All 9 functional domains routed to Go in `staging/caddy/canary_domains.conf`.
- **Active Connections:** All keep-alive client connections to the Rust service were drained over a 30-second window prior to SIGTERM.

---

## 3. Worker Queue Transition

- **Queue Draining:** Influx of jobs to Rust background worker halted. Active report export jobs and bulk employee imports were permitted to finish cleanly.
- **Go Activation:** `WORKER_MAX_CONCURRENT_JOBS` on Go service increased from 0 (canary isolation mode) to 5 (production mode).
- **Split-Brain Prevention:** Queue processing is exclusively handled by Go; Redis keys `worker:jobs:*` and `report:meta:*` are serviced without interference.

---

## 4. Configuration Cutover

1. **`leedevkit.toml`:**
   - `services.apiserver` switched from `lang = "rust"`, `cargo = true` to `lang = "go"`, `go = true`, `workdir = "apiserver"`.
   - `targets.api` and `targets.all` point to Go apiserver.
   - `test.dependencies` updated for Go test profiles.
2. **`docker-compose.yml`:**
   - Service `apiserver` builds from `apiserver/Dockerfile.go` (target: `runtime`).
   - Standard Go environment contract injected (`APP_PORT=3000`, `APP_HOST=0.0.0.0`, `DB_PRIMARY_URL`, `DB_POOLER_TX_URL`).
3. **`docker-compose.staging.yml`:**
   - Legacy Rust container `apiserver` moved to `profiles: ["legacy-retired"]`.

---

## 5. Source Tree Cleanup & Retention Boundary

- **`apiserver/src/`, `apiserver/tests/`, `apiserver/Cargo.*`:** Completely removed from active tree after 100% 1-1 verification (403/403 endpoints covered, all domain packages in-process). Historical code preserved in git commits.
- **Root `Cargo.toml`:** `members` updated to retain exclusively `agent-main`.
- **`apiserver/migrations/`:** Active and shared across all services; PostgreSQL migrations remain authoritative.
- **`agent-main/`:** Active; native Rust hardware orchestrator communicating with biometric devices via `rustzk` remains in Rust.
