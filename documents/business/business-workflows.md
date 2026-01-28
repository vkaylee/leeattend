# LeeAttend Business Workflows & Processes

**Version**: 1.0
**Last Updated**: 2026-01-28
**Major Workflows**: 5 documented

---

## Overview

LeeAttend's business processes span from device-level data collection through final payroll payment. Each workflow includes decision points, error handling, and audit trail requirements.

---

## Workflow 1: Device-to-Cloud Sync Pipeline

### Purpose
Collect attendance data from biometric devices at remote sites, cache locally during outages, and synchronize to central server.

### Participants
- **Edge Agent** (Rust orchestrator)
- **Device Worker** (Python sidecar)
- **Biometric Device** (ZKTeco/Hikvision/Suprema)
- **Central API Server**

### Process Flow

#### Phase 1: Edge Agent Polling

```
[Edge Agent] → [Semaphore Acquire]
                    ↓
            [Start Fair Round-Robin]
                    ↓
       [For Each Device in Rotation]
                    ↓
        [Spawn Brand-Specific Worker]
                    ↓
     [Worker Connects to Device]
                    ↓
        [Worker Calls device.get_logs()]
                    ↓
         [Device Returns Punch Records]
                    ↓
       [Worker Returns JSON via RPC]
                    ↓
         [Agent Receives Log Array]
                    ↓
            [Semaphore Release]
```

**Key Business Rules**:
- BR-01: Fair round-robin ensures no device starvation
- BR-02: Semaphore limits concurrent workers (max configurable)
- BR-03: Worker crashes don't cascade (isolated processes)
- BR-04: Retry on worker failure with exponential backoff (1s-60s)

#### Phase 2: Local Persistence

```
[Agent Receives Logs]
        ↓
[Validate Format]
        ↓
[Buffer Batch (100 records or 200ms)]
        ↓
[Single SQLite Transaction]
        ↓
[INSERT INTO attendance_logs (is_synced=0)]
        ↓
[Commit Transaction]
        ↓
[FIFO Order Guaranteed (ORDER BY id ASC)]
```

**Key Business Rules**:
- BR-05: Batch size of 100 records or 200ms timeout (balances latency vs throughput)
- BR-06: Single transaction prevents partial writes
- BR-07: is_synced=0 flags unsynchronized records
- BR-08: FIFO ordering maintained for deterministic processing

#### Phase 3: Cloud Synchronization

```
[Background Sync Task (Every 5-10s)]
        ↓
[Query WHERE is_synced=0]
        ↓
[Batch Records (max 1000)]
        ↓
[Open WebSocket Connection]
        ↓
[Send {type: "logs", batch: [...]}]
        ↓
[Wait for {type: "ack", ids: [...]}]
        ↓
[Server ACKs Successfully Received IDs]
        ↓
[UPDATE attendance_logs SET is_synced=1]
        ↓
[Only on ACK (Prevents Double-Sync)]
```

**Key Business Rules**:
- BR-09: ACK-based sync prevents data loss
- BR-10: Retry logic with exponential backoff (up to 60s)
- BR-11: Stable connection for 30s resets retry counter
- BR-12: Local cache survives network outages indefinitely

#### Phase 4: Central Server Processing

```
[Server Receives Log Batch via WebSocket]
        ↓
[Start Transaction]
        ↓
[For Each Log in Batch]
        ├─→ [Lookup Employee by Device User Mapping]
        ├─→ [INSERT INTO raw_attendance_logs]
        │       (immutable audit record)
        └─→ [Trigger Pairing Service]
        ↓
[Commit Transaction]
        ↓
[Send ACK {ids: [1,2,3...]}]
        ↓
[Agent Marks Synced]
```

**Key Business Rules**:
- BR-13: Raw logs immutable (never update/delete)
- BR-14: Each log creates one immutable record
- BR-15: Pairing triggered immediately on new log arrival

### Pairing Engine (Sub-Process)

