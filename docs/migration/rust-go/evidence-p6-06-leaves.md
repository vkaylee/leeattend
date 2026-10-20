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

## ESS Profile and Emergency Contacts Slice

### Implemented Endpoints
1. `GET /api/v1/ess/profile`
   - Resolves the caller's employee from the authenticated user and `X-Workspace-Id`.
   - Returns employee, optional profile, and emergency contacts.
2. `PUT /api/v1/ess/profile`
   - Validates `date_of_birth` as `YYYY-MM-DD` before upsert.
   - Upserts the employee profile by `employee_id`.
3. `GET /api/v1/ess/emergency-contacts`
   - Returns emergency contacts ordered by primary first, then oldest first.
4. `POST /api/v1/ess/emergency-contacts`
   - Validates non-empty contact name, relationship, and phone.
   - When `is_primary` is true, demotes existing primary contacts in the same tenant transaction.
5. `DELETE /api/v1/ess/emergency-contacts/{contact_id}`
   - Deletes only contacts owned by the resolved employee, returning 404 when absent.

### Source Files
- `internal/api/ess.go`: Typed ESS DTOs and handlers.
- `internal/repository/ess.go`: Tenant-scoped profile and emergency contact queries.
- `internal/api/ess_test.go`: Forbidden-without-employee, profile fetch, date validation, contact validation, and delete coverage.

## Approval Workflow Slice

### Implemented Endpoints
1. `GET /api/v1/workspaces/{ws_id}/approval-workflows`
   - Requires `approval_workflows:read`; optional `entity_type` filter.
2. `POST /api/v1/workspaces/{ws_id}/approval-workflows`
   - Requires `approval_workflows:manage`.
   - Validates entity type, name, and at least one step with positive step order.
   - Creates workflow and steps atomically.
3. `POST /api/v1/workspaces/{ws_id}/approval-workflows/approve`
   - Validates supported actions (`approve`, `reject`, `return`, `reassign`, `skip` and aliases).
   - Requires `overtime:approve_workflow` for `ot_request`.
   - Writes `approval_action_log` and updates the pending instance status transactionally.
4. `GET /api/v1/workspaces/{ws_id}/approval-workflows/history/{type}/{id}`
   - Returns action history ordered by `acted_at`.

### Source Files
- `internal/api/approval_workflow.go`: Typed workflow DTOs and handlers.
- `internal/repository/approval_workflow.go`: Workflow creation, action processing, and history queries.
- `internal/api/approval_workflow_test.go`: Permission denial, create, invalid action rejection, and history coverage.

## Verification Status (ESS and Approval)
- `scripts/_go-gates.sh`: ALL GATES PASSED.
- `./leedevkit test api-go`: ALL GATES PASSED.
