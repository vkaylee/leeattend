# Phase 1 — External Dependency & Integration Inventory

> Task ID: `P1-08`  
> Status: Completed  
> Reference Implementation: Rust `apiserver/src/`  
> Target Implementation: Go `apiserver/`  

---

## 1. Overview & Architecture Boundaries

The `apiserver` component interfaces with external infrastructure services, third-party network boundaries, local filesystems, and stateful socket connections. Every boundary listed below must be replicated in Go with identical timeout guarantees, retry semantics, credential resolution, error mapping, and cleanup rules.

| Integration | Type | Protocol | Rust Symbol (`path:line`) | Default Timeout | Retry Policy | Auth Mechanism |
|---|---|---|---|---|---|---|
| **S3 Object Storage** | Storage | HTTP/REST (AWS S3 SDK) | `apiserver/src/services/storage/s3_storage.rs:20` | **UNKNOWN — needs decision** (SDK default not set in Rust) | **UNKNOWN — needs decision** (no application retry configured; SDK behavior not pinned) | Static AWS credentials (`access_key`, `secret_key`) |
| **Workspace S3 Resolver** | Storage | In-memory Cache + SQL | `apiserver/src/services/storage/resolver.rs:11` | N/A (local cache / DB) | None in resolver | Credentials come from DB config; encryption handling **UNKNOWN — needs decision** |
| **ClamAV Daemon** | Security Scan | TCP Socket (clamd) | `apiserver/src/services/storage/scanner/clamav.rs:6` | 10s scan timeout | None; scheduler logs error and leaves file pending | None (TCP daemon) |
| **Mock Scanner** | Security Scan | In-process Fallback | `apiserver/src/services/storage/scanner/mock.rs:6` | 10ms simulated delay | None | None |
| **SMTP Mail Transport** | Notification | SMTP / StartTLS / TLS | `apiserver/src/services/email_service.rs:100` | 10s | None (Redis rate-limited: 5/min, 20/hr, 100/day) | SMTP username/password when both configured |
| **HTTP Email Provider** | Notification | HTTP/REST (reqwest) | `apiserver/src/services/email_service.rs:449` | 10s | None (Redis rate-limited) | Configured auth header/value |
| **Supavisor Admin API** | Multi-tenancy | HTTP/REST | `apiserver/src/utils/env_registry.rs:1009` | **UNKNOWN — needs decision** (no client call found in `apiserver/src`) | **UNKNOWN — needs decision** | Intended JWT setting is registered; active call/auth path **UNKNOWN — needs decision** |
| **E-Contract Dispatcher** | Legal Sign | Provider Interface | `apiserver/src/services/econtract/dispatcher.rs:9` | **UNKNOWN — needs decision** (mock only) | None in current mock | Provider-specific auth is **UNKNOWN — needs decision** |
| **Agent Terminal WebSocket** | IoT / Presence | RFC 6455 WebSocket | `apiserver/src/handlers/agents.rs:66` | Continuous; explicit timeout **UNKNOWN — needs decision** | Reconnect policy is client-owned; server has no retry | Headers `x-agent-id`, `x-token` validated against DB |
| **Report Streaming WebSocket** | Client Stream | RFC 6455 WebSocket | `apiserver/src/handlers/report_websocket.rs:74` | Continuous; explicit timeout **UNKNOWN — needs decision** | Client reconnect; server no retry | JWT in query `?token=` or cookie `access_token` |
| **Worker Monitoring SSE** | Admin Stream | HTTP Server-Sent Events | `apiserver/src/handlers/worker_monitor.rs:2443` | Continuous stream | Client reconnect; server no retry | Bearer JWT + PlatformAdmin / Support role |
| **PostgreSQL Database** | Relational DB | PostgreSQL wire via PgBouncer | `apiserver/src/diesel_pool/config.rs:3` | Pool acquire: 5s API / 30s Worker | Pool library policy **UNKNOWN — needs decision** | Connection URL credentials / PgBouncer auth |
| **Redis Cache & Queue** | Cache & Queue | Redis RESP via multiplexed client | `apiserver/src/queue/redis_client.rs:44` | 5s connection attempt; BLPOP caller timeout | Up to 5 connection attempts (`redis_client.rs:70-93`); no command retry contract | URL credentials / optional `requirepass` |
| **Local Filesystem** | File Storage | POSIX FS | `apiserver/src/utils/mod.rs:20` | Synchronous / tokio::fs | None | OS file permissions |

