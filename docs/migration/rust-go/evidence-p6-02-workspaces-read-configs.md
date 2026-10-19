# P6-02 Workspace Read and Workspace Configs Evidence

Status: in progress; typed HTTP handlers, repository queries, and unit tests implemented. Full endpoint parity and live integration proof remain open.

## Implemented Endpoints
1. `GET /api/v1/workspaces`
   - Requires authenticated user context (`UserID`).
   - Queries `workspaces` table via `INNER JOIN workspace_members` filtering by `user_id = $1`.
   - Returns 200 OK with `[]WorkspaceRecord` (empty array `[]` when no memberships).
   - Omit `custom_db_url_encrypted` to match Rust `#[serde(skip_serializing)]`.
2. `GET /api/v1/workspaces/{ws_id}`
   - Requires authenticated user context (`UserID`).
   - Queries workspace by ID; returns 404 Not Found with error key `error.workspace_not_found` when absent.
   - Returns 200 OK with `WorkspaceRecord`.
3. `GET /api/v1/workspaces/{ws_id}/configs`
   - Requires authenticated user context in workspace with `workspace:settings:read` permission.
   - Queries `workspace_config_entries` for given `workspace_id` ordered by `created_at DESC`.
   - Returns 200 OK with `[]WorkspaceConfigEntry`.
4. `GET /api/v1/workspaces/{ws_id}/configs/{key}`
   - Requires authenticated user context in workspace with `workspace:settings:read` permission.
   - Checks effective config in `workspace_config_entries` and returns 200 OK with Rust-compatible mock `ResolvedConfig` (`{"key":"...","value":{"Text":"mock"},"scope_type":"workspace","scope_id":null}`).
5. `DELETE /api/v1/workspaces/{ws_id}/configs/{key}`
   - Path parameter `key` parses UUID entry ID matching Rust handler behavior.
   - Requires authenticated user context in workspace with `workspace:settings:write` permission.
   - Deletes config entry from `workspace_config_entries` scoped by `id = $1 AND workspace_id = $2`.
   - Returns 200 OK.

## Source Files
- `internal/api/workspaces.go`: Typed handlers (`List`, `Get`, `ListConfigs`, `GetConfig`, `DeleteConfig`), DTOs, and permission checks.
- `internal/api/workspaces_test.go`: Unit tests for workspace read, config CRUD, and permission rejection.
- `internal/repository/workspaces.go`: SQL queries for workspace membership filtering, ID lookup, typed config insert/upsert, config listing, and deletion.
- `internal/repository/workspaces_test.go`: SQL placeholder and parameter binding tests for workspace and config queries.
- `main.go`: Registered the 5 endpoints on `ServeMux`.

## Verification Status
- `./scripts/_go-gates.sh`: ALL GATES PASSED (`gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, `go test -race`).
- `./leedevkit test api-go`: ALL GATES PASSED.
- Endpoint metrics:
  - Contract routes: **403**
  - Go route registrations: **127** (+5)
  - Contract routes covered: **111** (+5)
  - Contract routes missing: **292** (-5)
