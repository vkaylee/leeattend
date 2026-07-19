# Kế Hoạch Giải Quyết 3 Nợ Kỹ Thuật Nghiêm Trọng Nhất

> **Ngày:** 2026-07-19 | **Dựa trên:** Phân tích codebase toàn diện (graph analysis, file size, cognitive complexity, similarity detection)

---

## Phân Tích & Phát Hiện

Qua phân tích toàn bộ codebase (38,113 nodes, 141,553 edges), các metrics chính được đánh giá:

| Metric | Backend (apiserver) | Frontend (webdashboard) |
|--------|---------------------|-------------------------|
| File handler lớn nhất | `import_termination_preview` (361 dòng) | `UnrecordedShiftsTab` (510 dòng) |
| Cognitive complexity cao nhất | - | `useMonthlyRoster` (34) |
| N+1 Query phát hiện | 1 (`for year in years { .await }`) | N/A |
| Code duplication (SIMILAR_TO > 85%) | `create_workflow`/`update_workflow` (96.9%) | `PatternGrid`/`PatternList` (100%) |
| eslint-disable còn tồn tại | N/A | 1 (legitimate) |

### Tiêu Chí Đánh Giá (theo CLAUDE.md)

1. **Enterprise Premium Standard** — N+1 queries, god components là vi phạm nghiêm trọng
2. **Boy Scout Rule** — Code duplication, hardcoded i18n, business logic trong handler
3. **Performance** — Sequential DB calls, missing batch operations
4. **Maintainability** — Files > 300 dòng với business logic inline

---

## TD-1: N+1 Query + God Handler `import_termination_preview`

**File:** `apiserver/src/handlers/employees.rs:577-937` (361 dòng)  
**Mức độ:** CRITICAL  
**Loại:** Performance + Code Organization

### Mô tả vấn đề

Handler `import_termination_preview` chứa 3 vấn đề chính:

1. **N+1 Query tại dòng 776:**
   ```rust
   for year in years {
       let batch_balances = state
           .leaves()
           .balance_repo()
           .get_employee_balances_batch(&ctx.auth_token, &employee_ids, year)
           .await?;  // ← Gọi DB riêng cho từng năm
       balances_by_year_and_employee.insert(year, batch_balances);
   }
   ```
   File import có thể chứa nhân viên nghỉ việc ở nhiều năm khác nhau → mỗi năm là một DB round-trip riêng.

2. **Business logic nằm trong handler (361 dòng):**
   - Logic validate dữ liệu (dòng 639-732)
   - Logic tìm active employment (dòng 810-845)
   - Logic tính proration fraction (dòng 889-893)
   - Logic tính prorated leaves (dòng 901-913)
   - Tất cả đều là business logic, lẽ ra phải ở service layer

3. **Hardcoded i18n với locale checks lặp lại:**
   ```rust
   let reason = if locale == "vi" {
       format!("Lý do nghỉ việc không hợp lệ: '{}'...", raw_reason, ...)
   } else {
       format!("Invalid termination reason: '{}'...", raw_reason, ...)
   };
   ```
   Xuất hiện 4 lần trong cùng một hàm (dòng 682-695, 706-712, 818-824, 847-866)

### Giải pháp đề xuất

```
apiserver/src/
├── handlers/employees.rs          ← Chỉ còn orchestration (~80 dòng)
├── services/termination_service.rs ← MỚI: Business logic + validation
└── repositories/
    └── leave_balance_repository.rs ← THÊM: get_balances_for_employees_by_years()
```

**Bước 1: Fix N+1 query** — Thêm batch method:
```rust
// Repository trait
async fn get_balances_for_employees_by_years(
    &self,
    auth_token: &TenantAuthToken,
    employee_ids: &[Uuid],
    years: &[i32],
) -> Result<HashMap<i32, HashMap<Uuid, Vec<EmployeeLeaveBalance>>>, AppError>;

// SQL: WHERE employee_id = ANY($1) AND year = ANY($2)
```

**Bước 2: Extract service** — `TerminationService` với các method:
- `parse_and_validate_rows()` — validate CSV rows
- `resolve_active_employments()` — tìm active employment
- `calculate_proration_preview()` — tính proration + merge với balances