```
[New Raw Log Received]
        ↓
[Deduplication Check]
(exact timestamp match = skip)
        ↓
[Fetch Workspace Timezone & Algorithm]
        ↓
[Convert Device Timestamp to Local Date]
        ↓
[Determine Processing Windows]
├─→ [Primary: Current date]
└─→ [Lookback: Previous date if punch before 12pm]
        ↓
[Delete Old time_entries for Window]
        ↓
[Fetch Effective Shift for Each Date]
(Roster > Pattern > Assignment Rule precedence)
        ↓
[Run Selected Pairing Algorithm]
        ├─→ [Anchor-First: First In + Last Out within 16h]
        └─→ [FSM: Explicit punch status from device]
        ↓
[Insert New time_entries]
        ↓
[Store Generation Metadata]
(source log IDs, algorithm, timestamp)
```

**Key Business Rules**:
- BR-16: Duplicate detection by exact timestamp
- BR-17: 16-hour session window prevents multi-day sessions
- BR-18: Overnight shift detected if punch before 12pm (lookback)
- BR-19: Previous day reprocessed (allows accurate night shift pairing)
- BR-20: Generation metadata immutable (audit trail of calculation)

### Error Handling

| Scenario | Action | Recovery |
|----------|--------|----------|
| Device offline | Retry with backoff | Resume on reconnect |
| Network outage | Store locally | Auto-sync when connected |
| Parse error | Log to parsing_errors table | Manual correction by admin |
| Duplicate punch | Skip (dedup) | No action needed |
| Worker crash | Spawn new worker | No data loss (local cache) |
| Database error | Rollback transaction | Retry on next cycle |

### Success Criteria

- [ ] 100% of punch records received from device
- [ ] No data loss during network outages
- [ ] Sync latency <10 seconds (P95)
- [ ] Pairing accuracy >95% without manual correction
- [ ] Audit trail complete (raw logs preserved forever)

---

## Workflow 2: Shift Assignment Process

### Purpose
Assign employees to work shifts using patterns, rosters, and automated rules.

### Participants
- **HR Admin**
- **Admin Dashboard**
- **Assignment Rules Engine**
- **Shift Resolution Service**

### Process Flow

#### Phase 1: Assignment Rule Creation

```
[HR Admin Opens Assignment Rules]
        ↓
[Create New Rule]
├─→ [Set Name: "NightShift"]
├─→ [Set Pattern: Night Pattern (8pm-4am)]
├─→ [Define Conditions]
│   ├─→ [Department: Security]
│   ├─→ [Tags: ["night_shift", "external"]]
│   └─→ [Employee IDs: Optional specific list]
└─→ [Set Priority: 100 (higher = first evaluated)]
        ↓
[Save Rule to Database]
        ↓
[Dashboard Confirms Save]
```

**Key Business Rules**:
- BR-21: Rules can target department, tags, or specific employee IDs
- BR-22: Multiple conditions combined as AND
- BR-23: Priority determines evaluation order

#### Phase 2: Rule Evaluation (Manual or Scheduled)

```
[Trigger: Admin clicks "Apply Rules" or Scheduled Task]
        ↓
[Fetch All Active Rules]
        ↓
[Sort by Priority DESC]
        ↓
[For Each Rule in Priority Order]
├─→ [Find Employees Matching Conditions]
│   ├─→ [Department Match]
│   ├─→ [Tag Match (JSONB)]
│   └─→ [Employee ID Match]
├─→ [For Each Matching Employee]
│   ├─→ [Check if Already Assigned]
│   ├─→ [Create employee_working_patterns]
│   ├─→ [Set effective_from: Today]
│   ├─→ [Set effective_to: Null (ongoing)]
│   └─→ [Store Trace Hints (why rule matched)]
└─→ [Report: X employees assigned]
        ↓
[Confirm Completion]
```

**Key Business Rules**:
- BR-24: Rules evaluated in priority order (first match wins)
- BR-25: Employee matches multiple rules = higher priority rule applies
- BR-26: Existing assignments can be updated or preserved (configurable)
- BR-27: Trace hints stored for audit (explains assignment rationale)

#### Phase 3: Effective Shift Resolution

