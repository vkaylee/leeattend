# Background Job & Worker Contract — Rust to Go Migration Inventory

**Scope:** `apiserver/src/queue/`, `apiserver/src/worker/`, and all asynchronous job producers across `apiserver/src/handlers/`, `apiserver/src/services/`, database triggers, and background schedulers.  
**Ledger Task:** P1-07 (`rust-go-migration-task-ledger.md`)  
**Target Artifact:** `docs/migration/rust-go/job-contract.md`

Every symbol, configuration setting, key, and queue routing rule below is cited with exact `path:line` evidence from the Rust codebase.

---

## 1. Asynchronous Architecture & Queue Topology

### 1.1 Multi-Queue Routing & Priority Dequeue
The background processing infrastructure uses **Redis LIST** keys with three priority tiers (`apiserver/src/queue/job_queue.rs:31-36`):
- `High` (`worker:jobs:high`): User-facing report batches where clients actively await completion via WebSocket or polling (`JOB_PRIORITY = High`).
- `Normal` (`worker:jobs:normal`): Default priority for batch background jobs: leave accrual, bulk attendance reprocessing, import finalization, and bulk terminations.
- `Low` (`worker:jobs:low`): Background maintenance tasks, attendance log ingestion, nightly housekeeping, and contract expiry scans.

#### Dequeue Fairness Algorithm (`apiserver/src/queue/job_queue.rs:202-289`)
- Workers dequeue via **weighted fairness**: an atomic counter `dequeue_counter` (`AtomicU64`, `:75,234`) is incremented on every `dequeue_job` call.
- Every `LOW_QUEUE_WEIGHT`-th call (`LOW_QUEUE_WEIGHT = 10`, `:41,235`):
  - Worker executes a non-blocking `LPOP` against `worker:jobs:low` (`:247-250`).
  - If a low-priority job is available, it returns immediately (guaranteeing low-priority tasks receive at least ~9.1% of capacity).
  - If empty, it falls through to blocking dequeue.
- Normal/Fallback path: blocking `BLPOP` checking keys in left-to-right priority order (`worker:jobs:high`, `worker:jobs:normal`, `worker:jobs:low`) with timeout `poll_timeout_secs` (`:268-274`).

### 1.2 Redis Connection Isolation (HOL Blocking Protection)
To prevent `BLPOP` head-of-line blocking, `RedisPool` provides two distinct connection managers (`apiserver/src/queue/redis_client.rs:44-50,110-120`):
- `conn()` (`conn_manager`): Multiplexed connection for fast non-blocking commands (`HSET`, `GET`, `PUBLISH`, `INCR`, `SET`).
- `blocking_conn()` (`blocking_conn`): Dedicated connection isolated solely for `BLPOP`/`BRPOP`.
*Failure without this isolation*: `BLPOP` holding a shared connection caused concurrent `HSET` result writes and progress `PUBLISH` calls to queue for 27–54 seconds (`job_queue_tests.rs:545-560,652-700`).

### 1.3 Dual-Layer Durability Architecture
The system supports three distinct durability backends depending on the job domain:

1. **Redis Priority Queues with Redis Delivery Store (`apiserver/src/queue/ingestion_delivery.rs:1-60`)**:
   - Used for `LogIngestion` jobs sent from hardware agents.
   - Key: `ingest:delivery:{agent_id}:{msg_id}` (STRING, JSON `DeliveryRecord`, TTL 1,800s / 30m).
   - Atomic Lua script (`record_and_enqueue`, `:178-202`) validates queue capacity, records delivery state (`Ready`), and pushes to `worker:jobs:low`.
   - Worker acquires time-bounded lease (`DEFAULT_LEASE_SECS = 600`, `:58,237-265`) upon claiming.
   - Lease reaper loop (`runner.rs:763-784`) runs every 30s to re-queue expired processing leases as `Retryable` or move to DLQ (`Permanent`).

2. **PostgreSQL Authoritative Infrastructure Ledger (`apiserver/src/services/infra_job_service.rs:1-97`)**:
   - Backed by table `public.infra_job_records` and audit table `public.infra_job_events` (`apiserver/migrations/system/20260810000001_create_infra_job_ledger/up.sql`).
   - States: `dispatching` $\to$ `queued` $\to$ `processing` $\to$ `completed` | `failed` | `retryable` | `dead_letter` | `lost`.
   - Before publishing to Redis, the job record is inserted with `state = 'dispatching'`. Upon successful queue enqueue, transitioned to `queued`.
   - Worker atomically claims the row with `lease_token` (UUID) and `lease_expires_at = now + job_timeout_secs + 30s` (`runner.rs:487-540`).
   - Outcome transition (`completed`, `retryable`, or `dead_letter`) is persisted with audit reason codes (`runner.rs:581-738`).

3. **PostgreSQL Transactional Outbox (`apiserver/src/repositories/diesel_outbox_job_repository.rs:1-88`)**:
   - Table `public.outbox_jobs` in tenant schemas (`apiserver/migrations/tenant/20260609000001_init_tenant/up.sql:1160`).
   - Handled via `OutboxSweepScheduler` (`apiserver/src/worker/outbox_sweep.rs:1-95`), which sweeps pending and stuck jobs using `FOR UPDATE SKIP LOCKED`.

4. **In-Memory Channel (`apiserver/src/worker/pairing_scheduler.rs:77-84`)**:
   - Unbounded Tokio mpsc channel connecting `AttendanceLogService` to `PairingScheduler`.
   - Drained every 2s to deduplicate pairing calculations.

---

## 2. Complete Background Job Inventory

### Job Envelope Schema
All jobs sent over Redis share the outer envelope `Job` (`apiserver/src/queue/job/mod.rs:31-56`):

```json
{
  "job_id": "uuid",
  "parent_job_id": "uuid",
  "batch_index": 0,
  "task_type": "attendance_report",
  "workspace_id": "uuid",
  "payload": {
    "type": "attendance_report",
    "data": { }
  },
  "priority": "high",
  "created_at": "2026-10-19T00:00:00Z",
  "retry_count": 0,
  "max_retries": 3,
  "trace_context": "uuid-string-or-null"
}
```

*Note on Serde Encoding*: `TaskType` is `snake_case` string (`types.rs:13`). `JobPayload` is an internally-tagged enum: `#[serde(tag = "type", content = "data")]` (`types.rs:230`).

---

### Job 1: `AttendanceReport`
- **TaskType**: `TaskType::AttendanceReport` (`apiserver/src/queue/job/types.rs:15`)
- **Trigger**:
  - API Enqueue: `POST /api/reports/export` $\to$ `ReportCoordinator::create_report_job` (`apiserver/src/services/report_coordinator.rs:142-174`).
  - Outbox Sweep: `OutboxSweepScheduler::process_attendance_report` (`apiserver/src/worker/outbox_sweep.rs:162-216`) re-enqueues report jobs stuck $>5$ min.
- **Payload Schema (`ReportPayload`, `types.rs:74-79`)**:
  - `employee_ids`: `Vec<Uuid>` (batched in chunks of 50, `BATCH_SIZE = 50`, `report_coordinator.rs:25`).
  - `date_range`: `DateRange` (`start: NaiveDate`, `end: NaiveDate`).
  - `config`: `ReportConfig` (`report_type: ReportType` [`Summary` | `Detail`], `timezone: String`, `rounding_policy: RoundingPolicy` [`Floor`], `work_unit_minutes: i32` [30]).
