# P6-03 Employee Detail Evidence

Status: partial slice implemented; full employee/employment/assignment parity remains open.

## Implemented

- `GET /api/v1/workspaces/{ws_id}/employees/{id}`
  - Requires `employees:read`.
  - Reads only active employee rows scoped by `workspace_id`.
  - Returns `404` when the employee is missing or soft-deleted.
- `PUT /api/v1/workspaces/{ws_id}/employees/{id}`
  - Requires `employees:update`.
  - Supports Rust-compatible nullable `user_id`, `full_name`, `employee_id_number`, `tags`, `recruitment_source_id`, and `force_transfer` fields.
  - Updates only supplied fields and preserves tenant filtering.
  - `force_transfer=true` clears an existing user link in the same workspace before assignment.

## Files

- `apiserver/internal/api/employees.go`
- `apiserver/internal/api/employees_test.go`
- `apiserver/internal/repository/employees.go`
- `apiserver/main.go`

## Verification

- `./scripts/_go-gates.sh`: ALL GATES PASSED.
- `./leedevkit test api-go`: ALL GATES PASSED.
- Endpoint metrics: contract routes **403**, Go registrations **149**, covered **126**, missing **277**.

## Remaining P6-03 scope

Employee conditions/history, contracts, employments, assignments, lifecycle, payroll configuration, salary components, and import/upload routes remain unmigrated and require separate evidence.
