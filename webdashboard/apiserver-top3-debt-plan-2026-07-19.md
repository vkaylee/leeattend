# Top 3 API Technical Debt Resolution Plan (2026-07-19)

**Scope:** `apiserver/` — Rust/Axum backend  
**Methodology:** Static analysis via codebase-memory graph (38K nodes, 142K edges) + manual audit  
**Previous rounds:** Round 1 (a78a137), Round 2 (e459658), Round 3 (83cac27)

---

## Executive Summary

After deep analysis of **148K lines** of production Rust code across **753 `.rs` files**, three systemic debts emerge as most critical — measured against the project's core standards: "Explicit AppError handling", "No N+1 queries", "Strict transaction boundaries".

| # | Debt | Severity | Primary File | Key Metric |
|---|------|----------|-------------|------------|
| **#1** | **Error Swallowing in Payroll** | 🔴 CRITICAL | `payroll_service.rs` | **38** `unwrap_or_default` on DB results |
| **#2** | **God Service Monoliths** | 🔴 CRITICAL | `payroll_service.rs` (1987L) + `leave_service.rs` (2289L) + `attendance_summary_service.rs` (2204L) | **3 services** = 6480 lines |
| **#3** | **Clone Epidemic** | 🟠 HIGH | `leave_service.rs` (33) + `payroll_service.rs` (29) | **80+ clones** across 3 files |

---

## 🔴 DEBT #1: Error Swallowing in Payroll Batch Processing

### What It Is

`PayrollService::process_payroll_chunk()` uses `.unwrap_or_default()` on **Result types from database queries**, silently converting errors into empty/default data. This directly violates **"Explicit AppError handling — NEVER throw generic Error or unwrap() blindly"**.

### Evidence

```rust
// apiserver/src/services/payroll_service.rs:1056-1060
let employments_history = self
    .employment_repo
    .get_employments_by_ids(&d.token, &chunk_employment_ids)
    .await
    .unwrap_or_default();  // 🔴 DB failure → empty vec, no log
```

**38 occurrences** of `unwrap_or_default`/`unwrap_or_else` in `payroll_service.rs`:

| Line | Query | Consequence if DB fails |
|------|-------|------------------------|
| 1060 | `get_employments_by_ids` | Empty list → all employees silently skipped |
| 1155-1159 | `employees_res` (mapped) | `return Vec::new()` — entire chunk dropped |
| 1162 | `salary_histories` | `unwrap_or_default()` → salary = 0, corrupt payslip |
| 1163 | `dept_histories` | `unwrap_or_default()` → department data gone |
| 1186 | `policies_res` | `unwrap_or_default()` → policy rules missing |
| 1201-1203 | `get_department_payroll_configs_batch` | `unwrap_or_default()` → dept configs missing |
| 1206-1209 | `get_employee_payroll_configs_batch` | `unwrap_or_default()` → emp configs missing |

Also: **`leave_service.rs`** has 4 similar occurrences, and **349 `.expect()` calls** exist across the apiserver (mostly in test code, but some in production like `gen_hash.rs`, `openapi.rs`, `worker/handlers/payroll_report.rs`).

### Root Cause

Batch processing code prioritizes "partial success" over correctness, but goes too far — DB connection failures, schema mismatches, and constraint violations all get the same silent treatment as "no data found."

### Proposed Solution

**Phase 1: Add structured error logging (non-breaking, ~2-4h)**
- Before each `.unwrap_or_default()` that touches a DB Result, insert:
  ```rust
  .unwrap_or_else(|e| {
      tracing::error!(%workspace_id, error = %e, query = "get_employments_by_ids",
          "Payroll batch fetch failed — returning empty");
      Vec::new()
  })
  ```
- This makes failures observable in production logs without changing behavior

**Phase 2: Introduce `BatchFetchResult` type (~4-6h)**
```rust
enum BatchFetchResult<T> {
    Success(T),
    Degraded(T),  // data available but some errors occurred
}
```
- Replace `unwrap_or_default()` on key queries with explicit match + log
- Individual `BatchCalculateResult` already has `error_message: Option<String>` — use it