- **Storage**:
  - Queue: Redis LIST `worker:jobs:high` (`JobPriority::High`).
  - Outbox Mirror: PostgreSQL tenant table `outbox_jobs` (`job_type = 'AttendanceReport'`).
  - Metadata: Redis STRING `report:meta:{parent_job_id}` + ZSET `report:job_list`.
  - Batch Results: Redis HASH `report:results:{parent_job_id}` (`batch_00000` $\to$ JSON chunk).
- **Visibility / Execution Timeout**:
  - Worker poll: 30s BLPOP (`poll_timeout_secs: 30.0`).
  - Execution timeout: 300s (`job_timeout_secs: 300`, `runner.rs:385-389`).
  - Report max total duration: 3,600s (`JOB_TIMEOUT_SECS = 3600`, `report_coordinator.rs:26`).
- **Retry / Backoff Policy**:
  - Max attempts: 3 (`WORKER_MAX_RETRIES`, `runner.rs:114,623`).
  - Backoff: `durable_retry_backoff(attempt)` = $5 \times 2^{\min(\max(\text{attempt}-1, 0), 6)}$ seconds (5s, 10s, 20s, 40s, 80s, 160s, max 300s) (`runner.rs:35-42`).
  - Failure behavior: Batch failure increments `report:counter:{parent_id}:failed`, updates `report:meta:{parent_id}.error_message`, moves job to `report:dlq`.
- **Idempotency Key**: `(parent_job_id, batch_index)` tuple.
- **Handler**: `AttendanceHandler::handle` (`apiserver/src/worker/handlers/attendance/mod.rs:48-80`).

---

### Job 2: `PayrollReport`
- **TaskType**: `TaskType::PayrollReport` (`apiserver/src/queue/job/types.rs:16`)
- **Trigger**: None (`ORPHAN`). No HTTP endpoint or service enqueues this task type.
- **Payload Schema (`ReportPayload`, `types.rs:74-79`)**:
  - Shared with `AttendanceReport`: `employee_ids: Vec<Uuid>`, `date_range: DateRange`, `config: ReportConfig`.
- **Storage**: Redis LIST `worker:jobs:normal` (`JobPriority::Normal`).
- **Visibility / Execution Timeout**: Poll 30s, worker timeout 300s.
- **Retry / Backoff Policy**: Max retries 3.
- **Idempotency Key**: `(parent_job_id, batch_index)`.
- **Handler**: `PayrollReportHandler::handle` (`apiserver/src/worker/handlers/payroll_report.rs:9-21`).
  - Current implementation: Logs `"PayrollReport task received but not yet implemented (No-Op)"` and returns `Ok(())`.
- **Status**: **ORPHAN** (Consumer exists; no producer exists in codebase).

---

### Job 3: `MergeResults`
- **TaskType**: `TaskType::MergeResults` (`apiserver/src/queue/job/types.rs:17`)
- **Trigger**: Internal batch-completion cascade. Enqueued by `ReportCoordinator::report_batch_completed` (`apiserver/src/services/report_coordinator.rs:405-446`) when all batches finish (`completed_count == total_batches`).
- **Payload Schema (`MergePayload`, `types.rs:82-86`)**:
  - `date_range`: `DateRange` (`start: NaiveDate`, `end: NaiveDate`).
  - `config`: `ReportConfig` (`report_type: ReportType`, `timezone: String`, `rounding_policy: RoundingPolicy`, `work_unit_minutes: i32`).
- **Storage**: Redis LIST `worker:jobs:high` (`JobPriority::High`).
  - Final merged binary: Redis STRING `report:results:{parent_job_id}:data` (TTL 3,600s / 1h) and `report:results:{parent_job_id}:filename` (TTL 3,600s) (`job_queue.rs:606-646`).
- **Visibility / Execution Timeout**: Poll 30s, worker timeout 300s.
- **Retry / Backoff Policy**: Max retries 3.
- **Idempotency Key**: Redis SETNX on `report:merge_triggered:{parent_job_id}` (`job_queue.rs:457-479`). Guarantees exactly one worker executes merge even under concurrent batch completions.
- **Handler**: `MergeHandler::handle` (`apiserver/src/worker/handlers/merge/mod.rs:39-82`).

---

### Job 4: `LeaveAccrual`
- **TaskType**: `TaskType::LeaveAccrual` (`apiserver/src/queue/job/types.rs:18`)
- **Trigger**: API Enqueue: `POST /api/leaves/accrual` (`apiserver/src/handlers/leaves.rs:608-615`).
- **Payload Schema (`AccrualPayload`, `types.rs:89-95`)**:
  - `date`: `NaiveDate` (target accrual calculation date).
  - `force`: `bool` (re-calculate if already calculated).
  - `employee_ids`: `Option<Vec<Uuid>>` (optional subset filter).
- **Storage**: Redis LIST `worker:jobs:normal` (`JobPriority::Normal`).
  - Metadata: Redis STRING `report:meta:{parent_job_id}`.
- **Visibility / Execution Timeout**: Poll 30s, worker timeout 300s.
- **Retry / Backoff Policy**: Max retries 3.
- **Idempotency Key**: Checked in DB against table `leave_accrual_logs` for `(employee_id, leave_type_id, date)` unless `force = true`.
- **Handler**: `AccrualHandler::handle` (`apiserver/src/worker/handlers/accrual.rs:14-72`).

---

### Job 5: `LogIngestion`
- **TaskType**: `TaskType::LogIngestion` (`apiserver/src/queue/job/types.rs:19`)
- **Trigger**: Agent WebSocket incoming frame (`CentralMessage::PunchLog` / `punch_log`) $\to$ `agents.rs:186-248`.
- **Payload Schema (`LogIngestionPayload`, `types.rs:99-124`)**:
  - `workspace_id`: `Uuid`
  - `employee_id_number`: `String`
  - `punch_time_raw`: `String` (raw timestamp from biometric device)
  - `device_id`: `Option<String>`
  - `raw_data`: `Option<serde_json::Value>`
  - `timezone`: `String`
  - `date_fmt`: `String`
  - `time_fmt`: `String`
  - `msg_id`: `Option<String>` (unique UUID per agent message)
  - `agent_id`: `Option<String>` (agent UUID)
  - `source_timestamp`: `Option<String>`
  - `source_timezone`: `Option<String>`
  - `timestamp_kind`: `Option<String>`
- **Storage**:
  - Queue: Redis LIST `worker:jobs:low` (`JobPriority::Low`).
  - Delivery Store: Redis STRING `ingest:delivery:{agent_id}:{msg_id}` (state machine: `Ready` $\to$ `Processing` $\to$ `Processed` | `Retryable` | `Permanent`).
- **Visibility Timeout & Lease**:
  - Lease duration: `DEFAULT_LEASE_SECS = 600` (10 minutes) leased to `worker_id:job_id` (`runner.rs:898-905`).
  - Monitored by periodic reaper loop (`runner.rs:763-784`, runs every 30s).
