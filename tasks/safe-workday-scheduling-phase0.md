# Safe Workday Scheduling — Phase 0 Checklist

## Rule IDs

- `development-workflow.md §2–7`: test impact matrix, regression protection, and completion evidence.
- `testing-standards.md §1–4`: behavior tests, negative paths, and full LeeDevKit gates.
- `coding-standards.md §40, §46`: validate trusted boundaries and keep calculation logic repository-independent.
- `api-governance.md §1–2`: preserve additive API compatibility while calculation contracts evolve.
- `database-rules.md §3–4`: preserve transactional lifecycle and reversible migration requirements.

## Scope

Phase 0 and Phase 1 only. No UI, roster mutation, raw-event schema, or Shift change-impact endpoint in this slice.

## Current inventory

- `apiserver/internal/domain/payroll/buckets/buckets.go`: current break deduction and daily bucket logic.
- `apiserver/internal/domain/payroll/buckets/buckets_test.go`: current unit coverage for deduction and overtime overlap.
- `apiserver/internal/repository/attendance.go`: raw attendance logs, `time_entries`, required-break state transitions, and idempotency.
- `apiserver/internal/repository/schedule_lifecycle.go`: materialized `breaks_snapshot` and selected Shift Version on published work segments.
- `apiserver/internal/repository/payroll.go`: Work Policy loading still exposes transitional `break_auto_deduct` and `break_is_overtime` fields.
- `apiserver/internal/repository/reports.go`: reads derived attendance minute buckets.
- `apiserver/migrations/20261119020000_shift_break_policy/up.sql`: interval-level legacy break flags.
- `apiserver/migrations/20261119030000_break_capture_policy/up.sql`: punch mode, credit mode, break records, idempotency, and summary buckets.

## Locked calculation invariants

- `break_punch_mode = none + scheduled` uses scheduled deduction only.
- `break_punch_mode = none + none` has no break deduction.
- `break_punch_mode = required` uses actual completed break pairs only; scheduled deduction is zero.
- Missing required break pairs produce an exception; they do not silently receive scheduled deduction.
- Planned/approved break-worked minutes remove scheduled deduction for covered minutes.
- `payable_minutes` never subtracts actual and scheduled break minutes for the same break.
- `duration_minutes` remains derived compatibility output, never source-of-truth policy.
- Overnight intervals use absolute local ordering; split intervals calculate independently.

## Test impact matrix

| Dimension | Phase 1 coverage |
|---|---|
| Changed behavior | Pure calculation returns separate minute buckets and exception code. |
| Existing behavior | Existing unit rounding and early/late OT behavior remains unchanged. |
| Inputs/boundaries | Zero/negative duration, missing punches, missing break pair, overnight and split intervals. |
| State | Scheduled 2-punch, required 4-punch, required missing break, planned break-worked. |
| Failures | Invalid ordering and unsupported break mode return explicit calculation errors/exceptions. |
| Security | No repository or DB dependency enters pure calculator; no PII in outputs. |
| Compatibility | Existing `WorkBuckets` and `duration_minutes` consumers remain supported until migration. |
| Operations | Deterministic pure function; no I/O, retries, or logging. |

## Checkpoints

- [x] Duplicate-deduction regression test added; current unsafe legacy path remains isolated from new calculator.
- [x] Pure calculator implements locked buckets and exception behavior.
- [x] Focused calculator unit tests pass through `scripts/_go-gates.sh --unit-only --package ./internal/domain/payroll/buckets`.
- [ ] Full API and all required target gates run; every relevant `.test_logs/` phase is read.
- [x] `git diff --check` passes.

## Verification limitation

`leedevkit test api --unit-only` was attempted but could not start because the repository wrapper is `0.11.2` while `leedevkit.toml` requires `0.11.1`; the compatible runtime exists only in `.leedevkit.bak`. Focused Go verification used the tracked `scripts/_go-gates.sh` wrapper in the project container. Full LeeDevKit gates remain pending until the runtime mismatch is repaired by the project owner.
