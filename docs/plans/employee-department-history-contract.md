# Employee Department History Contract Fix

Status: Approved

## Goal
Remove stale `employee_department_history` runtime dependencies and use current `employments` department segments, preventing PostgreSQL relation errors without losing transfer history behavior.

## Rules
- DB-01: Use current single-organization schema and parameterized PostgreSQL queries.
- DB-02: Preserve historical department data through existing `employments` segments; no destructive migration.
- TEST-01: Add regression coverage for schedule and payroll department history consumers.
- COMPAT-01: Preserve API behavior and mixed-version safety; avoid introducing a replacement table.

## Test impact matrix
- Changed behavior: schedule rule matching and payroll segment boundaries read department history from `employments`.
- Existing behavior: employee department history API and transfer writes remain unchanged.
- Boundaries: no department segments, deleted segments, future segments, multiple transfers, end-date cutoff.
- Failures: query errors propagate from schedule loader; payroll remains tolerant only for optional ancillary data, not stale relation absence.
- Compatibility: fresh baseline and existing migrated databases already contain `employments`; no data migration required.
- Operations: verify no runtime SQL references `employee_department_history`; run full API and all suites.

## Tasks
- [ ] Confirm canonical employment columns and test doubles.
- [ ] Replace stale schedule/payroll queries with `employments` queries.
- [ ] Add regression tests for query shape and department history semantics.
- [ ] Run migration/schema and full test verification.

## Done when
- No active Go runtime query references `employee_department_history`.
- Schedule and payroll consumers use deleted-filtered employment segments.
- Regression tests fail against stale relation usage and pass with canonical queries.
- `leedevkit test api` and `leedevkit test all` pass; `git diff --check` passes.
