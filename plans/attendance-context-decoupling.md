# Plan: Giải quyết nợ AttendanceContext

## 📋 Context

`AttendanceContext` đã được tách ra thành module standalone (`apiserver/src/services/attendance_context.rs`) với **zero dependency** vào `payroll_service`. Tuy nhiên, vẫn còn 3 khoản nợ chính:

1. **Không có constructor trực tiếp** — `impl AttendanceContext {}` trống, mọi caller đều phải đi qua `From<&PayrollContext>`
2. **Test coupling nặng** — 6 tests trong `attendance_matcher/tests_mod.rs` mỗi test phải construct full `PayrollContext` (~30 fields, ~45 dòng) chỉ để convert sang `AttendanceContext` rồi discard 13 payroll-only fields
3. **Encapsulation vi phạm** — tất cả 17 fields đều `pub(crate)`, vi phạm Coding Standards Rule 10 (strict encapsulation)
4. **Dead code** — `start_date` và `end_date` đánh dấu `#[allow(dead_code)]` cho "Phase 4" nhưng không ai dùng

### Key Insight

Non-payroll services (`AttendanceService`, `AttendanceSummaryService`, worker `AttendanceHandler`) **KHÔNG** dùng `AttendanceContext` — chúng dùng pipeline khác (`AttendanceService::compute_daily_attendance()` với per-day `ComputeDailyInput`). Đây là 2 abstraction level khác nhau:
- `AttendanceMatcher` + `AttendanceContext` = monthly-segment pipeline (payroll)
- `AttendanceService` + `ComputeDailyInput` = per-day pipeline (worker/summary/report)

→ **KHÔNG** cố migrate non-payroll callers sang `AttendanceContext`. Thay vào đó, tập trung giảm coupling thực sự.

---

## 🎯 Scope: 4 Phases

### Phase 1: Builder pattern + encapsulation (Core)

**Mục tiêu:** `AttendanceContext` có `builder()` constructor, fields chuyển từ `pub(crate)` sang private + getter methods.

**File:** `apiserver/src/services/attendance_context.rs`

1. Chuyển tất cả 17 fields từ `pub(crate)` → private (bỏ `pub(crate)`)
2. Thêm `AttendanceContextBuilder` struct với fluent API:
   ```rust
   pub struct AttendanceContextBuilder { /* 17 Option fields */ }

   impl AttendanceContext {
       pub fn builder() -> AttendanceContextBuilder { ... }
   }

   impl AttendanceContextBuilder {
       pub fn employee_id(mut self, v: Uuid) -> Self { ... }
       pub fn tz(mut self, v: Tz) -> Self { ... }
       // ... 15 more setter methods
       pub fn build(self) -> Self {
           // Defaults: empty Vec, Arc::new(HashMap::new()), Uuid::nil(), etc.
       }
   }
   ```
3. Getter methods cho mỗi field mà `AttendanceMatcher` thực sự dùng (15 getters):
   - `employee_id() -> Uuid`
   - `tz() -> Tz`
   - `start_date() -> NaiveDate`
   - `end_date() -> NaiveDate`
   - `daily_entries_map() -> &HashMap<NaiveDate, Vec<TimeEntry>>`
   - `holidays_map() -> &Arc<HashMap<NaiveDate, Holiday>>`
   - `roster_data() -> &Arc<RosterData>`
   - `emp_id_number() -> &str`
   - `emp_tags() -> Option<&[String]>`
   - `payroll_roster_map() -> &HashMap<(Uuid, NaiveDate), Option<Uuid>>`
   - `overtime_data() -> &Arc<WorkspaceOvertimeData>`
   - `active_assignments() -> &[EmployeeAssignment]`
   - `primary_location_id() -> Uuid`
   - `all_dept_history() -> &[EmployeeDepartmentHistory]`
   - `active_employments() -> &[Employment]`
   - `working_conditions() -> &[WorkingCondition]`
   - `condition_type_catalog() -> &Arc<HashMap<String, ConditionTypeDefinition>>`

**Files cần update (consumer side):**
- `apiserver/src/services/attendance_matcher/mod.rs` — `ctx.employee_id` → `ctx.employee_id()`
- `apiserver/src/services/attendance_matcher/pairing.rs` — `ctx.active_employments` → `ctx.active_employments()`, etc.
- `apiserver/src/services/attendance_matcher/scoring.rs` — `ctx.overtime_data` → `ctx.overtime_data()`, etc.
- `apiserver/src/services/payroll_service/context.rs` — `From` impl update để dùng builder

