# Phase 1 — Configuration & Secrets Contract

> Task ID: `P1-09`  
> Status: Completed  
> Canonical SSOT: Rust `apiserver/src/utils/env_registry.rs` (`ALL_ENV_VARS`)  
> Target: Go `internal/config/`  
> **Security Notice**: No secret values are contained in this document. Secret settings are documented by name, sensitivity, and validation contract only.

---

## 1. Governance & Startup Fail-Closed Protocol

The LeeAttend API Server enforces a **strict fail-closed configuration gate** at process startup before opening network sockets, running migrations, or establishing database pools.

### 1.1 Resolution Sequence (`main.rs:66-82`)
1. **Registry Resolution**: `EnvRegistry::resolve()` executes before any logging framework or database connection.
2. **Primary Key vs Aliases**: For each defined variable, the primary key is checked first, followed by declared aliases in order, and then the default value.
3. **Mandatory Fail-Fast Validation**:
   - Variables marked `required_always = true` (`DB_PRIMARY_URL`, `DB_POOLER_TX_URL`, `DB_POOLER_SESS_URL`, `REDIS_URL`) must resolve in all environments (`development`, `test`, `production`). If any are missing, the process outputs a formatted failure block to `stderr` and terminates with exit code `1` (`main.rs:81`).
   - In `production` mode (`APP_ENV=production`), variables marked `required_in_prod = true` (`JWT_SECRET`, `ENCRYPTION_KEYS_JSON`, `ALLOWED_ORIGINS`, `S3_ENDPOINT_URL`, `S3_BUCKET_NAME`, `S3_ACCESS_KEY`, `S3_SECRET_KEY`) must resolve. Startup aborts if any are missing (`env_registry.rs:191-205`, `config.rs:47-76`).
4. **Format & Type Validation**: Numerical bounds, boolean syntax, connection URL scheme prefixes (`postgres://` / `postgresql://`), and JSON structure are validated before connection initialization (`diesel_pool/config.rs:122`).
5. **Redacted Startup Summary**: Safe public settings are emitted to logs, while secrets (`Sensitivity::Secret`) and password-bearing connection strings (`Sensitivity::Connection`) are masked (`env_registry.rs:43-53, 243-358`).

---

## 2. Canonical Environment Variable Inventory

Total Canonical Env Vars in Registry: **47**  
Total Container / Deployment Env Vars: **17**  

### 2.1 Core Category

| Env Var Name | Aliases | Required | Default | Type / Bounds | Sensitivity | Fail-Closed Rule | Code Citation (`path:line`) | Config Citations |
|---|---|---|---|---|---|---|---|---|
| `APP_ENV` | `API_ENV` | Optional | `"development"` | Enum: `development`, `production`, `test`, `testing` | Public | None; falls back to `"development"` | `apiserver/src/utils/env_registry.rs:536`, `apiserver/src/utils/config.rs:21` | `.env.example:12` |
| `APP_PORT` | `API_PORT` | Optional | `3000` | Integer: `u16` ($1 \le P \le 65535$) | Public | None; falls back to `3000`. If port in use, searches next 10 ports | `apiserver/src/utils/env_registry.rs:546`, `apiserver/src/main.rs:219` | `.env.example:16`, `docker-compose.yml:204` |
| `APP_HOST` | `API_HOST` | Optional | `"0.0.0.0"` | String: IPv4, IPv6, or Hostname | Public | None; falls back to `"0.0.0.0"` | `apiserver/src/utils/env_registry.rs:556`, `apiserver/src/utils/config.rs:28` | `.env.example:20`, `docker-compose.yml:203` |
| `PORT_FILE_NAME` | *(None)* | Optional | `".api_port"` | String: filesystem file path | Public | None; falls back to `".api_port"` | `apiserver/src/utils/env_registry.rs:566`, `apiserver/src/main.rs:212` | `.env.example:23` |

### 2.2 Database Category

