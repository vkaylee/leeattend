# P6-03 Departments Domain Evidence

Status: in progress; typed HTTP handlers, repository queries, and unit tests implemented. Full endpoint parity and live integration proof remain open.

## Implemented Endpoints
1. `GET /api/v1/workspaces/{ws_id}/departments`
   - Requires authenticated user context in workspace.
   - Parses pagination `page` (default 1) and `limit` (default 50, clamped max 3000).
   - Computes subquery for `active_employee_count`.
   - Returns 200 OK with `DepartmentListResponse`.
2. `POST /api/v1/workspaces/{ws_id}/departments`
   - Requires `departments:create` permission.
   - Validates non-empty `name`.
   - Returns 201 Created with `DepartmentRecord`.
   - Maps unique constraint violation on `(workspace_id, lower(name))` to 409 Conflict.
3. `PATCH /api/v1/workspaces/{ws_id}/departments/{id}`
   - Requires `departments:update` permission.
   - Dynamically updates non-null fields (`name`, `description`, `manager_id`).
   - Returns 200 OK with updated `DepartmentRecord`.
   - Returns 404 Not Found if department does not exist in workspace or is soft-deleted.
4. `DELETE /api/v1/workspaces/{ws_id}/departments/{id}`
   - Requires `departments:delete` permission.
   - Performs soft-delete (`UPDATE departments SET deleted_at = NOW()`).
   - Returns 204 No Content.
5. `GET /api/v1/workspaces/{ws_id}/departments/{id}/payroll-config`
   - Requires authenticated user context in workspace.
   - Reads `payroll.fixed_std_days` and `payroll.hours_per_day` from `workspace_config_entries`.
   - Returns 200 OK with `[]DepartmentPayrollConfig` (empty array if none exist).
6. `POST /api/v1/workspaces/{ws_id}/departments/{id}/payroll-config`
   - Requires authenticated user context in workspace.
   - Checks active department exists in workspace; returns 404 with key `error.department_not_found` if absent.
   - Upserts entries into `workspace_config_entries` for both keys.
   - Returns 201 Created with `DepartmentPayrollConfig`.

## Source Files
- `internal/api/departments.go`: Typed handlers, DTOs, and error mapping.
- `internal/api/departments_test.go`: Unit tests for CRUD, pagination, permission rejection, and payroll config endpoints.
- `internal/repository/departments.go`: SQL queries with tenant isolation (`workspace_id`), active employee subquery, and config upserts.
- `internal/repository/departments_test.go`: SQL construction tests for CRUD and config upserts.
- `main.go`: Wired `DepartmentRepository` and registered all 6 department routes.

## Verification Status
- `./scripts/_go-gates.sh`: ALL GATES PASSED (`gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, `go test -race`).
- `./leedevkit test api-go`: ALL GATES PASSED.
- Endpoint metrics:
  - Contract routes: **403**
  - Go route registrations: **122** (+6)
  - Contract routes covered: **106** (+6)
  - Contract routes missing: **297** (-6)
