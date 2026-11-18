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
- Boundaries: CSV/XLSX, empty report, multiple pages/batches, missing tenure mapping, current number changed after enqueue, malformed queue payload.
- State: pending, processing, completed, failed, duplicate/retry, cancellation.
- Failures: repository query, Redis read/write/publish, invalid payload, output generation, context cancellation; no partial terminal success.
- Security: authenticated requester metadata retained; no raw PII logs; worker uses service-side data and does not bypass report authorization model.
- Compatibility: additive payload fields only; legacy queue payload remains readable where safe.
- Operations: bounded batch/page size, retry/lease semantics using existing queue contract, masked metrics.

## Completion evidence
- Focused worker tests pass, including date-effective number selection and pagination beyond the 3000-row page limit.
- Disposable Redis integration proves queued job consumption, historical number application, terminal metadata, and stored CSV result.
- Disposable PostgreSQL integration proves historical number snapshot and signed artifact immutability.
- Export request survives current-number change with original effective number.
- CSV download is proven; XLSX is only a minimal one-sheet workbook and the business two-sheet contract remains unproven.
- Full API/web/all gates pass on current HEAD: `./leedevkit test all --json` returned `3120/3120`, with API unit/integration/vet/gofmt and web unit/Playwright E2E/i18n/lint/typecheck all passing; log project `leedevkit-test-aabe3bf3`.
- Signed-document disposable integration passes when run directly inside owned disposable API container after the repository wrapper's phased fixture mount failed: `go test -tags=integration -count=1 ./internal/repository -run TestIntegrationSignedDocumentCannotBeDeletedOrVersioned` returned `ok`.
- Wrapper retry blocker: `_go-gates.sh --integration` could not apply `phased_schema_contract.sql` because DB container has no `/workspace` mount; wrapper then called undefined `fail`. No shared stack was changed.
- Browser export E2E remains mock-backed; it proves UI state/download wiring, not live API-to-Redis-worker execution. Live worker proof remains disposable Redis integration only.
