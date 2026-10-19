# P6-04 Attendance/Time-Tracking Slice Evidence

Status: in progress; Go punch-in API handler, repository, and unit tests implemented and passing all quality gates.

## Implemented

- `internal/api/attendance.go`:
  - `PunchInRequest`: strictly typed JSON request (`employment_id`, `entry_type`, `note`).
  - `TimeEntryRecord`: typed DTO mapping the canonical time entry contract (`punch_in_time`, `punch_out_time`, `duration_minutes`, `is_break_overtime`, `entry_type`, `note`, `generation_metadata`, `created_at`, `updated_at`).
  - `AttendanceHandler.PunchIn`: verifies workspace path UUID, enforces `ess:clock` permission via authorizer, rejects empty or nil UUID payloads with validation errors, and writes HTTP 201 Created on success.
- `internal/repository/attendance.go`:
  - `PunchIn`: transactional data mutation wrapping active employment validation (`SELECT status = 'active' FROM employments`), raw log insertion into `raw_attendance_logs` with lowercase enum `log_source_type` (`'device'`), deduplication handling on `unique_raw_log` constraint violation, and time entry creation in `time_entries`.
- `internal/api/attendance_test.go`:
  - `TestPunchIn_PermissionDenied`: verifies 403 Forbidden when `ess:clock` permission is missing.
  - `TestPunchIn_ValidationFailed`: verifies validation error rejection on nil UUID, malformed JSON, and empty body.
  - `TestPunchIn_Success`: verifies 201 Created response matching the canonical schema.
- `main.go`:
  - Route `POST /api/v1/workspaces/{ws_id}/time/punch-in` registered on the standard `net/http.ServeMux`.

## Verification

- `./scripts/_go-gates.sh`: ALL GATES PASSED (gofmt, go vet, staticcheck, govulncheck, go mod verify, go test -race).
- Uncached unit tests run:
  ```
  === RUN   TestPunchIn_PermissionDenied
  --- PASS: TestPunchIn_PermissionDenied (0.00s)
  === RUN   TestPunchIn_ValidationFailed
  --- PASS: TestPunchIn_ValidationFailed (0.00s)
  === RUN   TestPunchIn_Success
  --- PASS: TestPunchIn_Success (0.00s)
  PASS
  ok github.com/vkaylee/leeattend/apiserver/internal/api 1.018s
  ```

## Implemented in this round

- `internal/api/attendance.go`:
  - `PunchOut`: accepts `PunchOutRequest` on `POST /api/v1/workspaces/{ws_id}/time/punch-out/{employee_id}`, checks `attendance:force_clock_out`, resolves employee ownership through repository, and returns updated `TimeEntryRecord`.
  - `TimeHistory`: accepts `GET /api/v1/workspaces/{ws_id}/time/history/{employee_id}?start=&end=`, enforces `attendance:logs:read` or self-ownership, and returns descending RFC3339-filtered history.
  - `CreateAdjustment`: accepts canonical single-punch request fields `employment_id`, `time_entry_id`, `requested_punch_time`, `reason`, and `attachment_file_ids`.
  - `ListAdjustments`: supports workspace list permission, `status`, `employee_id`, page, and limit filters.
  - `ListEmployeeAdjustments`: enforces admin permission or self ownership using workspace context.
  - `ApproveAdjustment`: validates terminal `approved`/`rejected` status and requires `attendance:adjustments:approve`.
- `internal/repository/attendance.go`:
  - `resolveActiveEmployment` maps employee IDs to workspace-scoped active employment IDs.
  - `PunchOut` inserts raw checkout audit data and closes the active entry transactionally.
  - `ListTimeHistory` uses `template_id`/`segment_id` projections and descending order.
  - `CreateAdjustment` uses current `attendance_adjustment_requests` schema.
  - `ListAdjustments` and `ListEmployeeAdjustments` join through `employments` to preserve workspace isolation.
  - `ApproveAdjustment` locks the request, updates status and approver, and inserts a manual raw attendance log for approved requests.
- `main.go`: registered workspace adjustment list, approval, and employee self-list routes.
- `internal/api/attendance_test.go`: added permission, validation, pagination, listing, and approval handler tests.

## Verification

- `./scripts/_go-gates.sh`: ALL GATES PASSED (`gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, `go test -race`).
- `./leedevkit test api-go`: ALL GATES PASSED.
- PostgreSQL integration remains open: current test target executes Go race/gate suite but no seeded tenant CRUD test exercises attendance SQL against a migrated tenant database.

## Remaining P6-04 scope

- Integrate pairing service behavior after raw punch events; current repository closes the active entry directly while Rust invokes synchronous pairing.
- Implement submit-on-behalf scope enforcement and approval workflow/outbox integration.
- Add seeded PostgreSQL integration and Rust-vs-Go differential fixtures before marking P6-04 verified.
