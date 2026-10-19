# P6-02 / P6-03 vertical slice evidence

Status: in progress; typed HTTP, authorization, and PostgreSQL adapters implemented. Full endpoint parity and live integration proof remain open.

## Implemented

- `internal/api/workspaces.go`: typed workspace creation, settings PATCH validation, role creation, feature-flag update/check, and tenant config write handlers.
- `internal/repository/workspaces.go`: PostgreSQL workspace/role/config/feature-flag queries with workspace IDs carried through every tenant query and unique-constraint mapping to `409`.
- `internal/repository/workspace_authz.go`: DB-backed platform-admin and workspace permission checks; default deny on missing DB/error.
- `internal/api/employees.go`: typed employee create/list/soft-delete/rehire/custom-field handlers with permission checks, pagination validation, and no raw PII logging.
- `internal/repository/employees.go`: tenant-scoped employee queries, soft-delete filter, duplicate-constraint mapping, rehire, and custom-field upsert.
- `main.go`: workspace and employee route registration plus tenant pool injection.

## Verification

- `./scripts/_go-gates.sh`: PASS after these changes. Includes `gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, and `go test -race`.
- API tests cover regular-user workspace denial, empty settings PATCH validation, employee permission denial, employee soft-delete status, and employee create validation.
- Fixture preflight remains PASS: 59 fixtures, unique IDs, all required sections.

## Remaining proof

- Live PostgreSQL integration with migrations and tenant pool transaction boundaries.
- Full workspace/employee fixture execution against Rust and Go.
- Audit event assertions, PII redaction snapshots, and query-count/N+1 evidence.
- Complete P6-02/P6-03 endpoint matrix rows before ledger status can become `verified`.
