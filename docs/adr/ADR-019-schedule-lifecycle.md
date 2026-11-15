# ADR-019: Hybrid Schedule Lifecycle

| Property | Value |
|---|---|
| Status | Accepted; implementation in progress, production activation pending |
| Date | 2026-11-13 |
| Owner | LeeAttend backend and attendance domain owners |
| Scope | Schedule preview, publication, amendment, attendance closure, and official schedule consumers |
| Source | `schedule-lifecycle.md`; `schedule-lifecycle-spec.md` |

## Context

LeeAttend currently calculates schedules on read through `resolveEffectiveShift` and returns a paginated schedule view. Current precedence is roster, individual working pattern, assignment rule, then fixed shift. The resolver currently uses the current shift representation, reads only `sort_order = 0` intervals, performs rotation arithmetic with elapsed hours, and has no official schedule revision read path.

Current schedule inputs and consumers are mutable and not lifecycle-aware:

- `shift_rosters`, working patterns, assignment rules, fixed shifts, contracts, assignments, holidays, department history, and shift templates feed schedule resolution.
- `work_segments` support CRUD and bulk replacement but have no schedule revision link or immutable official-state guard.
- Reports read segments directly; no-show logic still has a synthetic `09:00–17:00` path.
- Payroll periods already provide employer/month locking, versioning, `FOR UPDATE`, and same-transaction audit, but attendance closure is not yet a prerequisite.
- Audit accepts UUID actors; system actions need a nullable actor ID plus explicit system actor metadata.
- Outbox producers exist, but the source does not prove a PostgreSQL outbox consumer or schedule publisher.

Mutable source inputs cannot provide reproducible evidence for publication, amendment, closure, or later audit. Missing schedules also cannot safely be interpreted as days off. A lifecycle needs explicit distinction between provisional resolution, official work, explicit off, missing coverage, conflict, superseded history, and closed attendance evidence.

## Decision

Adopt a hybrid schedule lifecycle that reuses `resolveEffectiveShift` for both preview and publication. Do not create a second resolver, microservice, event-sourcing platform, or second payroll lock.

### Schedule states and identity

- Preview is provisional, persisted only in a short-lived sanitized `schedule_previews` record, and never official evidence.
- Official identity is `(employment_id, business_date)`, not employee ID.
- A current official revision is either `published` or `explicit_off`, selected by a partial unique index.
- Historical revisions remain immutable and can become `superseded`.
- `missing` and `conflict` are resolver outcomes. They never become current official rows; both block publication or closure as specified.
- `explicit_off` requires an explicit nullable schedule source, not absence of a matching schedule.
- Revision numbers use `MAX + 1` over all revisions while holding the employment lock. Legacy backfill rows use `legacy_unverified` provenance and never satisfy closure coverage.

Each official revision stores source snapshot, resolver version, employer, timezone, actor metadata, reason, publication timestamp, and links to its publication batch. Official segments link to the exact revision and selected shift version, preserve every interval and break, and store concrete timezone-aware timestamps. Published segments cannot be updated or deleted.

### Publication and amendment

- Office publication requires an approved effective policy and runs through a transaction.
- Rotating schedules publish through an explicit batch route.
- First publication cannot replace an existing current revision.
- Amendment uses a separate replan preview and server-generated diff/hash confirmation. It creates a new revision and supersedes the expected current revision without mutating prior content.
- Publication time comes from the server transaction. Clients cannot supply `published_at`.
- Late publication requires human authority, amendment permission, reason, and explicit retroactive confirmation. System publication cannot publish retrospectively.
- Every mutating request requires idempotency. Same key and canonical request replay the stored response; same key with a different request returns conflict.
- Audit insertion is part of the same transaction. Audit failure rolls back publication, revision, segment, and batch writes.

### Attendance closure and payroll interaction

Introduce attendance periods and immutable numbered closures keyed by employer and month. Closing captures exact schedule revision, attendance, calculation, and evidence snapshots for every employment/day. Missing coverage, conflicts, open punches, and unresolved required approvals block closure.

