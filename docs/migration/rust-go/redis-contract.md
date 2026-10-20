# Redis contract — Rust apiserver inventory

Scope: `apiserver/src` Redis call sites.
Key count: **43 key/channel families** (including fixed keys and dynamic templates).
Every key below is traceable to Rust source `path:line`.

## Client and failure semantics

### Client topology
- Rust client: `redis` async `ConnectionManager` with multiplexed connection.
  - `RedisPool` establishes two `ConnectionManager` instances: `conn_manager` (shared async) and `blocking_conn` (for `BLPOP` to avoid blocking other async operations) (`apiserver/src/queue/redis_client.rs:44-50,66-75`).
  - Pub/Sub uses a dedicated multiplexed connection `get_pubsub_connection()` converted via `.into_pubsub()` (`apiserver/src/queue/redis_client.rs:148-150`, `apiserver/src/queue/pubsub.rs:79-82`).
- Go target client:
  - Base: `github.com/redis/go-redis/v9` client (`*redis.Client` or `redis.UniversalClient`).
  - Blocking queue consumer: separate dedicated client or connection pool dedicated to `BLPop` routines.
  - Pub/Sub: `client.Subscribe(ctx, channel)` dedicated connection manager.

### Failure policy: fail-open vs fail-closed
There are **6 fail-open call sites** (or non-blocking degradations) where Redis errors or absence do not abort user requests:
1. `apiserver/src/lib.rs:128-130`: `is_token_revoked` returns `false` (token valid) if Redis pool is `None` or Redis command fails (`exists` unwrap_or `false`).
2. `apiserver/src/handlers/auth.rs:170-185,200-215`: login attempt counter increment failure defaults to `0` (`unwrap_or(0)`), and `expire` / `set_ex` errors are ignored via `let _: Result<(), _> = ...`.
3. `apiserver/src/mw/rate_limit.rs:185-199`: when Redis fails or is absent, the middleware logs an error and falls back to in-memory Moka cache instead of dropping traffic.
4. `apiserver/src/services/email_service.rs:225-242,244-260`: if Redis read for `smtp:config` fails, the service falls through to DB `system_settings` or environment fallback.
5. `apiserver/src/services/email_service.rs:402-423`: rate limit counters for SMTP return `0` on `INCR` error (`unwrap_or_else(|_| 0)`) and ignore `EXPIRE` failures.
6. `apiserver/src/cache/roster_cache.rs:450-473`: when Redis `roster:version:{workspace_id}` fails, `fetch_version` returns `None`, which bypasses the local Moka cache and forces a direct PostgreSQL load (`apiserver/src/cache/roster_cache.rs:524-526`), keeping roster queries operational.

Fail-closed / error-propagating call sites:
1. Password reset token creation & verification (`apiserver/src/services/auth_service.rs:252-295`): requires Redis; returns `AppError::Redis` if Redis is not configured or fails.
2. Global session revocation (`apiserver/src/services/auth_service.rs:238-247`): writing `user:revoked_at:{user_id}` returns `AppError::Redis` on failure.
3. Queue operations (`apiserver/src/queue/job_queue.rs`, `apiserver/src/repositories/queue_repository.rs`): `enqueue_jobs`, `dequeue_job`, `create_job_metadata`, `update_job_metadata`, `store_result` return `redis::RedisError` / `String` errors up the stack.
4. Durable attendance ingestion (`apiserver/src/queue/ingestion_delivery.rs`): `record_and_enqueue` and `claim` fail closed so unacknowledged frames trigger agent retries.

---

## Complete Key Inventory