---

## 2. Integration Deep-Dives

### 2.1 S3 / MinIO Object Storage

- **Rust Symbols**:
  - `S3Config`: `apiserver/src/services/storage/s3_storage.rs:10`
  - `S3Storage`: `apiserver/src/services/storage/s3_storage.rs:20`
  - `ObjectStorage` trait: `apiserver/src/services/storage/mod.rs:25`
  - `FileService`: `apiserver/src/services/storage/file_service.rs:19`
  - `StorageResolver`: `apiserver/src/services/storage/resolver.rs:11`
  - `KeyGenerator`: `apiserver/src/services/storage/key_generator.rs:33`
- **Client Library**:
  - `aws-sdk-s3` v1, `aws-config` v1, `aws-credential-types` v1 (`apiserver/Cargo.toml:57-59`).
  - Go Target: `github.com/aws/aws-sdk-go-v2/service/s3` + `github.com/aws/aws-sdk-go-v2/config`.
- **Configuration & Resolution**:
  - Central Fallback: Configured from env vars (`S3_ENDPOINT_URL`, `S3_BUCKET_NAME`, `S3_ACCESS_KEY`, `S3_SECRET_KEY`, `S3_REGION`, `S3_USE_PATH_STYLE`).
  - Per-Workspace Override: Stored in system database table `storage_configs` (`apiserver/src/repositories/storage_config_repo.rs`). Credentials encrypted with AES-256-GCM (`ENCRYPTION_KEYS_JSON`).
  - Caching: `StorageResolver` caches workspace storage clients for 1 hour (`moka::future::Cache`, TTL 3600s, max capacity 1000, `resolver.rs:20`).
- **Storage Key Naming Contract**:
  - Workspace file: `{workspace_id}/{category}/{YYYY/MM}/{uuid}_{sanitized_filename}` (`key_generator.rs:37`).
  - System file: `_system/{category}/{YYYY/MM}/{uuid}_{sanitized_filename}` (`key_generator.rs:44`).
  - Filename sanitization: replaces non-alphanumeric/hyphen/underscore chars with `_` (`key_generator.rs:10-30`).
- **Presigning & Proxying**:
  - Threshold: `FILE_PROXY_THRESHOLD_BYTES` (default 1,048,576 bytes / 1MB, `file_service.rs:34`).
  - Downloads $\le$ threshold: Proxied via server as raw bytes (`FileDownloadResponse::ProxyBytes`, `file_service.rs:188`).
  - Downloads $>$ threshold: Client redirected to presigned S3 GET URL (`FileDownloadResponse::PresignedUrl`, `file_service.rs:192`).
  - Direct Uploads: Client requests presigned PUT URL via `/api/v1/workspaces/:ws_id/files/presigned` (`file_service.rs:285`).
  - URL TTL: `PRESIGNED_URL_TTL_SECS` (default 900 seconds / 15 minutes, `file_service.rs:37`).
- **Timeout & Retry Policy**:
  - Timeout: **UNKNOWN — needs decision** (Rust `S3Storage::new` at `s3_storage.rs:72` sets no explicit HTTP timeout on `aws_sdk_s3::Config::builder()`; relies on AWS SDK defaults).
  - Retry Policy: **UNKNOWN — needs decision** (no custom retry policy is configured on the Rust client builder).
  - Healthcheck: `list_objects_v2` with `max_keys=1` (`s3_storage.rs:253`).
