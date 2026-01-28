# LeeAttend Business Glossary & Domain Terminology

**Version**: 1.0
**Last Updated**: 2026-01-28
**Total Terms**: 100+

---

## Overview

This glossary defines business terms, domain concepts, and their relationships in LeeAttend. Terms are organized by domain area with technical mappings for developer reference.

---

## A

### Accrual
**Definition**: The process of accumulating entitlements (e.g., leave days) over time according to a policy.

**Context**: Employees accrue annual leave at a rate of 1 day per month.

**Related Terms**: Leave Balance, Leave Policy, Leave Type

**Database**: `leave_balances.accrued` (column)

---

### Adjustment Request
**Definition**: An employee-initiated request to correct their attendance punch times, requiring managerial approval.

**Example**: Employee requests correction of missed check-out after leaving early.

**Workflow**: Pending → Approved/Rejected

**Related Terms**: Time Entry, Raw Log, Audit Trail

**Database**: `attendance_adjustment_requests` (table)

---

### Agent (Edge Agent)
**Definition**: A software component deployed on-premise at a customer site to collect attendance data from biometric devices and cache it locally.

**Architecture**: Orchestrator (Rust) + Brand-Specific Workers (Python sidecars)

**Responsibilities**: Device polling, local persistence, sync to central server

**Related Terms**: Device, Sidecar, Central Server

**Database**: `authorized_agents` (table)

---

### Anchor-First Algorithm
**Definition**: Pairing algorithm that matches the first check-in punch with the last check-out punch within a 16-hour window.

**Advantage**: Simple, prevents multi-day sessions, clear audit trail

**Limitation**: Assumes single work period per day

**Related Terms**: Pairing, FSM Algorithm, Time Entry

**Configuration**: `workspace.pairing_algorithm` setting

---

### Assignment Rule
**Definition**: Automated logic to assign employees to working patterns based on conditions (department, tags, employee IDs).

**Example**: "All employees with tag 'NightShift' in Security department → Night Working Pattern"

**Precedence**: Priority-based (higher priority evaluated first)

**Related Terms**: Working Pattern, Shift Roster, Trace Hints

**Database**: `assignment_rules` (table)

---

### Audit Trail
**Definition**: Immutable record of all sensitive system actions for compliance and dispute resolution.

**Includes**: Raw attendance logs, policy changes, adjustments, approvals

**Retention**: Typically indefinite for legal/regulatory compliance

**Related Terms**: Raw Log, Immutability, Compliance

**Database**: Multiple tables with `created_at`, `updated_at`, audit logs

---

## B

### Biometric Device
**Definition**: Physical hardware (fingerprint, face, RFID reader) that records employee attendance by capturing punch events.

**Brands**: ZKTeco, Hikvision, Suprema, ANVIZ

**Data**: Punch time, user ID, device status

**Related Terms**: Device, Raw Log, Sidecar

**Database**: `devices` (table)

---

### Break
**Definition**: Unpaid or paid time off during a work shift (e.g., lunch break, prayer break).

**Configuration**: Break start/end times defined in shift

**Impact**: Deducted from worked hours (except if overtime-eligible)

**Related Terms**: Shift, Time Entry, Overtime

**Calculation**: Duration = shift_end - shift_start - break_duration

---

### Break Deduction
**Definition**: Reduction in worked hours due to configured break periods.

**Example**: 8-hour shift with 1-hour lunch → 7 hours paid work

**Flexibility**: Can be marked as overtime-eligible

**Related Terms**: Break, Time Entry, Payroll Calculation

**Database**: `shifts.break_duration_minutes` (column)

---

## C

### Carry Forward (Carryover)
**Definition**: Leave days unused in one fiscal year that roll over to the next year.

**Rules**: Limited by policy (e.g., max 5 days carry forward)

**Example**: 2 unused annual leave days from 2025 → available in 2026

**Related Terms**: Leave Type, Leave Balance, Leave Policy

**Database**: `leave_balances.carry_forward` (column)

---

### Central Server
**Definition**: Cloud-based API server that processes attendance data, calculates payroll, and manages multi-tenant workspaces.

**Stack**: Rust + Axum + PostgreSQL

**Endpoints**: REST API + WebSocket for agent sync

**Related Terms**: Agent, API, Workspace

**Location**: `/home/elt1541/leeattend/apiserver/`

---

### Change Point
**Definition**: Event within a payroll period that requires segmentation (salary change, department transfer, policy override).

**Detection**: Automatic scan of configuration history

**Impact**: Period split into segments with different rates

**Related Terms**: Payslip Segment, Prorating, Payroll Calculation

**Example**: Employee transferred on mid-month = 2 segments

---

