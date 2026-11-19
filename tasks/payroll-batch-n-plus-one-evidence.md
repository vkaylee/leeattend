# Payroll batch calculation N+1 evidence

## Rule IDs

- `DB-02`: no query inside an unbounded batch loop.
- `TEST-01`: characterize existing partial-success and ordering behavior before optimization.
- `ARCH-01`: preserve existing calculation path and transaction semantics.

## Findings

`PayrollRepository.CalculateBatchPayroll` currently loops employment IDs and calls `calculateEmploymentPayslip` per item. Each calculation loads salary history, department history, policy assignments, overtime assignments, attendance, and writes payslip snapshots inside the shared transaction. This is a real N+1 path.

No bulk rewrite in this slice. Bulk prefetch would need a calculation-context boundary for salary history, department changes, policy segments, overtime, attendance, and writes; changing only the first employment lookup would leave deeper N+1 queries and could change rollback/error semantics.

## Evidence

- Batch API rejects duplicate IDs and caps input at 500.
- Partial missing employment returns per-item error while later items continue.
- Response preserves request order.
- Existing calculation integration and locked-period tests remain required before any bulk refactor.

## Next safe optimization gate

Measure query count and latency for representative batch sizes, then introduce one bulk-fetched immutable calculation input context with parity tests for legacy/no-policy, policy boundary, partial failure, locked period, and rollback. Do not optimize by adding a second calculation engine.
