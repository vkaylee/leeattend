# P6-11 Lifecycle, Condition Types, System, and Admin Evidence

Status: in-progress; condition-type catalog CRUD vertical slice implemented and verified through Go gates plus tenant-schema SQL smoke.

## Implemented Slice

Routes:
- `GET /api/v1/workspaces/{ws_id}/condition-types?active_only=true`
- `POST /api/v1/workspaces/{ws_id}/condition-types` — 201
- `GET /api/v1/workspaces/{ws_id}/condition-types/{id}`
- `PATCH /api/v1/workspaces/{ws_id}/condition-types/{id}`
- `DELETE /api/v1/workspaces/{ws_id}/condition-types/{id}` — 204

Behavior:
- Workspace-scoped `condition_type_definitions` queries.
- `workspace:settings:read` and `workspace:settings:write` authorization.
- Uppercase code validation.
- Duplicate workspace/code maps to 409.
- System condition types cannot be deleted; maps to 403.
- Missing records map to 404.
- JSONB `name_i18n` and `affects_rules` preserved as typed `json.RawMessage`.

Source files:
- `apiserver/internal/api/condition_types.go`
- `apiserver/internal/api/condition_types_test.go`
- `apiserver/internal/repository/condition_types.go`
- `apiserver/main.go`

## Verification

- `scripts/_go-gates.sh`: ALL GATES PASSED.
- `./leedevkit test api-go`: ALL GATES PASSED.
- Tenant schema SQL smoke executed in `leeattend-test_db_system_1`: INSERT, ordered SELECT, UPDATE, DELETE, and transaction ROLLBACK succeeded.

## Remaining P6-11 Work

- Lifecycle/status transitions.
- Custom-field definitions and values.
- Remaining system/admin route families and parity fixtures.