**Phase 3: Per-employment error isolation (~6-8h)**
- Each employment in a chunk can fail independently
- Propagate errors into the **existing** `BatchCalculateResult::error_message` field
- Remove all silent `unwrap_or_default()` from the chunk processing path

### Files to Modify
- `apiserver/src/services/payroll_service.rs` — primary (38 error sites)
- `apiserver/src/services/leave_service.rs` — secondary (4 error sites)
- `apiserver/src/services/attendance_matcher.rs` — related error paths

### Effort Estimate: ~16 hours

---

## 🔴 DEBT #2: God Service Monoliths (3 Services = 6480 Lines)

### What It Is

Three service files dominate the codebase with oversized, multi-concern implementations:

| Service | Lines | Public Methods | Clone Count | Key Issue |
|---------|-------|---------------|-------------|-----------|
| `leave_service.rs` | 2289 | 9 | 33 | Mixed concerns: balance, request, history, approval |
| `attendance_summary_service.rs` | 2204 | 6 | 2 | Summary + OT + recalculation all in one |
| `payroll_service.rs` | 1987 | 4 | 29 | 30-field `PayrollContext`, 300-line chunk processor |

**Also critical but secondary:**
- `attendance_matcher.rs` — 1623 lines, **1 single public method** (extreme God Function)
- Repository files: `diesel_employee_repository/core.rs` (1570L), `diesel_payroll_repository.rs` (1830L), `diesel_roster_repository.rs` (1531L)

### Evidence — The 30-Field God Context

```rust
// apiserver/src/services/payroll_service.rs:33-72
pub struct PayrollContext {
    pub(crate) employee_id: Uuid,
    pub(crate) workspace: Workspace,
    pub(crate) tz: Tz,
    pub(crate) salary_history: Vec<...>,           // compensation
    pub(crate) all_dept_history: Vec<...>,          // org structure
    pub(crate) daily_entries_map: HashMap<...>,     // attendance
    pub(crate) holidays_map: HashMap<...>,           // calendar
    pub(crate) roster_data: Arc<...>,               // scheduling
    pub(crate) payroll_roster_map: HashMap<...>,
    pub(crate) working_conditions: Vec<...>,        // work rules
    pub(crate) condition_type_catalog: HashMap<...>,
    pub(crate) attendance_policy: AttendanceRules,  // policy
    pub(crate) emp_payroll_configs: Vec<...>,
    pub(crate) ws_payroll_settings_history: Vec<...>,
    pub(crate) department_payroll_configs: HashMap<...>,
    pub(crate) overtime_data: ...,                  // OT policy
    pub(crate) ot_base_component_ids: HashSet<Uuid>,
    pub(crate) active_contracts: Vec<...>,
    pub(crate) active_assignments: Vec<...>,
    pub(crate) active_employments: Vec<...>,
    pub(crate) primary_location_id: Uuid,
    pub(crate) month_end_payroll_settings: Option<...>,
    pub(crate) month_end_attendance_policy: AttendanceRules,
    pub(crate) leave_breakdown: ...,
    pub(crate) active_policies_with_tiers: Vec<...>, // incentives
    pub(crate) salary_base_groups: Vec<...>,         // compensation again
    pub(crate) unpaid_holiday_days: i64,
    pub(crate) rule_assignments_with_tiers: Vec<...>, // rules
    // 30 fields mixing 8 different domains
}
```

Also, `process_payroll_chunk` (lines 1040-1331) is **~300 lines** and does:
- Employment resolution (lines 1056-1068)
- 8+ batch DB queries (lines 1071-1203)
- Department/employee mapping (lines 1191-1266)
- Rule assignment building (lines 1268-1277)
- PayrollContext building (lines 1280-1302)
- Concurrent calculation dispatch (lines 1305-1330)

