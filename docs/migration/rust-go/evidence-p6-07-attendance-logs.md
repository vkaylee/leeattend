# P6-07 Attendance Logs, Imports, Staging, and Paper OT Evidence

Status: implemented; verification deferred to parent gate run.

## Implemented routes

- `POST /api/v1/workspaces/{ws_id}/attendance/upload`: multipart Excel upload, 10 MiB bound, typed `AttendanceImportJob`, adjustment import target, and `{job_id}` response.
- `POST /api/v1/workspaces/{ws_id}/attendance/logs/upload`: same upload contract with raw-log import target.
- `POST /api/v1/workspaces/{ws_id}/attendance/bulk-upload-paper-ot`: multipart CSV, 10 MiB/10,000-row bounds, typed per-row validation/result.
- `POST /api/v1/workspaces/{ws_id}/attendance/enter-paper-ot/{employee_id}`: typed paper OT JSON, positive integer minutes, ISO date, reason, active employment and daily-summary upsert.
- `GET /api/v1/workspaces/{ws_id}/attendance/import-jobs/latest` and `GET /api/v1/workspaces/{ws_id}/attendance/import-jobs/{id}`: workspace-scoped import job schema with optional target filter.
- `GET /api/v1/workspaces/{ws_id}/attendance/import-jobs/{id}/error-file`: workspace-scoped XLSX download with exact content type and attachment name.
- `GET /api/v1/workspaces/{ws_id}/attendance/logs`: paired entries with employee identity, RFC3339 filters, page/limit defaults 1/20 and max 100.
- `GET /api/v1/workspaces/{ws_id}/attendance/raw-logs`: raw punches with employee/device identity and same filters/pagination.
- `GET /api/v1/workspaces/{ws_id}/attendance/staging-logs`: pending/ambiguous rows, page/limit defaults 1/50 and max 100.
- `POST /api/v1/workspaces/{ws_id}/attendance/staging-logs/{id}/resolve`: typed employee/employment mapping, date-coverage validation, duplicate-safe promotion.
- `DELETE /api/v1/workspaces/{ws_id}/attendance/staging-logs/{id}`: workspace-scoped dismiss, HTTP 204.

Paper-form aliases are also wired at `/attendance/paper-forms/overtime/{employee_id}` and `/attendance/paper-forms/overtime/bulk` for parity with Rust route modules.

## Contract behavior

- Every handler checks workspace path, user context, and Rust permission equivalent.
- Repository queries always filter workspace. Raw log insertion uses `unique_raw_log` conflict key `(workspace_id, employment_id, punch_time, punch, status)`.
- Unknown imported employees stage into `attendance_staging_logs`; employment/import failures persist typed `attendance_log_errors` records.
- Staging resolve uses transaction row lock, validates employee/employment/date ownership, promotes raw punch, and marks processed. Dismiss only changes open statuses.
- Bulk paper OT reports independent line failures. No float arithmetic used for minutes or dates.

## Files

- `apiserver/internal/api/attendance_logs.go`
- `apiserver/internal/api/attendance_logs_test.go`
- `apiserver/internal/repository/attendance_logs.go`
- `apiserver/main.go`
- `docs/migration/rust-go/endpoint-coverage-gap.md`

## Verification status

Parent agent must run Go tests/build/gates after sibling changes land. This worker did not run checks by assignment.
