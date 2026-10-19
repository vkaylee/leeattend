# P6-12 Domain Completion Review

Status: in-progress. Measured coverage contradicts a completion claim; this file records the audited gap.

## Measured Coverage (machine-derived)

Source of truth: `endpoint-matrix.csv` (403 contract routes) compared against the Go route table in `apiserver/main.go`, normalized per path shape. Full gap list: `endpoint-coverage-gap.md`.

- Contract routes: **403**
- Go route registrations: **108**
- Contract routes covered: **92**
- Contract routes missing: **311**

Missing by method: GET 145, POST 87, DELETE 41, PUT 23, PATCH 15.

## Domain Slices Implemented So Far

| Task | Domain | Implemented | Evidence |
|---|---|---|---|
| P6-01 | Auth / Token | login, refresh, logout, `/me` | `evidence-p6-01-auth.md` |
| P6-02 | Workspaces & Settings | create, patch, roles, feature flags, configs | `evidence-p6-02-p6-03.md` |
| P6-03 | Employees & Custom Fields | create, list, delete, rehire, custom-field definitions CRUD, entity values GET/PUT | `evidence-p6-03-custom-fields.md` |
| P6-04 | Attendance & Time | punch in/out, history, adjustments, approval | `evidence-p6-04-attendance.md` |
| P6-05 | Shifts & Rosters | shifts, patterns, rosters, swaps, segments | `evidence-p6-05-shifts.md` |
| P6-06 | Leave & ESS | leave types, submit, approve, balances, ESS profile/contacts, workflows | `evidence-p6-06-leaves.md` |
| P6-07 | Notifications | list, unread count, mark read, mark all read | `evidence-p6-07-notifications.md` |
| P6-08 | Payroll & Payslips | workspace and employee payslips with decimal amounts | `evidence-p6-08-payroll.md` |
| P6-09 | Reports & Exports | async export enqueue (`outbox_jobs` + Redis), status polling, cancel, download | `evidence-p6-09-reports.md`, `evidence-p6-07-report-export.md` |
| P6-10 | Files & Signatures | presigned upload, metadata, download URL, soft delete, signature requests | `evidence-p6-10-files.md` |
| P6-11 | Condition Types | workspace condition-type catalog CRUD | `evidence-p6-11-condition-types.md` |

## Verification Evidence

- `scripts/_go-gates.sh`: PASSED (gofmt, go vet, staticcheck, govulncheck, go mod verify, go test -race).
- `./leedevkit test api-go`: PASSED.
- Tenant schema smoke for condition types executed against `leeattend-test_db_system_1`.

## Blocking Criteria For P6-12 Completion

P6-12 can only be marked complete when the endpoint matrix has no `unmapped`/`implemented` rows and every missing route above is implemented and differentially verified.