**Bước 3: Clean up handler** — Chỉ còn:
- Parse multipart upload
- Gọi `TerminationService::build_preview()`
- Return JSON response

**Bước 4: Fix i18n** — Sử dụng `I18nKey` enum thay vì hardcoded locale check:
```rust
// Thay vì:
if locale == "vi" { format!("...") } else { format!("...") }
// Dùng:
crate::utils::i18n::translate(&locale, I18nKey::InvalidTerminationReason)
```

### Files bị ảnh hưởng

| File | Thay đổi |
|------|----------|
| `apiserver/src/handlers/employees.rs` | Giảm từ 361 → ~80 dòng |
| `apiserver/src/services/termination_service.rs` | **MỚI** — ~200 dòng |
| `apiserver/src/repositories/leave_balance_repository.rs` | **THÊM** trait method |
| `apiserver/src/repositories/diesel_leave_balance_repository.rs` | **THÊM** diesel impl |
| Test files liên quan | Cập nhật mock implementations |

### Rủi ro & Mitigation

| Rủi ro | Mitigation |
|--------|-----------|
| Breaking change API response | Giữ nguyên response structure |
| Performance regression | Batch query thực tế nhanh hơn N+1 |
| Test regression | 2315 tests phải pass; thêm unit test cho service mới |

---

## TD-2: God Component `UnrecordedShiftsTab`

**File:** `webdashboard/src/pages/attendance/components/unrecorded-shifts-tab.tsx` (510 dòng)  
**Mức độ:** CRITICAL  
**Loại:** Code Organization / Maintainability

### Mô tả vấn đề

Component lớn nhất toàn bộ frontend (510 dòng), nhồi nhét quá nhiều concerns:

| Concern | Vị trí | Dòng |
|---------|--------|------|
| State management (filters, tick, export) | 125-150 | ~25 |
| Infinite query + auto-drain | 163-207 | ~45 |
| Employee map building | 209-218 | ~10 |
| Zone computation (useNoShowRows) | 230-235 | ~5 |
| Filter logic (department, zone-b) | 242-258 | ~17 |
| Export handler (handleExport) | 261-321 | ~60 |
| Clear filters | 323-328 | ~5 |
| Column definitions | 330-338 | ~8 |
| Trace map building | 340-350 | ~11 |
| JSX rendering | 352-633 | ~282 |

Component này vi phạm Single Responsibility Principle — mỗi khi cần sửa export logic, filter, hoặc zone display, developer phải navigate qua 500+ dòng code.

### Giải pháp đề xuất

```
webdashboard/src/pages/attendance/components/
├── unrecorded-shifts-tab.tsx          ← Còn ~200 dòng (orchestration)
├── unrecorded-shifts/
│   ├── NoShowFilterBar.tsx            ← MỚI: Filter controls (dept, shift type, zone)
│   ├── NoShowTable.tsx                ← MỚI: Table rendering + infinite scroll
│   ├── NoShowExportButton.tsx         ← MỚI: Export button + logic
│   └── hooks/
│       └── useNoShowExport.ts         ← MỚI: Export logic extracted from handleExport
```

**Bước 1: Extract `useNoShowExport` hook:**
```typescript
// ~70 dòng — toàn bộ logic drain-pages + build-rows + export-to-excel
export function useNoShowExport({ wsId, queryKey, employeeMap, ... }) {
  const queryClient = useQueryClient();
  const [isExporting, setIsExporting] = useState(false);
  
  const exportFn = useCallback(async (selectedDate, selectedDept) => {
    // ... drain pages, build NoShowRow[], call exportNoShowToExcel
  }, [...]);
  
  return { exportFn, isExporting };
}
```

**Bước 2: Extract `NoShowFilterBar`:**
```typescript
// ~80 dòng — department select, shift type picker, zone-b toggle, clear button
export function NoShowFilterBar({ departments, ... }) { ... }
```

**Bước 3: Clean up `UnrecordedShiftsTab`:**
- Chỉ còn: query setup, employeeMap, composition của các sub-components
- Giảm từ 510 → ~200 dòng

### Files bị ảnh hưởng