- **Retry / Backoff Policy**:
  - Max retries: 3.
  - Reaper recovers expired leases: `attempt < max_retries` $\to$ re-enqueued as `Retryable`; `attempt >= max_retries` $\to$ moved to `report:dlq` and marked `Permanent`.
  - Worker notifies agent via `SharedAckSink` (`ack_sink.rs:21-34`): sends `Ack` with `status = "processed"`, `"retryable_error"`, or `"permanent_error"`.
- **Idempotency Key**: `(agent_id, msg_id)` stable composite key in Redis delivery store; PostgreSQL `raw_attendance_logs` uniqueness on `(workspace_id, employee_id, punch_time)`.
- **Handler**: `runner.rs:889-949` $\to$ `LogIngestionHandler::handle` (`apiserver/src/worker/handlers/log_ingestion.rs:29-75`).

---

### Job 6: `ReprocessAttendance`
- **TaskType**: `TaskType::ReprocessAttendance` (`apiserver/src/queue/job/types.rs:20`)
- **Trigger**: API Enqueue: `POST /api/attendance/reprocess` (`apiserver/src/handlers/attendance/logs.rs:61-68`).
- **Payload Schema (`ReprocessPayload`, `types.rs:127-135`)**:
  - `employee_id`: `Uuid`
  - `employment_id`: `Option<Uuid>` (explicit employment if pinned; otherwise auto-resolved)
  - `date_range`: `DateRange` (`start: NaiveDate`, `end: NaiveDate`)
- **Storage**: Redis LIST `worker:jobs:normal` (`JobPriority::Normal`).
  - Metadata: Redis STRING `report:meta:{parent_job_id}`.
- **Visibility / Execution Timeout**: Poll 30s, worker timeout 300s.
- **Retry / Backoff Policy**: Max retries 3.
- **Idempotency Key**: Synchronized via PostgreSQL advisory lock `pg_advisory_xact_lock(employment_id, date)` in pairing engine.
- **Handler**: `ReprocessAttendanceHandler::handle` (`apiserver/src/worker/handlers/reprocess_attendance.rs:16-143`).

---

### Job 7: `ClaimStagedLogs`
- **TaskType**: `TaskType::ClaimStagedLogs` (`apiserver/src/queue/job/types.rs:21`)
- **Trigger**: None (`ORPHAN`).
  - In `employee_service` (`core.rs:78`, `create.rs:61`) and `attendance_import_service.rs:412`, orphan staged logs are claimed **synchronously in-process** via `staging_repo.claim_staged_logs()`.
  - No background enqueue call exists in production routes or services.
- **Payload Schema (`ClaimStagedLogsPayload`, `types.rs:138-143`)**:
  - `workspace_id`: `Uuid`
  - `employee_id`: `Uuid`
  - `employee_id_number`: `String`
- **Storage**: Redis LIST `worker:jobs:normal` (low/normal priority).
- **Visibility / Execution Timeout**: Poll 30s, worker timeout 300s.
- **Retry / Backoff Policy**: Max retries 3.
- **Idempotency Key**: DB query updates unassigned rows in `staging_attendance_logs`.
- **Handler**: `ClaimStagedLogsHandler::handle` (`apiserver/src/worker/handlers/claim_staged_logs.rs:14-80`).
- **Status**: **ORPHAN** (Consumer and test harness exist; producer is executed synchronously in-process).

---

### Job 8: `LeaveImport`
- **TaskType**: `TaskType::LeaveImport` (`apiserver/src/queue/job/types.rs:22`)
- **Trigger**: API Enqueue: `POST /api/leaves/import` (`apiserver/src/handlers/leaves.rs:818-825`).
- **Payload Schema (`LeaveImportPayload`, `types.rs:146-156`)**:
  - `file_path`: `String` (path on local disk to uploaded spreadsheet)
  - `user_id`: `Uuid`
  - `auto_approve`: `bool`
  - `import_job_id`: `Uuid` (tracks row in `attendance_import_jobs` table)
- **Storage**: Redis LIST `worker:jobs:normal` (`JobPriority::Normal`).
  - Progress counters in Redis: `import_progress:{job_id}`, `import_chunks_done:{job_id}`, `import_total_chunks:{job_id}`, `import_errors:{job_id}`.
- **Visibility / Execution Timeout**: Poll 30s, worker timeout 300s.
- **Retry / Backoff Policy**: Max retries 3.
- **Idempotency Key**: `import_job_id` UUID primary key.
- **Handler**: `LeaveImportHandler::handle` (`apiserver/src/worker/handlers/leave_import.rs:22-197`).
  - Splits file into chunks of 500 rows (`get_import_chunk_size()`) and enqueues sub-jobs of type `LeaveImportChunk`.

---

### Job 9: `LeaveImportChunk`
- **TaskType**: `TaskType::LeaveImportChunk` (`apiserver/src/queue/job/types.rs:26`)
- **Trigger**: Sub-job enqueued by master `LeaveImportHandler` (`apiserver/src/worker/handlers/leave_import.rs:172-184`).
- **Payload Schema (`LeaveImportChunkPayload`, `types.rs:191-199`)**:
  - `import_job_id`: `Uuid`
  - `user_id`: `Uuid`
  - `file_path`: `String`
  - `auto_approve`: `bool`
  - `start_index`: `usize`
  - `end_index`: `usize`
- **Storage**: Redis LIST `worker:jobs:normal` (`JobPriority::Normal`).
- **Visibility / Execution Timeout**: Poll 30s, worker timeout 300s.
- **Retry / Backoff Policy**: Max retries 3.
- **Idempotency Key**: `(import_job_id, start_index, end_index)`.
- **Handler**: `LeaveImportChunkHandler::handle` (`apiserver/src/worker/handlers/leave_import.rs:200-332`).
  - On completing the final chunk, enqueues `FinalizeImport` (`:324-325`).

---

### Job 10: `EmployeeImport`
- **TaskType**: `TaskType::EmployeeImport` (`apiserver/src/queue/job/types.rs:24`)
- **Trigger**: API Enqueue: `POST /api/attendance/import/employee` (`apiserver/src/handlers/attendance/import.rs:174-186`).
  - Enqueued via `infra_job_service.enqueue_durable(&queue, job)` (`:184`), creating a durable row in `infra_job_records`.
- **Payload Schema (`EmployeeImportPayload`, `types.rs:172-178`)**:
  - `import_job_id`: `Uuid`
  - `user_id`: `Uuid`
- **Storage**: Redis LIST `worker:jobs:normal` (`JobPriority::Normal`) + PostgreSQL `infra_job_records`.
- **Visibility / Execution Timeout**: Poll 30s, worker timeout 300s, durable lease `lease_expires_at = now + 330s`.
- **Retry / Backoff Policy**: Max retries 3; exponential backoff (5s to 300s).
- **Idempotency Key**: `import_job_id` UUID.
- **Handler**: `EmployeeImportHandler::handle` (`apiserver/src/worker/handlers/employee_import.rs:23-198`).
  - Splits file into chunks of 500 rows and enqueues sub-jobs of type `EmployeeImportChunk`.

---