**Quy tắc:**
- Coding Standards Rule 10: strict encapsulation ✅
- Getter methods cho tất cả accessed fields ✅
- Builder self-documenting, test-friendly ✅

---

### Phase 2: Test helper + decouple tests khỏi PayrollContext

**Mục tiêu:** Tests construct `AttendanceContext` trực tiếp qua builder, không cần `PayrollContext`.

**File:** `apiserver/src/services/attendance_matcher/tests_mod.rs`

1. Migrate tất cả 6 tests: thay `PayrollContext { ... } + AttendanceContext::from(&ctx)` bằng builder:
   ```rust
   let att_ctx = AttendanceContext::builder()
       .employee_id(employee_id)
       .tz(chrono_tz::UTC)
       .start_date(date)
       .end_date(date)
       .daily_entries_map(daily_entries_map)
       .emp_id_number("EMP001".to_string())
       .active_employments(vec![employment])  // chỉ set khi test cần
       .build();  // 10+ fields còn lại tự động default
   ```
2. Xóa `use crate::services::payroll_service::PayrollContext;` khỏi test imports
3. Giảm ~270 dòng boilerplate (6 tests × ~45 dòng payroll-only fields)

**Expected result:** Mỗi test chỉ cần set 5-8 fields (thay vì 30), không có field nào bị discard.

---

### Phase 3: Clean up dead code

**Mục tiêu:** Quyết định số phận `start_date` và `end_date`.

**Options:**
- **Option A (Recommended):** Giữ lại nhưng thêm doc comment rõ ràng giải thích mục đích Phase 4, bỏ `#[allow(dead_code)]` bằng cách sử dụng chúng trong `new()` constructor
- **Option B:** Xóa nếu Phase 4 không còn relevant (non-payroll services đã có pipeline riêng)

**Decision:** Giữ lại (Option A) vì chúng có ý nghĩa semantic — context nên biết date range của nó. Sử dụng trong getter để bỏ dead_code warning.

---

### Phase 4: Từ điển hóa From impl (Bridge cleanup)

**Mục tiêu:** `From<&PayrollContext>` dùng builder thay vì field-by-field assignment.

**File:** `apiserver/src/services/payroll_service/context.rs`

```rust
impl From<&PayrollContext> for AttendanceContext {
    fn from(pc: &PayrollContext) -> Self {
        AttendanceContext::builder()
            .employee_id(pc.employee_id)
            .tz(pc.tz)
            .start_date(pc.start_date)
            .end_date(pc.end_date)
            .daily_entries_map(pc.daily_entries_map.clone())
            .holidays_map(Arc::clone(&pc.holidays_map))
            .roster_data(Arc::clone(&pc.roster_data))
            .emp_id_number(pc.emp_id_number.clone())
            .emp_tags(pc.emp_tags.clone())
            .payroll_roster_map(pc.payroll_roster_map.clone())
            .overtime_data(Arc::clone(&pc.overtime_data))
            .active_assignments(pc.active_assignments.clone())
            .primary_location_id(pc.primary_location_id)
            .all_dept_history(pc.all_dept_history.clone())
            .active_employments(pc.active_employments.clone())
            .working_conditions(pc.working_conditions.clone())
            .condition_type_catalog(Arc::clone(&pc.condition_type_catalog))
            .build()
    }
}
```

---

## 📁 Files thay đổi

| File | Thay đổi |
|------|----------|
| `apiserver/src/services/attendance_context.rs` | Private fields + `new()` + getters |
| `apiserver/src/services/attendance_matcher/mod.rs` | `.field` → `.field()` |
| `apiserver/src/services/attendance_matcher/pairing.rs` | `.field` → `.field()` |
| `apiserver/src/services/attendance_matcher/scoring.rs` | `.field` → `.field()` |
| `apiserver/src/services/attendance_matcher/tests_mod.rs` | Direct `AttendanceContext` construction |
| `apiserver/src/services/payroll_service/context.rs` | `From` impl dùng `new()` |

## 🔍 Verification

1. `./leedevkit test api --lint-only` — verify compile + clippy
2. `./leedevkit test api --unit-only --pattern 'attendance_matcher'` — verify tests pass
3. `./leedevkit test api --unit-only --pattern 'payroll'` — verify payroll tests pass
4. `./leedevkit test api` — full suite

## ⚠️ Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Getter method rename cascade | IDE refactoring + compiler catches all usage sites |
| Test helper defaults không match real data | Copy exact default patterns từ existing tests |
| `From` impl thay đổi behavior | Giữ nguyên logic, chỉ thay construction path |
