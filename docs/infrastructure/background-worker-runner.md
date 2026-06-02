# Background Worker Runner Architecture

This document describes the design, implementation, and operations of the background worker runner model in **LeeAttend**. The background system is responsible for handling long-running, resource-intensive, or asynchronous operations (e.g., generating report batches, importing employees/leaves, reprocessing attendance records, nightly maintenance) without blocking the responsive HTTP interface of the `apiserver`.

---

## 1. System Overview & Architecture

LeeAttend employs a distributed, asynchronous job-processing pipeline built around a **Postgres Outbox Pattern** for durability, a **Redis Queue** for fast delivery and horizontal scaling, and a **Semaphore-controlled Worker Pool** for execution.

```
                  ┌────────────────────────────────────────┐
                  │            Client Request              │
                  └───────────────────┬────────────────────┘
                                      │ (HTTP POST)
                                      ▼
                        ┌───────────────────────────┐
                        │      apiserver (Axum)     │
                        └─────────────┬─────────────┘
                                      │
               ┌──────────────────────┴──────────────────────┐
               │ 1. Transaction                              │ 2. Async Enqueue
               ▼                                             ▼
  ┌─────────────────────────┐                   ┌─────────────────────────┐
  │      PostgreSQL         │                   │      Redis Queue        │
  │   (outbox_jobs table)   │                   │  (report:jobs:queue)    │
  └─────────────────────────┘                   └────────────┬────────────┘
               ▲                                             │
               │                                             │ (BLPOP Poll)
               │ (Update state / status)                     ▼
               │                                ┌─────────────────────────┐
               └────────────────────────────────┤     JobWorker Pool      │
                                                └─────────────────────────┘
                                                             │ (Execute Task)
                                                             ▼
                                                ┌─────────────────────────┐
                                                │   Task-Specific Handler │
                                                └─────────────────────────┘
```

---

## 2. Core Components