### Clock In / Clock Out
**Definition**: The action of recording arrival (clock in) or departure (clock out) via biometric device or manual entry.

**Frequency**: Multiple times per day (work + breaks)

**Data**: Timestamp, device/source, employee, status

**Related Terms**: Raw Log, Punch, Check-In/Check-Out

**Database**: `raw_attendance_logs.log_type` (enum)

---

## D

### Department
**Definition**: Organizational unit within a workspace (e.g., Sales, HR, Finance).

**Hierarchy**: Workspace → Departments → Employees

**Payroll**: Can have department-specific working standards

**Related Terms**: Workspace, Employee, Manager

**Database**: `departments` (table)

---

### Department Payroll Config
**Definition**: Department-level override of payroll policies (working days, OT rates, bonuses).

**Precedence**: Overrides workspace settings, overridden by employee settings

**Temporal**: Effective dates for historical accuracy

**Related Terms**: Payroll Settings, Employee Payroll Config, 3-Tier Hierarchy

**Database**: `department_payroll_configs` (table)

---

### Device
**Definition**: Registered biometric hardware in a workspace with configuration (IP, port, brand).

**Attributes**: Name, location, timezone, enabled/disabled status

**Owner**: Managed by one authorized agent

**Related Terms**: Biometric Device, Authorized Agent, Device Brand

**Database**: `devices` (table)

---

### Device Brand
**Definition**: Manufacturer of biometric devices (ZKTeco, Hikvision, Suprema, ANVIZ, Mock).

**Sidecar**: Each brand has a Python worker implementing JSON-RPC interface

**Protocol**: Custom binary format → standardized JSON output

**Related Terms**: Sidecar, Device, Worker

**Configuration**: `devices.brand` (enum)

---

### Diligence Bonus
**Definition**: Performance incentive awarded based on attendance and behavior criteria.

**Criteria**: Low late count, low unpaid leave, good punctuality

**Tiers**: Bronze, Silver, Gold with different thresholds

**Related Terms**: Incentive Policy, Tier, Payroll Incentive

**Example**: Employees with <2 late arrivals → Bronze bonus (500K VND)

---

## E

### Early Leave
**Definition**: Permission for employee to leave work earlier than scheduled (e.g., parental leave, breastfeeding).

**Approval**: Often automatic based on working condition

**Duration**: Temporary (with start/end dates)

**Related Terms**: Working Condition, Leave Type, Paid Time Off

**Database**: `working_conditions` (table)

---

### Effective Date
**Definition**: The date on which a policy, configuration, or rule becomes active.

**Usage**: Point-in-time lookups (find policy effective on specific date)

**Range**: effective_from to effective_to (nullable for ongoing)

**Related Terms**: Temporal Data, Point-in-Time Lookup, Policy Version

**Pattern**: `WHERE effective_from <= target_date AND (effective_to IS NULL OR effective_to >= target_date)`

---

### Employee
**Definition**: Person employed by workspace with associated HR data, shifts, leave, payroll records.

**Unique Identifier**: employee_id (custom number like "EMP0015")

**Attributes**: Name, email, phone, hire date, department, status

**Related Terms**: User, Workspace Member, Department

**Database**: `employees` (table)

---

### Employee Payroll Config
**Definition**: Individual employee override of payroll policies (highest precedence).

**Use Case**: Part-time employees, contractors, executives with custom rates

**Temporal**: Effective dates for retroactive corrections

**Related Terms**: Payroll Settings, Department Payroll Config, 3-Tier Hierarchy

**Database**: `employee_payroll_configs` (table)

---

### Employee Self-Service (ESS)
**Definition**: Portal allowing employees to view/manage personal attendance, leave, and payroll data.

**Features**: Leave request submission, payslip access, balance viewing

**Scope**: Personal data only (cannot see colleagues)

**Related Terms**: Portal, Authentication, Identity Linking

**Components**: `/webdashboard/src/pages/ess/` (UI), `/apiserver/src/handlers/ess.rs` (API)

---

### ESS
**See**: Employee Self-Service

---

## F

### Fiscal Year
**Definition**: The 12-month period used for leave entitlement and accrual.

**Typical**: Calendar year (Jan 1 - Dec 31) or financial year

**Usage**: Leave balance tracked separately per fiscal year

**Related Terms**: Leave Balance, Leave Type, Accrual

**Database**: `leave_balances.fiscal_year` (column)

---

### FSM Algorithm
**Definition**: Finite State Machine pairing algorithm using explicit punch status (check-in/check-out) from device.

**Advantage**: Handles complex schedules, accounts for explicit intent

**Requirement**: Device must transmit punch type (0=check-in, 1=check-out)

