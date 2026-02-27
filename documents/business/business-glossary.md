# LeeAttend — Business Glossary

## Core Terms

| Term | Definition |
|---|---|
| **Workspace** | An isolated tenant/organization unit. All data is scoped to a workspace. Equivalent to a company or branch. |
| **Employee** | A staff member tracked in the attendance system. May or may not have a user account for self-service. |
| **Agent** | An on-premise Rust application that communicates with ZKTeco devices and syncs data to the central API server. |
| **Device** | A ZKTeco biometric attendance machine (fingerprint, face recognition, or card) used for punch-in/punch-out. |

## Attendance Terms

| Term | Definition |
|---|---|
| **Punch In / Punch Out** | A single biometric event recorded by a device. Raw data before pairing. |
| **Time Entry** | A paired punch-in and punch-out record representing a complete work session. |
| **Pairing** | The process of matching raw punch events into in/out pairs. Multiple algorithms available (greedy, optimal). |
| **Raw Attendance Log** | Unprocessed device data including timestamp, device ID, and user identifier. |
| **Attendance Daily Summary** | An aggregated daily record per employee containing actual work minutes, OT minutes, and status flags. |

## Scheduling Terms

| Term | Definition |
|---|---|
| **Shift** | A defined work schedule including start/end times, break policies, and OT rules. Example: "8:00-17:00 with 60min lunch". |
| **Shift Roster** | A direct per-employee, per-date shift assignment. Highest priority in resolution chain. |
| **Working Pattern** | A weekly template (Mon-Sun) specifying which shift applies on each day. Can be assigned to employees or departments. |
| **Assignment Rule** | An automated rule that matches employees to shifts based on criteria (department, job title, etc.). |
| **Report Symbol** | A short code displayed in Excel timesheet views (e.g., "S8" for standard 8-hour shift). |

## Time Calculation Terms

| Term | Definition |
|---|---|
| **Work Unit** | The minimum granularity for time calculations. Configured per workspace (e.g., 30 minutes). |
| **Standard Work Units** | Work time within the scheduled shift hours. |
| **OT Work Units** | Work time exceeding the scheduled shift hours. |
| **Rounding Policy** | How work time is rounded to work units (round up, round down, round nearest). |
| **Grace Period** | Minutes before/after scheduled start that are not counted as late/early. |
| **Early Arrival Strategy** | How early punch-ins are handled: `allow` (full credit), `block` (reject), `clamp` (cap at shift start). |

## Holiday Terms

| Term | Definition |
|---|---|
| **Holiday** | A calendar date marked as a non-working day (national, company, or regional). |
| **Holiday Type** | Classification: `national` (government-mandated), `company` (organization-specific), `regional` (location-specific). |
| **OT Multiplier** | The pay rate multiplier for working on a holiday. Default 3.0 (300%) per Vietnamese labor law. |
| **Holiday Status** | `holiday` = no punch on a holiday; `holiday_work` = employee worked on a holiday. |
| **Holiday Overnight Strategy** | How shifts crossing midnight are handled: split at midnight, by punch-in date, or by majority hours. |

## Leave Terms

| Term | Definition |
|---|---|
| **Leave Type** | A category of leave (annual, sick, unpaid, maternity, etc.). |
| **Leave Policy** | Rules governing a leave type: accrual frequency, amount, carry-over limit, seniority bonus. |
| **Accrual** | Periodic credit of leave days to employee balance (monthly, quarterly, yearly). |
| **Carry-Over** | Leave balance that transfers to the next period, subject to configurable limits. |
| **Seniority Bonus** | Additional leave days granted based on years of service. |

## Payroll Terms

| Term | Definition |
|---|---|
| **Payslip** | A monthly payroll calculation result for one employee, comprising multiple segments. |
| **Payslip Segment** | One component of a payslip (base salary, OT pay, incentive bonus, deduction). |
| **Salary Component** | A definition of a pay element (fixed amount, hourly rate, or percentage). |
| **Incentive Tier** | A bonus level determined by attendance compliance (e.g., 0 late days = 150k bonus). |
| **Diligence Bonus** | An incentive payment for consistent attendance, evaluated by late/early arrival count. |
| **Late/Early Count** | Number of days an employee arrived late or left early. Holidays are excluded from this count. |

## Technical Terms

| Term | Definition |
|---|---|
| **sqlx** | Rust SQL toolkit with compile-time query verification against a live PostgreSQL database. |
| **Axum** | Async Rust web framework used for the API server. |
| **rustzk** | Rust library for ZKTeco device communication (TCP/UDP protocol). |
| **Moka** | In-memory caching library for Rust (async-compatible). |
| **Tauri** | Framework for building cross-platform desktop apps with web technologies (used for the web dashboard). |
| **run-tests.sh** | The unified test orchestrator that runs all tests inside Docker containers. Required on WSL2 environments. |

## Status Values

| Status | Context | Meaning |
|---|---|---|
| `present` | Daily report | Employee attended and punched on time |
| `late` | Daily report | Employee attended but arrived late |
| `absent` | Daily report | Employee did not attend a scheduled day |
| `off` | Daily report | No shift scheduled for the day |
| `holiday` | Daily report | Holiday and employee did not work |
| `holiday_work` | Daily report | Holiday but employee worked |
| `pending` | Leave request | Request submitted, awaiting approval |
| `approved` | Leave request / OT claim | Approved by manager/admin |
| `rejected` | Leave request / OT claim | Rejected by manager/admin |
