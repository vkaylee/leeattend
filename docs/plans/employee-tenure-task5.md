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
- Current API gate completed successfully: `./leedevkit test api` exit 0; Go unit, integration phase, format, and vet phases selected by LeeDevKit passed. The integration phase emitted no test details in captured output, so this does not replace focused PostgreSQL evidence.
- Signed-file characterization tests pass through the API unit phase.
- Added PostgreSQL integration test `TestIntegrationSignedDocumentCannotBeDeletedOrVersioned`; execution remains blocked before test start by shared environment (`leeattend-test_apiserver_1` and `test_database`). Ownership inspection proved active stack mounts `/home/elt1541/lee/leeattend`, not this worktree; no stop/remove/reset performed.
- Current web evidence: `Unit_Tests_webdashboard.log` reports 326 test files passed and 6 failed, 3014 tests passed and 39 failed; `Integration_&_E2E_playwright-e2e.log` reports 55 passed and 5 failed. Typecheck/i18n and lint phases emitted no failure status, but full web target is failed. Report-export E2E produced `TypeError: Failed to fetch` and failed URL/progress assertions.
- `./scripts/_go-gates.sh --integration --package ./internal/repository` with a unique project/port was blocked by broken Podman network metadata before test execution; no shared environment cleanup performed.
- Current source audit confirms request/enqueue only: `AttendanceReport` has no production consumer, report result writer, status transition, or worker startup. Export payload stores request metadata but no immutable tenure/number snapshot.
- Disposable reconciliation fixture metrics captured before cleanup: employees=7 (2 masked seed rows + 5 documented-role rows), tenure rows=5, pending reconciliation=0, blocked reconciliation=0, affected accounts=5. These are disposable-fixture metrics, not production counts.
- Redis worker integration test now passes on disposable stack: `TestIntegrationReportWorkerConsumesRedisJobAndStoresResult`; `./scripts/_go-gates.sh --integration --package ./internal/repository --test-pattern ReportWorker` returned `ALL GATES PASSED` after bounded BLPOP shutdown fix.
- Last safe rollback point remains before first tenure write, per `20261115030000_employee_tenures/down.sql`; after tenure writes, retain history and roll forward.
- Disposable UI lifecycle execution was blocked by browser network policy: Playwright sessions could not navigate to `127.0.0.1:18082`, `localhost:18082`, or `host.docker.internal:18082` (`ERR_BLOCKED_BY_CLIENT`). No lifecycle mutation was executed.
- Task 5 implementation is present: production Redis report worker, immutable export tenure snapshot, date-effective readers, and pagination beyond 3000 rows are covered by focused tests and disposable Redis/PostgreSQL evidence.
- Latest `./leedevkit test all --json` passed `3120/3120` with no blocked or skipped phases. Signed-document immutability integration now passes through corrected disposable wrapper: `./scripts/_go-gates.sh --integration --package ./internal/repository --test-pattern SignedDocumentCannotBeDeletedOrVersioned` returned `ALL GATES PASSED`; `phased schema contract: PASS`.
- Remaining Task 5 acceptance limits: browser export E2E uses API mocks, XLSX is minimal one-sheet output, and production reconciliation aggregates are unavailable; disposable fixture counts remain employees=7, tenure_rows=5, pending=0, blocked=0, affected_accounts=5.
