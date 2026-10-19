# P6-06 Leave, ESS, and Approval Domain Evidence

Status: in-progress; Leave Request & Approval Lifecycle Slice implemented and verified through container build gates.

## Leave Request & Approval Lifecycle Slice

### Implemented Endpoints
1. `GET /api/v1/workspaces/{ws_id}/leave/types`
   - Returns non-deleted leave types for the workspace, sorted by name.
2. `POST /api/v1/workspaces/{ws_id}/leave/types`
   - Requires `leave:types:manage` permission.
   - Creates workspace leave type with policy configuration. Maps unique constraint violations to HTTP 409 Conflict.
3. `POST /api/v1/workspaces/{ws_id}/leave/submit`
   - Requires `leave:requests:submit` permission.
   - Enforces partial-day rules:
     - `first_half` and `second_half` must have `start_date == end_date`.
     - `custom_hours` must have `start_date == end_date`, `start_time < end_time`.
   - Auto-approves (initial status `pre_approved`) if requester has any of `leave:requests:approve`, `leave:approve:l1`, `leave:approve:l2`; defaults to `pending` otherwise.
   - Prevents overlapping leaves via PostgreSQL exclusion constraint and maps violations to HTTP 409 Conflict.
4. `GET /api/v1/workspaces/{ws_id}/leave/requests`
   - Requires `leave:requests:read` permission.
   - Supports optional `status` filter.
5. `POST /api/v1/workspaces/{ws_id}/leave/requests/{id}/approve`
   - Requires `leave:requests:approve` (or L1/L2 approver permission).
   - Validates transition to `approved` or `rejected` only.
   - Transactionally updates status and sets `approved_by_user_id`.
6. `GET /api/v1/workspaces/{ws_id}/employees/{id}/leave/balances`
   - Requires `leave:balances:read` or self-employee context.
   - Calculates entitlement, carried over, accrued amount, used days count, and remaining balance.
   - Uses `shopspring/decimal` for `accrued_amount` and `remaining`. No `float64` in balances.

### Source Files
- `internal/api/leaves.go`: Typed handlers, DTOs, validation rules, and error mappings.
- `internal/repository/leaves.go`: PostgreSQL tenant transactions, exclusion constraint mappings, and decimal balance calculations.
- `internal/api/leaves_test.go`: Unit tests for submit defaults, custom hours validation, auto-approval permissions, approval status transitions, and decimal balance verification.
- `main.go`: Registered leave routes and wired `LeaveRepository`.

### Verification Status
- `scripts/_go-gates.sh`: ALL GATES PASSED (`gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, `go test -race`).
- `./leedevkit test api-go`: ALL GATES PASSED.