### Job 11: `EmployeeImportChunk`
- **TaskType**: `TaskType::EmployeeImportChunk` (`apiserver/src/queue/job/types.rs:25`)
- **Trigger**: Sub-job enqueued by master `EmployeeImportHandler` (`apiserver/src/worker/handlers/employee_import.rs:173-195`).
- **Payload Schema (`EmployeeImportChunkPayload`, `types.rs:181-188`)**:
  - `import_job_id`: `Uuid`
  - `user_id`: `Uuid`
  - `file_path`: `String`
  - `start_index`: `usize`
  - `end_index`: `usize`
- **Storage**: Redis LIST `worker:jobs:normal` (`JobPriority::Normal`).
- **Visibility / Execution Timeout**: Poll 30s, worker timeout 300s.
- **Retry / Backoff Policy**: Max retries 3.
- **Idempotency Key**: `(import_job_id, start_index, end_index)`.
- **Handler**: `EmployeeImportChunkHandler::handle` (`apiserver/src/worker/handlers/employee_import.rs:205-321`).
  - On completing the final chunk, enqueues `FinalizeImport` (`:312-313`).

---

### Job 12: `FinalizeImport`
- **TaskType**: `TaskType::FinalizeImport` (`apiserver/src/queue/job/types.rs:27`)
- **Trigger**: Enqueued by `EmployeeImportChunkHandler` (`apiserver/src/worker/handlers/employee_import.rs:313`) and `LeaveImportChunkHandler` (`apiserver/src/worker/handlers/leave_import.rs:325`) upon reaching `chunks_done == total_chunks`.
- **Payload Schema (`FinalizeImportPayload`, `types.rs:202-205`)**:
  - `import_job_id`: `Uuid`
- **Storage**: Redis LIST `worker:jobs:normal` (`JobPriority::Normal`).
- **Visibility / Execution Timeout**: Poll 30s, worker timeout 300s.
- **Retry / Backoff Policy**: Max retries 3.
- **Idempotency Key**: `import_job_id` UUID.
- **Handler**: `FinalizeImportHandler::handle` (`apiserver/src/worker/handlers/finalize_import.rs:12-167`).
  - Cleans up Redis tracking keys (`import_progress`, `import_errors`, `import_chunks_done`, `import_total_chunks`) and deletes temporary spreadsheet files from disk (`:137-165`).

---

### Job 13: `NightlyHousekeeping`
- **TaskType**: `TaskType::NightlyHousekeeping` (`apiserver/src/queue/job/types.rs:23`)
- **Trigger**: Cron / Hourly scheduler: `HousekeepingScheduler::start` (`apiserver/src/worker/scheduler.rs:23-85`).
  - Ticks every 3,600s (1h). Acquires daily Redis lock `lock:housekeeping:{YYYY-MM-DD}` via `SET NX EX 86400` (`:39-46`).
- **Payload Schema (`HousekeepingPayload`, `types.rs:159-163`)**:
  - `reference_date`: `NaiveDate` (current UTC date).
- **Storage**: Redis LIST `worker:jobs:low` (`JobPriority::Low`, `scheduler.rs:59`).
- **Visibility / Execution Timeout**: Poll 30s, worker timeout 300s.
- **Retry / Backoff Policy**: Max retries 3.
- **Idempotency Key**: Distributed Redis lock `lock:housekeeping:{today}` ensures only one enqueue per calendar day across all cluster nodes.
- **Handler**: `NightlyHousekeepingHandler::handle` (`apiserver/src/worker/handlers/housekeeping.rs:15-177`).
  - Provisions monthly PostgreSQL table partitions (`raw_attendance_logs`, `time_entries`) for current and upcoming month across all workspaces (`:50-130`).

---

### Job 14: `ExpiringContractsCheck`
- **TaskType**: `TaskType::ExpiringContractsCheck` (`apiserver/src/queue/job/types.rs:28`)
- **Trigger**: Cron / Hourly scheduler: `HousekeepingScheduler::start` (`apiserver/src/worker/scheduler.rs:65-79`).
  - Enqueued alongside `NightlyHousekeeping` upon acquiring the daily lock.
- **Payload Schema (`ExpiringContractsPayload`, `types.rs:166-169`)**:
  - `reference_date`: `NaiveDate`.
- **Storage**: Redis LIST `worker:jobs:low` (`JobPriority::Low`, `scheduler.rs:74`).
- **Visibility / Execution Timeout**: Poll 30s, worker timeout 300s.
- **Retry / Backoff Policy**: Max retries 3.
- **Idempotency Key**: Redis lock `lock:housekeeping:{today}`.
- **Handler**: `ExpiringContractsHandler::handle` (`apiserver/src/worker/handlers/expiring_contracts.rs:9-160`).
  - Queries contracts expiring in 30, 15, and 7 days; writes notifications to `notifications` table.

---

### Job 15: `SyncEmployeeHireDate`
- **TaskType**: `TaskType::SyncEmployeeHireDate` (`apiserver/src/queue/job/types.rs:29`)
- **Trigger**: Domain event hook: Contract creation or contract date modification in `apiserver/src/handlers/employee_contracts.rs:40-49`.
- **Payload Schema (`SyncHireDatePayload`, `types.rs:208-212`)**:
  - `employee_id`: `Uuid`
  - `triggered_by_contract_id`: `Option<Uuid>`
- **Storage**: Redis LIST `worker:jobs:normal` (`JobPriority::Normal`).
- **Visibility / Execution Timeout**: Poll 30s, worker timeout 300s.
- **Retry / Backoff Policy**: Max retries 3.
- **Idempotency Key**: Recalculates `employees.hire_date` as $\min(\text{contract.start\_date})$.
- **Handler**: `SyncHireDateHandler::process` (`apiserver/src/worker/handlers/sync_hire_date.rs:10-151`).
---

### Job 16: `BulkTermination`
- **TaskType**: `TaskType::BulkTermination` (`apiserver/src/queue/job/types.rs:30`)
- **Trigger**: API Enqueue: `POST /api/employees/bulk-terminate` (`apiserver/src/handlers/employees.rs:700-709`).
- **Payload Schema (`BulkTerminationPayload`, `types.rs:223-226`)**:
  - `rows`: `Vec<BulkTerminationRow>`
    - `employment_id`: `Uuid`
    - `termination_date`: `NaiveDate`
    - `termination_reason`: `String`
    - `termination_notes`: `Option<String>`
- **Storage**: Redis LIST `worker:jobs:normal` (`JobPriority::Normal`).
  - Metadata: Redis STRING `report:meta:{parent_job_id}`.
- **Visibility / Execution Timeout**: Poll 30s, worker timeout 300s.
- **Retry / Backoff Policy**: Max retries 3.
- **Idempotency Key**: Iterates rows; terminates each employment within its transaction.
- **Handler**: `BulkTerminationHandler::handle` (`apiserver/src/worker/handlers/bulk_termination.rs:11-85`).

---

### Job 17: Outbox Job — `RosterCacheInvalidation`
- **Job Type String**: `"RosterCacheInvalidation"`
- **Trigger**: PostgreSQL Tenant AFTER Trigger: `enqueue_roster_cache_invalidation()` on 11 roster-affecting tables:
  `shift_templates`, `shift_versions`, `employee_shifts`, `employee_working_patterns`, `working_pattern_details`, `working_patterns`, `assignment_rules`, `employments`, `employees`, `shift_default_intervals`, `work_segments` (`apiserver/migrations/tenant/20260811000000_roster_cache_invalidation_outbox/up.sql:5-141`).
