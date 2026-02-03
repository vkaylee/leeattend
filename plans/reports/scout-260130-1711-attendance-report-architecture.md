# Scout Report: Attendance Report Service Architecture

## Exploration Scope
- **Target**: Attendance report system architecture and data flow
- **Boundaries**: `apiserver/src/services/`, `apiserver/src/repositories/`
- **Key Files Analyzed**:
  - `services/report_service.rs` (515 lines)
  - `services/attendance_calculator.rs` (186 lines)
  - `services/attendance_summary_service.rs` (344 lines)
  - `repositories/employee_repository.rs` (429 lines)
  - `repositories/roster_repository.rs` (725 lines)
  - `repositories/time_entry_repository.rs` (189 lines)
  - `domain/policy/resolver.rs` (161 lines)

---

## Current Architecture Overview

### Data Flow Diagram
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           ReportService                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│  get_daily_attendance_report()          get_workspace_attendance_summary()  │
│            │                                         │                       │
│            ▼                                         ▼                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  1. Get Workspace (for timezone, rounding_policy, work_unit)        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│            │                                         │                       │
│            ▼                                         ▼                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  2. roster_repository::get_workspace_schedules()                     │   │
│  │     → Returns Vec<EmployeeSchedule> (ALL employees x ALL dates)     │   │
│  │     → Each EmployeeSchedule has Vec<EffectiveSchedule> per day      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│            │                                         │                       │
│            ▼ [N+1 QUERY PROBLEM]                     ▼ [SAME N+1]          │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  3. FOR EACH EMPLOYEE:                                               │   │
│  │     → time_entry_repository::get_employee_time_entries()            │   │
│  │     → PolicyResolver::resolve_attendance_rules() [PER DAY!]         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│            │                                         │                       │
│            ▼                                         ▼                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  4. AttendanceCalculator::calculate_daily_units()                    │   │
│  │     → Pure computation, no DB                                        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│            │                                         │                       │
│            ▼ [MEMORY BOTTLENECK]                     ▼                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  5. IN-MEMORY PAGINATION                                             │   │
│  │     → reports.into_iter().skip(offset).take(limit)                  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Key Functions Analyzed

### 1. ReportService::get_daily_attendance_report()
**Location**: `services/report_service.rs:22-239`

**Signature**:
```rust
pub async fn get_daily_attendance_report(
    pool: &PgPool,
    workspace_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    page: Option<i64>,
    limit: Option<i64>,
    employee_ids: Option<Vec<Uuid>>,
) -> Result<PaginatedResponse<DailyAttendanceReport>, StatusCode>
```

**Process**:
1. Fetch workspace settings (timezone, rounding_policy)
2. Fetch ALL schedules via `roster_repository::get_workspace_schedules()`
3. Loop through each employee:
   - Fetch time entries for the entire date range
   - Loop through each day in employee's schedule
   - Call `PolicyResolver::resolve_attendance_rules()` for EACH day
   - Call `AttendanceCalculator::calculate_daily_units()` for calculation
4. **In-memory pagination**: `skip(offset).take(limit)`

**Critical Issue**: Loads ALL data into memory BEFORE pagination

---

### 2. ReportService::get_workspace_attendance_summary()
**Location**: `services/report_service.rs:251-514`

**Signature**:
```rust
pub async fn get_workspace_attendance_summary(
    pool: &PgPool,
    workspace_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    sort: Option<AttendanceReportSort>,
    page: Option<i64>,
    limit: Option<i64>,
    employee_ids: Option<Vec<Uuid>>,
) -> Result<PaginatedResponse<AttendanceSummary>, StatusCode>
```

**Process**: Nearly identical to daily report but aggregates per employee.

**Note**: Code comment at line 242-248 explicitly states this method uses **same calculation logic** as daily report for consistency.

---

### 3. roster_repository::get_workspace_schedules()
**Location**: `repositories/roster_repository.rs:9-344`

**Signature**:
```rust
pub async fn get_workspace_schedules(
    pool: &PgPool,
    workspace_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    employee_ids: Option<&Vec<Uuid>>,
) -> Result<Vec<EmployeeSchedule>, sqlx::Error>
```