**Related Terms**: Pairing, Anchor-First Algorithm, Time Entry

**Complexity**: Higher than Anchor-First but more accurate for break scenarios

---

## G

### Generation Metadata
**Definition**: JSONB record stored in time_entry documenting which raw logs were paired and algorithm used.

**Contents**: `{ source_log_ids: [id1, id2, ...], algorithm: "anchor_first", timestamp: ... }`

**Purpose**: Audit trail showing calculation inputs

**Related Terms**: Audit Trail, Time Entry, Pairing

**Database**: `time_entries.generation_metadata` (column)

---

## H

### Hierarchical Policy
**Definition**: Multi-level policy configuration where more specific levels override general ones.

**Levels**: Workspace (base) → Department (override) → Employee (highest)

**Resolution**: Most specific level with effective date wins

**Related Terms**: 3-Tier Hierarchy, Point-in-Time Lookup, Policy

**Patterns**: Leave policies, payroll settings, OT policies

---

## I

### Immutable
**Definition**: Data that cannot be modified or deleted after creation.

**Purpose**: Compliance, audit trail, prevents accidental data loss

**Examples**: Raw logs, published payslips, completed segments

**Related Terms**: Audit Trail, Compliance, Raw Log

**Implementation**: Never issued UPDATE or DELETE on these tables

---

### Immutable Audit Record
**Definition**: System-generated record (raw log, segment, incentive award) that serves as proof of transactions.

**Properties**: Created once, preserved forever, versioned by timestamp

**Compliance**: Required for regulatory compliance and disputes

**Related Terms**: Immutable, Raw Log, Audit Trail

**Database**: `raw_attendance_logs`, `payslip_segments` (immutable tables)

---

### Incentive Policy
**Definition**: Specification for automatic bonus awards based on employee performance criteria.

**Components**: Policy name, criteria (late count, unpaid leave), tiers with thresholds

**Evaluation**: Automatic during payroll calculation

**Related Terms**: Tier, Payroll Incentive, Diligence Bonus

**Database**: `payroll_incentive_policies`, `payroll_incentive_tiers` (tables)

---

### Incentive Tier
**Definition**: Threshold level within incentive policy with specific eligibility criteria and payout.

**Example**: "Silver Tier: max 3 late arrivals, max 1 unpaid day → 750K VND bonus"

**Matching**: First matching tier wins (generosity principle)

**Related Terms**: Incentive Policy, Threshold, Payroll Incentive

**Database**: `payroll_incentive_tiers` (table)

---

### Identity Linking
**Definition**: Association between user account and employee record enabling ESS access.

**Relationship**: One-to-one (user ↔ employee)

**Activation**: HR admin links user to employee record

**Related Terms**: User, Employee, ESS, Authentication

**Database**: `users.id` ↔ `employees.user_id` (foreign key)

---

## L

### Late Arrival
**Definition**: Punch-in time after scheduled shift start.

**Measurement**: Minutes late = punch_in - shift_start

**Impact**: Tracked for incentive thresholds, discipline, reports

**Related Terms**: Punctuality, Shift, Time Entry

**Calculation**: Automatic from time_entry vs shift comparison

---

### Leave
**Definition**: Authorized absence from work, tracked by type (vacation, sick, maternity, etc.).

**Accrual**: Earned over time according to policy

**Approval**: Often requires managerial review

**Related Terms**: Leave Type, Leave Request, Leave Balance

**Database**: `leave_requests`, `leave_balances` (tables)

---

### Leave Accrual Log
**Definition**: Audit record of every leave balance change (addition, deduction).

**Entry**: Created when balance adjusted (approval, adjustment, carryover)

**Content**: Adjustment amount, reason, timestamp, user

**Related Terms**: Leave Balance, Adjustment, Audit Trail

**Database**: `leave_accrual_logs` (table)

---

### Leave Balance
**Definition**: Current available leave days for employee, tracked per leave type and fiscal year.

**Calculation**: Opening + Accrued + Carryover - Consumed = Closing

**Validation**: Prevents requesting more than available

**Related Terms**: Leave Type, Fiscal Year, Accrual

**Database**: `leave_balances` (table)

---

### Leave Policy
**Definition**: Template defining entitlement, accrual rate, carryover limits, and other rules for a leave type.

**Scope**: Assigned to workspace, department, or individual employee

**Hierarchy**: Employee > Department > Workspace

**Related Terms**: Leave Type, Leave Policy Assignment, 3-Tier Hierarchy

**Database**: `leave_policy_assignments`, `leave_policies` (tables)

---

### Leave Policy Assignment
**Definition**: Association of a leave policy template to a scope (workspace/department/employee).

**Temporal**: Effective dates with auto-close of previous assignment