| No. | Key / channel template | Type, encoding | TTL & Source | Writes (`path:line`) | Reads & Invalidation triggers (`path:line`) | Failure policy |
|---|---|---|---|---|---|---|
| 1 | `agent:presence:{agent_id}` | STRING, RFC3339 timestamp string | 60s (`set_ex`) | `agent_manager.rs:161,195` | MGET in `agent_manager.rs:141,145`; read in status calculation `:335-340`. No explicit invalidation (natural TTL). | Fail-open: logs error, reports agent offline if not locally connected |
| 2 | `login:attempts:{email}` | INTEGER counter via `INCR` | 900s (15m), set on 1st attempt | `handlers/auth.rs:171,175,201,205` | Read implicitly via `incr`. Invalidation/cleanup: deleted on successful login `handlers/auth.rs:223,227`. | Fail-open (`unwrap_or(0)`, expiry error ignored) |
| 3 | `login:lockout:{email}` | STRING, decimal attempt count | Dynamic exponential backoff: starts at 900s, increases by $2^{\text{attempts}-5} \times 900$ (capped at 86400s) | `handlers/auth.rs:172,184,202,214` | Read via `GET` at login start `handlers/auth.rs:148,151`. Invalidation: deleted on successful login `:224,227`. | Fail-open on Redis errors |
| 4 | `jwt:revoked:{sig}` | STRING, literal `"1"` | Token remaining lifetime or default 900s (`claims.exp - now`) | `handlers/auth.rs:456,468` (logout) | Read via `EXISTS` in auth middleware `lib.rs:135,142`. No explicit invalidation (natural expiration). | Fail-open if Redis missing/fails (`lib.rs:128-130,142`) |
| 5 | `user:revoked_at:{user_id}` | STRING, Unix epoch timestamp (seconds) | 900s | `services/auth_service.rs:239,244` | Read via `GET` and compared to token `iat` in `lib.rs:136,147`. Invalidation: natural TTL. | Write fail-closed (returns `AppError::Redis`); Read fail-open |
| 6 | `jwt:rotated:{refresh_token}` | STRING, user UUID | 604800s (7 days: $7 \times 24 \times 3600$) | `services/auth_service.rs:196,200` | Read via `GET` to detect refresh token reuse `auth_service.rs:158,161`. Triggers revocation of all user sessions. | Read miss/error treated as invalid token (fail-closed auth) |
| 7 | `pwd_reset:{token}` | STRING, user UUID | 900s (15 minutes) | `services/auth_service.rs:255,260` | Read via `GET` and single-use `DEL` in `services/auth_service.rs:273,277,286`. | Fail-closed: missing Redis or error aborts reset |
| 8 | `rate:{identity}:{tier}:{matched_path}` | INTEGER bucket counter via Lua script | Dynamic `window_secs` (default 60s) via Lua `EXPIRE` on first `INCR` | `mw/rate_limit.rs:175,290-302` | Read & write atomic inside Lua script `mw/rate_limit.rs:288-305`. | Fail-open to in-memory Moka rate limiter (`:186-193`) |
| 9 | `cache:workspace:{workspace_id}` | JSON `CachedWorkspaceConfig` | 300s (`DEFAULT_CACHE_TTL` in `cache/mod.rs:27`) | `cache/redis_cache.rs:82` via `workspace_cache.rs:47` | Read via `GET` in `workspace_cache.rs:42-60`. Invalidation: `DEL` on workspace settings update `workspace_cache.rs:64-66`, `state.rs:791-799`, `manager.rs:65`. | Fail-closed on cache error; DB fallback on cache miss |
| 10 | `cache:shifts:{workspace_id}` | JSON `Vec<CachedShift>` | 300s (`DEFAULT_CACHE_TTL`) | `cache/redis_cache.rs:82` via `shift_cache.rs:48` | Read via `GET` in `shift_cache.rs:42-76`. Invalidation: `DEL` on shift CRUD `shift_cache.rs:80-82`, `manager.rs:66`. | Fail-closed on cache error; DB fallback on cache miss |
| 11 | `cache:workspace_config:{workspace_id}` | JSON `Vec<WorkspaceConfigEntry>` | 300s (`DEFAULT_CACHE_TTL`) | `cache/redis_cache.rs:82` via `config_cache.rs:42` | Read via `GET` in `config_cache.rs:32-48`. Invalidation: `DEL` on config change `config_cache.rs:51-53`, `manager.rs:67`. | Fail-closed on cache error; DB fallback on cache miss |
| 12 | `roster:version:{workspace_id}` | STRING, generation UUID | No TTL (persists until updated) | `cache/roster_cache.rs:445,487` | Read via `GET` in `roster_cache.rs:456-460`. Checked against local Moka entry generation `:517-520`. Invalidation: generation bumped `:487`. | Fail-open: Redis error returns `None`, bypassing local cache to query PostgreSQL directly (`:450-453`) |
| 13 | `cache:auth:permissions:{workspace_id}:{user_id}` | JSON `CachedMemberContext` | 3600s (1 hour) | `handlers/common.rs:218,227` | Read in `handlers/common.rs:222`. Invalidation: `DEL` on member/permission updates `handlers/workspaces/permissions.rs:117`, `members.rs:146,215,277`, `roles.rs:128,188`; wildcard `cache:auth:permissions:{workspace_id}:*` on role changes `roles.rs:359,441`. | Cache miss/error falls back to DB; invalidation error ignored |
| 14 | `cache:system:roles:{user_id}` | JSON `CachedSystemRoles` (`roles: Vec<String>`) | 1800s (`SYSTEM_ROLE_CACHE_TTL`, 30m) | `handlers/system_auth.rs:104,149` | Read in `SystemContext` extractor `handlers/system_auth.rs:133`. Invalidation: `DEL` via `invalidate_cache` `:109-113`. | Cache miss/absence falls back to DB query (`:144-155`) |
| 15 | `cache:employment:{employment_id}` | JSON `Employment` | 300s (`DEFAULT_CACHE_TTL`) | `services/leave_service/caching.rs:18,23` | Read via `get_or_fetch` `:23-28`. Invalidation: natural TTL. | Cache errors map to `AppError::Internal` |
| 16 | `cache:leave_type_exists:{leave_type_id}` | JSON/plain boolean | 300s (`DEFAULT_CACHE_TTL`) | `services/leave_service/caching.rs:45,50` | Read via `get_or_fetch` `:50-55`. Invalidation: natural TTL. | Cache errors map to `AppError::Internal` |
| 17 | `cache:employee_leaves:{employee_id}` | JSON `Vec<LeaveRequest>` | 300s (`DEFAULT_CACHE_TTL`) | `services/leave_service/caching.rs:72,77` | Read via `get_or_fetch` `:77-82`. Invalidation: `DEL` on leave submission `services/leave_service/submission.rs:140-142`. | Cache errors map to `AppError::Internal` |
| 18 | `smtp:config` | HASH (`host`, `port`, `username`, `password`, `from`, `base_url`, `http_api`) | No TTL (persists until updated) | `services/email_service.rs:296-313` | Read via `HMGET` in `email_service.rs:250-254`. Invalidation: `DEL` on system admin save `handlers/system_admin.rs:460-463`. | Fail-open: falls through to PostgreSQL `system_settings` or env |
| 19 | `email:rate:min:{unix_window}` | INTEGER counter via `INCR` | 60s (`RATE_WINDOW_MINUTELY_SECS`) | `services/email_service.rs:388,407,416` | Checked in `services/email_service.rs:426,430`. | Fail-open: returns 0 on `INCR` error |
| 20 | `email:rate:hour:{unix_window}` | INTEGER counter via `INCR` | 3600s (`RATE_WINDOW_HOURLY_SECS`) | `services/email_service.rs:392,407,416` | Checked in `services/email_service.rs:427,434`. | Fail-open: returns 0 on `INCR` error |
| 21 | `email:rate:day:{unix_window}` | INTEGER counter via `INCR` | 86400s (`RATE_WINDOW_DAILY_SECS`) | `services/email_service.rs:396,407,416` | Checked in `services/email_service.rs:428,438`. | Fail-open: returns 0 on `INCR` error |
| 22 | `worker:jobs:high` (or `{ns}:worker:jobs:high`) | LIST, JSON serialized `Job` via `RPUSH` | No TTL (queue items bounded by max length 100,000) | `queue/job_queue.rs:105,170-174` | Dequeued via `BLPOP` in `queue/job_queue.rs:268-274`; monitored via `LLEN` in `handlers/worker_monitor.rs:961,1010`. | Fail-closed: errors propagate to enqueuer / worker runner |
| 23 | `worker:jobs:normal` (or `{ns}:worker:jobs:normal`) | LIST, JSON serialized `Job` via `RPUSH` | No TTL (bounded by max length 100,000) | `queue/job_queue.rs:114,170-174` | Dequeued via `BLPOP` in `queue/job_queue.rs:268-274`; monitored via `LLEN` in `handlers/worker_monitor.rs:962,1010`. | Fail-closed |
| 24 | `worker:jobs:low` (or `{ns}:worker:jobs:low`) | LIST, JSON serialized `Job` via `RPUSH` | No TTL (bounded by max length 100,000) | `queue/job_queue.rs:123,170-174` | Dequeued via `LPOP` (weighted fairness) or `BLPOP` in `queue/job_queue.rs:247-250,268-274`; monitored via `LLEN` in `handlers/worker_monitor.rs:963,1010`. | Fail-closed |
| 25 | `report:meta:{job_id}` | STRING, JSON `JobMetadata` | 86400s (24h, `JOB_TTL_SECS`), preserved on update | `queue/job_queue.rs:299,311,361,380-388` | Read via `GET` in `queue/job_queue.rs:339-340`. Invalidation: `DEL` in `cleanup_job` `job_queue.rs:586`. | Fail-closed: errors bubble to API callers |
| 26 | `report:job_list` | SORTED SET (ZSET), job UUID scored by created Unix timestamp | Scored elements pruned if $< \text{now} - 86400\text{s}$ via `ZREMRANGEBYSCORE` | `queue/job_queue.rs:47,313-322` | Read via `ZREVRANGEBYSCORE` in `queue/job_queue.rs:751-757,845-853`, `worker_monitor.rs:980-992`. | Monitor degrades gracefully to empty recent job list |
| 27 | `report:counter:{job_id}` | INTEGER counter (completed batches) | 86400s (`JOB_TTL_SECS`) | `queue/job_queue.rs:404,408-412` | Read via `GET` in `queue/job_queue.rs:442-444`. Invalidation: `DEL` in `cleanup_job` `:587`. | Fail-closed |
| 28 | `report:counter:{job_id}:failed` | INTEGER counter (failed batches) | 86400s (`JOB_TTL_SECS`) | `queue/job_queue.rs:427,430-434` | Invalidation: `DEL` in `cleanup_job` `queue/job_queue.rs:588`. | Fail-closed |
| 29 | `report:merge_triggered:{job_id}` | STRING, literal `"triggered"` (idempotency flag) | 86400s (`JOB_TTL_SECS`), set via `SET NX EX` | `queue/job_queue.rs:459,463-469` | Idempotency lock acquired atomically on set. Invalidation: `DEL` in `cleanup_job` `queue/job_queue.rs:592`. | Fail-closed |
| 30 | `report:results:{job_id}` | HASH, field `batch_{index:05}` $\to$ JSON result string | 86400s (`JOB_TTL_SECS`) on each chunk | `queue/job_queue.rs:495,498-502` | Read via `HGETALL` in `queue/job_queue.rs:518-524`. Invalidation: `DEL` in `delete_results` `:678-680`, `cleanup_job` `:589`. | Fail-closed |
| 31 | `report:results:{job_id}:data` | RAW BYTES (compiled Excel spreadsheet binary) | 3600s (1 hour) | `queue/job_queue.rs:613,625-626` | Read via `GET` in `queue/job_queue.rs:659,662`. Invalidation: `DEL` in `cleanup_job` `:590`. | Fail-closed |
| 32 | `report:results:{job_id}:filename` | STRING, export filename | 3600s (1 hour) | `queue/job_queue.rs:614,627-628` | Read via `GET` in `queue/job_queue.rs:660,663`. Invalidation: `DEL` in `cleanup_job` `:591`. | Fail-closed |
| 33 | `report:dlq` | LIST, JSON `{job, error, failed_at}` via `RPUSH` | No TTL (bounded via `LTRIM` to max 10,000 entries) | `queue/job_queue.rs:46,555-558` | Monitored via `LLEN` in `handlers/worker_monitor.rs:964`. | Permanent error path logs error |
| 34 | `stats:completed:{YYYY-MM-DD}` | INTEGER counter via `INCR` | 604800s (7 days, `STATS_TTL_SECS`) | `queue/job_queue.rs:688,691-695` | Read via `GET` in `handlers/worker_monitor.rs:966-972`. | Monitor returns `None` on read failure |
| 35 | `stats:failed:{YYYY-MM-DD}` | INTEGER counter via `INCR` | 604800s (7 days, `STATS_TTL_SECS`) | `queue/job_queue.rs:705,708-712` | Read via `GET` in `handlers/worker_monitor.rs:974-978`. | Monitor returns `None` on read failure |
| 36 | `import_progress:{job_id}` | INTEGER counter via `INCRBY` | 86400s (24h) initialized in worker, or untimed in repo | `repositories/queue_repository.rs:149-155`, `worker/handlers/leave_import.rs:136`, `employee_import.rs:152` | Read via `GET` in `handlers/leaves.rs:879-885`, `queue_repository.rs:161-166`, `worker/handlers/finalize_import.rs:31-37`. Invalidation: `DEL` in `finalize_import.rs:137`. | Writer errors propagate; reader ignores errors |
| 37 | `import_errors:{job_id}` | HASH, chunk index $\to$ error message string | No TTL (persists for job lifetime) | `repositories/queue_repository.rs:176-184` | Read via `HGETALL` in `worker/handlers/finalize_import.rs:39-46`. Invalidation: `DEL` in `finalize_import.rs:145`, `leave_import.rs:158`. | Writer and finalize errors propagate |
| 38 | `import_chunks_done:{job_id}` | INTEGER counter via `increment` | 86400s (24h) initialized in import worker | `worker/handlers/leave_import.rs:143`, `employee_import.rs:159`, `increment` in `leave_import.rs:301`, `employee_import.rs:289` | Read via `increment` return value `:303`. Invalidation: `DEL` in `finalize_import.rs:154`. | Chunk processor logs and continues |
| 39 | `import_total_chunks:{job_id}` | STRING integer | 86400s (24h) | `worker/handlers/leave_import.rs:150`, `employee_import.rs:166` | Read via `get_json` in `leave_import.rs:306-311`, `employee_import.rs:294-299`. Invalidation: `DEL` in `finalize_import.rs:164`. | Fail-closed: missing value defaults to 0 |
| 40 | `lock:housekeeping:{YYYY-MM-DD}` | STRING, literal `"1"` (distributed lock) | 86400s (24h), acquired via `SET NX EX` | `worker/scheduler.rs:34,39-46` | Lock acquisition check only (`:48`). No read. | Fail-safe: if Redis fails, scheduler skips turn and retries next hour |
| 41 | `ingest:delivery:{agent_id}:{msg_id}` | STRING, JSON `DeliveryRecord` | 1800s (30m, `DELIVERY_TTL_SECS`), refreshed on worker claim | Lua `SET EX` in `queue/ingestion_delivery.rs:134,178-202`, claim `:237-265` | Read via `GET` and Lua check `:180,239,283`; scanned via `SCAN MATCH ingest:delivery:*` by lease reaper `:354-360`. Invalidation: `DEL` on terminal outcome `:302`. | Fail-closed: returns error so agent preserves local copy and retries |
| 42 | `report:stream:{job_id}` | Pub/Sub CHANNEL, JSON `StreamMessage` (`CHUNK`, `DONE`, `ERROR`) | Ephemeral (Pub/Sub) | `queue/pubsub.rs:126,184,200` | Subscribed in WebSocket handler `handlers/report_websocket.rs:141-152`, `queue/pubsub.rs:71-82,87-95`. | Publish/Subscribe errors fail WebSocket session setup |
| 43 | `monitor:worker:{worker_id}` | STRING, JSON `WorkerHeartbeat` | 90s (`HEARTBEAT_TTL_SECS` in `worker/runner.rs:49`) | `worker/runner.rs:45,852,870-873` | Read via `SCAN MATCH monitor:worker:*` + chunked `MGET` in `handlers/worker_monitor.rs:1044,1060-1070`. Invalidation: `DEL` on graceful worker exit `runner.rs:880`. | Monitor marks worker source unavailable; doesn't break API |

