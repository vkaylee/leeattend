# Remove obsolete SaaS workspace surfaces

## Context

LeeAttend now uses one implicit organization, not SaaS workspaces. Active schema/runtime already preserve `organization_settings` and valid employer/department/employment scopes. Cleanup must remove stale workspace/tenant selectors, docs, tooling, and contradictory contracts without deleting required async infrastructure.

## Scope

Remove:

- Active workspace/tenant selectors, routes, claims, headers, browser state, SQL predicates, and compatibility aliases.
- Stale tenant migration/tooling/config references.
- Contradictory workspace language in infrastructure job contracts.
- Stale Rust/distributed-worker documentation that does not describe current Go runtime.
- Misleading UI/docs wording such as workspace selection and old `/api/v1/workspaces/:id/...` routes.

Preserve:

- Singleton `organization_settings` and organization/employer/department/employment authorization/config scopes.
- PostgreSQL `outbox_jobs`, lease/fencing, retry/dead-letter behavior, and `ScheduleOfficeWorker`.
- Redis auth token/revocation/limiter services, report Pub/Sub/transport, and required import/report producers.
- Worker monitor, heartbeats, durable job ledger, idempotency, and negative tests proving workspace rejection/non-exposure.
- Historical ADRs, migration evidence, fixtures, and tracked backups unless separately approved for deletion.

## Dependency graph

```text
ADR-013 / ADR-0005 + schema contract
  -> single-org schema/bootstrap truth
  -> active API, RBAC, repository SQL, frontend

schedule lifecycle
  -> PostgreSQL outbox_jobs
  -> ScheduleOfficeWorker
  -> schedule publication/evidence

report/import APIs
  -> Redis transport + metadata/PubSub
  -> external or future worker contract

Redis
  -> auth token/revocation/limiters
  -> report/import transport
  -> monitor/heartbeat signals
```

## Vertical tasks

### T0 — Inventory and baseline

- Capture current git status; preserve existing user changes.
- Run `./leedevkit --help` and `./leedevkit test --help`.
- Inventory tracked active, historical, generated, and untracked workspace references.
- Classify each match: remove, rewrite, preserve valid scope, preserve historical, or ignore technical `/workspace` mounts/WebSocket `ws`.

Acceptance: inventory complete; queue/Redis not marked removable merely because workspaces are gone.

Checkpoint: review classification before edits.

### T1 — Contract and schema truth

- Verify `apiserver/migrations/testdata/schema_contract.sql` and baseline catalog already reject workspace objects.
- Verify `infra_job_records`/`infra_job_events` are workspace-free.
- Update `docs/decisions/ADR-018-infrastructure-job-truth-and-reconciliation.md` to remove workspace ID and tenant-scoped requirements while preserving durable PostgreSQL ledger, leases, retries, reconciliation, idempotency, and fencing.
- Do not alter schema or queue behavior in this slice.

Acceptance: schema catalog unchanged; docs match singleton organization architecture.

### T2 — Active runtime/API cleanup

- Search Go routes, DTOs, auth claims, middleware, browser state, and repository SQL.
- Remove confirmed client-selectable workspace scope only.
- Preserve flat `/api/v1/...` routes, organization bootstrap, valid business scopes, fail-closed authorization, and old-route rejection tests.
- Re-check report worker path before touching it; graph data may be stale.

Acceptance: no active client workspace selector/header/query/JWT/storage contract; canonical routes and authorization remain intact.

### T3 — Queue boundary decision

- Prove call sites for `apiserver/internal/queue.Worker`.
- Remove it only if repository/build/CI checks prove no runtime or external contract depends on it; update tests/docs together.
- Do not remove `outbox.go`, `ScheduleOfficeWorker`, Redis client/keys, report/import producers, or monitor contracts.
- Treat absent Redis consumers as a separate behavior migration, not workspace cleanup.

Acceptance: no dead generic queue code, or explicit evidence explains why it remains.

### T4 — Documentation and config cleanup

- Rewrite `docs/operations/background-worker-runner.md` to current Go/PostgreSQL/Redis behavior, or clearly mark obsolete material historical.
- Update `docs/features/ot-claim-workflow.md` to flat routes.
- Replace workspace-configured timezone wording in `docs/features/time-tracking-refactor-phase-1.md` with organization settings.
- Remove obsolete `.coveragerc` path only after proving migration script is absent/unused.
- Update `scripts/security_audit.sh` only after active-path inventory; retain auth, revocation, sensitive-log, and forbidden-selector checks.
- Keep historical migration Rust/workspace fixtures and tracked tenant backups unless separately approved.

Acceptance: active docs/tooling do not instruct workspace or obsolete Rust architecture; historical records remain clearly historical.

### T5 — Generated artifacts and frontend wording

- Remove misleading selectable/create workspace UI copy while preserving organization-wide semantics.
- Do not hand-edit generated route cache; remove/regenerate tracked `.tanstack/tmp` only after reference/build proof.
- Preserve negative tests and accessibility/i18n behavior.

Acceptance: no active UI workspace selector flow; generated output matches source.

### T6 — Verification

Run focused checks, then full gates:

- Schema/migration contract and disposable DB checks.
- Outbox claim/lease/retry/dead-letter/fencing tests.
- Schedule worker tests.
- Report/import/Redis transport, auth revocation, login limiter, and rate-limit tests.
- Monitor tests and security audit.
- Active-source grep with documented allowlist.
- `./leedevkit test api`
- `./leedevkit test all`

Inspect per-phase logs. Do not rely on trailing green summary.

## Test impact matrix

| Dimension | Required protection |
|---|---|
| Changed behavior | Workspace selectors/routes disappear; flat routes and singleton organization remain usable. |
| Existing behavior | RBAC, organization config, schedule publication, report export, Redis auth services remain. |
| Boundaries | Old route, forged selector, stale DB, duplicate org row, valid scoped config. |
| State | Fresh DB, stale/partial markers, active/expired lease, completed/dead-letter job. |
| Failures | Redis outage, publisher failure, lease loss, retry exhaustion, migration failure. |
| Security | No client scope selection; fail-closed; no PII/secrets in job docs/payloads/logs. |
| Compatibility | Historical evidence preserved; active consumers updated together; no old-schema upgrade assumed. |
| Operations | Heartbeats, monitor freshness, queue recovery, migration recovery messages. |

## Non-goals

- Do not remove PostgreSQL outbox, Redis, schedule/report worker semantics, retries, leases, heartbeats, or dead-letter handling.
- Do not remove organization/employer/department/employment scope.
- Do not rewrite Git history or silently delete backups/historical fixtures.
- Do not modify `.leedevkit/`.
