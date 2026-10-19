# P6-05 Shifts, Roster, Working Patterns, and Compliance Evidence

Status: in progress; Shift Template Core Slice and initial shift assignment/compliance slice implemented, registered, and passing all quality gates plus `./leedevkit test api-go`.

## Implemented in this slice

- `internal/api/shifts.go`:
  - `CreateShiftRequest`, `UpdateShiftRequest`, `AssignShiftRequest`, `ShiftIntervalRequest`, `ShiftRecord`, `ShiftIntervalRecord`, `ShiftVersionRecord`, `PaginatedShifts`.
  - `ShiftHandler` methods:
    - `POST /api/v1/workspaces/{ws_id}/shifts` (`shifts:create`)
    - `GET /api/v1/workspaces/{ws_id}/shifts` (`shifts:read`)
    - `PUT /api/v1/workspaces/{ws_id}/shifts/{shift_id}` (`shifts:update`)
    - `GET /api/v1/workspaces/{ws_id}/shifts/{shift_id}/versions` (`shifts:read`)
    - `POST /api/v1/workspaces/{ws_id}/shifts/assign` (`shifts:assign`)
  - Strict validation of time format (`15:04:05`), date format (`2006-01-02`), non-empty name, non-empty intervals, non-negative break and grace durations, and bounded assignment date ranges.
- `internal/domain/compliance/compliance.go`:
  - Pure typed validation for disabled rules, hard interval overlaps including overnight intervals, soft rest-period violations, weekly hours, and monthly overtime.
  - Integer-minute arithmetic only; no DB, network, global state, wall-clock reads, or floating-point calculations.
- `internal/repository/shifts.go`:
  - `CreateShift`: Atomic SCD Type-2 creation inserting template, version 1, intervals, audit log, and setting `current_version_id`.
  - `UpdateShift`: Atomic revision/correction creating version `N+1`, closing prior version's `effective_to` on revision, linking `superseded_by`, updating `shift_templates.current_version_id`, copying immutable interval snapshots, and emitting audit logs.
  - `ListShifts`: Tenant-scoped query filtering active versions with pagination and interval hydration.
  - `ListShiftVersions`: Batch-loads all versions for a template with a single `ANY($1)` intervals query, completely eliminating N+1 query patterns.
  - `AssignShift`: Tenant-scoped transactional employment/template checks, compliance precheck, duplicate mapping, and `employee_shifts` insertion.
- `apiserver/main.go`:
  - Registered all 5 implemented shift template, version, and assignment routes in `net/http.ServeMux`.
- `internal/api/shifts_test.go` and `internal/domain/compliance/compliance_test.go`:
  - Unit tests for creation, interval validation, pagination, permission checks, revision updates, version history, assignment date validation, compliance conflicts, overnight overlap, weekly hours, and monthly overtime.

- `internal/api/working_patterns.go`:
  - `WorkingPattern`, `WorkingPatternDetail`, `EmployeeWorkingPattern`, `CreateWorkingPatternRequest`, `AssignPatternRequest`, `BulkAssignPatternRequest`, `BulkAssignPatternResponse`.
  - Routes: create, list, detail lookup, update, soft delete, employment assignment, bulk assignment, and employment assignment history.
  - Validation: non-empty name, non-empty details, `weekly`/`rotation` pattern types, weekly without cycle length, rotation with positive cycle length, schedule day bounds, and assignment date ordering.
- `internal/repository/working_patterns.go`:
  - Transactional create/update with detail replacement, soft delete, assignment, bulk assignment with conflict pre-check, and employment history.
- `internal/api/working_patterns_test.go`:
  - Handler contract tests for permission denial, creation, listing, assignment, and bulk assignment.

## Quality Gates Verification

- `./scripts/_go-gates.sh`: ALL GATES PASSED (`gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, `go test -race`).
- `./leedevkit test api-go`: PASSED.
- Version history uses one batched `ANY($1)` interval query; no version-history N+1 query pattern.

- Pattern compliance pre-check for date-expanded pattern assignments.
- Daily roster scheduling (`shift_rosters`), bulk upsert, and attendance recalculation triggers.
- Shift swap workflows (`shift_swap_requests`).

## Work-segment management slice

- `internal/api/work_segments.go`:
  - Typed `WorkSegment`, create/update/bulk request DTOs, filters, and list response.
  - Routes: list, detail, create, update, delete, and bulk create under `/api/v1/workspaces/{ws_id}/work-segments`.
  - Permission checks use `rosters:read` and `rosters:manage`; create defaults `source` to `manual`; batch size is bounded at 1000.
  - Decimal rates use `shopspring/decimal`; JSON override fields use `json.RawMessage`.
- `internal/repository/work_segments.go`:
  - Tenant-scoped list/detail queries with date, employment, type, status, and source filters.
  - Transactional create, update, and bulk-create operations with reference validation and conflict mapping.
  - Explicit SQL uses `work_segments` and preserves decimal values without floating-point conversion.
- `internal/api/work_segments_test.go`:
  - Permission denial, source default, invalid clock, and empty bulk validation coverage.

## Daily roster scheduling slice

- `internal/api/rosters.go`:
  - `ShiftRoster`, `CreateRosterRequest`, `RosterItem`, `BulkRosterRequest`, `PaginatedShiftRosters`.
  - Routes:
    - `GET /api/v1/workspaces/{ws_id}/employees/{id}/rosters` (`rosters:read`)
    - `POST /api/v1/workspaces/{ws_id}/employees/{id}/rosters` (`rosters:manage`)
    - `POST /api/v1/workspaces/{ws_id}/employees/{id}/rosters/bulk` (`rosters:manage`)
    - `DELETE /api/v1/workspaces/{ws_id}/employees/{id}/rosters/{date}` (`rosters:manage`)
  - Strict validation of date format (`2006-01-02`), maximum bulk size (366 items), duplicate date prevention, and active employment resolution.
- `internal/repository/rosters.go`:
  - Tenant-scoped queries with pagination.
  - Upsert with atomic `ON CONFLICT (employment_id, date) DO UPDATE`.
  - Bulk upsert with template/employment reference validation within a single tenant transaction.
  - Atomic delete returning 404 when no row was deleted.
- `internal/api/rosters_test.go`:
  - Unit tests covering bulk duplicate rejection, 202 accepted response for bulk, and 200 confirmed response for single upsert.

## Remaining P6-05 scope

- Pattern compliance pre-check for date-expanded pattern assignments.
- Shift swap workflows (`shift_swap_requests`).