---

## Go target mapping

Client recommendation: `github.com/redis/go-redis/v9`. Centralize all key generation inside a typed package `internal/infra/rediskeys`.

| Key / Channel family | Go package & function | Serialization & Types | TTL Constant |
|---|---|---|---|
| `agent:presence:{agent_id}` | `rediskeys.AgentPresence(agentID string)` | String (RFC3339 timestamp) | `60 * time.Second` |
| `login:attempts:{email}` | `rediskeys.LoginAttempts(email string)` | Int counter (`Incr`) | `900 * time.Second` |
| `login:lockout:{email}` | `rediskeys.LoginLockout(email string)` | String integer | Dynamic ($900 \times 2^{\text{attempts}-5}\text{s}$, max 86400s) |
| `jwt:revoked:{sig}` | `rediskeys.JWTRevoked(sig string)` | String `"1"` | Token exp remaining, default `900 * time.Second` |
| `user:revoked_at:{user_id}` | `rediskeys.UserRevokedAt(userID uuid.UUID)` | String Unix timestamp | `900 * time.Second` |
| `jwt:rotated:{refresh_token}` | `rediskeys.JWTRotated(token string)` | String user UUID | `7 * 24 * time.Hour` |
| `pwd_reset:{token}` | `rediskeys.PasswordReset(token string)` | String user UUID | `900 * time.Second` |
| `rate:{identity}:{tier}:{path}` | `rediskeys.RateLimit(identity, tier, path string)` | Int counter (Lua atomic `INCR` + `EXPIRE`) | `time.Duration(windowSecs) * time.Second` |
| `cache:workspace:{workspace_id}` | `rediskeys.CacheWorkspace(workspaceID uuid.UUID)` | JSON `WorkspaceConfigDTO` | `300 * time.Second` |
| `cache:shifts:{workspace_id}` | `rediskeys.CacheShifts(workspaceID uuid.UUID)` | JSON `[]ShiftDTO` | `300 * time.Second` |
| `cache:workspace_config:{workspace_id}` | `rediskeys.CacheWorkspaceConfig(workspaceID uuid.UUID)` | JSON `[]WorkspaceConfigEntryDTO` | `300 * time.Second` |
| `roster:version:{workspace_id}` | `rediskeys.RosterVersion(workspaceID uuid.UUID)` | String UUID generation | None (persists until invalidation) |
| `cache:auth:permissions:{workspace_id}:{user_id}` | `rediskeys.CacheAuthPermissions(workspaceID, userID uuid.UUID)` | JSON `MemberContextDTO` | `3600 * time.Second` |
| `cache:system:roles:{user_id}` | `rediskeys.CacheSystemRoles(userID uuid.UUID)` | JSON `SystemRolesDTO` | `1800 * time.Second` |
| `cache:employment:{employment_id}` | `rediskeys.CacheEmployment(employmentID uuid.UUID)` | JSON `EmploymentDTO` | `300 * time.Second` |
| `cache:leave_type_exists:{leave_type_id}` | `rediskeys.CacheLeaveTypeExists(leaveTypeID uuid.UUID)` | String bool `"true"`/`"false"` | `300 * time.Second` |
| `cache:employee_leaves:{employee_id}` | `rediskeys.CacheEmployeeLeaves(employeeID uuid.UUID)` | JSON `[]LeaveRequestDTO` | `300 * time.Second` |
| `smtp:config` | `rediskeys.SMTPConfig()` (fixed key) | Hash fields (`host`, `port`, `username`, `password`, `from`, `base_url`, `http_api`) | None |
| `email:rate:min:{unix_window}` | `rediskeys.EmailRateMin(window uint64)` | Int counter | `60 * time.Second` |
| `email:rate:hour:{unix_window}` | `rediskeys.EmailRateHour(window uint64)` | Int counter | `3600 * time.Second` |
| `email:rate:day:{unix_window}` | `rediskeys.EmailRateDay(window uint64)` | Int counter | `86400 * time.Second` |
| `worker:jobs:{high,normal,low}` | `rediskeys.QueueKey(namespace string, priority Priority)` | List JSON `JobDTO` | None (max length 100,000) |
| `report:meta:{job_id}` | `rediskeys.ReportMeta(jobID uuid.UUID)` | String JSON `JobMetadataDTO` | `86400 * time.Second` (preserved on update) |
| `report:job_list` | `rediskeys.ReportJobList()` (fixed key) | Sorted Set member UUID score epoch | Pruned at $\text{now} - 86400\text{s}$ |
| `report:counter:{job_id}` | `rediskeys.ReportCounter(jobID uuid.UUID)` | Int counter | `86400 * time.Second` |
| `report:counter:{job_id}:failed` | `rediskeys.ReportCounterFailed(jobID uuid.UUID)` | Int counter | `86400 * time.Second` |
| `report:merge_triggered:{job_id}` | `rediskeys.ReportMergeTriggered(jobID uuid.UUID)` | String `"triggered"` (`SET NX EX`) | `86400 * time.Second` |
| `report:results:{job_id}` | `rediskeys.ReportResults(jobID uuid.UUID)` | Hash field `batch_%05d` $\to$ JSON chunk | `86400 * time.Second` |
| `report:results:{job_id}:data` | `rediskeys.ReportResultData(jobID uuid.UUID)` | Binary `[]byte` | `3600 * time.Second` |
| `report:results:{job_id}:filename` | `rediskeys.ReportResultFilename(jobID uuid.UUID)` | String filename | `3600 * time.Second` |
| `report:dlq` | `rediskeys.ReportDLQ()` (fixed key) | List JSON `DLQEntryDTO` | None (capped at 10,000 via `LTrim`) |
| `stats:completed:{YYYY-MM-DD}` | `rediskeys.StatsCompleted(date string)` | Int counter | `7 * 24 * time.Hour` |
| `stats:failed:{YYYY-MM-DD}` | `rediskeys.StatsFailed(date string)` | Int counter | `7 * 24 * time.Hour` |
| `import_progress:{job_id}` | `rediskeys.ImportProgress(jobID uuid.UUID)` | Int counter (`IncrBy`) | `86400 * time.Second` |
| `import_errors:{job_id}` | `rediskeys.ImportErrors(jobID uuid.UUID)` | Hash chunk index $\to$ error text | None |
| `import_chunks_done:{job_id}` | `rediskeys.ImportChunksDone(jobID uuid.UUID)` | Int counter (`Incr`) | `86400 * time.Second` |
| `import_total_chunks:{job_id}` | `rediskeys.ImportTotalChunks(jobID uuid.UUID)` | String integer | `86400 * time.Second` |
| `lock:housekeeping:{YYYY-MM-DD}` | `rediskeys.LockHousekeeping(date string)` | String `"1"` (`SET NX EX`) | `86400 * time.Second` |
| `ingest:delivery:{agent_id}:{msg_id}` | `rediskeys.IngestDelivery(agentID, msgID string)` | String JSON `DeliveryRecordDTO` | `1800 * time.Second` |
| `report:stream:{job_id}` | `rediskeys.ReportStream(jobID uuid.UUID)` | Channel Pub/Sub JSON `StreamMessageDTO` | Ephemeral |
| `monitor:worker:{worker_id}` | `rediskeys.MonitorWorker(workerID uuid.UUID)` | String JSON `WorkerHeartbeatDTO` | `90 * time.Second` |

