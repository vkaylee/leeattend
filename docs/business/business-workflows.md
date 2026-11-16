# LeeAttend — Business Workflows

## WF1: Attendance Data Flow

```text
[ZKTeco device]
        │ TCP
        ▼
[Attendance source connector]
        │ HTTP
        ▼
[Go apiserver]
        │
        ▼
[Raw attendance logs]
        │
        ▼
[Punch pairing and policy resolution]
        │
        ▼
[Time entries]
        │
        ▼
[Daily attendance summaries]
```

**Trigger**: Attendance-source sync (configurable interval)
**Output**: Daily attendance summary per employee

---

## WF2: Daily Report Generation

1. Admin selects date range and optional filters (department, employee)
2. `report_service.get_daily_attendance_report()` executes:
   - Fetches organization schedules, shifts, and employees
   - Fetches holidays for date range (batch lookup)
   - For each employee × date:
     - Resolves effective shift (roster → pattern → rule → department)
     - Finds time entries for the date
     - Calculates work units and OT
     - Determines status: `present`, `late`, `absent`, `off`, `holiday`, `holiday_work`
3. Returns paginated report with per-row data

---

## WF3: Excel Export

1. Admin clicks "Export" → POST `/reports/export`
2. Server creates async job → returns `job_id`
3. Frontend opens WebSocket to `/reports/ws/:job_id`
4. Server generates Excel file:
   - Sheet 1: **Timesheet View** — Calendar matrix with shift symbols, holiday highlighting
   - Sheet 2: **Raw Details** — Per-day breakdown with columns for punch times, status, OT, holiday
5. Server streams progress via WebSocket (0-100%)
6. On completion, frontend downloads file

---

## WF4: Payroll Calculation

```
┌─────────────────────────────────────────────────────────────┐
│  Monthly Payslip Calculation                                 │
│                                                              │
│  1. Fetch salary components for employee                     │
│  2. Fetch attendance summaries for month                     │
│  3. Fetch holidays for month (batch)                         │
│  4. For each working day:                                    │
│     a. Calculate standard work pay                           │
│     b. If has_ot: Calculate OT pay                          │
│        - If holiday: use holiday OT multiplier (300%)       │
│        - Else: use standard OT rules                        │
│  5. Calculate incentive/diligence bonus:                     │
│     a. Count late/early arrivals                            │
│     b. EXCLUDE holidays from count                          │
│     c. Match to incentive tier                              │
│  6. Generate payslip with segments                          │
└─────────────────────────────────────────────────────────────┘
```

---

## WF5: Leave Request Workflow

```
Employee submits request → Pending
       │
       ▼
Manager/Admin reviews
       │
  ┌────┴────┐
  ▼         ▼
Approved  Rejected
  │
  ▼
Balance deducted
```

---

## WF6: Holiday Management

1. Admin navigates to Settings → Holidays
2. Creates holiday with name, date, type, OT multiplier
3. System validates: unique date per organization
4. On create/update/delete:
   - Recalculates attendance summaries for affected date(s)
   - Updates `is_holiday` flag in daily summaries
   - Recalculates payroll if within a processed period

---

## WF7: Shift Assignment Resolution

Priority chain for determining an employee's shift on a given date:

```
1. Shift Roster (direct per-employee per-date assignment)
   ↓ not found
2. Working Pattern (weekly template assigned to employee)
   ↓ not found
3. Assignment Rule (rule-based matching by department/criteria)
   ↓ not found
4. Department Default Pattern
   ↓ not found
5. No shift (status = "off")
```

---

## WF8: OT Claim Workflow

```
System detects OT (attendance_summary) → System OT
       │
Employee submits claim → Claimed OT
       │
Manager reviews claim
       │
  ┌────┴────┐
  ▼         ▼
Approved  Rejected
  │
  ▼
Approved OT → used in payroll calculation
```

---

## WF9: Agent Device Sync

1. Agent starts → registers with API server (agent token auth)
2. Agent discovers ZKTeco devices on local network
3. Periodic sync cycle:
   a. Connect to device via TCP approved connector protocol
   b. Fetch attendance logs since last sync
   c. Store in local SQLite (resilience)
   d. POST logs to API server
   e. API server processes: dedup → store → pair → calculate
4. Agent sends heartbeat for monitoring
