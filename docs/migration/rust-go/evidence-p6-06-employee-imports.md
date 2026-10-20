# P6-06 Employee Excel Upload and Termination Import Evidence

Status: implemented; verification deferred to parent gate run.

## Implemented routes

- `POST /api/v1/workspaces/{ws_id}/employees/upload`: multipart `file`, 10 MiB bound, XLS/XLSX signature and extension validation, `attendance_import_jobs` pending row, durable employee-import queue envelope, and `{job_id}` response.
- `POST /api/v1/workspaces/{ws_id}/employees/import-termination/preview`: multipart Excel parser, normalized header matching, row validation, active-employment resolution, integer leave proration, and `{valid_rows, invalid_rows, can_commit}` response.
- `POST /api/v1/workspaces/{ws_id}/employees/import-termination/confirm`: typed JSON rows, UUID/date/reason validation, terminated-employment preflight rejection, job metadata, and bulk-termination queue envelope.
- `GET /api/v1/workspaces/{ws_id}/employees/import-termination/template`: generated XLSX binary with `Template`/`Instructions` sheets, localized `en`/`vi`/`zh` names and reason labels, exact download headers.

## Contract behavior

- All routes enforce `employees:upload` or `employees:update` through workspace authorization and reject invalid workspace UUIDs through existing path validation.
- Uploads are capped at 10 MiB while streaming, require multipart field `file`, reject CSV and MIME/extension/signature mismatches, and store under `STORAGE_DIR/imports` with restrictive file permissions.
- XLSX parsing uses stdlib `archive/zip` and XML. It supports shared strings, inline strings, normalized BOM/whitespace/case headers, sheet selection by header match, Excel serial date styles, duplicate/missing required header checks, and empty-row skipping. Binary XLS is recognized and rejected by parser because no external dependency is introduced.
- Termination preview resolves only non-terminated employments, chooses newest start date, reports multiple-employment warning, rejects unknown/duplicate codes and invalid date/reason rows, and uses integer rounding `(entitlement*days_worked + total_days/2)/total_days`; no float proration.
- Confirm queue payload uses typed envelopes matching Rust `Job`/`JobPayload` field names and tagged payload variants, with Redis metadata under `report:meta:{job_id}`. If Redis is unavailable, repository transaction performs termination update and leave-balance proration synchronously to preserve side effects.

## Files

- `apiserver/internal/api/employee_imports.go`
- `apiserver/internal/api/employee_import_excel.go`
- `apiserver/internal/api/employee_imports_test.go`
- `apiserver/internal/repository/employee_imports.go`
- `apiserver/internal/repository/employee_imports_test.go`
- `apiserver/main.go`
- `docs/migration/rust-go/endpoint-coverage-gap.md`

## Verification status

Parent agent must run Go tests/build/gates after sibling changes land. This worker did not run those checks by assignment.
