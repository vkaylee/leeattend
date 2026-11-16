# Department payroll reconciliation report

Status: verified

## Rules
- `DB-03`: read-only report must preserve legacy data and avoid unsafe inference.
- `MIG-01`: reconciliation stays additive, bounded, repeatable, and rollback-safe.
- `TEST-01`: cover authorization, pagination, statuses, and PostgreSQL view behavior.
- `API-01`: define bounded response and stable error behavior.

## Scope
Expose existing `department_payroll_reconciliation` view through a paginated payroll-read API. Do not backfill missing hours/effective dates or promote ambiguous legacy values.

## Test impact matrix
- Success: matched, missing canonical config, value mismatch, missing legacy value rows.
- Boundaries: page/limit, empty result, invalid status.
- Negative: unauthenticated/unauthorized access; malformed status.
- Persistence: read-only query; no writes.
- Compatibility: additive endpoint and response fields only.
- Operations: bounded page size; no PII or salary values in errors/logs.
