# P6-06 Leave Type Lifecycle Evidence

Status: implemented; live database integration and differential parity remain pending.

## Implemented Endpoints

1. `DELETE /api/v1/workspaces/{ws_id}/leave/types/{id}`
   - Requires `leave:types:manage`.
   - Rejects deletion with `409 CONFLICT` when leave balances reference the type.
   - Soft deletes via `deleted_at`; returns `204`.
   - Returns `404` when the active row is absent.
2. `PUT /api/v1/workspaces/{ws_id}/leave/types/{id}/config`
   - Requires `leave:types:manage`.
   - Replaces `policy_config`, returns the leave type with `200`.
   - Returns `404` when the row is absent.
3. `POST /api/v1/workspaces/{ws_id}/leave/types/{id}/restore`
   - Requires `leave:types:manage`.
   - Restores only a soft-deleted row, returns the leave type with `200`.
   - Returns `404` when missing or already active.

All queries constrain `workspace_id`; no cross-workspace access.

## Source Files

- `apiserver/internal/api/leaves.go`: lifecycle handlers and policy config validation.
- `apiserver/internal/repository/leaves.go`: update config, guarded soft delete, restore.
- `apiserver/internal/api/leaves_test.go`: success and forbidden-path tests.
- `apiserver/main.go`: route registration.

## Verification

- `./scripts/_go-gates.sh`: ALL GATES PASSED.
- `./leedevkit test api-go`: ALL GATES PASSED.
- Endpoint metrics: contract routes **403**, Go registrations **146**, covered **123**, missing **280**.
