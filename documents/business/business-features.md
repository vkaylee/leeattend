# LeeAttend — Feature Specifications

## Feature Matrix

| # | Feature | Status | Module |
|---|---|---|---|
| F1 | Authentication & Authorization | ✅ Live | `auth` |
| F2 | Device Agent Management | ✅ Live | `agents`, `agent-main` |
| F3 | Attendance Log Ingestion | ✅ Live | `attendance_log_service` |
| F4 | Punch Pairing | ✅ Live | `pairing_service` |
| F5 | Shift Management | ✅ Live | `shifts`, `shift_service` |
| F6 | Working Patterns | ✅ Live | `working_patterns` |
| F7 | Holiday Calendar | ✅ Live | `holidays`, `holiday_repository` |
| F8 | Leave Management | ✅ Live | `leaves`, `leave_service` |
| F9 | Attendance Reports | ✅ Live | `report_service`, `excel_writer` |
| F10 | Overtime Management | ✅ Live | `overtime`, `attendance_summary_service` |
| F11 | Payroll Processing | ✅ Live | `payroll_service`, `payroll_evaluator` |
| F12 | Employee Self-Service | ✅ Live | `ess` |
| F13 | Assignment Rules | ✅ Live | `assignment_rules`, `rule_evaluator` |
| F14 | Department Management | ✅ Live | `employees` |
| F15 | Dashboard | ✅ Live | `DashboardPage` |

---

## F1: Authentication & Authorization

- Email/password login with JWT tokens
- Token refresh flow
- Role-based access: `Owner`, `Admin`, `Member`
- `MemberContext` middleware for workspace-scoped authorization
- Dev mode bypass for local testing

## F3: Attendance Log Ingestion

- Raw logs from ZKTeco devices via agent sync
- CSV/Excel import for manual data entry
- Deduplication by device + timestamp hash
- Import job tracking with status reporting

## F4: Punch Pairing

- Multiple algorithms: `greedy`, `optimal`
- Configurable per workspace via `pairing_algorithm` setting
- Handles overnight shifts crossing midnight
- Break detection within shift windows

## F5: Shift Management

- CRUD for shift definitions with break policies
- Shift rosters (per-employee, per-date assignments)
- Temporal assignments with effective dates
- Version tracking for audit trails
- Configuration: grace periods, early punch-in policy, OT allowance

## F7: Holiday Calendar

- National/Company/Regional holiday types
- Per-holiday OT multiplier override
- Holiday-aware attendance status: `holiday` (no work) vs `holiday_work` (worked on holiday)
- Automatic recalculation when holidays are created/updated/deleted
- Excel export with holiday highlighting
- UI: CRUD page with year navigation

## F8: Leave Management

- Configurable leave types (annual, sick, unpaid, etc.)
- Leave policies with accrual rules (monthly/quarterly/yearly)
- Seniority-based bonus allocation
- Carry-over limits and expiry
- Leave request workflow: submit → approve/reject
- Balance tracking and history

## F9: Attendance Reports

- **Daily Report**: Per-employee status with punch times, work units, OT
- **Summary Report**: Period aggregation
- **Excel Export**: Async generation with multi-sheet output
  - Timesheet View (calendar matrix with shift symbols)
  - Raw Details (per-day breakdown with holiday column)
- WebSocket progress streaming during export
- Holiday-aware status badges (green for holiday, orange for holiday_work)

## F10: Overtime Management

- System-detected OT from attendance summaries
- Employee OT claim submission
- Manager approval workflow
- Per-day breakdown in claim view
- Holiday OT multiplier integration (default 300%)

## F11: Payroll Processing

- Monthly payslip calculation engine
- Salary component definitions (fixed, hourly, percentage)
- Employment period-aware calculations
- OT pay with configurable rules and holiday multipliers
- Incentive tiers based on attendance compliance
- Holiday exclusion from late/early penalty counts
- Payslip segments with metadata for audit