- **Payload Schema**: Empty JSON object `{}`.
- **Storage**: PostgreSQL tenant table `public.outbox_jobs` (`job_type = 'RosterCacheInvalidation'`, `status = 'pending'`).
- **Poll / Interval**: Checked every 30 seconds by `OutboxSweepScheduler::sweep_once` (`apiserver/src/worker/outbox_sweep.rs:60-69`).
  - Claims up to 100 jobs older than 30s using `FOR UPDATE SKIP LOCKED`.
- **Handler Action**: `OutboxSweepScheduler::process_roster_cache_invalidation` (`apiserver/src/worker/outbox_sweep.rs:97-160`):
  - Increments Redis `roster:version:{workspace_id}` generation UUID (`cache.bump_version(ws_id)`).
  - Evicts local Moka cache (`cache.evict_local(ws_id)`).
  - Updates `outbox_jobs.status = 'completed'`.

---

### Job 18: Outbox Job — `WorkflowCompleted`
- **Job Type String**: `"WorkflowCompleted"`
- **Trigger**: Approval Workflow Terminal State: `ApprovalWorkflowEngine::evaluate` (`apiserver/src/services/approval_workflow/engine.rs:537-544`) when workflow instance transitions to `approved`, `rejected`, or `returned`.
- **Payload Schema (`WorkflowCompletedPayload`, `apiserver/src/models/outbox_job.rs:6-13`)**:
  - `entity_type`: `String` (e.g., `"leave_request"`)
  - `entity_id`: `Uuid`
  - `final_status`: `String` (`"approved"` | `"rejected"` | `"returned"`)
  - `actor_user_id`: `Uuid`
  - `trace_context` (serde rename `_trace_context`): `Option<serde_json::Value>`
- **Storage**: PostgreSQL tenant table `public.outbox_jobs` (`job_type = 'WorkflowCompleted'`).
- **Handler**: `WorkflowCompletedHandler::handle` (`apiserver/src/worker/handlers/workflow_completed.rs:24-65`).
  - Dispatches to registered `WorkflowCallbackHandler` for the entity type.
- **Production Wiring Status**: **UNWIRED CONSUMER / OPERATIONAL DEFECT**.
  - In `apiserver/src/app_builder/workers.rs:65-69`, `OutboxSweepScheduler` is initialized without `with_workflow_handler(...)`.
  - When stuck jobs sweep runs (`outbox_sweep.rs:218-274`), `self.workflow_handler` is `None`, emitting an error: `"No workflow handler registered for WorkflowCompleted jobs"`.

---

### Job 19: Background Sweep — `FileScanScheduler`
- **Job Nature**: Autonomous background polling loop (`apiserver/src/worker/file_scan_scheduler.rs:11-57`).
- **Trigger**: Periodic interval every 30s (`interval(Duration::from_secs(30))`).
- **Storage**: PostgreSQL tables `workspace_files` and `system_files` (querying `scan_status = 'Pending'`).
- **Handler Action (`process_pending_files`, `:60-188`)**:
  - Iterates all workspaces via stream (`buffer_unordered(5)`).
  - Downloads pending file bytes via `FileService::download_file_direct`.
  - Invokes ClamAV / VirusScanner via `scan_service.scan(&bytes)`.
  - Updates file row to `'Clean'` or `'Infected'`.

---

### Job 20: In-Memory Stream — `PairingScheduler`
- **Job Nature**: In-process deferred pairing batch loop (`apiserver/src/worker/pairing_scheduler.rs:1-120`).
- **Trigger**: `AttendanceLogService::process_new_raw_log` sends `PairingNotification` over an unbounded Tokio mpsc channel (`tx.send()`, `:68-74`).
- **Payload Schema (`PairingNotification`, `:45-49`)**:
  - `workspace_id`: `Uuid`
  - `employment_id`: `Uuid`
  - `punch_time`: `DateTime<Utc>`
- **Storage**: Memory only (Tokio unbounded channel, `pairing_channel()`, `:78-84`).
- **Interval**: Drains channel every 2s (`DEFAULT_SWEEP_INTERVAL_SECS = 2`, `:95`, or `PAIRING_SWEEP_INTERVAL_SECS` env).
- **Handler Action (`sweep`, `:156-300`)**:
  - Drains all pending notifications.
  - Groups punches by `(workspace_id, employment_id)` and derives local calendar dates using the employee's timezone.
  - Applies overnight cutoff rule: punches before 12:00 PM also reprocess the previous day (`OVERNIGHT_CUTOFF_HOUR = 12`, `:92,243`).
  - Batches timezone lookups (1 SQL query per workspace, `get_employee_timezones_batch`).
  - Invokes `PairingProcessor::reprocess_attendance_range` **once per employment** instead of per punch, eliminating PostgreSQL `pg_advisory_xact_lock` contention on high-volume ingress.

---

## 3. Producer / Consumer Table

Proves every producer has a consumer and vice versa. Highlights orphans with evidence.