- **Auth Mechanism**:
  - AWS Static Credentials Provider (`aws_credential_types::Credentials::new`, `s3_storage.rs:76`).
  - Path-style addressing enforced when `S3_USE_PATH_STYLE=true` (critical for MinIO compatibility, `s3_storage.rs:89`).
- **Failure & Error Mapping**:
  - 404 / `NotFound` / `NoSuchKey` $\to$ `AppError::not_found_args(I18nKey::ErrorFileNotFound, ...)` (`s3_storage.rs:135, 238`).
  - Network / Auth / S3 Internal $\to$ `AppError::Internal("S3 ... error: {e}")` (`s3_storage.rs:117, 140, 161, 180, 206, 258`).
- **Cleanup Behavior**:
  - Object deletion: Exposed by `S3Storage::delete_object` (`s3_storage.rs:154`).
  - Database deletion semantics: `FileService::delete_file` and `delete_system_file` execute repository soft-delete only (`file_service.rs:322-331`), marking records inactive without automatically invoking `delete_object`.

---

### 2.2 SMTP & HTTP Email Services

- **Rust Symbols**:
  - `EmailService`: `apiserver/src/services/email_service.rs:181`
  - `EmailConfig`: `apiserver/src/services/email_service.rs:70`
  - `SmtpConfigData`: `apiserver/src/services/email_service.rs:28`
  - `HttpApiConfig`: `apiserver/src/services/email_service.rs:46`
  - `build_transport`: `apiserver/src/services/email_service.rs:100`
  - `send_via_http`: `apiserver/src/services/email_service.rs:449`
- **Client Libraries**:
  - `lettre` v0.11 (`features = ["builder", "rustls-tls", "smtp-transport"]`, `Cargo.toml:63`).
  - `reqwest` v0.12 (`features = ["json", "rustls-tls", "stream"]`, `Cargo.toml:55`).
  - Go Target: `net/smtp` / `gopkg.in/mail.v2` for SMTP; `net/http` standard client for HTTP.
- **Dynamic Configuration & Hot-Reload**:
  - Priority Hierarchy:
    1. Memory cache (`EmailConfig`, fresh for 60 seconds, `CONFIG_RELOAD_INTERVAL = 60s`, `email_service.rs:22`).
    2. Redis hash `smtp:config` (fast shared cache across API nodes, `email_service.rs:244`).
    3. System database table `system_settings` where `key = 'smtp'` (source of truth, `email_service.rs:277`).
  - On DB load, config is automatically written back to Redis cache (`email_service.rs:234`).
- **Dual Transport Modes**:
  - **Mode 1: HTTP API** (`HttpApiConfig`):
    - Takes precedence over SMTP when configured (`email_service.rs:319`).
    - Used for platforms blocking outbound SMTP ports 25/465/587 (e.g., Railway).
    - Template resolution: resolves `{{username}}` and `{{password}}` in `auth_value` at reload time (`email_service.rs:324-335`).
    - Body templating: substitutes `{{from_name}}`, `{{from_email}}`, `{{to}}`, `{{subject}}`, `{{body}}` (`email_service.rs:458-465`).
    - Optional JSON response verification: checks `success_field` matches `success_value` (`email_service.rs:493-507`).
  - **Mode 2: Native SMTP** (`SmtpTransport`):
    - Port 465 $\to$ Direct TLS relay (`SmtpTransport::relay(host)`, `email_service.rs:112`).
    - Port 587 or other $\to$ STARTTLS relay (`SmtpTransport::starttls_relay(host)`, `email_service.rs:114`).
    - Credentials: `username` + `password` (`email_service.rs:126`).
