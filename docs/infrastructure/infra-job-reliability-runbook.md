# Infrastructure Job Reliability Runbook

## Scope

Use this runbook for `/infra/monitor` incidents involving missing jobs, stale snapshots, Redis/worker failures, import/report divergence, retry backlog, dead-letter records, or explicit `lost` transitions.

The monitor is authoritative only when its banner reports `data_quality=complete` and the freshness age is within the configured SLO. `Live` describes valid recent snapshots, not merely an open SSE socket.

## First response

1. Record the monitor's snapshot time, freshness state, data-quality state, dependency error code, reconciliation lag, and affected job IDs.
2. Copy the displayed job ID, request ID, and trace ID. Do not copy raw payloads or unmasked PII into incident channels.
3. Check whether the issue is isolated to one workspace or affects the system-wide ledger.
4. Check `lost`, `dead_letter`, `retryable`, expired lease, and reconciliation-mismatch counts before retrying anything.
5. Preserve the original event history. Do not manually delete Redis keys or ledger rows as an initial response.

## Symptom matrix

| Symptom | Likely cause | First action |
|---|---|---|
| `Stale` while SSE says connected | Server stream loop or snapshot builder is failing | Inspect `monitor-error` events and server logs by request/trace ID; verify REST freshness |
| `Unavailable` or `Partial` overview | Redis, DB, or heartbeat scan failure | Identify the typed dependency error; do not interpret missing counters as zero |
| `lost > 0` | Reconciler could not recover a durable job/transport mismatch | Inspect event history and payload hash; quarantine before replay |
| Growing `retryable` backlog | Downstream dependency or handler failure | Group by sanitized error code and task type; confirm retry backoff is progressing |
| Growing `dead_letter` count | Permanent validation/business failures or exhausted retries | Review error code and run the task-specific recovery procedure |
| Job is `processing` past lease deadline | Worker crash, network partition, or stalled handler | Confirm lease owner heartbeat; allow fencing/reconciliation to requeue or mark lost |
| Parent report remains running | Child callback, merge, or outbox convergence failure | Inspect parent/child event history; do not create a second merge manually |
| Import progress stops after restart | Process-local task or Redis-only progress path | Inspect durable chunk records and resume through the idempotent finalizer |
| Jobs missing from a filtered page | Cursor or source mismatch | Retry with the opaque composite cursor and compare ledger totals; never infer absence from an empty page alone |

## Correlation and logs

Search structured production logs by `job_id`, then narrow by `request_id`, `trace_id`, `workspace_id`, and transition event. Log fields must be sanitized. Never search or paste access tokens, passwords, raw import rows, emails, phone numbers, or replay envelopes.

Expected lifecycle event names include:

- `infra_job_created`
- `infra_job_queued`
- `infra_job_claimed`
- `infra_job_lease_expired`
- `infra_job_requeued`
- `infra_job_completed`
- `infra_job_failed`
- `infra_job_dead_lettered`
- `infra_job_lost`
- `infra_job_duplicate_suppressed`
- `infra_job_reconciliation_mismatch`

## Recovery procedures

### Redis outage or transport mismatch

1. Confirm PostgreSQL ledger availability and snapshot data quality.
2. Stop manual queue mutation until the outage scope is known.
3. Restore Redis connectivity and run the reconciler in observe mode.
4. Requeue only records in `queued`/`retryable` whose idempotency key is not already leased or terminal.
5. Verify queue depth, ledger state counts, and event ordering after recovery.
6. Escalate any record that cannot be reconstructed as `lost` with a reason; do not silently mark it completed.

### Worker crash or expired lease

1. Confirm the worker heartbeat is stale and record the lease owner/fencing token.
2. Allow the lease reaper to transition the job to `retryable` and requeue it.
3. If the handler is not safe to retry, quarantine the job and require task-owner approval before replay.
4. Verify the stale worker cannot commit a terminal transition after recovery.
5. Inspect duplicate suppression and business side effects before closing the incident.

### Import failure

1. Identify the import and workspace from the job ledger, not from Redis progress alone.
2. Inspect durable chunk states and sanitized error codes.
3. Retry only failed/retryable chunks through the idempotent chunk service.
4. Re-run finalization after all required chunks have terminal outcomes.
5. Verify aggregate progress and final import status are derived from durable chunks.

### Report/outbox divergence

1. Inspect parent job and all child batch events by stable IDs.
2. Check for duplicate callbacks, merge trigger state, and outbox lease/attempt data.
3. Allow the convergence sweep to renew or complete the outbox record before manual action.
4. Requeue only when the ledger transition is idempotently reserved.
5. Verify the parent cannot report `completed` while a required child is failed or missing.

### Explicit `lost` record

1. Treat `lost` as a data-integrity incident, not a normal failure count.
2. Preserve the event history and payload hash.
3. Determine whether the source request can be safely re-submitted with the same idempotency key.
4. If replay is authorized, use the encrypted replay path and record the operator/action audit event.
5. If replay is not possible, communicate the exact job ID, workspace, transition gap, and affected business operation; never fabricate success.

## Release and rollback checks

Before enabling `ff_infra_monitor_truthful_states`, verify:

- migration up/down in staging;
- reconciliation mismatch count is understood and historical unknowns are documented;
- Redis and DB failure tests pass;
- `lost`, DLQ, stale snapshot, and queue-age alerts have runbooks;
- REST and SSE return the same snapshot quality and freshness semantics;
- Playwright confirms stale/degraded/unavailable states are visible on desktop and mobile.

To roll back the UI, disable the feature flag while retaining ledger writes and reconciliation. Do not roll back the ledger migration after records have been written unless the documented database disaster-recovery procedure is approved. Preserve event history for incident review.