### Root Cause

The payroll pipeline is inherently complex (fetch → resolve → compute → persist), but all concerns were pushed into one struct. The existing decomposition (`AttendanceMatcher`, `FinancialCalculator`, `PolicyResolverService`) was a good start but stopped short of a proper pipeline architecture.

### Proposed Solution

**Decompose `payroll_service.rs` into a focused module directory:**

```
apiserver/src/services/payroll/
├── mod.rs              // PayrollOrchestrator (~100L): thin coordinator
├── context.rs          // PayrollContext + builder (extract from L33-72)
├── fetch.rs            // PayrollDataFetcher (~300L): all batch DB queries
├── segments.rs         // SegmentBuilder (~200L): date range logic
├── calculator.rs       // PayrollCalculator (~300L): per-employment calc
└── persistence.rs      // PayslipPersistence (~150L): save results
```

**For `leave_service.rs` (2289 lines):**
- Extract: `LeaveBalanceService` (balance queries), `LeaveRequestService` (submit/approve), `LeaveHistoryService`
- Already has `LeaveAccrualService` at `leave_accrual_service/mod.rs` — follow that pattern

**For `attendance_matcher.rs` (1623 lines, 1 function):**
- Decompose `process_segment_attendance` into:
  - `shift_resolver.rs` — shift matching per day
  - `daily_unit_calc.rs` — std/OT unit computation
  - `holiday_credit.rs` — holiday premium calculation

### Effort Estimate: ~22 hours

---

## 🟠 DEBT #3: Clone Epidemic — 80+ Unnecessary Clones

### What It Is

`leave_service.rs` (33 clones) and `payroll_service.rs` (29 clones) amplify every request's memory footprint. The clones are not one-off convenience copies — they represent systemic ownership design flaws where data is cloned 3-4 times through the pipeline.

### Evidence

**Double-clone pattern in leave_service:**
```rust
// Lines 2050-2055: clone → clone of clone
let other_requests = vec![incoming_req.clone(), conflicting_req.clone()];
let other_requests_clone = other_requests.clone();  // clone of a clone!

// Additionally: Mutex-locked data cloned on every read
let data = self.workspace_balances.lock().unwrap().clone();  // Line 804
let balances = self.employee_balances.lock().unwrap().clone(); // Line 835
```

**Triple-clone path in payroll_service:**
```rust
// Step 1: Clone into Arc<ChunkSharedData> (lines 1012-1026, 7 clones)
workspace: workspace.clone(),
token: token.clone(),
condition_type_catalog: condition_type_catalog.clone(),
// ... 4 more

// Step 2: Clone from ChunkSharedData into each PayrollContext (lines 1431-1464, 10 clones)
workspace: d.workspace.clone(),
holidays_map: d.holidays_map.clone(),
// ... 8 more

// Step 3: Clone fields during calculation (lines 703, 744, 796, 847)
let attendance_policy = ctx.attendance_policy.clone(); // Line 744
for group in ctx.salary_base_groups.clone() {          // Line 847
```

**Total:** ~80 clones across just 3 files. Each `PayrollContext` clone copies 30 fields. With 50 employments per chunk × 10 concurrent = ~500 clone operations per batch.

### Root Cause

1. `PayrollContext` stores owned data but is built from references to data owned elsewhere
2. `ChunkSharedData` wraps owned data in `Arc`, but individual fields are still cloned
3. `Mutex<HashMap>` in leave_service is cloned on every read instead of using read-lock access patterns

### Proposed Solution

**1. Arc-wrap shared data at field level (~6-8h)**
```rust
struct ChunkSharedData {
    holidays_map: Arc<HashMap<NaiveDate, Holiday>>,        // was: owned HashMap
    condition_type_catalog: Arc<HashMap<String, ConditionType>>,
    overtime_data: Arc<WorkspaceOvertimeData>,
    // These are the same for ALL employments in a batch → Arc them
}
```

