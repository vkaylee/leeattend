# P6-08 Attendance Rules, Policy Versions, and Behalf Assignments Evidence

Status: implemented; verification deferred to parent gate run.

## Implemented routes

- `GET /api/v1/workspaces/{ws_id}/attendance/rules`: workspace default rules with typed `AttendanceRules` response and default fallback.
- `PATCH /api/v1/workspaces/{ws_id}/attendance/rules`: typed partial update; creates default policy/version when absent and returns `AttendancePolicyVersionResponse`.
- `GET /api/v1/workspaces/{ws_id}/attendance/rules/versions`: default-policy version history ordered by `valid_from DESC`.
- `GET /api/v1/workspaces/{ws_id}/attendance/policies/assignments`: active workspace policy assignments and active rules.
- `POST /api/v1/workspaces/{ws_id}/attendance/policies/assignments`: typed USER/GROUP assignment upsert with version close/open transaction semantics.
- `DELETE /api/v1/workspaces/{ws_id}/attendance/policies/assignments/{assignment_id}`: workspace-scoped soft close; always HTTP 204 after repository success, including zero affected rows.
- `GET /api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments`: typed attendance-adjustment behalf assignments.
- `POST /api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments`: typed behalf assignment creation with employee/department target XOR validation.
- `DELETE /api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments/{id}`: workspace-scoped physical delete; HTTP 200 on success and 404 when no row is affected.
- `GET /api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments/my-targets`: direct employee and current active department targets, deduplicated by employee.

## Contract behavior

- Every workspace query filters authenticated workspace path ID. Permissions match Rust: `workspace:settings:read`, `workspace:settings:write`, and `attendance:adjustments:manage_behalf`; `my-targets` requires authenticated user but no explicit Rust permission.
- Rules use typed JSON fields. Defaults are work unit 30, `floor`, `block`, nullable grace period. Version responses expose nullable grace as `15` domain default.
- `valid_from` accepts RFC3339 or `YYYY-MM-DD`; absent uses current UTC. Version updates close open versions at new `valid_from`, insert new open version, and commit atomically through `pgx.Tx`.
- Policy assignments preserve USER/GROUP target precedence and active windows. New assignments create non-default policy rows named `<TARGET_TYPE> <TARGET_ID> Override`.
- Behalf target resolution only returns active, non-deleted employees and expands current department targets through active employments.

## Files

- `apiserver/internal/api/attendance_policies.go`
- `apiserver/internal/api/attendance_policies_test.go`
- `apiserver/internal/repository/attendance_policies.go`
- `apiserver/internal/repository/attendance_policies_test.go`
- `apiserver/main.go`
- `docs/migration/rust-go/endpoint-coverage-gap.md`

## Verification status

Parent agent must run Go tests/build/gates after sibling changes land. This worker did not run checks by assignment.