```
[When Calculating Attendance for Date]
        ↓
[For Employee & Date]
        ↓
[Check 1: Shift Roster (Date-Specific)]
WHERE employee_id = ? AND assigned_date = ?
├─→ [Found: Use This Shift]
└─→ [Not Found: Continue]
        ↓
[Check 2: Working Pattern Assignment]
WHERE employee_id = ?
AND effective_from <= date
AND (effective_to IS NULL OR effective_to >= date)
├─→ [Found: Get Pattern for Day-of-Week]
│   ├─→ [Pattern maps Mon-Sun to shift IDs]
│   ├─→ [Get Shift for Today's Day-of-Week]
│   └─→ [Use This Shift]
└─→ [Not Found: Continue]
        ↓
[Check 3: Assignment Rules]
WHERE priority DESC (evaluate rules)
├─→ [First Matching Rule's Pattern → Shift]
└─→ [Not Found: Continue]
        ↓
[Result: Shift ID or NULL (Day Off)]
        ↓
[Store Trace Hints (which check matched)]
```

**Key Business Rules**:
- BR-28: Roster overrides pattern for specific date
- BR-29: Pattern used if no roster match
- BR-30: Rules used as fallback
- BR-31: Multiple resolution sources = precedence order matters

### Error Handling

| Scenario | Action | Result |
|----------|--------|--------|
| Employee already has shift | Update effective dates | Old pattern end-dated |
| Invalid pattern reference | Validation error | Admin notified, rollback |
| No matching rule | Employee unassigned | Day appears as off |
| Circular dependency | Prevention check | Admin notified before save |

### Success Criteria

- [ ] Bulk assignment completes in <5 seconds (100+ employees)
- [ ] Trace hints clearly show assignment source
- [ ] Precedence order respected (roster > pattern > rule)
- [ ] Audit trail tracks all changes
- [ ] Dashboard shows assignment history

---

## Workflow 3: Leave Request Workflow

### Purpose
Enable employees to request time off, receive approval, and automatically deduct from leave balance.

### Participants
- **Employee** (via ESS portal)
- **HR Admin** (approval)
- **Leave Management System**
- **Payroll Calculation** (downstream)

### Process Flow

#### Phase 1: Leave Request Submission

```
[Employee Accesses ESS Portal]
        ↓
[Navigates to Leave Requests]
        ↓
[Click "New Request"]
        ↓
[Select Leave Type: "Annual Leave"]
        ↓
[Set Start Date: 2026-02-15]
        ↓
[Set End Date: 2026-02-17]
        ↓
[Enter Reason: "Family vacation"]
        ↓
[System Validates]
├─→ [Check Balance]
│   ├─→ [Calculate Working Days (excl. weekends)]
│   ├─→ [3 days requested]
│   ├─→ [Check Current Balance: 12 days]
│   └─→ [Sufficient: ✓ Proceed]
├─→ [Check Overlaps]
│   ├─→ [Query Approved Requests for Same Dates]
│   ├─→ [Status = approved]
│   └─→ [No Conflicts: ✓ Proceed]
└─→ [Other Validations]
    ├─→ [Leave Type Active: ✓]
    ├─→ [Dates in Future: ✓]
    └─→ [All Pass: ✓]
        ↓
[Create leave_request]
├─→ [status: pending]
├─→ [employee_id: (current user)]
├─→ [leave_type_id: Annual Leave]
├─→ [start_date: 2026-02-15]
├─→ [end_date: 2026-02-17]
└─→ [reason: "Family vacation"]
        ↓
[Submit Button]
        ↓
[Success Message: "Request submitted for approval"]
```

**Key Business Rules**:
- BR-32: Leave cannot exceed available balance
- BR-33: Approved requests block overlapping new requests
- BR-34: Pending requests do not block new requests
- BR-35: Working days calculated (exclude weekends/holidays if configured)

#### Phase 2: Admin Review & Approval