- **Timeout & Retry Policy**:
  - Timeout: `SMTP_TIMEOUT = Duration::from_secs(10)` (`email_service.rs:25`). Applied to both SMTP operations and HTTP client (`reqwest::Client::builder().timeout(SMTP_TIMEOUT)`).
  - Rate Limiting: Redis sliding counters with TTL (`email_service.rs:14-19, 377-444`):
    - 5 emails / minute (`RATE_LIMIT_MINUTELY_MAX = 5`, window 60s)
    - 20 emails / hour (`RATE_LIMIT_HOURLY_MAX = 20`, window 3600s)
    - 100 emails / day (`RATE_LIMIT_DAILY_MAX = 100`, window 86400s)
    - If limit exceeded: returns `Ok(())` silently to prevent email spamming (`email_service.rs:521`).
- **Failure & Error Mapping**:
  - Unconfigured: `AppError::Internal("Email is not configured...")` (`email_service.rs:611`).
  - Transport Build / Recipient Parse: `AppError::Internal(format!("Failed to parse recipient: {e}"))` (`email_service.rs:555`).
  - HTTP failure: `AppError::Internal(format!("Email API returned HTTP {status}: {resp_body}"))` (`email_service.rs:488`).
  - SMTP failure: `AppError::Internal(format!("Failed to send email via SMTP: {e}"))` (`email_service.rs:567`).
- **Cleanup Behavior**:
  - Blocking SMTP calls dispatched to threadpool via `tokio::task::spawn_blocking` (`email_service.rs:562, 659`).

---

### 2.3 Virus Scanner (ClamAV / Mock)

- **Rust Symbols**:
  - `ClamAvScanner`: `apiserver/src/services/storage/scanner/clamav.rs:6`
  - `MockScanner`: `apiserver/src/services/storage/scanner/mock.rs:6`
  - `ScanProvider`: `apiserver/src/services/storage/scanner/mod.rs:20`
  - `VirusScanner` trait: `apiserver/src/services/storage/scanner/mod.rs:16`
  - `FileScanScheduler`: `apiserver/src/worker/file_scan_scheduler.rs:11`
- **Client Library**:
  - `clamav-client` v2.2.0 (`apiserver/Cargo.toml:60`).
  - Protocol: TCP communication to `clamd` daemon (`INSTREAM` / `SCAN` command).
  - Go Target: Custom TCP client or `github.com/dutchcoders/go-clamd`.
- **Configuration & Selection**:
  - `CLAMD_HOST` and `CLAMD_PORT` (default 3310, `env_registry.rs:985-1003`).
  - Fallback: If `CLAMD_HOST` is unset or empty, `ScanProvider` instantiates `MockScanner` (`scanner/mod.rs:27-37`), which simulates clean scan with a 10ms sleep (`mock.rs:13`).
- **Timeout & Retry Policy**:
  - ClamAV scan timeout: 10,000 ms (10 seconds) (`clamav.rs:28`).
  - Invocation: `tokio::task::spawn_blocking` (`clamav.rs:23`).
  - Retry: No retry.
- **Outcomes & Error Mapping**:
  - `ScanResult::Clean` $\to$ file marked Clean in DB.
  - `ScanResult::Infected(details)` $\to$ file marked Infected (`worker/file_scan_scheduler.rs:114-117`), download blocked.
  - Connection/Join Error $\to$ `AppError::Internal(format!("ClamAV scan connection error: {e}"))` (`clamav.rs:33`).
- **Worker File Scan Scheduler**:
  - Runs in background every 30 seconds (`file_scan_scheduler.rs:41`).
  - Queries pending files in chunks of 10 (`get_pending_scan_workspace_files`, `file_scan_scheduler.rs:76`).
  - Concurrency: buffered up to 5 workspaces concurrently (`buffer_unordered(5)`, `file_scan_scheduler.rs:133`).

---

### 2.4 Outbound HTTP Services

