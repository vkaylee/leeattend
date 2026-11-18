# Work-policy reconciliation evidence slice

## Rule IDs

- `DB-03`: preserve legacy data; no unsafe inferred mapping.
- `MIG-01`: additive, bounded, repeatable migration evidence with rollback.
- `TEST-01`: success, boundary, negative, persistence, and retry evidence.

## Test-impact matrix

| Dimension | Evidence |
|---|---|
| Changed behavior | No runtime behavior changed; this record documents the safe reconciliation boundary. |
| Existing behavior | Matched, missing canonical, missing legacy, mismatch statuses and pagination remain compatible. |
| Boundaries | Missing legacy/canonical values and duplicate effective candidates rejected by existing uniqueness. |
| State | Repeated read returns stable evidence; no write or activation state is introduced. |
| Failures | Duplicate canonical key is rejected by existing uniqueness constraint; transaction rollback remains unchanged. |
| Security | Existing `payroll:read` authorization remains; evidence contains no PII, salary, actor, or credential. |
| Compatibility | View and API additions are additive; legacy field remains untouched. |
| Operations | Pagination and deterministic ordering remain bounded. |

## Decision

Do not backfill legacy department payroll settings. Legacy departments lack hours, effective date, and source actor, so no automatic mapping is safe. Existing reconciliation view remains read-only and repeatable; no persisted activation or inferred provenance is added.