Reopen changes only attendance period state and version, retains prior closure snapshots, and requires reason and permission. Reclose creates a new immutable closure. Payroll periods remain the only payroll lock table; payroll locking requires closed attendance, while payroll lock blocks schedule amendment and recalculation. Late punches remain ingestible and create reconciliation signals without rewriting closed snapshots.

### Authorization and scoping

Add schedule permissions for read, preview, office publication, rotating publication, amendment, evidence export, and history; add attendance closure read, close, and reopen permissions. Authorization runs before business parsing or schedule existence checks.

Scope is evaluated at business date:

- Global managers can operate across employments.
- Employer-scoped managers operate within effective employer scope.
- Department-scoped shift managers can publish rotating schedules and amend within managed departments, but cannot close or reopen.
- Department-scoped team leads are read-only within managed departments.
- Employees can read only their own employment.

System office publication uses `actor_id = NULL`, `actor_type = 'system'`, and `actor_name = 'schedule-publisher'`. Actor fields are server-derived and never accepted from HTTP clients.

### API and consumer contract

Preserve `GET /api/v1/schedules` fields and add official metadata. Default to `view=official` after activation; official reads return current revision data and report missing days as missing, without synthetic times. Add endpoints for preview, office publication, rotating publication, replan preview/commit, history, evidence, close, and reopen according to the approved specification.

Move readers to the official contract: schedules, segments, reports, no-show, matching, attendance summaries, overtime, and payroll. Remove synthetic no-show schedule generation after cutover. Guard all schedule-affecting legacy writers, including roster, segment, pattern, assignment, shift, adjustment, import, bulk, and reprocess paths. Raw punch and raw attendance-log ingestion remains allowed after closure.

The office publisher stays in the Go process. It scans a bounded future horizon, uses durable outbox jobs with dedupe and leases, retries up to five attempts, and recovers committed publication through idempotency. No notification or acknowledgement is treated as publication evidence.

## Alternatives

### Continue calculating schedules on read

Rejected. It cannot preserve the exact decision, source inputs, revision, timezone conversion, or interval set used for a published day. It also cannot support reliable closure or reproducible evidence.

### Persist only one mutable schedule row

Rejected. Updates would destroy history, make amendments indistinguishable from original publication, and prevent exact closure snapshots.

### Treat missing as explicit off

Rejected. Absence of a source is not authorization for a day off and would hide coverage failures. Missing must remain visible and block closure.

### Create a second schedule resolver or service

Rejected. Duplicated precedence logic would drift from preview and existing schedule behavior. Reuse `resolveEffectiveShift` in the existing Go runtime.

### Use payroll periods as attendance closure

Rejected. Payroll lock and attendance closure have different state, evidence, reopen, and retention semantics. Payroll remains dependent on closure; it does not replace it.

### Use notifications or acknowledgements as publication proof

Rejected. Delivery and acknowledgement are asynchronous user-facing outcomes, not transactional schedule evidence.

### Use Redis or outbox delivery as the lifecycle ledger

Rejected. Delivery can be duplicated, lost, or reclaimed. PostgreSQL revisions, batches, closures, and audit rows are authoritative; outbox is transport/work scheduling only.

## Consequences

### Positive

- Official schedule state becomes reproducible by employment and business date.
- Preview, publication, amendment, closure, and evidence have explicit boundaries.
- Prior revisions, intervals, breaks, source decisions, actors, and reasons remain auditable.
- Missing, explicit off, conflict, late publication, and closed states cannot be silently collapsed.
- Reports, no-show, matching, OT, and payroll consume one official contract.
- Idempotent transactions and immutable closure snapshots protect against retries, races, and partial writes.
- Existing API fields and resolver ownership remain reusable during migration.

### Costs and constraints

