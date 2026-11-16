# LeeAttend — Domain Knowledge

## Scope

LeeAttend serves one organization per on-premise appliance. Organization scope comes from trusted server bootstrap; clients cannot select tenant or workspace scope.

## Core entities

- **Employee**: Staff member tracked for attendance, employment, leave, and payroll. May have a user account for self-service.
- **Employment**: Effective-dated employment relationship used by attendance and payroll calculations.
- **Attendance source**: Approved source of raw attendance events, including biometric device integrations and imports.
- **Time entry**: Paired punch-in and punch-out record representing one work session.
- **Daily summary**: Per-employee, per-date aggregate used by reports, overtime, and payroll.
- **Shift**: Work schedule with start/end times, breaks, grace periods, and overtime rules.
- **Working pattern**: Reusable schedule template assigned through effective-dated rules.
- **Holiday**: Organization calendar entry with paid status and optional overtime multiplier.
- **Leave type/policy**: Configurable leave category and accrual/carry-over rules.
- **Payslip**: Monthly payroll result with salary, overtime, incentive, and deduction segments.

## Organization settings

- `work_unit_minutes`: calculation granularity.
- `rounding_policy`: work-time rounding behavior.
- `timezone`: organization timezone for calculations.
- `early_arrival_strategy`: `allow`, `block`, or `clamp`.
- `pairing_algorithm`: punch pairing strategy.
- `holiday_overnight_strategy`: handling for shifts crossing midnight.
- `default_holiday_ot_multiplier`: organization default, commonly 3.0 for 300% holiday overtime.

## Data flow

```text
Attendance source → Go apiserver → raw attendance logs
                                 ↓
                         punch pairing and policy resolution
                                 ↓
                         daily attendance summaries
                                 ↓
                    reports, overtime, and payroll calculations
```

## Database boundary

PostgreSQL stores organization, identity, attendance, payroll, file, and audit data. The baseline schema uses `organization_settings`, `roles`, `role_permissions`, and `user_roles`; it does not expose workspace or tenant tables in the active runtime contract.

Repositories use parameterized `pgx` queries. Migrations under `apiserver/migrations/` are schema source of truth.

## API boundary

Canonical public routes use flat `/api/v1/...` paths. See [`../api/api-reference.md`](../api/api-reference.md). Authentication and explicit permission checks apply to protected routes.

## Business rules

1. Work time rounds using organization `work_unit_minutes` and `rounding_policy`.
2. Holiday status distinguishes no work from work performed on a holiday.
3. Holiday overtime uses the holiday multiplier or organization default.
4. Holiday late/early counts do not affect diligence bonus eligibility.
5. Leave accrual supports monthly, quarterly, and yearly schedules with seniority and carry-over rules.
6. Shift resolution follows roster → pattern → assignment rule → department/default schedule.