**Process** (10 batch queries + N iterations):
1. Fetch employees (`WHERE workspace_id AND is_active AND optional filter`)
2. Fetch all shifts (for lookup)
3. Fetch rosters in range
4. Fetch fixed assignments (overlapping)
5. Fetch pattern assignments (overlapping)
6. Fetch department patterns & history
7. Fetch department default shifts
8. Fetch dynamic assignment rules
9. Fetch pattern details for rules
10. Fetch employee department history

Then iterates each employee x each date to resolve effective schedule via priority:
- P1: Roster (priority 100)
- P2: Individual Pattern (priority 80)
- P3: Dynamic Rules (evaluated via RuleEvaluator)
- P4: Department Pattern (priority 40)
- P5: Fixed Assignment (priority 20)
- P6: Department Fixed (priority 10)

**Returns**: `Vec<EmployeeSchedule>` where each contains `Vec<EffectiveSchedule>` for all dates

---

### 4. time_entry_repository::get_employee_time_entries()
**Location**: `repositories/time_entry_repository.rs:85-99`

**Signature**:
```rust
pub async fn get_employee_time_entries(
    pool: &PgPool,
    employee_id: Uuid,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<TimeEntry>, sqlx::Error>
```

**Query**: Simple SELECT with employee_id and time range filter.

**Issue**: Called once PER employee in report loop (N+1 pattern)

---

### 5. AttendanceCalculator::calculate_daily_units()
**Location**: `services/attendance_calculator.rs:155-185`

**Signature**:
```rust
pub fn calculate_daily_units<T: TimeZone>(
    punch_in: DateTime<T>,
    punch_out: DateTime<T>,
    shift_start_dt: Option<DateTime<T>>,
    shift_end_dt: Option<DateTime<T>>,
    scheduled_break_minutes: i32,
    actual_break_minutes: i32,
    work_unit: i32,
    rounding_policy: &str,
    rules: &PolicyRules,
) -> (i32, i32)  // (standard_units, ot_units)
```

**Process**:
1. Calls `calculate_buckets()` to get `WorkBuckets { standard_minutes, early_ot_minutes, late_ot_minutes }`
2. Applies rounding via `calculate_units()`
3. Returns tuple of (std_units, ot_units)

**Pure Function**: No DB calls, efficient. Not a bottleneck.

---

### 6. PolicyResolver::resolve_attendance_rules()
**Location**: `domain/policy/resolver.rs:91-137`

**Signature**:
```rust
pub async fn resolve_attendance_rules(
    pool: &PgPool,
    employee_id: Uuid,
    workspace_id: Uuid,
    date: NaiveDate,
) -> Result<PolicyRules, sqlx::Error>
```

**Process**:
1. Get workspace defaults
2. Get employee's working_conditions for date
3. Merge into PolicyRules

**Issue**: Called for EACH day x EACH employee in report loop

---

## Performance Bottlenecks Identified

### 1. N+1 Query Problem (CRITICAL)
**Location**: `report_service.rs:74-82` and `:346-355`

```rust
for emp_sched in schedules {
    let entries = time_entry_repository::get_employee_time_entries(...)  // N queries!
```

**Impact**: For 100 employees, 100 additional queries per report request.

**Solution**: Batch fetch all time entries with single query:
```rust
// Proposed: Get all entries for all employees in one query
pub async fn get_workspace_time_entries(
    pool: &PgPool,
    workspace_id: Uuid,  // Can join through employees table
    employee_ids: &[Uuid],
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<TimeEntry>, sqlx::Error>
```

---

### 2. In-Memory Pagination (CRITICAL)
**Location**: `report_service.rs:217-231` and `:492-506`

```rust
let total = reports.len();
let data = reports.into_iter().skip(offset).take(limit).collect();
```

**Impact**: For 100 employees x 30 days = 3000 records loaded into memory even if client only wants page 1 with 20 items.

**Solution Options**:
- A) Push pagination to database (requires restructuring query)
- B) Use cursor-based pagination
- C) Pre-compute summaries in `attendance_daily_summaries` table (already exists!)

---

### 3. Per-Day Policy Resolution (MODERATE)
**Location**: `report_service.rs:117-127` and `:392-409`