- Requires additive schema, transaction services, guards, API changes, reader/writer cutover, and frontend state handling.
- PostgreSQL storage and locking increase for revisions, segments, previews, batches, closures, and audit evidence.
- Existing legacy data needs provenance-aware backfill and reconciliation; it cannot be promoted to official evidence without proof.
- Every schedule-affecting writer must honor lifecycle and closure guards.
- Feature activation must be per employer and must stop on integrity mismatch, unauthorized legacy mutation, shadow mismatch, or publisher failure thresholds.
- After official evidence exists, rollback is roll-forward only; lifecycle data and compatibility evidence cannot be deleted to restore the legacy path.

## Security and operational impact

- Check authenticated permission and business-date scope before querying schedule existence to avoid authorization leakage.
- Validate UUIDs, dates, IANA timezones, range limits, actor ownership, preview expiry, expected revisions, idempotency hashes, and unknown JSON fields at trust boundaries.
- Never store or return employee names, codes, email, phone, salary, raw punches, tokens, or credentials in lifecycle snapshots, audit metadata, preview results, or job payloads.
- Protect official segments, shift versions, revisions, closure days, and evidence with restrictive foreign keys and database triggers.
- Preserve human UUID attribution. Represent system actors with nullable UUID plus fixed actor type/name; never pass a string as `actor_id`.
- Use the documented lock order for publication, amendment, close, reopen, and payroll operations. Keep outbox claim transactions separate from domain locks.
- Monitor publication latency, missing and late coverage, resolver conflicts, stale revisions, lock waits, retries, dead letters, and shadow mismatches. Alert before shift start when required office coverage is missing.
- Retain immutable closure evidence for the attendance retention period and audit records for the documented audit period. Preview cleanup must not remove publication evidence.

## Migration approach

1. Add the lifecycle schema and guards additively. Do not edit the baseline migration. Include guarded `up.sql` and `down.sql` with an owned migration marker.
2. Backfill at most one `legacy_unverified` revision per employment/day under employment locks. Keep legacy segments unlinked and preserve original IDs and time-entry links.
3. Extend the existing resolver for calendar-date rotation, effective shift versions, all intervals, breaks, timezone conversion, overlays, explicit off, missing, and rule ties.
4. Add preview, publication, amendment, history, evidence, attendance closure, and outbox worker capabilities with transaction and authorization tests.
5. Run shadow reads and reconciliation against legacy output. Resolve all multi-interval, overnight, off, missing, effective-version, and timezone mismatches.
6. Enable lifecycle per employer only after zero schema integrity errors, current-key duplicates, unauthorized legacy writes, and activation-scope shadow mismatches; abort at the specified publisher failure or dead-letter thresholds.
7. Cut over official readers and guarded writers together. Reject old application versions that cannot enforce lifecycle guards.
8. Remove legacy compatibility reads and writes only after all employers reconcile, no old application remains connected, evidence exports reproduce exact chains, and cleanup receives recorded approval.

## Rollback approach

Before employer activation, stop the publisher, disable the feature flag, restore legacy readers and writers, and run the guarded down migration only when no lifecycle or evidence rows exist. Down migration must refuse safely when any preview, batch, revision, official segment link, attendance period, closure, closure day, or lifecycle-managed legacy row exists.

After official publication or closure evidence exists, do not down-migrate, delete lifecycle records, or revert to an unguarded legacy writer. Fix forward, reconcile data, and preserve revision and closure history. A committed idempotent response remains replayable without mutation even if its preview expires or its revision is later superseded.

## Verification gate

Implementation remains incomplete until repository-wrapper tests cover resolver semantics, migration up/down boundaries, publication atomicity, audit failure rollback, idempotency, concurrency, authorization scope, immutable segments, closure snapshots, payroll dependency, late punches, legacy writer bypass, outbox retry/dead-letter recovery, API contracts, frontend states, and the complete preview → publish → amend → close → reopen → reclose → evidence chain. Existing focused tests prove foundation only; unit-only success is insufficient, and integration, role-scoped E2E and full-suite phases are required.
