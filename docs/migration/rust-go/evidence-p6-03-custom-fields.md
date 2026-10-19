# P6-03 Custom Fields Domain Evidence

Status: in-progress; workspace-scoped custom field definitions CRUD and entity values GET/PUT implemented and verified through container build gates.

## Custom Fields Domain Slice

### Implemented Endpoints
1. `POST /api/v1/workspaces/{ws_id}/custom-fields/definitions` (201 Created)
   - Requires `workspace:settings:write` permission.
   - Enforces valid `entity_type` (e.g., `employee`, `employment`, `department`), non-empty `field_key` and `field_label`, and non-empty `field_type`.
   - Prevents duplicate `(workspace_id, entity_type, field_key)` via unique constraint handling (409 Conflict).
2. `GET /api/v1/workspaces/{ws_id}/custom-fields/definitions/{id}`
   - Requires `workspace:settings:read` permission.
   - Returns 404 for non-existent or soft-deleted definitions.
3. `GET /api/v1/workspaces/{ws_id}/custom-fields/definitions`
   - Requires `workspace:settings:read` permission.
   - Optional `entity_type` filter query parameter.
   - Ordered by `display_order ASC, field_key ASC`.
4. `PATCH /api/v1/workspaces/{ws_id}/custom-fields/definitions/{id}`
   - Requires `workspace:settings:write` permission.
   - Updates label, options, default value, requirements, flags, display order, section name, or validation rules.
5. `DELETE /api/v1/workspaces/{ws_id}/custom-fields/definitions/{id}` (204 No Content)
   - Requires `workspace:settings:write` permission.
   - Soft-deletes definition by setting `deleted_at = NOW()`.
6. `GET /api/v1/workspaces/{ws_id}/custom-fields/values/{entity_type}/{entity_id}`
   - Requires `employees:read` for employee/employment or `departments:read` for department entities.
   - Returns all active definitions for the workspace with associated entity values.
7. `PUT /api/v1/workspaces/{ws_id}/custom-fields/values/{entity_type}/{entity_id}`
   - Requires `employees:update` for employee/employment or `departments:update` for department entities.
   - Atomic upsert of field values into `custom_field_values` with `ON CONFLICT (definition_id, entity_id)` handling typed columns (`value_text`, `value_number`, `value_date`, `value_bool`, `value_json`).

### Source Files
- `internal/api/custom_fields.go`: Typed handlers, DTOs, JSON validation, and error mappings.
- `internal/api/custom_fields_test.go`: Unit tests for definitions CRUD and values GET/PUT with mock repository.
- `internal/repository/custom_fields.go`: Postgres queries on `custom_field_definitions` and `custom_field_values`.
- `main.go`: Registered routes and wired `CustomFieldRepository`.

### Verification Status
- `scripts/_go-gates.sh`: ALL GATES PASSED (`gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, `go test -race`).
- `./leedevkit test api-go`: ALL GATES PASSED.
- Clean cutover: Obsolete narrow `employeeHandler.CustomFields` shim and method removed from `api/employees.go` and `repository/employees.go`.
