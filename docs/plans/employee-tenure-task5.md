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
- `./leedevkit test api`: PASS on current branch (Go unit, integration phase, format, vet).
- Focused tenure integration: PASS for historical report old/new number and ambiguous import rejection.
- Signed-file characterization tests: PASS through API unit gate.
- Added PostgreSQL integration test `TestIntegrationSignedDocumentCannotBeDeletedOrVersioned`; execution blocked before test start by shared environment (`leeattend-test_apiserver_1` already exists and `test_database` already exists). No container/database cleanup performed because concurrent work may own them.
- `./leedevkit test web`: FAIL. Current log: 326 unit files passed, 6 failed; E2E 55 passed, 5 failed; typecheck/i18n passed; lint 0 errors with warnings. Report-export E2E failures keep export compatibility unverified.
- `./scripts/_go-gates.sh --integration --package ./internal/repository`: prior run failed on unrelated payroll/schedule/work-policy tests; focused tenure run passed.
- Task 5 remains incomplete: active Go report worker is absent, export payload has no immutable tenure snapshot, export E2E fails, and reconciliation aggregate metrics need a target dataset.