- **Rust Symbols**:
  - Email HTTP Client: `apiserver/src/services/email_service.rs:188`
  - Supavisor Admin API: `apiserver/src/utils/env_registry.rs:1009` (`SUPAVISOR_API_URL`, `API_JWT_SECRET` / `SUPAVISOR_API_JWT`)
  - E-Contract Dispatcher: `apiserver/src/services/econtract/dispatcher.rs:9`
- **Client Library**:
  - `reqwest` v0.12 (rustls-tls, connection pooled).
  - Go Target: `net/http.Client` with custom `http.Transport` (keep-alive, connection pooling).
- **Supavisor Admin API Boundary**:
  - Target: Supavisor pooler control-plane API for tenant registration.
  - Settings: `SUPAVISOR_API_URL`, `API_JWT_SECRET` / `SUPAVISOR_API_JWT` (`env_registry.rs:1009, 1019`).
  - Implementation Status: **UNKNOWN — needs decision**. Settings are registered in `env_registry.rs:1008-1027` and i18n error keys exist (`i18n.rs:81-82`), but no outbound HTTP client call to Supavisor exists anywhere in `apiserver/src`. Tenant migrations currently run directly against the provided database URL via `TenantMigrator` (`tenant_provisioner.rs:115`). Timeout, retry, and auth-header format are therefore not observable.
- **E-Contract Third-Party Boundary**:
  - Interface `EContractProvider`: `create_envelope`, `get_signing_url`, `parse_webhook`, `download_signed_document` (`econtract/provider.rs:9-31`).
  - Current implementation: `MockEContractProvider` (`econtract/mock.rs:1`).
  - Production adapters: **UNKNOWN — needs decision**. FPT/Viettel/DocuSign are described only in comments (`dispatcher.rs:37-41`); no live endpoint, timeout, retry, or credential handling exists to inventory.

---

### 2.5 WebSocket & Server-Sent Events (SSE)

- **1. Agent Ingestion WebSocket (`GET /api/v1/ws`)**:
  - **Symbol**: `apiserver/src/handlers/agents.rs:66` (`ws_handler`, `handle_socket`)
  - **Auth**: Headers `x-agent-id` and `x-token` (`agents.rs:71-77`). Validated against database table `authorized_agents` using SHA-256 token hashing (`crypto::hash_agent_token`, `crypto.rs:151`). Returns 401 UNAUTHORIZED on failure.
  - **Connection State**: Registered in `AgentManager` in-memory map `agent_id -> mpsc::UnboundedSender<Message>` (`agent_manager.rs:217`).
  - **Framing Protocol**:
    - Inbound `log` frame: parsed into `AttendanceLog`, enqueued into Redis `queue:jobs` via `record_and_enqueue` (`agents.rs:240-250`).
    - Immediate Inbound ACK: sent back over socket as `accepted` (durable delivery in Redis) or `processed` (if deduplicated/already complete) (`agents.rs:252-263`).
    - Inbound `ping`: updates `last_seen_at` heartbeat in Redis/memory.
    - Outbound `command`: sends administrative ping or synchronization commands to agent device (`agent_manager.rs:340`).
  - **Shutdown Behavior**: Server close frame sent to all connected agents on graceful shutdown (`agent_manager.rs:390-410`).

- **2. Report Export Streaming WebSocket (`GET /api/v1/ws/reports/:job_id`)**:
  - **Symbol**: `apiserver/src/handlers/report_websocket.rs:74` (`report_stream_ws`, `handle_report_socket`)
  - **Auth**: `ws_auth_middleware` (`report_websocket.rs:33`) extracts JWT from `?token=` query param or `access_token` cookie. Verifies JWT signature and workspace membership (`report_websocket.rs:107-128`). Returns 401 if unauthenticated, 403 if user outside workspace.
  - **Backend Subscription**: Subscribes to Redis Pub/Sub channel `reports:stream:{job_id}` (`ReportSubscriber`, `report_websocket.rs:141`).
  - **Race-Condition Handling**: If worker completes before client connects, handler queries Redis job metadata and synthesizes the `DONE` message immediately (`report_websocket.rs:155-175`).
  - **Frame Messages**: JSON text frames: `{"type": "BATCH", ...}`, `{"type": "DONE", ...}`, `{"type": "ERROR", ...}`.