```
[HR Admin Accesses Dashboard]
        ↓
[Navigates to Leave Requests Tab]
        ↓
[Views Pending Requests]
├─→ [Shows: Employee Name, Type, Dates, Reason]
├─→ [Shows: Current Leave Balance]
├─→ [Shows: Team Schedule for Period]
└─→ [Shows: Request Created Date]
        ↓
[Select Request: "John Doe - Annual Leave"]
        ↓
[Click "Review"]
        ↓
[Admin Evaluates]
├─→ [Check: "Is coverage available?" → Yes]
├─→ [Check: "Balance sufficient?" → Yes (12 days available, 3 requested)]
├─→ [Check: "Any prior issues?" → No]
└─→ [Decision: "Approve"]
        ↓
[Select Action: Approve / Reject]
        ↓
[If Approve:]
        ├─→ [Add Optional Note: "Approved - good timing"]
        ├─→ [Set Status: approved]
        ├─→ [Record approved_by_user_id: (admin ID)]
        └─→ [Record approval_date: NOW()]
        ↓
[If Reject:]
        ├─→ [Add Rejection Reason: "Insufficient coverage"]
        ├─→ [Set Status: rejected]
        └─→ [Balance NOT deducted]
        ↓
[Click Submit]
        ↓
[Notification Sent to Employee]
(Email: "Your leave request for 2026-02-15 has been approved")
```

**Key Business Rules**:
- BR-36: Approver recorded for audit trail
- BR-37: Rejection prevents balance deduction
- BR-38: Approval timestamp recorded
- BR-39: Notification sent immediately on decision

#### Phase 3: Automatic Balance Deduction

```
[On Leave Request Status = approved]
        ↓
[Calculate Leave Duration]
├─→ [Start: 2026-02-15]
├─→ [End: 2026-02-17]
├─→ [Working Days: 3 days (Mon-Wed, excl. Sat-Sun)]
└─→ [Duration: 3.0 days]
        ↓
[Get Fiscal Year from Request]
├─→ [Year: 2026]
└─→ [Leave Type: Annual Leave]
        ↓
[Fetch leave_balance for (employee, leave_type, fiscal_year)]
├─→ [Current Balance: 12.0 days]
└─→ [Update Balance]
        ↓
[Create leave_accrual_log]
├─→ [adjustment: -3.0 (negative = consumption)]
├─→ [reason: "Approved leave request (2026-02-15 to 2026-02-17)"]
├─→ [leave_request_id: (reference)]
└─→ [timestamp: NOW()]
        ↓
[Update leave_balance]
├─→ [closing_balance: 12.0 - 3.0 = 9.0 days]
└─→ [updated_at: NOW()]
        ↓
[Transaction Commit]
        ↓
[Payroll System Queried: leave request recorded]
```

**Key Business Rules**:
- BR-40: Deduction happens only on approval (not pending)
- BR-41: Working days counted (weekends excluded)
- BR-42: Accrual log created for every balance change
- BR-43: Balance immutable after deduction (only via adjustments)

### Payroll Impact

```
[During Payroll Calculation for Period 2026-02]
        ↓
[Query: Leave Requests Status = approved]
WHERE start_date >= 2026-02-01 AND end_date <= 2026-02-28
        ↓
[For Each Matching Leave Request]
├─→ [Check Leave Type: is_paid?]
├─→ [If Paid (sick, vacation, etc.)]
│   └─→ [Count as standard working days (no pay deduction)]
├─→ [If Unpaid]
│   └─→ [Deduct from base_pay (prorated)]
└─→ [Aggregate Total Leave Days by Type]
        ↓
[Include in Payslip Calculation]
├─→ [Standard Days: 22 - 3 (leave) = 19 days]
├─→ [Base Pay: Adjusted for 19 days]
└─→ [Payslip Segment: Leave deduction recorded]
```

**Key Business Rules**:
- BR-44: Paid leave counts as working days (no pay impact)
- BR-45: Unpaid leave reduces base pay (prorated calculation)
- BR-46: Leave classification determined by leave_type.is_paid flag

### Error Handling

| Scenario | Action | Result |
|----------|--------|--------|
| Insufficient balance | Block submission | Error message shown |
| Overlapping approved request | Block submission | Show conflicting request |
| Leave type inactive | Block submission | Error: "Type not available" |
| Invalid dates | Block submission | Error: "Invalid date range" |
| Approval but balance change | Deduct available | Use most recent balance |

### Success Criteria