| Env Var Name | Aliases | Required | Default | Type / Bounds | Sensitivity | Fail-Closed Rule | Code Citation (`path:line`) | Config Citations |
|---|---|---|---|---|---|---|---|---|
| `DB_PRIMARY_URL` | `DB_URL`, `DATABASE_URL` | **Always** | *(None)* | Connection string: `postgres://` or `postgresql://` | Connection | **Fatal abort**: Missing or invalid prefix aborts startup immediately | `apiserver/src/utils/env_registry.rs:580`, `apiserver/src/diesel_pool/config.rs:36` | `.env.example:29`, `docker-compose.yml:194` |
| `DB_POOLER_TX_URL` | `DATABASE_POOLER_TX_URL` | **Always** | *(None)* | Connection string: `postgres://` port `6543` | Connection | **Fatal abort**: Missing or invalid prefix aborts startup; Supavisor transaction mode enforced | `apiserver/src/utils/env_registry.rs:590`, `apiserver/src/diesel_pool/config.rs:47` | `.env.example:33`, `docker-compose.yml:195` |
| `DB_POOLER_SESS_URL` | `DATABASE_POOLER_SESS_URL` | **Always** | *(None)* | Connection string: `postgres://` port `5452` | Connection | **Fatal abort**: Missing or invalid prefix aborts startup; Supavisor session mode enforced | `apiserver/src/utils/env_registry.rs:600`, `apiserver/src/diesel_pool/config.rs:58` | `.env.example:37`, `docker-compose.yml:196` |
| `DB_TENANT_POOLER_TX_URL` | `DATABASE_TENANT_POOLER_TX_URL` | Optional | Fallback to `DB_POOLER_TX_URL` | Connection string: `postgres://` port `6543` | Connection | Aborts if set and prefix does not start with `postgres://` | `apiserver/src/utils/env_registry.rs:610`, `apiserver/src/diesel_pool/config.rs:69` | `.env.example:41`, `docker-compose.yml:197` |
| `DB_TENANT_POOLER_SESS_URL` | `DATABASE_TENANT_POOLER_SESS_URL` | Optional | Fallback to `DB_POOLER_SESS_URL` | Connection string: `postgres://` port `5452` | Connection | Aborts if set and prefix does not start with `postgres://` | `apiserver/src/utils/env_registry.rs:620`, `apiserver/src/diesel_pool/config.rs:76` | `.env.example:45`, `docker-compose.yml:198` |
| `DB_READER_URL` | `DATABASE_READER_URL` | Optional | Fallback to `DB_POOLER_TX_URL` | Connection string: `postgres://` | Connection | Aborts if set and prefix does not start with `postgres://` | `apiserver/src/utils/env_registry.rs:630`, `apiserver/src/diesel_pool/config.rs:83` | `.env.example:49` |
| `DB_API_MAX_CONN` | *(None)* | Optional | `20` | Integer: `u32` ($1 \le N \le 1000$) | Public | None; falls back to `20` | `apiserver/src/utils/env_registry.rs:640`, `apiserver/src/diesel_pool/config.rs:98` | `.env.example:52`, `docker-compose.yml:200` |
| `DB_API_ACQUIRE_TIMEOUT_SECS` | *(None)* | Optional | `5` | Integer: `u64` seconds | Public | None; falls back to `5` | `apiserver/src/utils/env_registry.rs:650`, `apiserver/src/diesel_pool/config.rs:99` | `.env.example:55` |
| `DB_WORKER_ACQUIRE_TIMEOUT_SECS` | *(None)* | Optional | `30` | Integer: `u64` seconds | Public | None; falls back to `30` | `apiserver/src/utils/env_registry.rs:660`, `apiserver/src/diesel_pool/config.rs:103` | `.env.example:58` |
| `DB_TENANT_MAX_CONN` | *(None)* | Optional | `5` | Integer: `u32` ($1 \le N \le 100$) | Public | None; falls back to `5` per tenant | `apiserver/src/utils/env_registry.rs:670`, `apiserver/src/diesel_pool/config.rs:106` | `.env.example:61` |
| `PGBOUNCER_HOST` | `DATABASE_HOST` | Optional | `"localhost"` | String: hostname or IP | Public | None; falls back to `"localhost"` | `apiserver/src/utils/env_registry.rs:680`, `apiserver/src/handlers/system_tenants.rs:87` | `.env.example:65` |
| `PGBOUNCER_PORT` | `PGPORT` | Optional | `5432` | Integer: `u16` ($1 \le P \le 65535$) | Public | None; falls back to `5432` | `apiserver/src/utils/env_registry.rs:690`, `apiserver/src/handlers/system_tenants.rs:94` | `.env.example:69` |

