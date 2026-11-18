# ADR-018: Infrastructure Job Truth and Reconciliation

- **Status:** Accepted
- **Date:** 2026-08-10
- **Author:** Lee + AI
- **Domain:** Background jobs, imports, reports, and `/infra/monitor`

## Context

The infrastructure monitor currently combines Redis queue lengths, Redis counters, worker heartbeats, and best-effort job metadata. Those values are useful signals but are not an authoritative record of job existence or lifecycle. A Redis `BLPOP` removes a job before durable worker claim state exists; Redis-only import progress can disappear on restart; report parent/child counters can diverge; and an open SSE connection can remain technically connected while snapshots stop arriving.

The operational failure mode is silent ambiguity: an operator sees zero, empty, or live-looking data without being able to determine whether there is genuinely no work, a dependency is unavailable, a worker died between transitions, or a record was lost.

## Decision

PostgreSQL is the authoritative source of infrastructure job lifecycle. Redis remains a delivery transport and cache, not the source of truth.

Every accepted asynchronous job must have a durable record before Redis dispatch. Every lifecycle mutation must be represented by an append-only event and an idempotent state transition. The supported states are:

```text
dispatching -> queued -> processing
                       ├-> completed
                       ├-> failed
                       ├-> retryable -> queued
                       ├-> dead_letter
                       └-> lost
```

A job may be delivered more than once. Handlers must therefore remain idempotent, and stale workers must be fenced from committing after lease ownership changes.

The monitor API must use one snapshot builder for REST and SSE. Snapshot responses include freshness, data quality, unknown fields, dependency error codes, reconciliation counts, and an ordered event/sequence identifier. Missing or failed sources are never converted to zero.

## Durable record requirements

`infra_job_records` stores only operational metadata:

- stable job ID and optional parent/batch ID;
- task type/priority within the single implicit organization;
- current state, attempt count, lease owner/deadline, and next retry time;
- created/queued/claimed/finished timestamps;
- sanitized error code and summary;
- request ID, trace ID, and payload hash;
- encrypted replay envelope only when the task is explicitly approved for replay.

`infra_job_events` is append-only and stores transition type, previous/new state, attempt, actor/worker identity, correlation IDs, sanitized reason, and event timestamp. Raw payloads, tokens, passwords, and T1/T2 data are not written to logs or returned by monitor list endpoints.

Retention and purge must be implemented as an audited housekeeping operation. Any replay capability requires authorization, encryption-key management, payload minimization, and a documented retention period.

## Transition invariants

1. A job cannot become `queued` unless its durable record exists.
2. Only the current lease owner/fencing token may renew or complete a processing job.
3. A terminal transition is idempotent and cannot be overwritten by a stale attempt.
4. Retryable transitions persist next-attempt metadata before requeue.
5. A popped-but-unclaimed job is recoverable by reconciliation.
6. A job that cannot be recovered is explicitly marked `lost`; it is never represented as an ordinary zero/empty dashboard state.
7. Parent jobs aggregate child outcomes by stable child identity; duplicate callbacks do not overcount.
8. Business authorization remains explicit at the handler boundary; no client-selected organization or workspace scope is part of job identity.

## Rollout and compatibility

The first rollout is additive and observable:

1. deploy migrations and ledger writes;
2. dual-read Redis and ledger in reconciliation-only mode;
3. measure mismatches and repair producers/consumers;
4. switch monitor snapshots to ledger-backed counts with explicit quality metadata;
5. enable the new UI presentation using `ff_infra_monitor_truthful_states`;
6. remove the flag after full rollout and the required review window.

The existing numeric jobs cursor remains temporarily supported. The opaque composite cursor is introduced additively as `cursor_token`/`next_cursor_token` and becomes the frontend default before the numeric cursor is deprecated. No existing monitor field is removed in the first API version.

## Consequences

### Positive

- Operators can trace a job from creation through recovery or loss.
- Redis outages and worker crashes become recoverable, measurable states.
- REST and SSE have consistent semantics.
- UI consumers can distinguish no data from unavailable data.
- Retry, DLQ, duplicate, and reconciliation behavior becomes testable and auditable.

### Costs and risks

- PostgreSQL writes increase for job creation and transitions.
- Schema, worker, import, report, API, and frontend changes must be rolled out in order.
- Replay envelopes require careful encryption, access control, retention, and key rotation.
- Existing Redis-only records require a migration/reconciliation policy; records that cannot be reconstructed must be reported as historical unknowns rather than fabricated as completed.

## Rejected alternatives

- **Redis as the authoritative ledger:** rejected because queue pop, TTL expiry, counter loss, and partial writes make historical reconstruction impossible.
- **UI-only hardening:** rejected because a truthful UI cannot infer jobs that were never durably recorded.
- **SSE-only freshness:** rejected because a connected transport does not prove that a valid, current snapshot was received.
- **Unbounded event sourcing of business payloads:** rejected for PII, retention, and storage reasons; only sanitized operational events are retained.

## Verification gate

This ADR is complete only when migration rollback, queue crash recovery, import/report restart recovery, Redis/DB failure behavior, monitor REST/SSE consistency, and browser-visible stale/degraded states are covered by repository-wrapper integration and E2E tests. Structured telemetry must demonstrate request/trace/job correlation without PII.