- [ ] Request submission validates balance
- [ ] Approval prevents duplicate overlapping requests
- [ ] Balance automatically deducted on approval
- [ ] Audit trail complete (logs and approver tracked)
- [ ] Payroll correctly handles both paid and unpaid leave

---

## Workflow 4: Payroll Calculation Process

### Purpose
Calculate accurate monthly payroll with prorating for mid-month changes, incentives, and overtime.

### Participants
- **Payroll Manager**
- **Payroll Calculation Service**
- **Policy Resolution Engine**
- **Overtime Service**
- **Incentive Evaluation Service**

### Process Flow

#### Phase 1: Change Point Detection

```
[Payroll Manager Triggers Calculation]
├─→ [Employee: John Doe]
├─→ [Period: February 2026]
└─→ [Click: Calculate Payroll]
        ↓
[System Scans Period for Changes]
        ↓
[Find Salary Component Changes]
├─→ [Query employee_salary_components]
│   WHERE effective_date BETWEEN 2026-02-01 AND 2026-02-28
├─→ [Result: No changes for John]
└─→ [Skip segmentation for salary]
        ↓
[Find Department Transfers]
├─→ [Query employee_department_history]
│   WHERE change_date BETWEEN 2026-02-01 AND 2026-02-28
├─→ [Result: Transferred from Sales → Marketing on 2026-02-15]
└─→ [Department Change Point: 2026-02-15]
        ↓
[Find Payroll Config Changes]
├─→ [Query employee_payroll_configs]
│   WHERE effective_date BETWEEN 2026-02-01 AND 2026-02-28
├─→ [Result: New OT rate effective 2026-02-20]
└─→ [Config Change Point: 2026-02-20]
        ↓
[Consolidate Change Points]
└─→ [Segments: 2026-02-01 to 2026-02-14, 2026-02-15 to 2026-02-19, 2026-02-20 to 2026-02-28]
```

**Key Business Rules**:
- BR-47: All changes detected automatically (salary, transfer, policy)
- BR-48: Multiple changes on same date = single segment
- BR-49: Segments ordered chronologically

#### Phase 2: Per-Segment Calculation

```
[For Each Segment]
        ↓
[SEGMENT 1: 2026-02-01 to 2026-02-14]
        ↓
[Step 1: Policy Resolution]
├─→ [Check employee_payroll_configs]
│   WHERE employee_id = John
│   AND effective_date <= 2026-02-01
│   ORDER BY effective_date DESC LIMIT 1
│   → Result: None (no employee override)
├─→ [Check department_payroll_configs]
│   WHERE department_id = Sales (from history on 2026-02-01)
│   AND effective_date <= 2026-02-01
│   ORDER BY effective_date DESC LIMIT 1
│   → Result: Sales uses 20 standard days/month
├─→ [Check payroll_settings]
│   WHERE workspace_id = workspace
│   AND effective_date <= 2026-02-01
│   ORDER BY effective_date DESC LIMIT 1
│   → Result: Global default 26 days/month, 8 hours/day
        ↓
└─→ [Resolution: Use Sales Config (20 days, 8 hours, OT multiplier 1.5)]
        ↓
[Step 2: Rate Derivation]
├─→ [Fetch employee_salary_components (effective 2026-02-01)]
│   ├─→ [Basic Salary: 20,000,000 VND]
│   ├─→ [Housing: 2,000,000 VND (is_base_for_ot: true)]
│   ├─→ [Meal: 500,000 VND (is_base_for_ot: false)]
│   └─→ [Total Base for OT: 22,000,000 VND]
├─→ [Calculate Hourly Rate]
│   └─→ [22,000,000 / (20 days * 8 hours) = 137,500 VND/hour]
└─→ [Store for Later Use]
        ↓
[Step 3: Attendance Aggregation]
├─→ [Fetch time_entries for segment (2026-02-01 to 2026-02-14)]
│   ├─→ [9 full days worked: 72 hours standard]
│   ├─→ [2 half days: 8 hours standard]
│   ├─→ [5 hours overtime]
│   ├─→ [6 hours breaks (deducted)]
│   └─→ [Total: 79 hours standard + 5 hours OT]
└─→ [Classify by type: standard, OT, break]
        ↓
[Step 4: Financial Computation]
├─→ [Standard Pay: 79 hours * 137,500 VND = 10,862,500 VND]
├─→ [OT Pay: 5 hours * 137,500 * 1.5 = 1,031,250 VND]
├─→ [Break Deduction: 6 * minimum_break_rate (if applicable)]
├─→ [Leave Impact: None for segment 1]
└─→ [Segment Subtotal: 10,862,500 + 1,031,250 = 11,893,750 VND]
        ↓
[Step 5: Persist Segment]
├─→ [INSERT payslip_segment]
│   ├─→ [segment_start: 2026-02-01]
│   ├─→ [segment_end: 2026-02-14]
│   ├─→ [base_salary: 22,000,000]
│   ├─→ [applied_hours_per_day: 8]
│   ├─→ [standard_days: 20]
│   ├─→ [standard_pay: 10,862,500]
│   ├─→ [overtime_pay: 1,031,250]
│   ├─→ [leave_deduction: 0]
│   └─→ [segment_subtotal: 11,893,750]
└─→ [Immutable snapshot stored]
        ↓
[SEGMENT 2: 2026-02-15 to 2026-02-19]
(Similar process with new department config)
        ↓
[SEGMENT 3: 2026-02-20 to 2026-02-28]
(Similar process with new OT rate)
```