### 2.3 Redis Category

| Env Var Name | Aliases | Required | Default | Type / Bounds | Sensitivity | Fail-Closed Rule | Code Citation (`path:line`) | Config Citations |
|---|---|---|---|---|---|---|---|---|
| `REDIS_URL` | *(None)* | **Always** | *(None)* | Connection string: `redis://` or `rediss://` | Connection | **Fatal abort**: Missing var or unreachable server (5s timeout) terminates process | `apiserver/src/utils/env_registry.rs:704`, `apiserver/src/main.rs:201, 314` | `.env.example:74`, `docker-compose.yml:202` |

### 2.4 Security Category

| Env Var Name | Aliases | Required | Default | Type / Bounds | Sensitivity | Fail-Closed Rule | Code Citation (`path:line`) | Config Citations |
|---|---|---|---|---|---|---|---|---|
| `JWT_SECRET` | *(None)* | **In Prod** | Dev: `"dev_secret_key_leeattend_2026"` | String: $\ge 32$ bytes | **Secret** | **Fatal abort** in production mode if unset | `apiserver/src/utils/env_registry.rs:718`, `apiserver/src/utils/config.rs:47` | `.env.example:79`, `docker-compose.yml:205` |
| `ENCRYPTION_KEYS_JSON` | *(None)* | **In Prod** | Dev dummy key | JSON: `{"keys":{"v1":"..."},"active_key":"v1"}` | **Secret** | **Fatal abort** in production if unset or invalid AES-256-GCM key schema | `apiserver/src/utils/env_registry.rs:728`, `apiserver/src/utils/config.rs:71`, `apiserver/src/app_builder/mod.rs:39` | `.env.example:82`, `docker-compose.yml:210` |
| `ALLOWED_ORIGINS` | *(None)* | **In Prod** | Dev: `"http://localhost:5173,http://localhost:3000,http://127.0.0.1:5173,http://127.0.0.1:3000"` | Comma-separated URI list | Public | **Fatal abort** in production if unset; invalid origins ignored by parser | `apiserver/src/utils/env_registry.rs:738`, `apiserver/src/lib.rs:262-275`, `apiserver/src/utils/config.rs:59` | `.env.example:85`, `docker-compose.yml:206` |
| `COOKIE_DOMAIN` | *(None)* | Optional | *(None)* | String: e.g. `".vlee.dev"` | Public | None; defaults to host domain if omitted | `apiserver/src/utils/env_registry.rs:748` | `.env.example:88`, `docker-compose.yml:207` |
| `MAX_REQUEST_BODY_BYTES` | *(None)* | Optional | `2097152` (2MB) | Integer: bytes (`usize`) | Public | Global body limit enforced via Axum layer (`DefaultBodyLimit::max`) | `apiserver/src/utils/env_registry.rs:758`, `apiserver/src/lib.rs:347` | `.env.example:91` |
| `MAX_FILE_UPLOAD_BODY_BYTES` | *(None)* | Optional | `20971520` (20MB) | Integer: bytes (`usize`) | Public | Route layer cap on `/files/upload` | `apiserver/src/utils/env_registry.rs:768`, `apiserver/src/routes/files.rs:12` | `.env.example:94` |
| `MAX_BULK_UPLOAD_BODY_BYTES` | *(None)* | Optional | `10485760` (10MB) | Integer: bytes (`usize`) | Public | Route layer cap on bulk Excel routes (`/attendance/upload`, `/employees/upload`, etc.) | `apiserver/src/utils/env_registry.rs:778`, `apiserver/src/routes/attendance.rs:14` | `.env.example:97` |

### 2.5 Storage Category (S3 / MinIO)