---

## Drift risks and migration hazards

1. **Inline format strings**:
   - In Rust, key strings are constructed at arbitrary call sites with `format!("...")`. For example, `jwt:revoked:{sig}` in `auth.rs:456` and `lib.rs:135`, and `import_progress:{job_id}` vs `import_progress:{}` in different worker handlers.
   - **Hazard**: Minor typo in separator (`-` vs `:`) or missing curly brace will isolate cache/queue without compiler error.
   - **Remedy**: Go migration MUST forbid raw `fmt.Sprintf` for Redis keys and require constructors from `internal/infra/rediskeys`.
2. **Wildcard key deletion**:
   - `handlers/workspaces/roles.rs:359,441` calls `delete_pattern(&format!("cache:auth:permissions:{}:*", ctx.workspace_id))`, which in Rust executes Redis `KEYS` followed by `DEL` (`cache/redis_cache.rs:100-105`).
   - **Hazard**: `KEYS` is a blocking $O(N)$ operation that stalls Redis instances in production.
   - **Remedy**: In Go, rewrite to use `SCAN MATCH ... COUNT 100` in a bounded loop or track active permission keys per workspace in a Redis SET.
3. **Session storage clarify**:
   - Refresh tokens and access sessions are primary-stored in PostgreSQL (`sessions` table). Redis is used solely for revoked signature blocks (`jwt:revoked:{sig}`), post-revocation cutoff stamps (`user:revoked_at:{user_id}`), and rotated refresh token reuse detection (`jwt:rotated:{refresh_token}`). Go implementers must not move primary session records to Redis.
4. **No unused Redis commands**:
   - Source scan confirms **no occurrences** of Redis `SADD` (sets), `LPUSH` (lists use `RPUSH` + `BLPOP`/`LPOP`), or custom distributed locks other than the daily housekeeping `SET NX EX`.