**Key Business Rules**:
- BR-50: Policies in effect during segment used (point-in-time lookup)
- BR-51: Salary components identified for OT calculation (is_base_for_ot = true)
- BR-52: Prorating exact to the hour and minute
- BR-53: Segments immutable (audit trail)

#### Phase 3: Incentive Evaluation

```
[After All Segments Calculated]
        ↓
[Aggregate Context for Period]
├─→ [Total Days Worked: 20 days]
├─→ [Total Days Late/Early: 3 instances]
├─→ [Unpaid Leave Days: 0]
├─→ [Sick Leave Days: 1 day]
└─→ [Other Metrics: TBD]
        ↓
[Fetch Incentive Policies (workspace)]
├─→ [Active policies ordered by ID]
├─→ [Policy 1: Diligence Bonus]
│   ├─→ [Criteria: Attendance + behavior]
│   └─→ [Tiers: Bronze, Silver, Gold]
└─→ [Policy 2: Safety Bonus (Phase 3)]
        ↓
[Evaluate Diligence Bonus Tiers]
├─→ [Tier 1 (Gold): max_late_count=0, max_unpaid_leave=0]
│   └─→ [John: 3 lates + 1 unpaid = No match]
├─→ [Tier 2 (Silver): max_late_count=2, max_unpaid_leave=1]
│   └─→ [John: 3 lates = No match]
├─→ [Tier 3 (Bronze): max_late_count=5, max_unpaid_leave=2]
│   └─→ [John: 3 lates, 0 unpaid = Match! ✓]
└─→ [First Matching Tier: Bronze (payout = 500,000 VND)]
        ↓
[Create payslip_incentive]
├─→ [incentive_policy_id: Diligence]
├─→ [tier_id: Bronze]
├─→ [awarded_amount: 500,000 VND]
└─→ [awarded_date: NOW()]
        ↓
[Evaluate Other Policies (if any match)]
```

**Key Business Rules**:
- BR-54: Tiers evaluated in order (first match wins = generosity principle)
- BR-55: Incentive awarded only if all thresholds met
- BR-56: Separate incentive record per policy per period

#### Phase 4: Final Aggregation