**2. PayrollContext borrows from shared data (~6-8h)**
```rust
struct PayrollContext<'a> {
    shared: &'a ChunkSharedData,        // borrow, don't clone
    employee_specific: EmployeeData,    // only per-employee owned data
}
```
Eliminates 10 clones per employment (step 2 above).

**3. Fix leave_service Mutex clones (~4-6h)**
```rust
// Before (33 clones):
let data = self.workspace_balances.lock().unwrap().clone();

// After:
let data = self.workspace_balances.lock().unwrap();
// Return &HashMap or Arc<HashMap> instead of owned clone
```

### Files to Modify
- `apiserver/src/services/payroll_service.rs` — 29 clones → eliminate ~20
- `apiserver/src/services/leave_service.rs` — 33 clones → eliminate ~25
- `apiserver/src/services/attendance_matcher.rs` — 16 clones → eliminate ~10

### Effort Estimate: ~18 hours

---

## Implementation Order & Rationale

| Order | Debt | Why This Order |
|-------|------|---------------|
| **1st** | #1 Error Swallowing | Fix data integrity first. Add logging → introduce BatchFetchResult → remove silent defaults |
| **2nd** | #2 God Service Split | Structural refactor creates clean module boundaries — makes #3 easier by reducing scope per file |
| **3rd** | #3 Clone Epidemic | Ownership optimization — safer after module boundaries are clear and each file ≤400 lines |

**Total estimated effort: ~56 hours**

---

## Metrics — Current vs Target

| Metric | Current | Target | Debt # |
|--------|---------|--------|--------|
| `payroll_service.rs` error swallows | 38 `unwrap_or_default` | **0** (all explicit) | #1 |
| `payroll_service.rs` size | 1987 lines | **≤400 lines** (split into 5 modules) | #2 |
| `leave_service.rs` size | 2289 lines | **≤600 lines** (split into 3 modules) | #2 |
| `attendance_summary_service.rs` size | 2204 lines | **≤800 lines** (deferred to next round) | #2 |
| `attendance_matcher.rs` size | 1623 lines | **≤600 lines** (split into 3 modules) | #2 |
| `leave_service.rs` clones | 33 `.clone()` | **≤8** | #3 |
| `payroll_service.rs` clones | 29 `.clone()` | **≤8** | #3 |
| `PayrollContext` fields | 30 | **≤12** (rest in sub-contexts) | #2 |
| `process_payroll_chunk` size | ~300 lines | **≤100 lines** | #2 |

---

## Verification Checklist

After each debt resolution:
- [ ] `./leedevkit test` — all tests pass, 0 regression
- [ ] `cargo clippy -- -D clippy::clone_on_copy` — no new clone warnings
- [ ] Manual batch payroll test: empty workspace, 1 employee, 50 employees, missing salary data
- [ ] Leave balance query: verify response unchanged after Mutex refactor
- [ ] Attendance recalculation: verify daily summaries match before/after
- [ ] Check logs for any new ERROR-level messages from Phase 1 logging

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|-----------|
| Error logging changes behavior | Low | Phase 1 only adds `tracing::error!` before existing `.unwrap_or_default()` |
| Service split breaks wiring | Medium | Keep old `PayrollService` facade delegating to new modules; update `state.rs` incrementally |
| Ownership refactor (#3) causes borrow checker cascade | High | Do #2 first (smaller files); use `Arc` not lifetimes for shared data |
| AttendanceMatcher changes affect payroll calculation | High | Extensive integration tests exist in `tests/integration/payroll/` — run after each change |

---

## Related Past Work

- Round 1 (a78a137): Top-3 critical debts (webdashboard + apiserver cross-cutting)
- Round 2 (e459658): Top-3 critical debts (apiserver specific)
- Round 3 (83cac27): Top-3 critical debts (cross-cutting)
- **Model to follow:** `apiserver/src/services/pairing/` directory (12 files, well-decomposed, clear module boundaries)