| ID | Task / Job Name | Producer Symbol (`path:line`) | Consumer Symbol (`path:line`) | Queue / Storage Key | Match Status | Notes |
|:---|:---|:---|:---|:---|:---:|:---|
| J01 | `AttendanceReport` | `report_coordinator.rs:209` (`create_report_job`) & `outbox_sweep.rs:173` | `runner.rs:969` $\to$ `attendance/mod.rs:48` (`AttendanceHandler::handle`) | Redis LIST `worker:jobs:high` & PG `outbox_jobs` | **PROVEN** | User report generation |
| J02 | `PayrollReport` | **NONE** (No route/service enqueues this job) | `runner.rs:983` $\to$ `payroll_report.rs:9` (`PayrollReportHandler::handle`) | Redis LIST `worker:jobs:normal` | <mark>**ORPHAN**</mark> | Handler is a no-op placeholder |
| J03 | `MergeResults` | `report_coordinator.rs:425` (`report_batch_completed`) | `runner.rs:980` $\to$ `merge/mod.rs:39` (`MergeHandler::handle`) | Redis LIST `worker:jobs:high` | **PROVEN** | Cascade merge on parent completion |
| J04 | `LeaveAccrual` | `handlers/leaves.rs:608` (`trigger_accrual`) | `runner.rs:984` $\to$ `accrual.rs:14` (`AccrualHandler::handle`) | Redis LIST `worker:jobs:normal` | **PROVEN** | Manual/API accrual trigger |
| J05 | `LogIngestion` | `handlers/agents.rs:214,230` (`websocket_handler`) | `runner.rs:987` $\to$ `log_ingestion.rs:29` (`LogIngestionHandler::handle`) | Redis LIST `worker:jobs:low` + `ingest:delivery:*` | **PROVEN** | Durable delivery & agent lease |
| J06 | `ReprocessAttendance` | `handlers/attendance/logs.rs:61` (`reprocess_attendance`) | `runner.rs:988` $\to$ `reprocess_attendance.rs:16` (`ReprocessAttendanceHandler`) | Redis LIST `worker:jobs:normal` | **PROVEN** | Bulk manual attendance recalculation |
| J07 | `ClaimStagedLogs` | **NONE** in queues (Synchronous in `employee_service/core.rs:79`) | `runner.rs:991` $\to$ `claim_staged_logs.rs:14` (`ClaimStagedLogsHandler`) | Redis LIST `worker:jobs:normal` | <mark>**ORPHAN**</mark> | Async job path unused in prod |
| J08 | `LeaveImport` | `handlers/leaves.rs:818` (`import_leave_requests`) | `runner.rs:992` $\to$ `leave_import.rs:22` (`LeaveImportHandler::handle`) | Redis LIST `worker:jobs:normal` | **PROVEN** | Excel leave import orchestrator |
| J09 | `LeaveImportChunk` | `leave_import.rs:178` (`LeaveImportHandler::handle_internal`) | `runner.rs:1016` $\to$ `leave_import.rs:200` (`LeaveImportChunkHandler`) | Redis LIST `worker:jobs:normal` | **PROVEN** | 500-row chunk processor |
| J10 | `EmployeeImport` | `handlers/attendance/import.rs:175` (`import_employees`) | `runner.rs:996` $\to$ `employee_import.rs:23` (`EmployeeImportHandler::handle`) | Redis LIST `worker:jobs:normal` + `infra_job_records` | **PROVEN** | Durable employee import master |
| J11 | `EmployeeImportChunk`| `employee_import.rs:178` (`EmployeeImportHandler::handle_internal`) | `runner.rs:1000` $\to$ `employee_import.rs:205` (`EmployeeImportChunkHandler`) | Redis LIST `worker:jobs:normal` | **PROVEN** | 500-row chunk processor |
| J12 | `FinalizeImport` | `employee_import.rs:306` & `leave_import.rs:319` | `runner.rs:1009` $\to$ `finalize_import.rs:12` (`FinalizeImportHandler`) | Redis LIST `worker:jobs:normal` | **PROVEN** | Cleans Redis keys & temp files |
| J13 | `NightlyHousekeeping`| `scheduler.rs:50` (`HousekeepingScheduler`) | `runner.rs:1024` $\to$ `housekeeping.rs:15` (`NightlyHousekeepingHandler`) | Redis LIST `worker:jobs:low` | **PROVEN** | Provisions DB table partitions |
| J14 | `ExpiringContractsCheck`| `scheduler.rs:66` (`HousekeepingScheduler`) | `runner.rs:1012` $\to$ `expiring_contracts.rs:9` (`ExpiringContractsHandler`) | Redis LIST `worker:jobs:low` | **PROVEN** | Sends 30/15/7-day contract notices |
| J15 | `SyncEmployeeHireDate`| `handlers/employee_contracts.rs:41` | `runner.rs:1027` $\to$ `sync_hire_date.rs:10` (`SyncHireDateHandler::process`) | Redis LIST `worker:jobs:normal` | **PROVEN** | Contract hire date resync |
| J16 | `BulkTermination` | `handlers/employees.rs:701` (`bulk_terminate_employments`) | `runner.rs:1028` $\to$ `bulk_termination.rs:11` (`BulkTerminationHandler`) | Redis LIST `worker:jobs:normal` | **PROVEN** | Multi-employee termination |
| O01 | `RosterCacheInvalidation`| PG Triggers `trg_roster_cache_invalidation_*` (11 tables) | `outbox_sweep.rs:97` (`process_roster_cache_invalidation`) | PostgreSQL table `outbox_jobs` | **PROVEN** | Evicts Moka & updates Redis gen |
| O02 | `WorkflowCompleted` | `engine.rs:538` (`ApprovalWorkflowEngine::evaluate`) | `outbox_sweep.rs:218` $\to$ `workflow_completed.rs:24` | PostgreSQL table `outbox_jobs` | <mark>**UNWIRED**</mark> | Sweep consumer lacks handler in DI |
| S01 | `FileScanScheduler` | Autonomous timer (30s) | `file_scan_scheduler.rs:60` (`process_pending_files`) | PostgreSQL `workspace_files` | **PROVEN** | Virus scan on uploaded files |
| S02 | `PairingScheduler` | `attendance_log_service.rs:541` (`notifier.notify`) | `pairing_scheduler.rs:156` (`PairingScheduler::sweep`) | Memory `mpsc::UnboundedReceiver` | **PROVEN** | 2s punch deduplication loop |

---

## 4. Concurrency Limits & Worker Pool Sizing

| Parameter | Rust Source | Env Var / Default | Purpose |
|:---|:---|:---|:---|
| Max Concurrent Jobs | `runner.rs:111,123,134` | `WORKER_MAX_CONCURRENT_JOBS` = `5` | Tokio `Semaphore` permits governing concurrent worker executions |
| Worker Poll Timeout | `runner.rs:113,124,138` | `WORKER_POLL_TIMEOUT_SECS` = `30.0`s | Maximum wait per `BLPOP` before cycle (clamped 0.1s–300s) |
| Max Job Retries | `runner.rs:115,125,144` | `WORKER_MAX_RETRIES` = `3` | Attempts before marking `dead_letter` in ledger or moving to `report:dlq` |
| Job Execution Timeout | `runner.rs:117,126,148` | `WORKER_JOB_TIMEOUT_SECS` = `300`s | `tokio::time::timeout` wrapper around handler execution |
| Max Priority Queue Length | `job_queue.rs:51` | `MAX_QUEUE_LENGTH` = `100,000` | Redis Lua enqueue backpressure limit |
| Max DLQ Length | `job_queue.rs:54` | `MAX_DLQ_LENGTH` = `10,000` | Trim threshold for `report:dlq` via `LTRIM` |
| Max Serialized Job Size | `job_queue.rs:57` | `MAX_SERIALIZED_JOB_BYTES` = `1,048,576` (1MB) | Rejects oversized JSON payloads before Redis write |
| Database Worker Pool Size | `diesel_pool/config.rs:23,102` | `worker_max_conn` = `5` | Dedicated Diesel connection pool for worker database queries |
| DB Acquire Timeout | `diesel_pool/config.rs:25,104` | `DB_WORKER_ACQUIRE_TIMEOUT_SECS` = `30`s | Pool checkout deadline for background tasks |
| Ingestion Lease Duration | `ingestion_delivery.rs:58` | `DEFAULT_LEASE_SECS` = `600`s (10m) | Safe window before reaper recovers dead worker claim |
| Ingestion Reaper Interval | `runner.rs:30` | `REAP_INTERVAL_SECS` = `30`s | Lease expiration sweep frequency |
| Worker Heartbeat Interval | `runner.rs:46,49` | Interval `5`s, TTL `90`s | Heartbeat key `monitor:worker:{worker_id}` |
| Pairing Sweep Interval | `pairing_scheduler.rs:95` | `PAIRING_SWEEP_INTERVAL_SECS` = `2`s | Notification channel batching window |
| Outbox Sweep Interval | `outbox_sweep.rs:44` | Fixed `30`s | Interval for PostgreSQL outbox polling |
| File Scan Interval | `file_scan_scheduler.rs:41`| Fixed `30`s | Interval for pending file virus scans |

---

## 5. Shutdown Behavior & Lifecycle Guarantees

Grounding: `apiserver/src/main.rs:244-272,350-376`, `apiserver/src/worker/runner.rs:278-485`, `apiserver/src/agent_manager.rs:367-386`.