```
[Sum All Segments]
├─→ [Segment 1 Subtotal: 11,893,750 VND]
├─→ [Segment 2 Subtotal: 10,500,000 VND]
├─→ [Segment 3 Subtotal: 9,200,000 VND]
└─→ [Subtotal Before Incentives: 31,593,750 VND]
        ↓
[Add All Incentives]
├─→ [Diligence Bonus: +500,000 VND]
└─→ [Total Incentives: +500,000 VND]
        ↓
[Apply Deductions (Phase 3+)]
├─→ [Income Tax: -Configurable]
├─→ [Social Insurance: -Configurable]
└─→ [Total Deductions: -XXX VND]
        ↓
[Calculate Net Pay]
└─→ [Net = Subtotal + Incentives - Deductions]
└─→ [Net Pay: 32,093,750 VND]
        ↓
[Create/Update Payslip (Master Record)]
├─→ [employee_id: John]
├─→ [payroll_period_start: 2026-02-01]
├─→ [payroll_period_end: 2026-02-28]
├─→ [gross_salary: 32,093,750]
├─→ [net_salary: 32,093,750] (if no tax/insurance)
└─→ [status: draft] (Admin reviews before publishing)
        ↓
[Admin Reviews Payslip]
├─→ [Check calculations are reasonable]
├─→ [Verify no data entry errors]
└─→ [If OK: Click Publish]
        ↓
[Payslip Status: published]
├─→ [Sent to Employee via ESS]
├─→ [Archived for compliance]
└─→ [Immutable (no further changes)]
```

**Key Business Rules**:
- BR-57: Segments summed exactly (no rounding until final)
- BR-58: Payslip in draft status initially (admin review gate)
- BR-59: Published payslips immutable (compliance)
- BR-60: Calculation inputs preserved in segments (audit trail)

### Error Handling

| Scenario | Action | Recovery |
|----------|--------|----------|
| Missing salary component | Error dialog | Admin adds component, recalculate |
| Invalid effective date | Validation error | Admin corrects date |
| Employee with no time entries | Warning | Allow zero-pay payslip or exclude |
| Overlapping salary components | Use highest priority | Clear warning to admin |
| Negative final pay | Error | Admin reviews and adjusts |

### Success Criteria

- [ ] Calculation completes in <2 minutes for 100 employees
- [ ] Segment-based prorating accurate to the hour
- [ ] Change points detected automatically
- [ ] Payslip review prevents publishing errors
- [ ] Audit trail complete (segments preserved)

---

## Workflow 5: Employee Self-Service Access

### Purpose
Enable employees to view and manage their personal attendance, leave, and payroll data without HR intervention.

### Participants
- **Employee**
- **ESS Portal** (web interface)
- **API Server**
- **Personal Data Service**

### Process Flow

#### Phase 1: Identity Linking

```
[User Creates Account]
├─→ [Email: john.doe@company.com]
├─→ [Password: xxxxxxxx]
└─→ [Confirms Email]
        ↓
[HR Admin Links User to Employee]
├─→ [Search Employee Database]
├─→ [Find: John Doe (Employee ID: EMP0015)]
├─→ [Create Link: user_id ↔ employee_id]
└─→ [Employee Now Has ESS Access]
        ↓
[Employee Logs In with Email/Password]
        ↓
[System Issues JWT with employee_id in Claims]
        ↓
[All ESS Queries Automatically Scoped to Own Employee Record]
(WHERE employee_id = ? implicit in all API calls)
```

**Key Business Rules**:
- BR-61: User-employee link one-to-one (one user = one employee)
- BR-62: ESS APIs auto-scope queries (WHERE employee_id = ?)
- BR-63: No cross-employee data visible (multi-tenancy at employee level)

#### Phase 2: Available Self-Service Features