| Env Var Name | Aliases | Required | Default | Type / Bounds | Sensitivity | Fail-Closed Rule | Code Citation (`path:line`) | Config Citations |
|---|---|---|---|---|---|---|---|---|
| `S3_ENDPOINT_URL` | *(None)* | **In Prod** | Dev: `"http://localhost:9000"` | String: valid HTTP/HTTPS URL | Public | **Fatal abort** in production mode if missing | `apiserver/src/utils/env_registry.rs:792`, `apiserver/src/services/storage/s3_storage.rs:30` | `.env.example:102`, `docker-compose.yml:211` |
| `S3_BUCKET_NAME` | *(None)* | **In Prod** | Dev: `"leeattend"` | String: bucket identifier | Public | **Fatal abort** in production mode if missing | `apiserver/src/utils/env_registry.rs:802`, `apiserver/src/services/storage/s3_storage.rs:35` | `.env.example:105`, `docker-compose.yml:212` |
| `S3_ACCESS_KEY` | *(None)* | **In Prod** | Dev: `"minioadmin"` | String: access key ID | **Secret** | **Fatal abort** in production mode if missing | `apiserver/src/utils/env_registry.rs:812`, `apiserver/src/services/storage/s3_storage.rs:40` | `.env.example:108`, `docker-compose.yml:213` |
| `S3_SECRET_KEY` | *(None)* | **In Prod** | Dev: `"minioadmin"` | String: secret access key | **Secret** | **Fatal abort** in production mode if missing | `apiserver/src/utils/env_registry.rs:822`, `apiserver/src/services/storage/s3_storage.rs:45` | `.env.example:111`, `docker-compose.yml:214` |
| `S3_REGION` | *(None)* | Optional | `"us-east-1"` | String: AWS region identifier | Public | None; falls back to `"us-east-1"` | `apiserver/src/utils/env_registry.rs:832`, `apiserver/src/services/storage/s3_storage.rs:50` | `.env.example:114`, `docker-compose.yml:215` |
| `S3_USE_PATH_STYLE` | *(None)* | Optional | `"true"` | Boolean: `true` / `false` | Public | None; defaults to `true` (required for MinIO path addressing) | `apiserver/src/utils/env_registry.rs:842`, `apiserver/src/services/storage/s3_storage.rs:55` | `.env.example:117`, `docker-compose.yml:216` |
| `FILE_PROXY_THRESHOLD_BYTES` | *(None)* | Optional | `1048576` (1MB) | Integer: bytes (`i64`) | Public | Files larger than this use S3 presigned URLs; smaller files proxied via API server | `apiserver/src/utils/env_registry.rs:852`, `apiserver/src/services/storage/file_service.rs:33` | `.env.example:120`, `docker-compose.yml:217` |
| `PRESIGNED_URL_TTL_SECS` | *(None)* | Optional | `900` (15 min) | Integer: seconds (`u64`) | Public | None; falls back to `900` | `apiserver/src/utils/env_registry.rs:862`, `apiserver/src/services/storage/file_service.rs:36` | `.env.example:123`, `docker-compose.yml:218` |
| `STORAGE_DIR` | *(None)* | Optional | `"storage"` | String: filesystem path | Public | None; falls back to `"storage"` (local dev) or `"/app/storage"` (container) | `apiserver/src/utils/env_registry.rs:872`, `apiserver/src/utils/mod.rs:20` | `.env.example:126`, `docker-compose.yml:209` |

### 2.6 Worker Category

| Env Var Name | Aliases | Required | Default | Type / Bounds | Sensitivity | Fail-Closed Rule | Code Citation (`path:line`) | Config Citations |
|---|---|---|---|---|---|---|---|---|
| `WORKER_MAX_CONCURRENT_JOBS` | *(None)* | Optional | `5` | Integer: `usize` ($1 \le N \le 100$) | Public | None; falls back to `5` | `apiserver/src/utils/env_registry.rs:886`, `apiserver/src/worker/runner.rs:134` | `.env.example:131`, `docker-compose.yml:201` |
| `WORKER_POLL_TIMEOUT_SECS` | *(None)* | Optional | `30` | Float: `f64` ($0.0 < T \le 300.0$) | Public | None; clamped to $[0.1, 300.0]$, falls back to `30.0` | `apiserver/src/utils/env_registry.rs:896`, `apiserver/src/worker/runner.rs:138` | `.env.example:134` |
| `WORKER_MAX_RETRIES` | *(None)* | Optional | `3` | Integer: `u32` ($0 \le N \le 20$) | Public | Exhausted retries move job to dead-letter queue (DLQ) | `apiserver/src/utils/env_registry.rs:906`, `apiserver/src/worker/runner.rs:143` | `.env.example:137` |
| `WORKER_JOB_TIMEOUT_SECS` | *(None)* | Optional | `300` (5 min) | Integer: seconds (`u64`) | Public | Active job exceeding timeout is cancelled and released | `apiserver/src/utils/env_registry.rs:916`, `apiserver/src/worker/runner.rs:147` | `.env.example:140` |
| `IMPORT_CHUNK_SIZE` | *(None)* | Optional | `500` | Integer: rows (`usize`) | Public | None; falls back to `500` | `apiserver/src/utils/env_registry.rs:926`, `apiserver/src/worker/handlers/utils.rs:72` | `.env.example:143` |
| `PAIRING_SWEEP_INTERVAL_SECS` | *(None)* | Optional | `2` | Integer: seconds (`u64`) | Public | None; falls back to `2` | `apiserver/src/utils/env_registry.rs:936`, `apiserver/src/worker/pairing_scheduler.rs:111` | `.env.example:146` |

