# P6-13 ESS (Employee Self-Service) Endpoints Evidence

Task 4.3 ESS endpoints (13 missing routes) completed in Go API, achieving 100% route coverage for the ESS domain (20/20 routes).

## Route Coverage Status

- Contract routes: **403**
- Go route registrations: **343** (+13)
- Contract routes covered: **318** (+13)
- Contract routes missing: **85** (-13)
- ESS domain coverage: **20/20 (100%)**

---

## 13 Newly Implemented & Wired Routes

All 13 routes wired in `apiserver/main.go` and implemented in `internal/api/ess.go`, `internal/repository/ess.go`, `internal/repository/attendance.go`, and `internal/repository/shift_swaps.go`:

### 1. Documents & Acknowledgements (3)
1. `GET /api/v1/ess/documents/pending` — 200 OK `[]AcknowledgementRecord`. Resolves workspace and employee, queries pending document acknowledgements from `file_acknowledgements`.
2. `POST /api/v1/ess/documents/{id}/acknowledge` — 200 OK `{"status": "acknowledged"}`. Marks acknowledgement record as acknowledged with timestamp in `file_acknowledgements`.
3. `GET /api/v1/ess/documents` — 200 OK `[]EmployeeDocumentRecord`. Lists active employee documents for calling employee from `employee_documents`.

### 2. Payroll (1)
4. `GET /api/v1/ess/payslips` — 200 OK `[]PayslipRecord`. Resolves employee context and calls `PayrollRepository.ListEmployeePayslips`.

### 3. Attendance & Time Tracking (3)
5. `GET /api/v1/ess/attendance` — 200 OK `[]TimeEntryRecord`. Query params `start_date` and `end_date` (YYYY-MM-DD), default last 30 days. Queries time history via `AttendanceRepository.ListTimeHistory`.
6. `POST /api/v1/ess/punch` — 201 Created `TimeEntryRecord`. Toggles punch session: if open time entry exists (`punch_out_time IS NULL`), punches out and computes duration; otherwise punches in with entry type (default "work") and note via `AttendanceRepository.ESSPunch`.
7. `GET /api/v1/ess/time-entries` — 200 OK `[]TimeEntryRecord`. Query params `start_date` and `end_date` (RFC3339 timestamp), queries history via `AttendanceRepository.ListTimeHistory`.

### 4. Leaves (2)
8. `GET /api/v1/ess/leave/balances` — 200 OK `[]EssLeaveBalance`. Queries current year balances via `LeaveRepository.GetEmployeeLeaveBalances`, maps Decimal fields to float64 DTO (`leave_type`, `entitlement`, `accrued_amount`, `carried_over`, `used`, `remaining`).
9. `GET /api/v1/ess/leave/requests` — 200 OK `[]LeaveRequestRecord`. Lists leave requests for calling employee via `LeaveRepository.ListEmployeeLeaveRequests`.

### 5. Shift Swaps (4)
10. `GET /api/v1/ess/shift-swaps` — 200 OK `[]ShiftSwapRequestWithNames`. Lists shift swap requests where calling employee is requester or target, with resolved employee and shift names.
11. `POST /api/v1/ess/shift-swaps` — 201 Created `ShiftSwapRequest`. Validates target employee, active employments, absence of conflicting active/approved swaps, and non-identical shifts. Initial status `pending_target`.
12. `PUT /api/v1/ess/shift-swaps/{id}/respond` — 200 OK. Caller must be `target_id`. Status transition: `accept: true` -> `pending_admin`; `accept: false` -> `rejected`. Non-target -> 403 Forbidden; non-pending -> 409 Conflict.
13. `DELETE /api/v1/ess/shift-swaps/{id}` — 204 No Content. Caller must be `requester_id`. Status must be `pending_target` or `pending_admin`. State transition: `cancelled`. Non-requester -> 403 Forbidden; other status -> 409 Conflict.

---

## Complete ESS Route Map (20/20)