### 5.1 Signal Trap (`main.rs:350-376`)
- Handles POSIX signals `SIGINT` (Ctrl+C) and `SIGTERM` via `tokio::select!`.
- Passes signal to `axum::serve.with_graceful_shutdown()`, halting new HTTP connection acceptance.

### 5.2 Shutdown Sequence Order
1. **HTTP Termination (`main.rs:244-246`)**: Server stops accepting inbound HTTP traffic; active requests finish.
2. **Worker Cancellation Signal (`main.rs:252`)**: Calls `workers.job_worker_token.cancel()`.
   - Propagates cancellation to `JobWorker`, `HousekeepingScheduler`, `OutboxSweepScheduler`, `FileScanScheduler`, and `PairingScheduler`.
3. **Queue Dequeue Abort (`runner.rs:279-299`)**:
   - The worker loop detects `cancel_token.is_cancelled()`.
   - If currently blocked inside `worker.queue.dequeue_job`, `tokio::select!` cancels the BLPOP future immediately, releases the reserved semaphore permit (`drop(permit)`), and breaks the loop (`:293-296`).
4. **In-Flight Task Drain (`runner.rs:455-475`)**:
   - In-flight jobs run in decoupled `tokio::spawn` tasks and are **not killed**. They are permitted to run until completion or individual 300s timeout.
   - The runner measures pending jobs: `pending = max_concurrent_jobs - semaphore.available_permits()`.
   - Runner invokes `semaphore.acquire_many(max_concurrent_jobs)`. Because permits are held until tasks finish, this blocks until **100% of in-flight jobs finish** (`:464-470`).
5. **Heartbeat & Reaper Cleanup (`runner.rs:477-483`)**:
   - Deletes Redis heartbeat key `monitor:worker:{worker_id}`.
   - Awaits termination of `heartbeat_task` and `reaper_task`.
6. **Task Join in Main (`main.rs:253-264`)**:
   - Awaits `workers.job_worker_handle.await`.
   - Awaits `workers.outbox_sweep_handle.await`.
7. **Agent Disconnect Notice (`main.rs:266`, `agent_manager.rs:367-386`)**:
   - Iterates all active agent WebSocket connections and sends system frame `SERVER_SHUTDOWN` (`"API Server is shutting down gracefully."`).
8. **Resource Drop (`main.rs:268`)**: Drops DB connection pools and Redis pool.

### 5.3 Crash & Kill Recovery
- **Process Abruptly Killed (`SIGKILL` / OOM)**:
  - Jobs popped from Redis but not finished are recovered automatically:
    - `LogIngestion`: Lease expires after 600s; another worker's `reap_loop` finds it and re-pushes to `worker:jobs:low` (`runner.rs:772-780`).
    - Durable Jobs (`infra_job_records`): `reap_expired_durable_leases` (`runner.rs:792-847`) scans `state = 'processing' AND lease_expires_at < now`, transitions to `retryable`, and re-enqueues.
    - Outbox Jobs (`outbox_jobs`): Swept after 5 minutes by `OutboxSweepScheduler` via `FOR UPDATE SKIP LOCKED`.

---

## 6. Go Target Mapping

### 6.1 Package Structure & Topology
```text
internal/
├── worker/
│   ├── runner.go              # Semaphore worker loop, BLPop dequeue, weighted fairness
│   ├── config.go              # WorkerConfig (concurrency, timeouts, retries)
│   ├── types.go               # Job envelope, TaskType constants, JobPayload interface
│   ├── reaper.go              # 30s lease reaper for Redis & PostgreSQL ledgers
│   ├── heartbeat.go           # 5s heartbeat runner
│   └── handlers/
│       ├── attendance_report.go
│       ├── merge_results.go
│       ├── leave_accrual.go
│       ├── log_ingestion.go
│       ├── reprocess_attendance.go
│       ├── leave_import.go
│       ├── employee_import.go
│       ├── finalize_import.go
│       ├── housekeeping.go
│       ├── expiring_contracts.go
│       ├── sync_hire_date.go
│       └── bulk_termination.go
├── scheduler/
│   ├── housekeeping.go        # 1h ticker + Redis SET NX EX lock
│   ├── outbox_sweep.go        # 30s ticker + FOR UPDATE SKIP LOCKED
│   ├── file_scan.go           # 30s ticker ClamAV virus scanner
│   └── pairing.go             # 2s channel drain + overnight deduplication
└── infra/
    ├── queue/
    │   └── redis_queue.go     # Priority routing, BLPop, LPop fairness, DLQ
    └── rediskeys/
        └── keys.go            # Centralized typed key constructors
```

### 6.2 Go Payload Structs Matching Rust Wire Contract
All structs must maintain JSON field parity with `serde`:

```go
package worker

import (
	"encoding/json"
	"time"

	"github.com/google/uuid"
)

type TaskType string

const (
	TaskAttendanceReport       TaskType = "attendance_report"
	TaskPayrollReport          TaskType = "payroll_report"
	TaskMergeResults           TaskType = "merge_results"
	TaskLeaveAccrual           TaskType = "leave_accrual"
	TaskLogIngestion           TaskType = "log_ingestion"
	TaskReprocessAttendance    TaskType = "reprocess_attendance"
	TaskClaimStagedLogs        TaskType = "claim_staged_logs"
	TaskLeaveImport            TaskType = "leave_import"
	TaskNightlyHousekeeping    TaskType = "nightly_housekeeping"
	TaskEmployeeImport         TaskType = "employee_import"
	TaskEmployeeImportChunk    TaskType = "employee_import_chunk"
	TaskLeaveImportChunk       TaskType = "leave_import_chunk"
	TaskFinalizeImport         TaskType = "finalize_import"
	TaskExpiringContractsCheck TaskType = "expiring_contracts_check"
	TaskSyncEmployeeHireDate   TaskType = "sync_employee_hire_date"
	TaskBulkTermination        TaskType = "bulk_termination"
)

type JobPriority string

const (
	PriorityHigh   JobPriority = "high"
	PriorityNormal JobPriority = "normal"
	PriorityLow    JobPriority = "low"
)

// Generic Job Envelope matching Rust Job
type Job struct {
	JobID        uuid.UUID       `json:"job_id"`
	ParentJobID  uuid.UUID       `json:"parent_job_id"`
	BatchIndex   uint32          `json:"batch_index"`
	TaskType     TaskType        `json:"task_type"`
	WorkspaceID  uuid.UUID       `json:"workspace_id"`
	Payload      json.RawMessage `json:"payload"` // Envelope {"type": "...", "data": {...}}
	Priority     JobPriority     `json:"priority"`
	CreatedAt    time.Time       `json:"created_at"`
	RetryCount   uint32          `json:"retry_count"`
	MaxRetries   uint32          `json:"max_retries"`
	TraceContext *string         `json:"trace_context,omitempty"`
}

// Tagged payload envelope {"type": "...", "data": {...}}
type TaggedPayload[T any] struct {
	Type string `json:"type"`
	Data T      `json:"data"`
}

type DateRange struct {
	Start string `json:"start"` // YYYY-MM-DD
	End   string `json:"end"`   // YYYY-MM-DD
}

type ReportConfig struct {
	ReportType      string `json:"report_type"` // "summary" | "detail"
	Timezone        string `json:"timezone"`
	RoundingPolicy  string `json:"rounding_policy"`
	WorkUnitMinutes int32  `json:"work_unit_minutes"`
}

type ReportPayload struct {
	EmployeeIDs []uuid.UUID  `json:"employee_ids"`
	DateRange   DateRange    `json:"date_range"`
	Config      ReportConfig `json:"config"`
}

type MergePayload struct {
	DateRange DateRange    `json:"date_range"`
	Config    ReportConfig `json:"config"`
}

type AccrualPayload struct {
	Date        string       `json:"date"` // YYYY-MM-DD
	Force       bool         `json:"force"`
	EmployeeIDs *[]uuid.UUID `json:"employee_ids,omitempty"`
}

type LogIngestionPayload struct {
	WorkspaceID      uuid.UUID       `json:"workspace_id"`
	EmployeeIDNumber string          `json:"employee_id_number"`
	PunchTimeRaw     string          `json:"punch_time_raw"`
	DeviceID         *string         `json:"device_id,omitempty"`
	RawData          json.RawMessage `json:"raw_data,omitempty"`
	Timezone         string          `json:"timezone"`
	DateFmt          string          `json:"date_fmt"`
	TimeFmt          string          `json:"time_fmt"`
	MsgID            *string         `json:"msg_id,omitempty"`
	AgentID          *string         `json:"agent_id,omitempty"`
	SourceTimestamp  *string         `json:"source_timestamp,omitempty"`
	SourceTimezone   *string         `json:"source_timezone,omitempty"`
	TimestampKind    *string         `json:"timestamp_kind,omitempty"`
}

type ReprocessPayload struct {
	EmployeeID   uuid.UUID  `json:"employee_id"`
	EmploymentID *uuid.UUID `json:"employment_id,omitempty"`
	DateRange    DateRange  `json:"date_range"`
}

type ClaimStagedLogsPayload struct {
	WorkspaceID      uuid.UUID `json:"workspace_id"`
	EmployeeID       uuid.UUID `json:"employee_id"`
	EmployeeIDNumber string    `json:"employee_id_number"`
}

type LeaveImportPayload struct {
	FilePath    string    `json:"file_path"`
	UserID      uuid.UUID `json:"user_id"`
	AutoApprove bool      `json:"auto_approve"`
	ImportJobID uuid.UUID `json:"import_job_id"`
}

type LeaveImportChunkPayload struct {
	ImportJobID uuid.UUID `json:"import_job_id"`
	UserID      uuid.UUID `json:"user_id"`
	FilePath    string    `json:"file_path"`
	AutoApprove bool      `json:"auto_approve"`
	StartIndex  int       `json:"start_index"`
	EndIndex    int       `json:"end_index"`
}

type EmployeeImportPayload struct {
	ImportJobID uuid.UUID `json:"import_job_id"`
	UserID      uuid.UUID `json:"user_id"`
}

type EmployeeImportChunkPayload struct {
	ImportJobID uuid.UUID `json:"import_job_id"`
	UserID      uuid.UUID `json:"user_id"`
	FilePath    string    `json:"file_path"`
	StartIndex  int       `json:"start_index"`
	EndIndex    int       `json:"end_index"`
}

type FinalizeImportPayload struct {
	ImportJobID uuid.UUID `json:"import_job_id"`
}

type HousekeepingPayload struct {
	ReferenceDate string `json:"reference_date"` // YYYY-MM-DD
}

type ExpiringContractsPayload struct {
	ReferenceDate string `json:"reference_date"` // YYYY-MM-DD
}

type SyncHireDatePayload struct {
	EmployeeID             uuid.UUID  `json:"employee_id"`
	TriggeredByContractID *uuid.UUID `json:"triggered_by_contract_id,omitempty"`
}

type BulkTerminationRow struct {
	EmploymentID      uuid.UUID `json:"employment_id"`
	TerminationDate   string    `json:"termination_date"` // YYYY-MM-DD
	TerminationReason string    `json:"termination_reason"`
	TerminationNotes  *string   `json:"termination_notes,omitempty"`
}

type BulkTerminationPayload struct {
	Rows []BulkTerminationRow `json:"rows"`
}

type WorkflowCompletedPayload struct {
	EntityType   string           `json:"entity_type"`
	EntityID     uuid.UUID        `json:"entity_id"`
	FinalStatus  string           `json:"final_status"`
	ActorUserID  uuid.UUID        `json:"actor_user_id"`
	TraceContext *json.RawMessage `json:"_trace_context,omitempty"`
}
```

### 6.3 Goroutine Pool & Weighted Dequeue Implementation
In Go, worker concurrency is implemented using a buffered permit channel (`chan struct{}` of capacity `maxConcurrent`) combined with `sync.WaitGroup`:

```go
func (w *Worker) Run(ctx context.Context) error {
	sem := make(chan struct{}, w.config.MaxConcurrent)
	var wg sync.WaitGroup

	for {
		select {
		case <-ctx.Done():
			// Wait for in-flight jobs to complete
			wg.Wait()
			w.removeHeartbeat(context.Background())
			return nil
		case sem <- struct{}{}:
			// Permit acquired
		}

		job, err := w.dequeueWithFairness(ctx)
		if err != nil {
			<-sem
			if ctx.Err() != nil {
				wg.Wait()
				return nil
			}
			time.Sleep(1 * time.Second)
			continue
		}
		if job == nil {
			<-sem
			continue
		}

		wg.Add(1)
		go func(j *Job) {
			defer func() {
				<-sem
				wg.Done()
			}()
			w.processJobWithTimeout(ctx, j)
		}(job)
	}
}

func (w *Worker) dequeueWithFairness(ctx context.Context) (*Job, error) {
	ticket := atomic.AddUint64(&w.dequeueCounter, 1)
	isLowTurn := ticket%10 == 0

	if isLowTurn {
		// Non-blocking LPop on low queue
		res, err := w.redisBlockingClient.LPop(ctx, w.keys.QueueLow()).Result()
		if err == nil && res != "" {
			return unmarshalJob(res)
		}
	}

	// BLPop checking high -> normal -> low
	timeout := time.Duration(w.config.PollTimeoutSecs) * time.Second
	res, err := w.redisBlockingClient.BLPop(ctx, timeout, 
		w.keys.QueueHigh(), w.keys.QueueNormal(), w.keys.QueueLow()).Result()
	if errors.Is(err, redis.Nil) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return unmarshalJob(res[1])
}
```

### 6.4 Backoff & Retry Implementation
```go
func DurableRetryBackoff(attempt int) time.Duration {
	exp := attempt - 1
	if exp < 0 {
		exp = 0
	}
	if exp > 6 {
		exp = 6
	}
	multiplier := int64(1 << exp)
	secs := 5 * multiplier
	if secs > 300 {
		secs = 300
	}
	return time.Duration(secs) * time.Second
}
```

### 6.5 Graceful Shutdown Protocol
1. Context propagation: root context cancelled on `SIGTERM` / `SIGINT`.
2. HTTP server: `server.Shutdown(shutdownCtx)`.
3. Worker runner: cancels loop, halts `BLPop`, waits on `wg.Wait()`.
4. Reaper & Schedulers: `select` on `ctx.Done()`, join routines.
5. Agent WebSocket: broadcast `SERVER_SHUTDOWN` frame before socket termination.
6. DB & Redis: Close connection pools.
