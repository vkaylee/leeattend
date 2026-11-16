# ADR: Work-policy activation boundary

| Property | Value |
|---|---|
| Status | Blocked pending canonical cross-consumer gate |
| Date | 2026-11-16 |
| Owner | LeeAttend backend, payroll, attendance, and reporting domain owners |
| Scope | Work-policy configuration activation and consumer cutover |

## Context

Work-policy groups, immutable versions, effective-dated memberships, validation, point-in-time resolution, payroll provenance, simulation, and read-only legacy reconciliation now exist. No persisted activation state is introduced.

Current source evidence:

- API resolution uses `WorkPolicyRepository.ResolveWorkPolicy` and exclusive effective ranges in `apiserver/internal/repository/work_policy.go`.
- Payroll calculation resolves policy assignments and stores per-segment `labor_policy_group_version_id` in `apiserver/internal/repository/payroll.go`.
- Reports resolve work-credit policy per attendance date in `apiserver/internal/repository/reports.go`.
- Draft simulation reuses payroll calculation through `simulateEmployeePayroll` in `apiserver/internal/repository/payroll.go`.
- Impact staleness reuses the point-in-time resolver, but batch impact compares persisted runs rather than serving as a runtime activation gate.
- `GetAttendanceSummaries` still returns raw daily summary fields in `apiserver/internal/repository/attendance.go`; it is not yet a policy-aware consumer.
- Batch payroll invokes per-employment calculation in a loop, so it is not evidence of a bounded shared activation read path.
- Calculation-run input provenance stores one period-level policy object while payslip segments can contain multiple policy versions during a mid-period change.

## Decision

Do not implement activation state, an `activated` flag, an activation table, or a configuration flag for work policies yet.

Activation remains a documented architecture boundary, not a runtime feature. A future activation contract must provide one canonical source of truth and one shared gate/resolver consumed by API, payroll, reports, attendance summaries, batch workers, draft simulation, and snapshots. Read-only validation and impact comparison are not activation.

## Required gate before activation

A future implementation must prove all of these in one contract:

1. Canonical source-of-truth ownership and consumer inventory.
2. Shared effective-date resolver/gate for every consumer, including workers and reports.
3. Transaction revalidation immediately before cutover.
4. Expected-version or stale fingerprint rejection.
5. Concurrency conflict returned as `409`.
6. Idempotent replay behavior.
7. Audit event without PII, salary, or credentials.
8. Rollback/roll-forward behavior that preserves payroll history and immutable snapshots.
9. Integration coverage for success, stale state, concurrent edits, retry, authorization, and rollback.
10. Reconciliation evidence showing legacy and policy-aware results match or every difference has an explicit disposition.

## Consequences

- Existing policy-aware consumers continue operating without claiming activation.
- Legacy consumers remain identifiable gaps instead of silently reading a competing gate.
- Payroll history remains unchanged; no destructive migration or dual-write activation path is added.
- Cutover cannot be declared complete until attendance summaries, batch execution, worker paths, snapshot provenance, and controlled rehearsal have evidence.

## Rollback and removal

No runtime activation switch exists, so no activation rollback is claimed. Future cutover must first support a pre-evidence abort path. After official policy-backed payroll or closure evidence exists, rollback must be roll-forward/reconciliation only; it must not delete immutable history or silently restore an unguarded competing writer.

## Verification

- Source and graph audit performed for payroll, reports, attendance summaries, batch, draft simulation, snapshots, and resolver paths.
- Coverage caveat: excluded or partially indexed files were checked directly from source.
- No activation state or read-only validation endpoint is treated as activation evidence.