- **3. Worker Monitoring SSE Stream (`GET /api/v1/monitor/stream`)**:
  - **Symbol**: `apiserver/src/handlers/worker_monitor.rs:2443` (`monitor_stream_sse`)
  - **Auth**: `SystemContext` checking `PlatformAdmin` or `Support` role (`worker_monitor.rs:2447`). Returns 401/403.
  - **Protocol**: `Content-Type: text/event-stream`, keep-alive enabled (`worker_monitor.rs:2476`).
  - **Cadence**: Pushes `monitor-update` event every 2 seconds (`SSE_INTERVAL_SECS = 2`, `worker_monitor.rs:68, 2449`).
  - **Error Event**: Pushes `monitor-error` on Redis query failure (`worker_monitor.rs:2463`).

---

### 2.6 Local Filesystem Boundaries

- **Rust Symbols**:
  - Base Directory Resolver: `apiserver/src/utils/mod.rs:20` (`get_storage_dir`)
  - Temporary File Ingestion: `apiserver/src/utils/upload.rs:106` (`extract_uploaded_file`)
  - Import Error Reports: `apiserver/src/services/import_pipeline.rs:350`
  - Port File Lock: `apiserver/src/main.rs:21` (`PortFileGuard`), `main.rs:236`
- **Configuration**:
  - `STORAGE_DIR`: defaults to `"storage"` (local dev) or `"/app/storage"` (Docker volume, `apiserver/Dockerfile:78`).
  - `PORT_FILE_NAME`: defaults to `".api_port"` (`env_registry.rs:567`).
- **Paths & Subdirectories**:
  - `{STORAGE_DIR}/imports/`: Temporary storage for uploaded Excel spreadsheets (attendance, employee, leave imports) prior to async parsing (`upload.rs:108`).
  - `{STORAGE_DIR}/imports/error_{job_id}.xlsx`: Generated rejection/error workbooks containing invalid rows for user download (`import_pipeline.rs:351`, `attendance/import.rs:353`).
- **Lifecycle & Cleanup**:
  - Upload files: Read into memory and deleted immediately after initial parse or ingestion (`tokio::fs::remove_file`, `handlers/employees.rs:613`, `import_pipeline.rs:482`).
  - Port file: Deleted on application graceful shutdown by `PortFileGuard` drop implementation (`main.rs:27`).

---

### 2.7 Database (PostgreSQL & PgBouncer) Boundary

- **Rust Symbols**:
  - `DbPoolConfig`: `apiserver/src/diesel_pool/config.rs:3`
  - `DieselPools`: `apiserver/src/diesel_pool/core.rs:48`
  - `TenantPoolManager`: `apiserver/src/diesel_pool/tenant_manager.rs:26`
- **Protocol & Network Endpoints**:
  - Direct Primary Port `5432`: Used strictly for startup migrations and schema backup (`main.rs:139, 171`).
  - Transaction Pooler Port `6543`: Used for all HTTP request handlers and CRUD operations (`DB_POOLER_TX_URL`, `DB_TENANT_POOLER_TX_URL`).
  - Session Pooler Port `5452`: Used for session-bound operations (`DB_POOLER_SESS_URL`, `DB_TENANT_POOLER_SESS_URL`).
- **Timeouts & Connection Limits**:
  - API Pool Acquire Timeout: `DB_API_ACQUIRE_TIMEOUT_SECS` (default 5s, `config.rs:100`).
  - Worker Pool Acquire Timeout: `DB_WORKER_ACQUIRE_TIMEOUT_SECS` (default 30s, `config.rs:104`).
  - API Pool Max Connections: `DB_API_MAX_CONN` (default 20, `config.rs:98`).
  - Tenant Pool Max Connections: `DB_TENANT_MAX_CONN` (default 5, `config.rs:107`).