### 2.7 Rate Limiting Category

| Env Var Name | Aliases | Required | Default | Type / Bounds | Sensitivity | Fail-Closed Rule | Code Citation (`path:line`) | Config Citations |
|---|---|---|---|---|---|---|---|---|
| `RATE_LIMIT_MAX_READ` | *(None)* | Optional | `120` | Integer: requests (`u64`) | Public | None; falls back to `120` requests / window | `apiserver/src/utils/env_registry.rs:950`, `apiserver/src/mw/rate_limit.rs:46` | `.env.example:151` |
| `RATE_LIMIT_MAX_WRITE` | *(None)* | Optional | `30` | Integer: requests (`u64`) | Public | None; falls back to `30` requests / window | `apiserver/src/utils/env_registry.rs:960`, `apiserver/src/mw/rate_limit.rs:49` | `.env.example:154` |
| `RATE_LIMIT_WINDOW_SECS` | *(None)* | Optional | `60` | Integer: seconds (`u64`) | Public | None; falls back to `60` seconds | `apiserver/src/utils/env_registry.rs:970`, `apiserver/src/mw/rate_limit.rs:52` | `.env.example:157` |

### 2.8 Virus Scanning Category

| Env Var Name | Aliases | Required | Default | Type / Bounds | Sensitivity | Fail-Closed Rule | Code Citation (`path:line`) | Config Citations |
|---|---|---|---|---|---|---|---|---|
| `CLAMD_HOST` | *(None)* | Optional | *(None)* | String: hostname or IP | Public | If unset, ClamAV is disabled and `MockScanner` is used (clean return) | `apiserver/src/utils/env_registry.rs:984`, `apiserver/src/services/storage/scanner/mod.rs:27` | `.env.example:162` |
| `CLAMD_PORT` | *(None)* | Optional | `3310` | Integer: `u16` ($1 \le P \le 65535$) | Public | None; falls back to `3310` | `apiserver/src/utils/env_registry.rs:994`, `apiserver/src/services/storage/scanner/mod.rs:29` | `.env.example:165` |

### 2.9 Integration Category

| Env Var Name | Aliases | Required | Default | Type / Bounds | Sensitivity | Fail-Closed Rule | Code Citation (`path:line`) | Config Citations |
|---|---|---|---|---|---|---|---|---|
| `SUPAVISOR_API_URL` | *(None)* | Optional | *(None)* | String: valid HTTP/HTTPS URL | Public | Required if dynamic tenant DB registration is executed | `apiserver/src/utils/env_registry.rs:1008`, `apiserver/src/utils/i18n.rs:81` | `.env.example:170` |
| `API_JWT_SECRET` | `SUPAVISOR_API_JWT` | Optional | *(None)* | String: JWT signing secret | **Secret** | Required if dynamic tenant DB registration is executed | `apiserver/src/utils/env_registry.rs:1018`, `apiserver/src/utils/i18n.rs:82` | `.env.example:174` |

### 2.10 Feature Flags Category

| Env Var Name | Aliases | Required | Default | Type / Bounds | Sensitivity | Fail-Closed Rule | Code Citation (`path:line`) | Config Citations |
|---|---|---|---|---|---|---|---|---|
| `FF_NO_SHOW_TIMELINE` | *(None)* | Optional | `"false"` | Boolean: `true` / `false` | Public | Emergency kill-switch for no-show scrubber; DB flag takes priority | `apiserver/src/utils/env_registry.rs:1032` | `.env.example:179` |

