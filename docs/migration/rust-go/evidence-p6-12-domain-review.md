# P6-12 Domain Completion Review

Status: in-progress. Measured coverage contradicts a completion claim; this file records the audited gap.

## Measured Coverage (machine-derived)

Source of truth: `endpoint-matrix.csv` (403 contract routes) compared against the Go route table in `apiserver/main.go`, normalized per path shape. Script: `scratch/endpoint_coverage.py`. Full gap list: `endpoint-coverage-gap.md`.

- Contract routes: **403**
- Go route registrations: **100**
- Contract routes covered: **83**
- Contract routes missing: **320**

Missing by method: GET 149, POST 88, DELETE 43, PUT 24, PATCH 16.

## Domain Slices Implemented So Far

| Task | Domain | Implemented | Evidence |
|---|---|---|---|
| P6-01 | Auth / Token | login, refresh, logout, `/me` | `evidence-p6-01-auth.md` |
| P6-02 | Workspaces & Settings | create, patch, roles, feature flags, configs | `evidence-p6-02-p6-03.md` |
| P6-03 | Employees & Custom Fields | create, list, delete, rehire, custom fields | `evidence-p6-02-p6-03.md` |
| P6-04 | Attendance & Time | punch in/out, history, adjustments, approval | `evidence-p6-04-attendance.md` |
| P6-05 | Shifts & Rosters | shifts, patterns, rosters, swaps, segments | `evidence-p6-05-shifts.md` |
| P6-06 | Leave & ESS | leave types, submit, approve, balances, ESS profile/contacts, workflows | `evidence-p6-06-leaves.md` |
| P6-07 | Notifications | list, unread count, mark read, mark all read | `evidence-p6-07-notifications.md` |
| P6-08 | Payroll & Payslips | workspace and employee payslips with decimal amounts | `evidence-p6-08-payroll.md` |
| P6-09 | Reports & Exports | async export enqueue (`outbox_jobs` + Redis) and status polling | `evidence-p6-09-reports.md` |
| P6-10 | Files & Signatures | presigned upload, metadata, download URL, soft delete, signature requests | `evidence-p6-10-files.md` |
| P6-11 | Condition Types | workspace condition-type catalog CRUD | `evidence-p6-11-condition-types.md` |

## Verification Evidence

- `scripts/_go-gates.sh`: PASSED (gofmt, go vet, staticcheck, govulncheck, go mod verify, go test -race).
- `./leedevkit test api-go`: PASSED.
- Tenant schema smoke for condition types executed against `leeattend-test_db_system_1`.

## Blocking Criteria For P6-12 Completion

P6-12 can only be marked complete when the endpoint matrix has no `unmapped`/`implemented` rows and every missing route above is implemented and differentially verified.