### 2.1. `ReportCoordinator`
- **Location**: [`report_coordinator.rs`](file:///home/elt1541/lee/leeattend/apiserver/src/services/report_coordinator.rs)
- **Role**: Coordinates distributed jobs, generates metadata, and splits large datasets into manageable micro-batches.
- **Batching**: Splits massive operations (e.g., generating attendance reports for 7,000+ employees) into micro-batches of `BATCH_SIZE = 50`. Each batch handles up to 50 employees, which keeps peak memory consumption within a safe ~50MB limit per container.

### 2.2. `OutboxJobService`
- **Location**: [`outbox_job_service.rs`](file:///home/elt1541/lee/leeattend/apiserver/src/services/outbox_job_service.rs)
- **Role**: Standardized API for managing Postgres-persisted background jobs.
- **Outbox Pattern**: Before enqueuing any job payload into Redis, the service writes the job's parameters and states directly to the PostgreSQL `outbox_jobs` table. This guarantees that jobs are never lost in case of a Redis failover or worker crash.

### 2.3. `JobWorker` (Runner)
- **Location**: [`runner.rs`](file:///home/elt1541/lee/leeattend/apiserver/src/worker/runner.rs)
- **Role**: The main worker engine. It polls Redis queues using cancellation-aware asynchronous `BLPOP` loops, dynamically routes tasks to their corresponding handlers, and updates status metrics.
- **Concurrency**: Governed by a `tokio::sync::Semaphore` to cap concurrent execution threads and protect workers from out-of-memory (OOM) crashes.

### 2.4. `OutboxSweepScheduler`
- **Location**: [`outbox_sweep.rs`](file:///home/elt1541/lee/leeattend/apiserver/src/worker/outbox_sweep.rs)
- **Role**: A background reconciliation routine that acts as a reliability layer.
- **Reconciliation**: Runs every **30 seconds** looking for stuck jobs older than **5 minutes** (jobs still in `pending` or `running` state). When a stuck job is found, it deserializes the saved Outbox payload and safely re-enqueues it to the Redis pipeline.

---

## 3. Outbox State Transitions & Reconciliation

To ensure 100% execution guarantees, the background worker relies on a dual-database state tracking model:

### 3.1. Job States in Postgres
1. **`pending`**: Initial state immediately after the client requests an action. The outbox row is created in Postgres before pushing the ticket to Redis.
2. **`running`**: Updated when a worker dequeues a task and begins processing or when the reconciliation loop initiates a re-enqueue.
3. **`completed`**: Updated upon successful processing of the job (or after a batch-based job's final merge is finished).
4. **`failed`**: Updated when a handler encounters an unrecoverable error or when a re-enqueue operation fails.

```
       [ Client Request ]
               │
               ▼
         ┌───────────┐
         │  pending  │  (Postgres Record Created)
         └─────┬─────┘
               │ (Worker pops from Redis)
               ▼
         ┌───────────┐
         │  running  │  (Worker Begins Execution)
         └─────┬─────┘
               │
       ┌───────┴───────┐
       ▼               ▼ (Succeeded)
 ┌───────────┐   ┌───────────┐
 │  failed   │   │ completed │
 └───────────┘   └───────────┘
```

### 3.2. Reconciliation Parameters
- **Check Interval**: Runs every **30 seconds**.
- **Stuck Threshold**: Jobs older than **5 minutes** (based on `updated_at` time stamp) that are in `pending` or `running` state.
- **Payload Recovery**: For `AttendanceReport` jobs, the scheduler deserializes a structured payload containing `CreateJobRequest` and `requester_id`, updates the outbox state to `running` to prevent double-sweeping, and passes it to `ReportCoordinator::enqueue_report_job_internal` for re-enqueuing.

---

## 4. Worker Configuration & Concurrency Control

### 4.1. Concurrency Management
Each worker process uses a `tokio::sync::Semaphore` to limit concurrent job handling.
- **Permit Acquisition**: The worker loop acquires a permit from the semaphore *before* polling a job from Redis. If the maximum concurrency is reached, the worker suspends polling until in-flight jobs complete and release permits.
- **Graceful Shutdown**: On cancellation signals, the runner attempts to acquire all permits to guarantee that all in-flight jobs complete safely before exiting.

### 4.2. Configuration Environment Variables
Configuration is loaded via `WorkerConfig::from_env()` with the following variables:

| Environment Variable | Default Value | Purpose |
|----------------------|---------------|---------|
| `DB_WORKER_MAX_CONN` | `5` | Maximum concurrent jobs handled by a single worker process (`max_concurrent_jobs`). |
| `WORKER_POLL_TIMEOUT_SECS` | `30.0` | BLPOP blocking timeout for polling the Redis queue. Must be `> 0` and `<= 300.0`. |
| `WORKER_MAX_RETRIES` | `3` | Maximum retry attempts for transient job failures. |
| `WORKER_JOB_TIMEOUT_SECS` | `300` | Job execution timeout (in seconds). Handlers exceeding this are forcibly aborted. |

### 4.3. Worker Heartbeat and Health Monitoring
Each `JobWorker` instance registers its presence and monitors health by writing heartbeats to Redis:
- **Redis Key Pattern**: `monitor:worker:{worker_id}`
- **Heartbeat Interval**: Every **5 seconds**.
- **Heartbeat TTL**: **15 seconds** (if 3 consecutive heartbeats are missed, the worker is considered dead).
- **Payload (`WorkerHeartbeat`)**:
  - `id`: Unique UUID of the worker.
  - `host`: The system hostname running the container.
  - `status`: `IDLE` or `PROCESSING`.
  - `current_task_type`: Name of the active task type (if processing).
  - `last_seen_at`: Timestamp of the last heartbeat.

---

## 5. Connection Pooling & Deadlock Prevention

### 5.1. The Single Connection Constraint
To avoid draining connection pool capacity and causing severe application-wide deadlocks:
- **Rule**: Every job thread/handler MUST strictly check out and use **exactly one database connection** at any given time.
- **Anti-pattern**: Acquiring a database connection, starting an async operation, and then checking out *another* connection from the pool (e.g., via nested transactions, helper methods, or triggering parallel db calls). If the connection pool is saturated, checking out the second connection will block indefinitely, resulting in a deadlock.

### 5.2. Deadlock Detection in Tests
This constraint is validated during integration tests:
- **Location**: [`worker_pool.rs`](file:///home/elt1541/lee/leeattend/apiserver/tests/integration/misc/worker_pool.rs)
- **Mechanism**: The test suite initializes the application DB pool with `max_connections(1)`. It then executes all registered `TaskType` jobs sequentially. If any handler violates the single connection rule by attempting to acquire a second connection, the test immediately deadlocks. The test is wrapped in a 15-second timeout and fails if a deadlock occurs, ensuring regressions are caught before shipping.

---

## 6. Job Execution & Task Routing

The `JobWorker` matches dequeued jobs against its handler registry inside `process_job`:

### 6.1. Task Routing Directory

| Task Type | Payload Struct | Handler Method |
|-----------|----------------|----------------|
| `AttendanceReport` | `ReportPayload` | `AttendanceHandler::handle` |
| `MergeResults` | `MergePayload` | `MergeHandler::handle` |
| `LeaveAccrual` | `AccrualPayload` | `AccrualHandler::handle` |
| `LogIngestion` | `LogIngestionPayload` | `LogIngestionHandler::handle` |
| `ReprocessAttendance` | `ReprocessPayload` | `ReprocessAttendanceHandler::handle` |
| `ClaimStagedLogs` | `ClaimStagedLogsPayload` | `ClaimStagedLogsHandler::handle` |
| `LeaveImport` | `LeaveImportPayload` | `LeaveImportHandler::handle` |
| `LeaveImportChunk` | `LeaveImportChunkPayload` | `LeaveImportChunkHandler::handle` |
| `EmployeeImport` | `EmployeeImportPayload` | `EmployeeImportHandler::handle` |
| `EmployeeImportChunk` | `EmployeeImportChunkPayload` | `EmployeeImportChunkHandler::handle` |
| `FinalizeImport` | `FinalizeImportPayload` | `FinalizeImportHandler::handle` |
| `NightlyHousekeeping` | `HousekeepingPayload` | `NightlyHousekeepingHandler::handle` |
| `SyncEmployeeHireDate` | `SyncHireDatePayload` | `SyncHireDateHandler::process` |

### 6.2. Batch Aggregation vs Standalone Jobs
- **Standalone Jobs** (e.g., `LogIngestion`, `SyncEmployeeHireDate`): Execute directly and complete independently.
- **Batch Jobs** (e.g., `AttendanceReport`):
  1. The `ReportCoordinator` splits a report request into multiple micro-jobs.
  2. Workers process individual micro-jobs and report completion back to the coordinator using `ReportCoordinator::on_batch_complete`.
  3. The coordinator increments completion counters in Redis.
  4. Once `completed_batches >= total_batches`, the coordinator enqueues a final `MergeResults` task to consolidate the results and write the final output (e.g., generating the Excel report, uploading to S3, and updating the outbox status to `completed`).
