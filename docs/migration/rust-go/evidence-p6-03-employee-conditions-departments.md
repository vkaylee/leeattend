# P6-03 Employee Conditions, Departments, Compensation History Evidence

Status: implemented in Go for endpoint-matrix rows 92-97.

## Routes

- `GET /api/v1/workspaces/{ws_id}/employees/{id}/conditions`
- `POST /api/v1/workspaces/{ws_id}/employees/{id}/conditions`
- `DELETE /api/v1/workspaces/{ws_id}/employees/{id}/conditions/{cond_id}`
- `GET /api/v1/workspaces/{ws_id}/employees/{id}/departments`
- `POST /api/v1/workspaces/{ws_id}/employees/{id}/departments`
- `GET /api/v1/workspaces/{ws_id}/employees/{id}/compensation-history`

## Contract evidence

- Condition reads require authenticated workspace membership and active employment. Condition writes use `employees:update` with current-department scope; malformed dates return validation errors; deletes are tenant and active-employment scoped.
- Department history derives from non-deleted `employments` rows with non-null departments. Transfer uses one tenant transaction, closes the predecessor at `effective_date - 1 day`, inserts a copied employment segment for mid-segment transfers, and updates contiguous later segments retaining predecessor department.
- Compensation history batch-loads employments, employers, departments, and salary component history. Timeline grouping, decimal-only totals/deltas, event classification, and newest-first ordering match Rust service behavior. Empty employment selection returns `{"timeline":[]}`.

## Go implementation

- API DTOs/handlers: `apiserver/internal/api/employee_conditions.go`, `employee_departments.go`, `compensation_history.go`
- Tenant repositories: `apiserver/internal/repository/employee_conditions.go`, `employee_departments.go`, `compensation_history.go`
- Route registration: `apiserver/main.go`
- Focused contract tests cover active-employment/not-found/permission paths, transfer date boundary and transaction callback, decimal timeline, and empty result.

No history table added. No endpoint-specific audit event fabricated. Money uses `shopspring/decimal`; dates remain ISO date strings and PostgreSQL `date` values.
