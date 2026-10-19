# P6-02 / P6-03 vertical slice evidence

Status: in progress; typed HTTP, authorization, and PostgreSQL adapters implemented. Full endpoint parity and live integration proof remain open.

## Implemented

- `internal/api/workspaces.go`: typed workspace creation, settings PATCH validation, role creation, complete feature-flag CRUD (list/create/get/update/upsert/delete/check) with Rust field names and `{key,enabled}` check response, and tenant config write handlers.
- `internal/repository/workspaces.go`: PostgreSQL workspace/role/config/feature-flag queries with workspace IDs carried through every tenant query, `flag_key` ordering, `(workspace_id, flag_key)` upsert conflict target, and unique-constraint mapping to `409`.
- `internal/repository/workspace_authz.go`: DB-backed platform-admin and workspace permission checks; default deny on missing DB/error.
- `internal/api/employees.go`: typed employee create/list/soft-delete/rehire/custom-field handlers with permission checks, pagination validation, and no raw PII logging.
- `internal/repository/employees.go`: tenant-scoped employee queries, soft-delete filter, duplicate-constraint mapping, rehire, and custom-field upsert.
- `main.go`: workspace and employee route registration plus tenant pool injection.

## Verification

- `./scripts/_go-gates.sh`: PASS after these changes. Includes `gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, and `go test -race` (`logs/run_go_gates_feature_flag_repo_tests5.log`).
- `./leedevkit test api-go`: PASS (`logs/run_leedevkit_api_go_feature_flags.log`).
- Feature-flag handler coverage: list/create/get/update/upsert/delete/check with read/manage permission gates and empty-key `400` validation in `internal/api/workspaces_test.go`.
- Feature-flag repository coverage: update placeholder/argument ordering and workspace-scoped key lookup in `internal/repository/workspaces_feature_flags_test.go`.
- Fixture preflight remains PASS: 59 fixtures, unique IDs, all required sections.

## Remaining proof

- Live PostgreSQL integration with migrations and tenant pool transaction boundaries.
- Full workspace/employee fixture execution against Rust and Go.
- Audit event assertions, PII redaction snapshots, and query-count/N+1 evidence.
- Complete P6-02/P6-03 endpoint matrix rows before ledger status can become `verified`.