```
[Employee Dashboard]
        ↓
[Option 1: Profile Management]
├─→ [View: Legal name, ID, hire date, department]
├─→ [Edit: Phone, address, emergency contacts]
├─→ [Update Avatar]
└─→ [Personal details only (not salary/benefits)]
        ↓
[Option 2: Attendance History]
├─→ [View: Raw logs (punch times from devices)]
├─→ [View: Paired sessions (calculated work periods)]
├─→ [Filter: By date range, by device]
├─→ [Export: History as CSV (future)]
└─→ [Automatic: Shows corrections made by admin]
        ↓
[Option 3: Leave Balance]
├─→ [Display: Real-time balance per leave type]
│   ├─→ [Annual Leave: 9/12 days]
│   ├─→ [Sick Leave: 8/10 days]
│   └─→ [Unpaid: Unlimited]
├─→ [Show: Accrual history (logs of changes)]
├─→ [Show: Upcoming carryover (if applicable)]
└─→ [Trend: Visual indicator (declining/increasing)]
        ↓
[Option 4: Leave Request Submission]
├─→ [Click: "New Request"]
├─→ [Select: Leave Type]
├─→ [Enter: Start/End Dates]
├─→ [Enter: Reason]
├─→ [System: Validates balance & conflicts]
├─→ [Submit: Request created with status=pending]
├─→ [Track: View pending/approved history]
└─→ [Notify: Get approval notification]
        ↓
[Option 5: Payslip Access]
├─→ [View: Monthly payslips]
├─→ [Display: Breakdown (base, OT, deductions)]
├─→ [Download: PDF copy]
├─→ [History: All payslips retained]
└─→ [Tax Info: Component breakdown visible]
        ↓
[Option 6: Adjustment Requests] (Phase 2)
├─→ [Request: Attendance correction]
├─→ [Provide: Reason for correction]
├─→ [Track: Approval status]
└─→ [History: Past requests and outcomes]
        ↓
[Option 7: Document Repository]
├─→ [View: Employment contract]
├─→ [View: Offer letter]
├─→ [View: Certificates]
├─→ [Download: Copies as PDF]
└─→ [Upload: None (read-only)]
        ↓
[Option 8: Self-Clock Interface] (Phase 3)
├─→ [Large Punch In/Out Buttons]
├─→ [Session Status: Working / On Break / Off Duty]
├─→ [History: Last 7 days timeline]
├─→ [Real-Time: 30-second auto-refresh]
└─→ [Validation: Geofencing (future)]
```

**Key Business Rules**:
- BR-64: Employees view own data only (no cross-employee access)
- BR-65: Payslips immutable (read-only access)
- BR-66: Leave requests self-submitted (no admin needed)
- BR-67: Attendance corrections require approval (audit)

#### Phase 3: Access Control

```
[Employee Makes API Request]
├─→ [GET /api/ess/attendance/logs]
├─→ [Includes: Authorization: Bearer {JWT}]
└─→ [JWT Contains: employee_id = EMP0015]
        ↓
[API Handler Extracts employee_id from JWT]
        ↓
[Repository Query]
├─→ [Query: SELECT * FROM time_entries]
├─→ [Filter: WHERE employee_id = EMP0015]
└─→ [Only Own Records Returned]
        ↓
[Response: Attendance data for EMP0015 only]
```

**Key Business Rules**:
- BR-68: Employee ID extracted from JWT (secure, user cannot override)
- BR-69: Query-level filtering (WHERE employee_id = ?) mandatory
- BR-70: No cross-tenant or cross-employee data leakage possible

### Error Handling

| Scenario | Action | Result |
|----------|--------|--------|
| Employee not linked to user | Block access | Redirect to admin |
| Invalid JWT | Reject request | Redirect to login |
| Employee separated | Disable login | HR must unlink account |
| Leave balance negative | Prevent request | Show error message |
| Payslip not published | Hide draft | Only show published |

### Success Criteria

- [ ] Employees submit 70%+ of leave requests via ESS
- [ ] 60%+ ESS adoption within 3 months
- [ ] Zero data leaks between employees
- [ ] Support tickets reduced by 40% (self-service adoption)
- [ ] Payslip downloads >80% online access rate

---

## Business Rules Summary

### Data Integrity (Critical)

| Rule | Impact | Enforcement |
|------|--------|------------|
| BR-01 thru BR-20 | Attendance collection | SQLite transactions, ACK-based sync |
| BR-21 thru BR-31 | Shift assignment | Precedence rules, trace hints |
| BR-32 thru BR-46 | Leave management | Balance validation, approval gates |
| BR-47 thru BR-70 | Payroll & ESS | Immutable segments, query-level filtering |

### Success Metrics

- **Attendance**: 100% collection, <10s sync latency, >95% pairing accuracy
- **Scheduling**: <5s bulk assignment, transparent trace hints
- **Leave**: <24h approval, 100% balance accuracy
- **Payroll**: <2min calculation, exact prorating, 99.9% accuracy
- **ESS**: 60%+ adoption, 70%+ self-service usage

---

**Document Version**: Final for Phase 1
**Next Update**: Post-Phase 2 completion (Q3 2026)
