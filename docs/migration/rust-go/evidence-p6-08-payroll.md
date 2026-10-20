# P6-08 Payroll and Payslip Domain Evidence

Status: in-progress; workspace and employee payslip queries implemented using typed decimal amounts and verified through container build gates.

## Payroll Domain Slice

### Implemented Endpoints
1. `GET /api/v1/workspaces/{ws_id}/payroll/payslips`
   - Requires `payroll:read` permission.
   - Enforces valid `year` and `month` query parameters.
   - Queries `payslips` in the period range with descending ordering.
   - All amounts (`base_pay_amount`, `overtime_pay_amount`, `deductions_amount`, `net_pay_amount`, `holiday_work_premium_amount`) use `shopspring/decimal`. No `float64`.
2. `GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/payslips`
   - Requires `payroll:read` permission.
   - Tenant-scoped query joining `employments` to prevent cross-workspace data leakage.
   - Returns all payslips for the given employee in descending order.

### Source Files
- `internal/api/payroll.go`: Typed handlers, DTOs, decimal amounts, and query validation.
- `internal/repository/payroll.go`: Database queries with decimal string conversion.
- `main.go`: Registered payroll routes and wired `PayrollRepository`.

### Verification Status
- `scripts/_go-gates.sh`: ALL GATES PASSED (`gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, `go test -race`).
- `./leedevkit test api-go`: ALL GATES PASSED.