```rust
for daily in emp_sched.schedules {
    let rules = PolicyResolver::resolve_attendance_rules(pool, emp_sched.employee_id, ...).await
```

**Impact**: For 100 employees x 30 days = 3000 PolicyResolver calls

**Solution**: 
- Batch fetch working_conditions once per employee (they rarely change daily)
- Cache workspace settings at start of function (already done)

---

### 4. roster_repository::get_workspace_schedules() Complexity (MODERATE)
**Location**: `repositories/roster_repository.rs:9-344`

**Impact**: 10 SQL queries + O(employees x dates) iteration for priority resolution

**Note**: Already reasonably optimized with batch queries. Main issue is the large result set returned.

---

## Data Models

### DailyAttendanceReport
```rust
pub struct DailyAttendanceReport {
    pub date: NaiveDate,
    pub employee_id: Uuid,
    pub employee_name: String,
    pub scheduled_start: Option<NaiveTime>,
    pub scheduled_end: Option<NaiveTime>,
    pub break_duration_minutes: Option<i32>,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_break_auto_deduct: Option<bool>,
    pub actual_punch_in: Option<DateTime<Utc>>,
    pub actual_punch_out: Option<DateTime<Utc>>,
    pub work_units: i32,
    pub status: String,  // "present", "late", "absent", "off"
    pub generation_metadata: Option<serde_json::Value>,
}
```

### AttendanceSummary
```rust
pub struct AttendanceSummary {
    pub employee_id: Uuid,
    pub employee_name: String,
    pub total_work_units: i32,
    pub total_leave_days: i64,
}
```

### EmployeeSchedule (from roster_repository)
```rust
pub struct EmployeeSchedule {
    pub employee_id: Uuid,
    pub full_name: String,
    pub schedules: Vec<EffectiveSchedule>,  // One per day
}
```

---

## Existing Related Infrastructure

### attendance_daily_summaries Table
**Used by**: `AttendanceSummaryService`

This table pre-computes daily summaries including:
- actual_work_minutes
- system_detected_ot_minutes
- claimed_ot_minutes
- approved_ot_minutes
- ot_status

**Opportunity**: Reports could potentially leverage this cached data instead of recalculating.

---

## Conventions Observed

### Naming Patterns
- Services: `{Domain}Service` struct with static methods
- Repositories: `{domain}_repository` modules with free functions
- Models: Flat structs in `models.rs`

### Error Handling
- Repositories return `Result<T, sqlx::Error>`
- Services convert to `StatusCode` for HTTP layer

### Pagination Pattern
```rust
PaginatedResponse {
    data: Vec<T>,
    total: usize,
    page: i64,
    limit: i64,
}
```

---

## Recommendations for Refactoring

### Priority 1: Batch Time Entry Fetching
Create new repository function:
```rust
// In time_entry_repository.rs
pub async fn get_workspace_time_entries_batch(
    pool: &PgPool,
    employee_ids: &[Uuid],
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<HashMap<Uuid, Vec<TimeEntry>>, sqlx::Error>
```

### Priority 2: Database-Level Pagination
Option A - Restructure to fetch only paginated employees first:
```rust
// Step 1: Get paginated employee list
let (employees, total) = employee_repository::get_employees_paginated(...);
// Step 2: Fetch schedules/entries only for those employees
```

Option B - Leverage attendance_daily_summaries table for reports.

### Priority 3: Batch Working Conditions
```rust
// In policy/resolver.rs
pub async fn batch_resolve_working_conditions(
    pool: &PgPool,
    employee_ids: &[Uuid],
    date: NaiveDate,
) -> Result<HashMap<Uuid, Vec<WorkingCondition>>, sqlx::Error>
```

---

## Warnings

- **Data Consistency**: Both summary and detail reports use same AttendanceCalculator. Any changes must update both.
- **Timezone Handling**: Complex timezone logic with overnight shift support. Test thoroughly.
- **Policy Override Cascade**: Shift-specific settings override PolicyResolver results (lines 130-132, 412-420)

---

## Unresolved Questions

1. What is typical dataset size? (employees x days)
2. Is there SLA for report generation time?
3. Can `attendance_daily_summaries` be leveraged for monthly summary reports?
4. Should we consider async streaming for large exports?
