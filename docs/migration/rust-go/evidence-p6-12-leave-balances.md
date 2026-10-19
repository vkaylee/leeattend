# P6-12 Leave Balances, Adjustments, Policies, Accruals, and Behalf Evidence

Task 4.2 Leave balances & adjustments completed in Go API, achieving 100% route coverage for the leave domain.

## Routes Covered (21)

All 21 routes wired in `apiserver/main.go` and implemented in `internal/api/leaves.go` and `internal/repository/leaves.go`:

### Core Balances & Adjustments (4)
1. `GET /api/v1/workspaces/{ws_id}/leave/balances` — Permission `leave:balances:read`, paginated workspace employee balance overview with active policy attribution.
2. `GET /api/v1/workspaces/{ws_id}/employees/{id}/leave/history` — Permission `leave:balances:read` or self/member context, ledger transaction stream (accrual, adjustment, usage).
3. `POST /api/v1/workspaces/{ws_id}/employees/{id}/leave/adjust` — Permission `leave:balances:adjust`, atomic balance upsert with accrual log record.
4. `GET /api/v1/employees/{id}/leave/requests` — MemberContext or self check, list leave requests for employee.

### Policies & Assignments (7)
5. `GET /api/v1/workspaces/{ws_id}/leave/policies` — MemberContext, list workspace leave policy definitions.
6. `POST /api/v1/workspaces/{ws_id}/leave/policies` — Permission `leave:policies:manage`, create typed leave policy definition with rule config.
7. `PATCH /api/v1/workspaces/{ws_id}/leave/policies/{policy_id}` — Permission `leave:policies:manage`, update policy definition.
8. `DELETE /api/v1/workspaces/{ws_id}/leave/policies/{policy_id}` — Permission `leave:policies:manage`, soft delete policy definition and associated assignments.
9. `POST /api/v1/workspaces/{ws_id}/leave/policies/assign` — Permission `leave:policies:manage`, create assignment with temporal auto-close of overlapping prior assignments.
10. `GET /api/v1/workspaces/{ws_id}/leave/policies/assignments` — Permission `leave:policies:manage`, list assignments joined with policy and leave type names.
11. `DELETE /api/v1/workspaces/{ws_id}/leave/policies/assignments/{assignment_id}` — Permission `leave:policies:manage`, soft delete assignment with timeline healing for preceding records.

### Accrual & Import Jobs (6)
12. `POST /api/v1/workspaces/{ws_id}/leave/accrual/run` — Permission `leave:accrual:run`, enqueue `leave_accrual` outbox job.
13. `GET /api/v1/workspaces/{ws_id}/jobs/{job_id}/status` — MemberContext, query outbox/import job execution status.
14. `POST /api/v1/workspaces/{ws_id}/leave/upload-excel` — Permission `leave:requests:submit`, multipart XLSX upload, parse rows, insert requests, and capture error workbook.
15. `GET /api/v1/workspaces/{ws_id}/leave/import/latest` — Permission `leave:requests:submit`, retrieve most recent leave import job.
16. `GET /api/v1/workspaces/{ws_id}/leave/import/{job_id}/errors` — Permission `leave:requests:submit`, download error XLSX workbook.
17. `GET /api/v1/workspaces/{ws_id}/leave/upload-template` — Permission `leave:requests:submit`, download 2-sheet XLSX import template (sample rows + workspace leave types).

### Leave Behalf Assignments (4)
18. `GET /api/v1/workspaces/{ws_id}/leave/behalf-assignments` — MemberContext, list behalf assignments for domain `leave_request`.
19. `POST /api/v1/workspaces/{ws_id}/leave/behalf-assignments` — Permission `leave:policies:manage`, create behalf assignment for domain `leave_request`.
20. `GET /api/v1/workspaces/{ws_id}/leave/behalf-assignments/my-targets` — MemberContext, list delegator targets for current user for domain `leave_request`.
21. `DELETE /api/v1/workspaces/{ws_id}/leave/behalf-assignments/{id}` — Permission `leave:policies:manage`, delete behalf assignment.

## Technical Implementation Details

- **Decimal Arithmetic**: All calculations for entitlement, accrued amount, remaining balance, adjustments, and bonuses use `shopspring/decimal.Decimal`. Zero `float64` in balances or ledger rows.
- **Permission Matrix**: Exact match with Rust Axum route layer (`leave:balances:read`, `leave:balances:adjust`, `leave:policies:manage`, `leave:accrual:run`, `leave:requests:submit`).
- **Temporal Policy Healing**:
  - `AssignPolicy`: Finds previous overlapping assignments on same leave type and assignee, adjusts `effective_to = ($start_date - 1 day)`, soft-deletes future assignments (`effective_from >= $start_date`).
  - `DeletePolicyAssignment`: When deleting an assignment with `effective_from`, heals preceding record by extending its `effective_to` to the deleted assignment's `effective_to`.
- **Behalf Delegation**: Reuses existing `AttendancePolicyRepository` (`BehalfAssignmentRepository`) with domain `"leave_request"`.
- **Import & Accrual**:
  - `CreateAccrualJob` writes to `outbox_jobs` table with job type `leave_accrual`.
  - Excel parser parses workbook with `parseEmployeeWorkbook`, creates pre-approved or pending requests, and writes error spreadsheet on failures.

## Test Evidence

- `internal/api/leaves_test.go`:
  - `TestLeaveHandlerWorkspaceBalances`: Verifies paginated workspace balance list and 403 permission guard.
  - `TestLeaveHandlerEmployeeHistory`: Verifies ledger history retrieval and permission handling.
  - `TestLeaveHandlerAdjustBalance`: Verifies balance adjustment call, 403 forbidden, and 422 validation.
  - `TestLeaveHandlerMyRequests`: Verifies member context leave request retrieval.
  - `TestLeaveHandlerPoliciesCRUD`: Covers List (200), Create (201), Update (200), Delete (204).
  - `TestLeaveHandlerPolicyAssignments`: Covers Assign (201), List (200), Delete (204).
  - `TestLeaveHandlerAccrualAndJobStatus`: Covers accrual triggering and job status retrieval.
  - `TestLeaveHandlerBehalfAssignments`: Covers behalf List, Create, Targets, Delete.
  - `TestLeaveHandlerImportEndpoints`: Covers template download, latest job query, and error spreadsheet download.
  - `TestLeaveDecimalCalculations`: Table-driven tests validating exact decimal arithmetic without floating point drift.

## Coverage Impact

- Go route registrations: 309 -> 330 (+21)
- Contract routes covered: 284 -> 305 (+21)
- Missing contract routes: 119 -> 98 (-21)
- Leave domain coverage: Complete (100% of leave routes covered)