| File | Thay đổi |
|------|----------|
| `unrecorded-shifts-tab.tsx` | Giảm 510 → ~200 dòng |
| `unrecorded-shifts/NoShowFilterBar.tsx` | **MỚI** — ~80 dòng |
| `unrecorded-shifts/NoShowTable.tsx` | **MỚI** — ~120 dòng |
| `unrecorded-shifts/hooks/useNoShowExport.ts` | **MỚI** — ~70 dòng |
| `__tests__/` | **MỚI** — Unit tests cho hook + components |

### Rủi ro & Mitigation

| Rủi ro | Mitigation |
|--------|-----------|
| Export behavior thay đổi | Giữ nguyên logic, chỉ extract |
| Filter state sync giữa các component | Props drilling qua parent (đơn giản, rõ ràng) |
| Live tick re-render | useEffect interval vẫn ở parent, pass `liveTick` xuống |

---

## TD-3: Code Duplication `create_workflow` / `update_workflow`

**File:** `apiserver/src/handlers/approval_workflow.rs`  
**Mức độ:** HIGH  
**Loại:** Code Duplication / DRY Violation

### Mô tả vấn đề

Hai handler `create_workflow` (line 97-151, 55 dòng) và `update_workflow` (line 206-261, 56 dòng) có **96.9% Jaccard similarity**. Cả hai cùng:

1. Build `ApprovalWorkflow` struct từ request (giống hệt nhau)
2. Map `steps` từ request sang `ApprovalWorkflowStep` (giống hệt nhau)
3. Serialize response với `steps` embedded (giống hệt nhau)

Điểm khác biệt DUY NHẤT:
- `create_workflow`: tạo `workflow_id` mới bằng `new_id()`, gọi `create_workflow()`
- `update_workflow`: dùng `id` từ path params, gọi `update_workflow_definition()`

```rust
// create_workflow (line 119-143) — GIỐNG HỆT update_workflow (line 226-250)
let workflow = ApprovalWorkflow {
    id: workflow_id,                    // ← KHÁC: new_id() vs id từ params
    workspace_id: ctx.workspace_id,
    entity_type: req.entity_type.clone(),
    name: req.name.clone(),
    // ... 7 fields giống hệt
};

let steps: Vec<ApprovalWorkflowStep> = req  // ← GIỐNG HỆT 100%
    .steps
    .into_iter()
    .map(|s| ApprovalWorkflowStep { ... })
    .collect();
```

Ngoài ra, còn có các cặp tương tự trong codebase (mức độ nhẹ hơn):
- `PatternGrid`/`PatternList` (100% similarity) — frontend, 54 dòng mỗi component
- `assign_employee_phase`/`update_employee_phase_status` (100% similarity) — 34 dòng mỗi handler

### Giải pháp đề xuất

**Bước 1: Extract shared builder function:**
```rust
// Trong approval_workflow.rs
fn build_workflow_and_steps(
    workspace_id: Uuid,
    user_id: Uuid,
    workflow_id: Uuid,
    req: CreateWorkflowRequest,
) -> (ApprovalWorkflow, Vec<ApprovalWorkflowStep>) {
    let workflow = ApprovalWorkflow {
        id: workflow_id,
        workspace_id,
        entity_type: req.entity_type,
        name: req.name,
        description: req.description,
        total_steps: i32::try_from(req.steps.len()).unwrap_or(0),
        is_active: req.is_active,
        created_by: Some(user_id),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let steps: Vec<ApprovalWorkflowStep> = req
        .steps
        .into_iter()
        .map(|s| ApprovalWorkflowStep {
            id: crate::utils::id::new_id(),
            workflow_id,
            step_order: s.step_order,
            step_name: s.step_name,
            approver_type: s.approver_type,
            approver_config: s.approver_config,
            action_required: s.action_required,
            can_skip: s.can_skip,
            created_at: chrono::Utc::now(),
        })
        .collect();

    (workflow, steps)
}
```

