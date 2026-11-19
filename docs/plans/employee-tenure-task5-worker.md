# Employee Tenure Task 5 Worker Slice

## Rule IDs
- `database-rules.md §3–4`
- `migration-and-rollback.md §1–4`
- `api-governance.md §1–2, §4–6`
- `access-control.md §1, §4`
- `testing-standards.md §1–2`
- `development-workflow.md §2–7`

## Scope
Implement only contract-compatible AttendanceReport processing on existing Redis queue and report metadata/result keys. Preserve existing API response fields and report totals/grouping. Add immutable effective tenure/employee-number snapshot to queued export payload. No second queue architecture or unrelated cleanup.

## Test impact matrix
- Changed behavior: queued report reaches terminal status and downloadable result; export captures effective tenure number at request time.
- Existing behavior: enqueue/status/download contracts, report totals/grouping, historical date-effective report numbers remain unchanged.
- Boundaries: XLSX-only output, empty report, multiple pages/batches, missing tenure mapping, current number changed after enqueue, malformed queue payload.
- State: pending, processing, completed, failed, duplicate/retry, cancellation.
- Failures: repository query, Redis read/write/publish, invalid payload, output generation, context cancellation; no partial terminal success.
- Security: authenticated requester metadata retained; no raw PII logs; worker uses service-side data and does not bypass report authorization model.
- Compatibility: additive payload fields only; legacy queue payload remains readable where safe.
- Operations: bounded batch/page size, retry/lease semantics using existing queue contract, masked metrics.

## Completion evidence
- Focused worker tests pass, including date-effective number selection and pagination beyond the 3000-row page limit.
- Disposable Redis integration proves queued job consumption, historical number application, terminal metadata, and stored XLSX result.
- Disposable PostgreSQL integration proves historical number snapshot and signed artifact immutability.
- Export request survives current-number change with original effective number.
- XLSX-only download is the export contract. Workbook is one-sheet `Report` with archive/XML content assertions; no two-sheet business contract exists in repository requirements.
- Full API/web/all gates pass on current HEAD: `./leedevkit test all --json` returned `3120/3120`, with API unit/integration/vet/gofmt and web unit/Playwright E2E/i18n/lint/typecheck all passing; log project `leedevkit-test-aabe3bf3`.
- Signed-document disposable integration passes when run directly inside owned disposable API container after the repository wrapper's phased fixture mount failed: `go test -tags=integration -count=1 ./internal/repository -run TestIntegrationSignedDocumentCannotBeDeletedOrVersioned` returned `ok`.
- Disposable signed-document integration now passes through the corrected wrapper: `PROJECT_ROOT=/home/elt1541/lee/leeattend COMPOSE_PROJECT_NAME=leeattend-tenure-signed-1795002056 PGBOUNCER_TEST_HOST_PORT=16847 ./scripts/_go-gates.sh --integration --package ./internal/repository --test-pattern SignedDocumentCannotBeDeletedOrVersioned` returned `ALL GATES PASSED`; `phased schema contract: PASS`. Wrapper fix committed in `8ee928240`.
- Live disposable API→Redis→worker→XLSX download proof completed with admin: export `202`, worker status `pending→completed`, download `200`, valid XLSX ZIP/XML archive, `Report` sheet. Browser E2E remains mock-backed; browser MCP lifecycle access is tracked separately in Task 6.
