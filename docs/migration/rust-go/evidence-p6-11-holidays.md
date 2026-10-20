# P6-11 Holidays and holiday leave evidence

Task 4.1 completed in Go API.

## Routes

`apiserver/main.go` registers all seven contract routes:

- `GET /api/v1/workspaces/{ws_id}/holidays`
- `POST /api/v1/workspaces/{ws_id}/holidays`
- `GET /api/v1/workspaces/{ws_id}/holidays/{id}`
- `PUT /api/v1/workspaces/{ws_id}/holidays/{id}`
- `PATCH /api/v1/workspaces/{ws_id}/holidays/{id}`
- `DELETE /api/v1/workspaces/{ws_id}/holidays/{id}`
- `POST /api/v1/workspaces/{ws_id}/holidays/{id}/process-leave`

## Contract behavior

`internal/api/holidays.go` uses typed request and response models, `shopspring/decimal.Decimal` for `ot_multiplier`, workspace membership for reads, and `holidays:manage` for mutations. Validation returns `error.validation`; duplicate dates return `error.holiday_already_exists`; missing holidays return `error.holiday_not_found`; unresolved paid leave types return `error.leave_type_not_found`.

`internal/repository/holidays.go` scopes every query by workspace, excludes soft-deleted rows, marks deletes with `deleted_at = NOW()`, and emits roster cache invalidation outbox work after mutation. Holiday leave processing handles paid holidays without writes. Unpaid holidays resolve active employees and date-valid active employments, skip approved leaves, check decimal balances, create approved full-day requests, upsert `on_leave` rosters, record accrual logs, and executes writes inside one tenant transaction.

## Tests

`internal/api/holidays_test.go` covers authentication, membership, pagination and filters, CRUD success/failure, permission checks, conflicts, and paid/deducted/unpaid processing response shapes. `internal/repository/holidays_test.go` covers typed record scanning, not-found/conflict mapping, soft-delete SQL, and paid processing short-circuit.

Endpoint coverage updated from 277/403 covered (126 missing) to 284/403 covered (119 missing).
