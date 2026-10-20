# P6-07 Report Export Job Cancellation and Download Evidence

Status: verified through container gates (`./scripts/_go-gates.sh`, `./leedevkit test api-go`).

## Scope

Rust reference: `apiserver/src/handlers/report_export.rs` (`cancel_job`, `download_report`), `apiserver/src/services/report_coordinator.rs` (`cancel_job`, `get_job_status`), `apiserver/src/queue/job_queue.rs` (`get_result_data`).

Implemented routes:

| Method | Path | Behavior |
|---|---|---|
| `DELETE` | `/api/v1/workspaces/{ws_id}/reports/jobs/{job_id}` | 204 on cancel; 409 `error.cannot_cancel_job` when job is `completed`/`failed`; 404 `error.job_not_found`; 403 on workspace mismatch |
| `GET` | `/api/v1/workspaces/{ws_id}/reports/jobs/{job_id}/download` | 200 with `Content-Type` + `Content-Disposition` and file bytes when `completed`; 202 plain-text polling hint while `pending`/`processing`; 500 when `failed`; 404 `error.no_report_file_generated` when the result payload is missing |

## Behavior parity notes

- Workspace ownership is enforced before mutation: a job belonging to another workspace returns 403, never 404 (matches Rust `metadata.workspace_id != ctx.workspace_id`).
- Cancel transitions `pending`/`processing` to `failed` with `error_message = "Cancelled by user"` and `completed_at = now`, matching `ReportCoordinator::cancel_job`.
- Cancel is idempotency-safe for terminal jobs: the second call returns 409, matching `ReportError::CannotCancel` → `AppError::conflict(I18nKey::CannotCancelJob)`.
- Redis metadata now uses the canonical key family `report:meta:{job_id}` (24h TTL, matching `JOB_TTL_SECS`), with the legacy `job:metadata:{job_id}` key still written/read during cutover.
- Result payload retrieval reads `report:results:{job_id}:data` and `report:results:{job_id}:filename` in one pipeline, matching `JobQueue::get_result_data`.
- Download content type is `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` unless the stored filename ends in `.csv`, in which case `text/csv` is returned.

## Tests

`apiserver/internal/api/reports_test.go`:

- `TestReportCancel` — pending job becomes `failed` with `"Cancelled by user"` and 204.
- `TestReportCancelRejectsCompletedJob` — 409 for terminal job.
- `TestReportCancelOtherWorkspaceForbidden` — 403 cross-workspace.
- `TestReportDownloadCompleted` — exact content type, disposition, and bytes.
- `TestReportDownloadPendingReturnsAccepted` — 202 polling hint.

## Verification

```
./scripts/_go-gates.sh        -> ALL GATES PASSED (gofmt, vet, staticcheck, govulncheck, mod verify, go test -race)
./leedevkit test api-go       -> ALL GATES PASSED
```

Measured contract coverage after this slice: 92/403 contract routes covered, 311 missing.