| # | Method | Path | Status | Handler |
|---|---|---|---|---|
| 1 | GET | `/api/v1/ess/profile` | 200 | `ESSHandler.GetProfile` |
| 2 | PUT | `/api/v1/ess/profile` | 200 | `ESSHandler.UpdateProfile` |
| 3 | GET | `/api/v1/ess/emergency-contacts` | 200 | `ESSHandler.GetContacts` |
| 4 | POST | `/api/v1/ess/emergency-contacts` | 201 | `ESSHandler.AddContact` |
| 5 | DELETE | `/api/v1/ess/emergency-contacts/{contact_id}` | 204 | `ESSHandler.DeleteContact` |
| 6 | GET | `/api/v1/ess/workspaces/{ws_id}/signatures/pending` | 200 | `FileHandler.ListPendingSignatures` |
| 7 | POST | `/api/v1/ess/workspaces/{ws_id}/signatures/{request_id}/sign` | 200 | `FileHandler.SignDocument` |
| 8 | GET | `/api/v1/ess/documents/pending` | 200 | `ESSHandler.GetPendingDocuments` |
| 9 | POST | `/api/v1/ess/documents/{id}/acknowledge` | 200 | `ESSHandler.AcknowledgeDocument` |
| 10 | GET | `/api/v1/ess/documents` | 200 | `ESSHandler.GetDocuments` |
| 11 | GET | `/api/v1/ess/payslips` | 200 | `ESSHandler.GetPayslips` |
| 12 | GET | `/api/v1/ess/attendance` | 200 | `ESSHandler.GetAttendance` |
| 13 | POST | `/api/v1/ess/punch` | 201 | `ESSHandler.Punch` |
| 14 | GET | `/api/v1/ess/time-entries` | 200 | `ESSHandler.GetTimeEntries` |
| 15 | GET | `/api/v1/ess/leave/balances` | 200 | `ESSHandler.GetLeaveBalances` |
| 16 | GET | `/api/v1/ess/leave/requests` | 200 | `ESSHandler.GetLeaveRequests` |
| 17 | GET | `/api/v1/ess/shift-swaps` | 200 | `ESSHandler.GetShiftSwaps` |
| 18 | POST | `/api/v1/ess/shift-swaps` | 201 | `ESSHandler.CreateShiftSwap` |
| 19 | PUT | `/api/v1/ess/shift-swaps/{id}/respond` | 200 | `ESSHandler.RespondShiftSwap` |
| 20 | DELETE | `/api/v1/ess/shift-swaps/{id}` | 204 | `ESSHandler.CancelShiftSwap` |

---

## Architecture & Auth Verification

- **Workspace Resolution**: `resolveContextWithUser` extracts workspace ID from HTTP context (`httpinfra.WorkspaceIDFromContext`), falling back to header `X-Workspace-Id`. Missing workspace returns 400 Bad Request.
- **Employee Resolution**: Extracted user ID is verified against workspace via `h.Repository.ResolveEmployeeForUser`. Missing or unlinked employee record returns 403 Forbidden with `apperror.KeyNoEmployeeRecord` (`"error.no_employee_record"`).
- **Error Mapping**:
  - `ErrESSNoEmployeeRecord` -> 403 `error.no_employee_record`
  - `ErrShiftSwapForbidden` -> 403 `error.forbidden`
  - `ErrShiftSwapNotFound`, `ErrESSNotFound`, `pgx.ErrNoRows` -> 404 `error.not_found`
  - `ErrShiftSwapConflict` -> 409 `error.conflict`
  - `ErrESSValidation`, `ErrNoActiveEmployment`, `ErrShiftSwapValidation` -> 400 `error.validation`

---

## Unit Test Coverage (`internal/api/ess_test.go`)

- `TestESSMissingWorkspaceBadRequest`: Verifies 400 Bad Request on missing workspace header/context.
- `TestESSGetProfileNoLinkedEmployeeForbidden`: Verifies 403 Forbidden on unlinked employee record.
- `TestESSGetPendingDocumentsSuccess`: Verifies 200 OK with `[]AcknowledgementRecord`.
- `TestESSAcknowledgeDocumentSuccess`: Verifies 200 OK with `{"status": "acknowledged"}`.
- `TestESSGetDocumentsSuccess`: Verifies 200 OK with `[]EmployeeDocumentRecord`.
- `TestESSGetPayslipsSuccess`: Verifies 200 OK with `[]PayslipRecord`.
- `TestESSGetAttendanceSuccess`: Verifies 200 OK with query bounds and `[]TimeEntryRecord`.
- `TestESSPunchToggle`: Verifies 201 Created punch-in (open session) and subsequent 201 Created punch-out (closed session).
- `TestESSPunchNoActiveEmployment`: Verifies 400 Bad Request when employee lacks active employment.
- `TestESSGetTimeEntriesSuccess`: Verifies 200 OK with RFC3339 query params and `[]TimeEntryRecord`.
- `TestESSGetLeaveBalancesSuccess`: Verifies 200 OK with `[]EssLeaveBalance` float64 mappings.
- `TestESSGetLeaveRequestsSuccess`: Verifies 200 OK with `[]LeaveRequestRecord`.
- `TestESSShiftSwapsCRUD`:
  - List employee swaps (200 OK)
  - Create shift swap with status `pending_target` (201 Created)
  - Self-target validation guard (400 Bad Request)
  - Target respond accept (`pending_admin`) (200 OK)
  - Target respond reject (`rejected`) (200 OK)
  - Target respond non-target caller (403 Forbidden)
  - Requester cancel swap (`cancelled`) (204 No Content)
  - Cancel swap non-requester caller (403 Forbidden)
