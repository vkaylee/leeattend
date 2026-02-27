# LeeAttend — Domain Knowledge

## Core Domain Entities

### Employee
Primary entity representing a staff member. Linked to user accounts for self-service.

| Field | Type | Description |
|---|---|---|
| `id` | UUID | Primary key |
| `workspace_id` | UUID | Workspace isolation |
| `user_id` | UUID (nullable) | Linked user account for ESS |
| `full_name` | String | Display name |
| `employee_id_number` | String | Company-assigned ID |
| `department_name` | String (nullable) | Current department |
| `hire_date` | Date | First employment date |
| `termination_date` | Date (nullable) | End of employment |
| `is_active` | Boolean | Active status |

**Relationships**: Has many `TimeEntry`, `ShiftRoster`, `EmploymentPeriod`, `EmployeeSalaryComponent`, `LeaveRequest`

### Workspace
Multi-tenant isolation unit. All data is scoped to a workspace.

Key configuration fields:
- `work_unit_minutes` — Granularity for work calculations (e.g., 30 = 30-minute units)
- `rounding_policy` — How to round work time
- `timezone` — Workspace timezone for time calculations
- `early_arrival_strategy` — `allow` | `block` | `clamp` for early punch-ins
- `pairing_algorithm` — Punch pairing strategy
- `holiday_overnight_strategy` — How overnight shifts split across holidays
- `default_holiday_ot_multiplier` — Default OT rate for holidays (e.g., 3.0 = 300%)

### Shift
Work schedule definition with break policies.

| Field | Type | Description |
|---|---|---|
| `start_time` / `end_time` | NaiveTime | Shift hours |
| `break_duration_minutes` | Integer | Break length |
| `break_start` / `break_end` | NaiveTime (nullable) | Fixed break window |
| `is_break_auto_deduct` | Boolean | Auto-deduct break from work time |
| `is_break_overtime` | Boolean | Count break as overtime |
| `report_symbol` | String (nullable) | Short code for reports (e.g., "S8") |
| `early_punch_in_grace_period_minutes` | Integer (nullable) | Grace period |

### Holiday
Workspace holiday calendar entry.

| Field | Type | Description |
|---|---|---|
| `name` | String | Holiday name (e.g., "Tết Dương Lịch") |
| `date` | Date | Holiday date (unique per workspace) |
| `holiday_type` | String | `national` | `company` | `regional` |
| `is_paid` | Boolean | Whether holiday is paid |
| `ot_multiplier` | Decimal (nullable) | Override workspace default |

### TimeEntry
A paired punch-in/punch-out record representing a work session.

### RawAttendanceLog
Raw biometric device data before pairing.

### AttendanceDailySummary
Aggregated daily attendance metrics used for OT claims and payroll.

Key fields: `actual_work_minutes`, `system_detected_ot_minutes`, `claimed_ot_minutes`, `approved_ot_minutes`, `is_holiday`, `holiday_name`

### Payslip
Monthly payroll calculation result with segments, incentives, and salary components.

### LeaveType / LeavePolicy
Configurable leave types (annual, sick, unpaid) with accrual rules, seniority bonuses, and carry-over limits.

## Database Schema

**Engine**: PostgreSQL 15+ via SQLx (compile-time verified)

**Migration count**: 89 migration files (incremental schema evolution)

**Key tables**:
| Table | Purpose |
|---|---|
| `workspaces` | Multi-tenant workspace config |
| `users` / `workspace_members` | Authentication and authorization |
| `employees` / `employment_periods` | Employee lifecycle |
| `devices` / `authorized_agents` | Hardware device management |
| `shifts` / `shift_rosters` | Schedule definitions and assignments |
| `working_patterns` / `working_pattern_details` | Weekly pattern templates |
| `time_entries` | Paired attendance records |
| `raw_attendance_logs` | Raw device data |
| `attendance_daily_summaries` | Aggregated daily metrics |
| `holidays` | Holiday calendar |
| `leave_types` / `leave_policies` | Leave configuration |
| `leave_balances` / `leave_requests` | Leave tracking |
| `salary_component_definitions` / `employee_salary_components` | Salary structure |
| `payslips` / `payslip_segments` | Payroll results |
| `payroll_incentive_policies` / `incentive_tiers` | Bonus policies |
| `departments` / `employee_department_history` | Organization structure |
| `overtime_rules` | OT calculation rules |
| `assignment_rules` | Auto shift assignment rules |

## API Contracts

### Authentication
| Endpoint | Method | Auth | Description |
|---|---|---|---|
| `/auth/login` | POST | Public | Email/password login → JWT |
| `/auth/register` | POST | Public | User registration |
| `/auth/refresh` | POST | Refresh token | Token refresh |
| `/auth/me` | GET | JWT | Current user profile |

### Holiday CRUD
| Endpoint | Method | Auth |
|---|---|---|
| `/workspaces/:ws_id/holidays` | GET | Member |
| `/workspaces/:ws_id/holidays` | POST | Admin |
| `/workspaces/:ws_id/holidays/:id` | GET | Member |
| `/workspaces/:ws_id/holidays/:id` | PUT | Admin |
| `/workspaces/:ws_id/holidays/:id` | DELETE | Admin |

### Reports
| Endpoint | Method | Description |
|---|---|---|
| `/workspaces/:ws_id/reports/daily` | GET | Daily attendance report (paginated) |
| `/workspaces/:ws_id/reports/summary` | GET | Period summary report |
| `/workspaces/:ws_id/reports/export` | POST | Async Excel export job |
| `/workspaces/:ws_id/reports/export/:job_id` | GET | Export job status |

### Payroll
| Endpoint | Method | Description |
|---|---|---|
| `/workspaces/:ws_id/payroll/calculate` | POST | Batch payslip calculation |
| `/workspaces/:ws_id/payroll/payslips` | GET | List payslips |

## Business Rules

1. **Work Unit Rounding**: All work time is rounded to workspace-configured `work_unit_minutes` (e.g., 30 min units)
2. **Holiday Status**: `holiday` (no punch on holiday) vs `holiday_work` (punched on holiday)
3. **OT Multiplier**: Holiday OT uses per-holiday or workspace-default multiplier (default 300%)
4. **Incentive Eligibility**: Late/early counts on holidays are excluded from diligence bonus evaluation
5. **Leave Accrual**: Monthly/quarterly/yearly accrual with seniority bonuses and carry-over limits
6. **Shift Resolution Priority**: Roster → Pattern → Assignment Rule → Department Pattern → Fixed Schedule