---

## 3. Ancillary & Container-Level Variables

These settings are consumed by the runtime container orchestration, Caddy, dev tools, and adjacent services:

| Variable Name | Component | Required | Purpose | Sensitivity | Source / Citation |
|---|---|---|---|---|---|
| `RUST_LOG` | Logging Filter | Optional | Controls tracing log levels (`info`, `debug`, `tower_http=debug`) | Public | `main.rs:91`, `docker-compose.yml:208` |
| `RUST_BACKTRACE` | Crash Debug | Optional | Set to `1` in development to print stack traces | Public | `.compose/docker-compose.dev.yml:80` |
| `DB_POOLER_TYPE` | Routing Tag | Optional | Indicates pooler type (`pgbouncer` or `supavisor`) | Public | `docker-compose.yml:199`, `.compose/docker-compose.test.yml:272` |
| `DEV_PORT` | Local Dev | Optional | Host port exposed for Caddy dev proxy (default `8080`) | Public | `.compose/docker-compose.dev.yml:147` |
| `REDIS_PASSWORD` | Redis Service | **Required in Prod** | Auth password configured on Redis container (`requirepass`) | **Secret** | `docker-compose.yml:72` |
| `POSTGRES_DB` | Database Service | Required | Name of initial PostgreSQL database created by image | Public | `docker-compose.yml:24` |
| `POSTGRES_USER` | Database Service | Required | Root/admin database user name | Public | `docker-compose.yml:25` |
| `POSTGRES_PASSWORD` | Database Service | **Required in Prod** | Root/admin database password | **Secret** | `docker-compose.yml:26` |
| `MINIO_ROOT_USER` | MinIO Service | Required | MinIO administrative access key (default `leeattend`) | Public | `docker-compose.yml:256` |
| `MINIO_ROOT_PASSWORD` | MinIO Service | **Required in Prod** | MinIO administrative secret key | **Secret** | `docker-compose.yml:257` |
| `AGENT_CENTRAL_SERVER_URL` | Agent IoT Daemon | Required | WebSocket endpoint URL (e.g. `ws://webdashboard:80/api/ws`) | Public | `docker-compose.yml:291` |
| `AGENT_DATABASE_URL` | Agent IoT Daemon | Required | SQLite local database path (e.g. `sqlite:///app/data/agent.db`) | Public | `docker-compose.yml:292` |
| `AGENT_ID` | Agent IoT Daemon | Required | Unique agent identity UUID matching `authorized_agents` | Public | `docker-compose.yml:293` |
| `AGENT_TOKEN` | Agent IoT Daemon | Required | Shared secret token verified via SHA-256 hash on server | **Secret** | `docker-compose.yml:294` |
| `MAX_MEMORY_LOGS` | Agent IoT Daemon | Optional | Max in-memory queue size for offline terminal punch logs | Public | `docker-compose.yml:295` |
| `CLOUDFLARE_API_TOKEN` | Webdashboard CD | Required in Prod | Cloudflare API token for dashboard edge deployment | **Secret** | `.env.production:1` |
| `VITE_API_URL` | Webdashboard Build | Required | Base API URL baked into React bundle (`https://attapi.vlee.dev/api`) | Public | `.env.production:5`, `docker-compose.dev.yml:126` |

---

## 4. Go Migration Implementation Requirements

1. **Package Location**: `internal/config/`
2. **Fail-Closed Loading**:
   - Create a typed `Config` struct with exact field types matching the table above.
   - Implement `Load() (*Config, error)` that verifies all `required_always` variables and, if `APP_ENV == "production"`, all `required_in_prod` variables.
   - Return multi-error list identifying all missing or malformed variables on startup.
3. **Secret Protection**:
   - Types holding secrets (`JWTSecret`, `EncryptionKeysJSON`, `S3SecretKey`, `RedisURL`) must implement `fmt.Stringer` and `fmt.GoStringer` returning `"[REDACTED]"`.
4. **Key Rotation Readiness**:
   - `ENCRYPTION_KEYS_JSON` supports multiple versioned keys (`v1`, `v2`) with `active_key` specified in JSON. Go implementation must match this exact rotation structure.
