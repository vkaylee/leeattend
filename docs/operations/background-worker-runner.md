# Background Worker Runner

## Scope

LeeAttend runs background work inside the Go `apiserver` process. PostgreSQL remains authoritative for durable schedule work; Redis provides transport, cache, authentication state, rate limiting, and report progress/pub/sub. No separate worker container or Rust runtime is part of the current deployment.

## Runtime topology

```text
HTTP request
  ├─ PostgreSQL outbox_jobs
  │    └─ ScheduleOfficeWorker
  │         ├─ claim with FOR UPDATE SKIP LOCKED
  │         ├─ lease heartbeat and fencing token
  │         ├─ publish official schedule work
  │         └─ complete, retry, or dead-letter
  └─ Redis report/import transport
       ├─ report metadata, progress, result, Pub/Sub
       └─ report worker consumes report jobs
```

`organization_settings` is singleton organization context. Job identity does not contain a client-selected workspace or tenant ID. Business authorization still uses authenticated identity, permissions, ownership, employer, department, and employment boundaries.

## PostgreSQL schedule outbox

### `ScheduleOfficeWorker`

Source: `apiserver/internal/service/schedule_office_worker.go`.

- Scans the bounded schedule horizon on startup and every 15 minutes.
- Claims `ScheduleOfficeCoverageRefresh` jobs through `ScheduleOfficeOutboxStore`.
- Uses a five-minute lease and one-minute heartbeat.
- Validates payloads before publication.
- Publishes official schedule work through the schedule repository.
- Completes only while the lease token is still valid.
- Routes malformed payloads, publisher failures, and lease loss through the failure path.
- Publishes and removes worker heartbeats during lifecycle changes.
- Shuts down through the application hook and drains the active processing path.

### Outbox invariants

Source: `apiserver/internal/repository/outbox.go`.

- Enqueue is durable in PostgreSQL before work is considered accepted.
- `ClaimScheduleOfficeJob` uses a transaction and `FOR UPDATE SKIP LOCKED`.
- Pending jobs and expired processing leases are claimable.
- Each claim increments attempts and receives a UUID lease token.
- Heartbeat, completion, and failure require the current lease token.
- Failed jobs retry with bounded backoff; exhausted jobs become `dead_letter`.
- Dedupe keys prevent duplicate schedule coverage work.
- A stale worker cannot complete a job after lease ownership changes.

PostgreSQL schedule revisions, publication batches, evidence, and audit rows remain business truth. The outbox is transport/work scheduling, not the schedule lifecycle ledger.

## Redis report worker

Source: `apiserver/internal/repository/report_worker.go` and `reports.go`.

- `ReportRepository.CreateExportJob` records the accepted report job.
- `RedisReportQueue` publishes the report envelope to `worker:jobs:high` and stores report metadata.
- `ReportWorker` consumes the high-priority queue with cancellation-aware `BLPOP`.
- Large employee selections are chunked; report pages are read with bounded page size.
- Progress and final result metadata are stored in Redis with expiry.
- Report result streams use Redis Pub/Sub for WebSocket progress updates.
- Invalid envelopes and invalid request IDs are ignored without executing work.
- Redis outage follows the repository's explicit report availability/fallback policy; it must not be represented as fabricated success.

Employee import also uses `EmployeeImportQueue` and has a synchronous database fallback for supported operations when Redis is unavailable. Do not remove these producers as part of workspace cleanup without a separate behavior migration and replacement consumer.

## Redis services unrelated to queue transport

Redis remains required for:

- JWT/session revocation and token state.
- Login and HTTP rate limiting.
- Report metadata, results, and progress streams.
- Worker heartbeat and monitor signals.
- Supported import/report transport.

Removing SaaS workspace scope does not remove Redis.

## Shutdown and recovery

The application registers worker shutdown hooks. A worker crash or expired lease makes eligible PostgreSQL schedule work claimable by a later scan. Lease fencing prevents stale workers from committing terminal state. Operators must preserve job IDs and event history, avoid manual deletion of Redis keys or PostgreSQL ledger rows, and use the infrastructure reliability runbook for reconciliation, retry, and dead-letter incidents.

## Security and data handling

Job payloads and operational logs must contain only minimized identifiers and required business fields. Never write passwords, tokens, raw attendance rows, email addresses, phone numbers, salary data, or unmasked PII to job payloads, logs, monitor responses, or replay material. Authorization is checked at the application boundary; clients cannot select organization scope.

## Verification

Run focused tests for:

- outbox deduplication, claim ordering, lease heartbeat, stale-lease fencing, retry backoff, and dead-letter transitions;
- schedule worker malformed payload, publisher failure, lease loss, startup scan, and shutdown;
- Redis report queue lifecycle, result storage, cancellation, Pub/Sub, and Redis-unavailable behavior;
- employee import Redis fallback;
- auth revocation, login limiting, rate limiting, and monitor freshness/degraded states.

Then run the applicable full LeeDevKit target. Inspect per-phase logs; a final green summary alone is not sufficient evidence.
