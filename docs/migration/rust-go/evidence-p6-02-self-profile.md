# P6-02 Self-Profile and Workspace Users Evidence

Status: in progress; profile read/update, workspace user listing, and effective permission introspection implemented with unit tests. Live Rust-vs-Go differential parity remains pending.

## Implemented Endpoints
1. `GET /api/v1/me`
   - Requires authenticated user context.
   - Returns full `UserResponse` with nullable profile fields, `created_at`, `system_roles`, and `avatar_file_id`.
   - Returns 404 with `error.user_not_found` when the user record is absent.
2. `PATCH /api/v1/me`
   - Updates only supplied `full_name`, `avatar_url`, `bio`, and `preferred_language`.
   - Absent or null fields remain unchanged, matching Rust `Option` semantics.
   - Returns 200 with `UserResponse`; `system_roles` is always `[]` for the update response.
3. `GET /api/v1/users`
   - Requires authenticated workspace membership (`X-Workspace-Id`).
   - Returns users joined through `workspace_members` ordered by `users.created_at DESC`.
   - Returns `[]` when no users match.
4. `GET /api/v1/me/permissions`
   - Requires authenticated workspace membership.
   - Returns role permissions, active extra permissions, role UUIDs, and `managed_department_ids`.
   - Non-global department scoping is reported as `null` until department-scope loading is implemented.

## Source Files
- `internal/service/auth.go`: `UserRecord` profile fields and profile/update/workspace-user methods.
- `internal/repository/auth.go`: expanded user projection, system role query, profile update, workspace user list, and transaction-backed registration.
- `internal/repository/workspace_authz.go`: membership existence plus effective role/extra permission query.
- `internal/api/auth.go`: profile, update, workspace users, and permissions handlers with Rust-compatible error keys.
- `internal/api/auth_test.go`, `internal/service/auth_test.go`, `internal/api/auth_fixture_test.go`: profile, update, membership, and permission contract tests.
- `main.go`: PATCH `/me`, `/users`, and `/me/permissions` routes plus authorizer wiring.

## Verification Status
- `./scripts/_go-gates.sh`: ALL GATES PASSED (`gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, `go test -race`).
- `./leedevkit test api-go`: ALL GATES PASSED.
- Endpoint metrics:
  - Contract routes: **403**
  - Go route registrations: **141** (+6)
  - Contract routes covered: **118** (+3)
  - Contract routes missing: **285** (-3)