**Bước 2: Đơn giản hóa handlers:**
```rust
// create_workflow — còn ~25 dòng
pub async fn create_workflow(...) -> AppResult<Json<serde_json::Value>> {
    ctx.require_permission(Permission::ApprovalWorkflowsManage)?;
    let workflow_id = crate::utils::id::new_id();
    let (workflow, steps) = build_workflow_and_steps(
        ctx.workspace_id, ctx.user_id, workflow_id, req
    );
    let (wf, created) = state.core().approval_workflow_service()
        .create_workflow(&ctx.auth_token, workflow, steps).await?;
    Ok(Json(serialize_workflow_with_steps(&wf, &created)?))
}

// update_workflow — còn ~25 dòng
pub async fn update_workflow(...) -> AppResult<Json<serde_json::Value>> {
    let (_, id) = params;
    ctx.require_permission(Permission::ApprovalWorkflowsManage)?;
    let (workflow, steps) = build_workflow_and_steps(
        ctx.workspace_id, ctx.user_id, id, req
    );
    let (wf, created) = state.core().approval_workflow_service()
        .update_workflow_definition(&ctx.auth_token, workflow, steps).await?;
    Ok(Json(serialize_workflow_with_steps(&wf, &created)?))
}
```

**Bước 3 (optional):** Áp dụng pattern tương tự cho `PatternGrid`/`PatternList`:
```typescript
// Unify thành một component với variant prop
<PatternView variant="grid" {...props} />   // thay cho <PatternGrid />
<PatternView variant="list" {...props} />   // thay cho <PatternList />
```

### Files bị ảnh hưởng

| File | Thay đổi |
|------|----------|
| `apiserver/src/handlers/approval_workflow.rs` | Giảm 110 → ~70 dòng (cả 2 handlers + helper) |
| (Optional) `webdashboard/.../pattern-grid.tsx` | Merge vào PatternView |
| (Optional) `webdashboard/.../pattern-list.tsx` | Merge vào PatternView |

### Rủi ro & Mitigation

| Rủi ro | Mitigation |
|--------|-----------|
| Helper function thay đổi behavior | Extract chính xác code hiện tại, không thay đổi logic |
| Test expectation về struct fields | Test coverage hiện có sẽ catch regression |

---

## Tổng Quan & Timeline

| # | Debt | Layer | Dòng hiện tại | Dòng sau | Effort |
|---|------|-------|--------------|---------|--------|
| TD-1 | N+1 Query + God Handler | Backend | 361 | ~80 (handler) + 200 (service) | 3-4h |
| TD-2 | God Component | Frontend | 510 | ~200 + 270 (extracted) | 2-3h |
| TD-3 | Code Duplication | Backend + Frontend | 110 + 108 | ~70 + ~60 | 1-2h |

**Tổng thời gian dự kiến:** 6-9 giờ

**Thứ tự thực hiện:** TD-1 → TD-2 → TD-3 (giảm dần theo mức độ critical)

**Verification:**
- Toàn bộ 2315 tests phải pass sau mỗi TD
- Không thay đổi API contract (request/response)
- Không thay đổi UI behavior
- Git diff review cho từng commit riêng biệt

---

## Rule Compliance Check

| Rule ID | Rule | Áp dụng |
|---------|------|---------|
| R1 | Enterprise Premium Standard | TD-1 fix N+1, TD-2 extract components |
| R2 | Boy Scout Rule | TD-1 fix hardcoded i18n, TD-3 eliminiate duplication |
| R3 | Structured Errors (AppError) | TD-1 service uses AppError, không unwrap() |
| R4 | Zero N+1 Queries | TD-1 batch query thay thế loop |
| R5 | Component Modularity | TD-2 extract hooks + sub-components |
| R6 | DRY Principle | TD-3 extract shared builders |

---

## Ký Hiệu Quy Ước Commit

```
refactor(apiserver): resolve top 3 critical technical debts

TD-1 — Fix N+1 query in import_termination_preview:
- Add get_balances_for_employees_by_years() batch repository method
- Extract TerminationService with parse/validate/proration logic
- Replace hardcoded i18n locale checks with I18nKey enum
- Handler reduced from 361 to ~80 lines

TD-2 — Decompose UnrecordedShiftsTab god component:
- Extract useNoShowExport hook (drain-pages + excel generation)
- Extract NoShowFilterBar (department, shift-type, zone-b controls)
- Extract NoShowTable (rendering + infinite scroll)
- Component reduced from 510 to ~200 lines

TD-3 — Eliminate create_workflow/update_workflow duplication:
- Extract shared build_workflow_and_steps() helper
- Both handlers reduced to pure orchestration (~25 lines each)
- Unify PatternGrid/PatternList into PatternView component

All 2315 tests pass with zero regressions.

Co-Authored-By: Claude <noreply@anthropic.com>
```
