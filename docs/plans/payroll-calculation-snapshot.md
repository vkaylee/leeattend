# Payroll calculation snapshot completion

## Scope
Ensure every payroll calculation records effective labor-policy provenance, even when no KPI formula exists. Preserve existing formula-run snapshots and payroll outputs.

## Test impact matrix
- Changed behavior: non-KPI payroll calculations create/update one linked calculation snapshot.
- Existing behavior: KPI formula snapshots, payslip amounts, locked-period protection, and finalized-payslip protection remain unchanged.
- Boundaries: missing policy, nil formula version, zero standard/OT units, repeated draft recalculation.
- Failures: snapshot serialization or persistence failure rolls back calculation transaction.
- Security: snapshot excludes salary, base pay, net pay, PII, and raw restricted inputs.
- Compatibility: nullable `formula_version_id` remains valid; existing formula snapshots stay queryable.
- Operations: no new external calls or logs; snapshot uses current transaction.

## Rule IDs
- `ARCH-01`: reuse existing payroll calculation-run storage.
- `DB-03`: update/insert snapshot inside payroll transaction.
- `TEST-01`: add unit coverage and run full API gate.

## Acceptance
- Every successful payroll calculation has one `engine_version = payroll-v1` snapshot linked to payslip.
- Recalculation updates same non-formula snapshot instead of creating duplicates.
- Policy metadata is point-in-time and safe.
- No salary or PII values enter snapshot payload.