**Lifecycle**: Transactional (new assignment auto-closes old)

**Related Terms**: Leave Policy, Temporal, Effective Date

**Database**: `leave_policy_assignments` (table)

---

### Leave Request
**Definition**: Employee application for time off during specified dates.

**Workflow**: Pending → Approved/Rejected

**Approval**: Manager review required (if policy configured)

**Related Terms**: Leave, Approval Workflow, Leave Balance

**Database**: `leave_requests` (table)

---

### Leave Type
**Definition**: Category of leave (Annual Leave, Sick Leave, Maternity, Unpaid, etc.).

**Attributes**: Name, code, is_paid, requires_approval, annual_entitlement

**Usage**: Leave requests must select a type

**Related Terms**: Leave, Paid Leave, Unpaid Leave

**Database**: `leave_types` (table)

---

## M

### Manager
**Definition**: Employee with supervisory responsibilities, often approves leave requests and attendance corrections.

**Assignment**: Linked to department or direct report employees

**Approval Scope**: Approves actions for assigned employees

**Related Terms**: Department, Employee, RBAC

**Database**: `departments.manager_id` (optional foreign key)

---

### Multi-Tenant
**Definition**: Architecture pattern where single system instance serves multiple independent organizations (workspaces).

**Isolation**: Data strictly separated at database query level

**Enforcement**: WHERE workspace_id = ? on all queries

**Related Terms**: Workspace, Tenant, Isolation

**Technology**: PostgreSQL with workspace_id partitioning

---

### Multi-Tenant Isolation
**Definition**: Guarantee that data from one tenant (workspace) cannot be accessed by another.

**Enforcement**: Query-level filtering (WHERE clause), not just application logic

**Testing**: `schema_isolation_tests.rs` validates all queries

**Related Terms**: Multi-Tenant, Workspace, Security

**Critical**: Zero cross-tenant data leaks allowed

---

## O

### Overnight Shift
**Definition**: Work schedule spanning two calendar days (e.g., 10pm-6am).

**Challenge**: Punch timestamps cross date boundary

**Solution**: Lookback to previous day if punch before 12pm

**Related Terms**: Shift, Time Entry, Pairing

**Pairing**: Automatic recalculation of previous day when late night punch received

---

### Overtime (OT)
**Definition**: Work hours beyond standard daily/weekly limits, compensated at premium rate.

**Rate**: Typically 1.5x base rate (configurable)

**Threshold**: Hours exceeding standard work day (e.g., >8 hours)

**Related Terms**: Time Entry, Payroll Calculation, OT Policy

**Database**: `overtime_policies` (table), `payslip_segments.overtime_pay` (column)

---

### Overtime Policy
**Definition**: Configuration for OT calculation including multiplier, thresholds, caps.

**Scope**: Workspace, department, or employee level

**Temporal**: Effective dates for policy versioning

**Related Terms**: OT, 3-Tier Hierarchy, Policy

**Database**: `overtime_policy_assignments`, `overtime_policies` (tables)

---

## P

### Paid Leave
**Definition**: Leave type where employee receives full or partial salary during absence.

**Examples**: Annual leave, sick leave (paid), maternity leave

**Impact**: Counted as working days in payroll (no deduction)

**Related Terms**: Leave Type, Unpaid Leave, Payroll Impact

**Configuration**: `leave_types.is_paid = true` (column)

---

## Acronyms & Abbreviations

| Acronym | Definition |
|---------|-----------|
| API | Application Programming Interface |
| RBAC | Role-Based Access Control |
| ESS | Employee Self-Service |
| OT | Overtime |
| VND | Vietnamese Dong (currency) |
| RPC | Remote Procedure Call |
| JSON | JavaScript Object Notation |
| WAL | Write-Ahead Logging (SQLite) |
| JWT | JSON Web Token |
| FIFO | First In, First Out |
| HR | Human Resources |
| PDF | Portable Document Format |
| CSV | Comma-Separated Values |
| YTD | Year-to-Date |
| ICP | Ideal Customer Profile |
| SLA | Service Level Agreement |
| P95 | 95th Percentile |
| TTL | Time-to-Live (retention) |

---

## Business Rules Summary

**BR-01 thru BR-70**: See business-workflows.md for complete rule documentation

**Key Enforcement Points**:
- Raw logs immutable (audit compliance)
- Multi-tenant isolation at query level
- Leave balance validated before approval
- Payslip segments immutable (historical accuracy)
- Pairing deterministic (same input = same output)

---

**Document Status**: Final reference guide for Phase 1
**Target Audience**: Business users, product managers, developers needing domain context
**Next Update**: Post-Phase 2 (Q3 2026)