- **Failure Mapping**:
  - Connection refused / acquire timeout $\to$ `AppError::Internal("Database connection error")` or `500`.
  - Constraint violations: Unique key violation mapped to `AppError::conflict(...)` (`409 Conflict`).
  - Not found: `diesel::result::Error::NotFound` mapped to `AppError::not_found(...)` (`404 Not Found`).

---

### 2.8 Redis Boundary

- **Rust Symbols**:
  - `RedisPool`: `apiserver/src/queue/redis_client.rs:44`
  - `JobQueue`: `apiserver/src/queue/job_queue.rs:68`
  - `IngestionDeliveryStore`: `apiserver/src/queue/ingestion_delivery.rs:120`
- **Connection Architecture**:
  - Split dual ConnectionManager:
    - Connection 1: Shared multiplexed `conn_manager` for standard commands (`GET`, `SET`, `HSET`, `PUBLISH`, `redis_client.rs:80`).
    - Connection 2: Dedicated `blocking_conn` reserved strictly for `BLPOP` worker loops to prevent head-of-line pipeline stalls (`redis_client.rs:81, 122`).
- **Timeout & Retry Policy**:
  - Connection timeout: 5 seconds on startup (`main.rs:321, 334`).
  - Worker BLPOP timeout: `WORKER_POLL_TIMEOUT_SECS` (default 30s, clamped max 300s, `job_queue.rs:216-230`).
  - Job execution lease: `DEFAULT_LEASE_SECS = 600` (10 minutes, `ingestion_delivery.rs:58`).
  - Delivery record retention: `DELIVERY_TTL_SECS = 1800` (30 minutes, `ingestion_delivery.rs:55`).
  - Job max retries: `WORKER_MAX_RETRIES` (default 3, `job/mod.rs:77`).
- **Failure Mapping**:
  - Startup connection failure $\to$ Process aborts (`main.rs:325`).
  - Runtime command error $\to$ `AppError::Redis("...")` mapped to HTTP 500 (`errors.rs:451`).

---

## 3. Go Implementation Mapping Guide

| Rust Module / Trait | Recommended Go Standard / Library | Parity Verification Check |
|---|---|---|
| `S3Storage` (`aws-sdk-s3`) | `aws-sdk-go-v2/service/s3` | Presigned URL generation matches expiration and signature algorithm; MinIO integration test |
| `EmailService` (`lettre` + `reqwest`) | `net/smtp` + `net/http` | Live mail capture via Mailpit/MailHog; template replacement unit tests |
| `ClamAvScanner` (`clamav-client`) | `net.DialTimeout("tcp", addr, 10*time.Second)` | EICAR test string scan returns `ScanResult::Infected` |
| `ws_handler` (`axum::extract::ws`) | `nhooyr.io/websocket` or `gorilla/websocket` | WebSocket client test asserting immediate `accepted` and delayed `processed` ACKs |
| `report_stream_ws` | `nhooyr.io/websocket` + Redis PubSub | End-to-end report export streaming test capturing all batch frames |
| `monitor_stream_sse` | `net/http` (`http.Flusher`) | SSE client test asserting 2s JSON heartbeats |
| `FileService` (local FS fallback) | `os.ReadFile`, `os.Remove`, `os.MkdirAll` | File permissions match 0644 / 0755; cleanup on failure verified |
| `DieselPools` (`diesel-async`) | `github.com/jackc/pgx/v5/pgxpool` | `pgxpool.Config` MaxConns=20, AcquireTimeout=5s; `SELECT 1` ping |
| `RedisPool` (`redis-rs`) | `github.com/redis/go-redis/v9` | Dedicated client instance for blocking commands; shared client for standard commands |
