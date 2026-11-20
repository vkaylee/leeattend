# Attendance Break Override Calculation Slice

## Scope

Consume approved `break_worked` intervals in pure attendance calculation without mutating Shift Versions, published segments, raw punches, or closed evidence.

## Rule IDs

- `coding-standards.md`: pure domain logic stays independent from repositories.
- `database-rules.md`: approved overrides are batch-loaded; no query inside calculation loops.
- `testing-standards.md`: changed business behavior requires success, boundary, negative, and integration coverage.
- `development-workflow.md`: preserve immutable evidence and verify partial/invalid states.

## Test impact matrix

- Changed behavior: approved partial break-worked coverage increases paid-break minutes and reduces scheduled unpaid deduction.
- Preserved behavior: existing planned-break calculation and no-double-deduction invariant remain unchanged.
- Boundaries: partial coverage, full coverage, overnight timestamps, empty override list.
- Negative: pending/rejected overrides ignored; overlapping or out-of-window approved intervals rejected.
- Persistence: schedule readers expose approved rows only; closed snapshots retain evidence.
- Security/data integrity: calculator trusts status and materialized timestamps, never client OT minutes.

## Verification

- Focused bucket unit tests.
- Repository schedule unit/integration tests.
- Go lint, unit, integration, and `git diff --check` through repository wrappers.
