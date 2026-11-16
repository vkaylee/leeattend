# Employee Tenure Task 5 — Historical Consumers and Compatibility

Status: in progress

## Rule IDs
- `database-rules.md §3–4`
- `migration-and-rollback.md §1–4`
- `api-governance.md §1–2`
- `testing-standards.md §1–2`

## Test impact matrix
- Changed behavior: historical readers resolve employee number from relationship tenure at effective date.
- Existing behavior: report totals/grouping, export payload compatibility, finalized/signed document snapshots remain unchanged.
- Boundaries: old/new tenure dates, leading-zero numbers, missing mappings, duplicate effective matches.
- State: finalized documents and completed export jobs are immutable; new reads may use current data only for non-snapshot templates.
- Failures: ambiguous number resolution rejects rather than selecting latest employer; missing mapping fails closed for affected historical operation.
- Security: no cross-tenure fallback through current employee projection.
- Compatibility: legacy request/response fields remain; projection stays read-only compatibility fallback only where no historical effective date exists.
- Operations: reconciliation metrics are masked and record last safe rollback point.

## Reconciliation metrics
- Total employees: not measured in this workspace; disposable integration DB is recreated per gate and no production dataset is accessible.
- Tenure rows: not measured in this workspace; migration creates rows only for explicit lifecycle writes.
- Unresolved mappings: migration seeds one `pending` reconciliation row per legacy employee; current dataset count unavailable.
- Ambiguous mappings: focused resolver test confirms rejection; aggregate count unavailable without target dataset.
- Affected accounts: not measured; no account identifiers recorded.
- Last safe rollback point: before first new tenure write, per `20261115030000_employee_tenures/down.sql`; after tenure writes, roll forward and retain history.

## Evidence log
- Source audit found no report worker implementation beyond enqueue metadata; export historical snapshot remains blocked.
- Runtime readers now resolve tenure number and return empty when tenure mapping is absent; legacy employee projection is not used by readers.
- Signed-file mutation paths now reject delete/version changes and signer endpoint requires authenticated identity; repository/API tests cover signing without file snapshot rewrite.
- `./leedevkit test api`: PASS (unit, integration phase had no reported test output, lint/vet passed).
- `./scripts/_go-gates.sh --integration --package ./internal/repository`: FAIL. Existing concurrent failures: payroll impact batch, payroll simulation rollback, schedule coverage, work-policy conflict. No tenure-specific failure shown.
- Task 5 remains incomplete until export snapshot/worker evidence and required integration tests run successfully.
