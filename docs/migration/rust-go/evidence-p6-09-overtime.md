# P6-09 Overtime Policies, Requests, Behalf, and Attendance OT Evidence

Status: implemented; verification deferred to parent gate run.

## Implemented routes

Overtime policies:

- `GET /api/v1/workspaces/{ws_id}/overtime/policies`: paginated `OvertimePolicyListResponse` ordered by `created_at DESC`.
- `POST /api/v1/workspaces/{ws_id}/overtime/policies`: requires `overtime:policies:manage`; creates a policy definition and returns HTTP 201.
- `GET /api/v1/workspaces/{ws_id}/overtime/policies/{policy_id}`: typed `OvertimePolicyDetail` with rules ordered by `priority DESC, id`.
- `POST /api/v1/workspaces/{ws_id}/overtime/policies/{policy_id}/rules`: requires `overtime:policies:manage`; validates positive `decimal.Decimal` multiplier, 7-bit `days_of_week`, and `HH:MM[:SS]` clock fields.
- `POST /api/v1/workspaces/{ws_id}/overtime/policies/assign`: requires `overtime:policies:manage`; validates `workspace`/`department`/`employee` assignee semantics and `YYYY-MM-DD` effective date.

Overtime requests:

- `GET /api/v1/workspaces/{ws_id}/overtime/requests`: requires `overtime:request:read`; paginated `OvertimeRequestListResponse`.
- `POST /api/v1/workspaces/{ws_id}/overtime/requests`: requires `overtime:request:create`; validates planned date/minutes/reason/employees; resolves active employments inside one transaction.
- `GET /api/v1/workspaces/{ws_id}/overtime/requests/{id}`: requires `overtime:request:read`; returns `OvertimeRequestDetail`.
- `POST /api/v1/workspaces/{ws_id}/overtime/requests/{id}/submit`: requires `overtime:request:create`; atomically sets `pending_approval`/step 1 and submits an `ot_request` workflow instance.
- `POST /api/v1/workspaces/{ws_id}/overtime/requests/{id}/consent/{emp_id}`: self target needs no permission; delegated target requires `overtime:request:create_on_behalf`; accepts only `accepted`/`declined`.

Overtime request behalf:

- `GET /api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments`: paginated assignments filtered to `overtime_request`.
- `POST /api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments`: requires `overtime:policies:manage`; employee/department target XOR.
- `GET /api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments/my-targets`: authenticated caller targets; expands department assignments through active employments.
- `DELETE /api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments/{id}`: requires `overtime:policies:manage`.

Attendance OT claim and approval:

- `GET /api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments`: list `overtime_claim` assignments (no explicit Rust permission guard).
- `POST /api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments`: requires `overtime:policies:manage`.
- `DELETE /api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments/{id}`: requires `overtime:policies:manage`.
- `POST /api/v1/workspaces/{ws_id}/attendance/claim-ot/{employee_id}`: self target bypasses permission; delegated target requires `overtime:approve` OR `overtime:claim_on_behalf`.
- `POST /api/v1/workspaces/{ws_id}/attendance/approve-ot/{employee_id}`: department-scoped `overtime:approve`; only `approved`/`rejected` statuses.

## Contract behavior

- Overtime rules use `shopspring/decimal` end to end; no `float64` arithmetic.
- Policy listing clamps through the shared pagination contract (`page >= 1`, `1..=3000` limit) and defaults to page 1, limit 50.
- Request creation and submission run in a single transaction per operation; submission reuses active approval workflow definitions or the Rust-equivalent `Direct Manager Approval (Fallback)` step snapshot, cancelling any prior pending instance.
- Claim OT upserts `attendance_daily_summaries` only from `pending_employee`/`rejected` states, then submits an `ot_claim` workflow instance. Approval only transitions rows in `pending_manager` state.
- Behalf assignments reuse the shared `BehalfAssignmentRepository` methods (create/list/delete/my-targets) already used by attendance adjustments; the assignment type distinguishes `overtime_request` from `overtime_claim`.
- Self-consent and self-claim bypasses mirror Rust exactly; delegated paths never write before permission checks.

## Files

- `apiserver/internal/api/overtime.go`
- `apiserver/internal/api/overtime_test.go`
- `apiserver/internal/repository/overtime.go`
- `apiserver/internal/repository/overtime_test.go`
- `apiserver/main.go`
- `docs/migration/rust-go/endpoint-coverage-gap.md`

## Verification status

Parent agent must run Go tests/build/gates after sibling changes land. This worker did not run checks by assignment.
