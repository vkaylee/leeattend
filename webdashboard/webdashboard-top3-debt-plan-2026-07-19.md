# Kế hoạch Top 3 Nợ Kỹ thuật Webdashboard (Round 4 — 2026-07-19)

> **Đo đạc:** 2026-07-19
> **Lint:** ✅ 0 errors, 0 warnings | **TSC:** ✅ Clean
> **Tests:** 🔴 **54 failures (7 suites)** | **Bundle:** 🔴 **employees.lazy vượt budget**

---

## Executive Summary

Ba đợt refactor trước (June–July 2026) đã xử lý xong toàn bộ các vấn đề:
- Type-safety: `no-explicit-any` → `error`, 0 `any` trong production code
- Complexity & monolith functions: 0 violations
- Type narrowing: 0 `no-unnecessary-condition`, 0 `prefer-nullish-coalescing`
- Large files: whitelist đã được audit và nhiều file đã được tách

**Tuy nhiên, Round 3 đã để lại 3 regression mới, tập trung vào test stability và bundle budget:**

| # | Debt | Violations | Severity | Root Cause |
|---|------|-----------|----------|------------|
| **#1** | **Test Suite Failures** | 54 tests / 7 suites | 🔴 Critical | Regression từ Round 3 refactor + bundle budget cascade |
| **#2** | **Bundle Budget Regression** | employees.lazy 300.3 kB > 300 kB | 🔴 Critical | employees page tăng 0.3 kB vượt ngưỡng hard-fail |
| **#3** | **Type Safety Circumvention** | 17 `as Record<string, unknown>` + 3 `@ts-ignore` | 🟠 High | Defensive casting khi type system không được thiết kế đủ chặt |

---

## DEBT #1: Test Suite Failures (54 tests / 7 suites)

**Tác động:** Nghiêm trọng nhất. Test failures chặn CI/CD pipeline, ngăn merge,
và có thể che giấu real bugs. Mỗi test fail là một regression chưa được phát hiện.

### Failing Test Suites (phân nhóm theo root cause)

#### Nhóm A: ContractFormModal Crash (5 tests)
| File | Tests | Root Cause |
|------|-------|-----------|
| `employee-contracts-tab.test.tsx` | 5 | `TypeError: Cannot read properties of undefined (reading 'start_date')` tại `contract-form-modal.tsx:224` |

**Phân tích:** ContractFormModal nhận `contract` prop là undefined/null nhưng code không
có guard clause. Đây là regression từ việc refactor type narrowing ở Round 3 — có thể
một guard clause đã bị xóa nhầm khi clean up `no-unnecessary-condition`.

#### Nhóm B: EmployeesPage + Code Splitting (20 tests)
| File | Tests | Root Cause |
|------|-------|-----------|
| `employees-page.test.tsx` | 18 | `employees.lazy` chunk vượt budget → test render fail |
| `code-splitting.test.tsx` | 2 | Lazy import employees page fail |

**Phân tích:** Các test này fail vì 2 lý do:
1. Mock setup bị thiếu sau khi refactor employees page (có thể hook mới được thêm vào)
2. Bundle budget hard-fail ảnh hưởng đến code splitting test

#### Nhóm C: Các test khác (29 tests, 4 suites)
Cần điều tra từng suite một để xác định root cause chính xác.

### Kế hoạch Fix

#### Phase 1A: Fix ContractFormModal Crash (critical — 30 phút)
1. Đọc `contract-form-modal.tsx:224` để hiểu context
2. Thêm guard clause: `if (!contract) return null` hoặc `<Skeleton>` 
3. Verify: `npx vitest run src/components/features/employees/__tests__/employee-contracts-tab.test.tsx`

#### Phase 1B: Fix EmployeesPage Tests (1-2 giờ)
1. Audit mock setup trong `employees-page.test.tsx` — so sánh với imports thực tế của `employees-page.tsx`
2. Xác định hook/data source mới được thêm vào (có thể liên quan đến `use-unrecorded-shifts-data.ts`)
3. Cập nhật mock providers/react-query wrappers
4. Fix từng test group một

#### Phase 1C: Fix Remaining Failures (2-3 giờ)
1. Điều tra từng suite còn lại
2. Phân loại: mock issue / regression / genuine bug
3. Fix theo thứ tự severity

#### Phase 1D: Ngăn chặn regression
- Thêm `--reporter=verbose` vào CI pipeline để dễ debug
- Kiểm tra xem có test nào flaky không (chạy 3 lần liên tiếp)

### Verification
- `npx vitest run` → 7/7 passing suites, 54/54 passing tests
- Chạy 3 lần liên tiếp → 0 flaky tests
- CI pipeline xanh

### Ước lượng
- Phase 1A: 30 phút
- Phase 1B: 1-2 giờ
- Phase 1C: 2-3 giờ
- Phase 1D: 30 phút
- **Tổng: ~4-6 giờ**

---

## DEBT #2: Bundle Budget Regression (employees.lazy 300.3 kB)

**Tác động:** Check-budget script hard-fail (exit 1), chặn build production.
Chỉ vượt **0.3 kB** (300 bytes) — nhưng đó là dấu hiệu của bundle creep.

### Phân tích nguyên nhân

`employees.lazy` chunk chứa toàn bộ code cho route `/employees`:
- `employees-page.tsx` (component chính)
- `employee-form-modal.tsx` (add/edit employee form)
- `employee-detail-drawer.tsx` (detail panel)
- `contract-form-modal.tsx` (hợp đồng)
- `lifecycle-tab.tsx` (345 dòng)
- `employee-documents-tab.tsx`
- Các hooks và sub-components

### Kế hoạch

#### Step 1: Phân tích bundle composition (15 phút)
```bash
npx vite build --mode production
npx vite-bundle-visualizer  # hoặc rollup-plugin-visualizer
```
Xác định module nào chiếm nhiều nhất trong `employees.lazy` chunk.

#### Step 2: Quick win — Lazy-load sub-components (30 phút)
Các component chỉ hiển thị khi user click (modal, drawer) nên được lazy-load:
- `ContractFormModal` → `React.lazy(() => import('./contract-form-modal'))`
- `EmployeeFormModal` → `React.lazy(() => import('./employee-form-modal'))`
- `EmployeeDetailDrawer` → `React.lazy(() => import('./employee-detail-drawer'))`
- `LifecycleTab` → `React.lazy(() => import('./lifecycle-tab'))`

**Ước tính giảm:** 50-80 kB từ employees chunk → xuống ~220-250 kB.

#### Step 3: Tree-shaking audit (30 phút)
- Kiểm tra các import barrel (`index.ts`) có đang kéo theo dead code không
- Đảm bảo các library import dùng named imports (không default import cả thư viện)

#### Step 4: Nếu cần — Tách route employees thành sub-routes
Nếu sau Step 2-3 vẫn > 300 kB, cân nhắc tách thành:
- `/employees` → list view (chính)
- `/employees/:id` → detail view (sub-route)

### Verification
- `node scripts/check-budget.mjs` → exit 0
- `employees.lazy` chunk < 280 kB (để có margin an toàn)
- `npx vitest run` → 0 regression

### Ước lượng
- Step 1 (phân tích): 15 phút
- Step 2 (lazy-load sub-components): 30 phút
- Step 3 (tree-shaking): 30 phút
- Step 4 (sub-routes, nếu cần): 1-2 giờ
- **Tổng: ~1.5-3 giờ**

---

## DEBT #3: Type Safety Circumvention (17 `as Record<string, unknown>` + 3 `@ts-ignore`)

**Tác động:** `as Record<string, unknown>` và `@ts-ignore` vô hiệu hóa type checking,
cho phép runtime errors lọt qua compile-time. Anti-pattern này vi phạm nguyên tắc
"TypeScript strict mode" của dự án.

### Top Offenders

| # | File | Count | Pattern |
|---|------|-------|---------|
| 1 | `payroll-rules-manager.tsx` | **7** | `formData.rule_config as Record<string, unknown>` trong JSX event handlers |
| 2 | `use-shift-editor.ts` | **4** | `editingShift as Record<string, unknown>` để truy cập dynamic fields |
| 3 | `use-leave-type-manager.ts` | 2 | Error body parsing |
| 4 | `shift-list.tsx` | 1 | Dynamic shift field access |
| 5 | 7 files khác | 1 mỗi file | Các pattern tương tự |
| 6 | 3 files | `@ts-ignore` | `employee-documents-tab.tsx` (2), `smart-table-row.tsx` (1) |

### Phân tích nguyên nhân gốc

3 pattern chính:
1. **Dynamic config object** — `rule_config: Record<string, unknown>` trong API response không được typed đúng cách. Fix: define `RuleConfig` discriminated union type.
2. **API error body** — Error response body là `unknown`, cast sang `Record<string, unknown>` để access nested fields. Fix: define `ApiErrorBody` type với optional fields.
3. **Third-party library gaps** — `@ts-ignore` dùng khi library type không đầy đủ. Fix: augment types hoặc dùng `@ts-expect-error` với comment giải thích.

### Kế hoạch

#### Step 1: Fix Payroll Rules Manager (1 giờ)
Đây là file worst offender với 7 instances:
1. Define `PayrollRuleConfig` discriminated union:
```typescript
type PayrollRuleConfig = 
  | { type: 'percentage'; rate: number; base: 'gross' | 'taxable_income' }
  | { type: 'fixed'; amount: number }
  | { type: 'tiered'; tiers: Array<{ from: number; to?: number; rate: number }>; base: string };
```
2. Dùng type-safe access thay vì cast: `if (config.type === 'percentage') { config.rate }`

#### Step 2: Fix Shift Editor + Shift List (1 giờ)
1. `use-shift-editor.ts`: Define `EditableShift` interface với optional fields thay vì cast
2. `shift-list.tsx`: Same approach — type the shift object properly

#### Step 3: Fix Error Handling (30 phút)
1. `use-leave-type-manager.ts`: Define `ApiErrorDetail` type với `retry` field
2. `import-preview-step.tsx`: Type the row data properly

#### Step 4: Fix @ts-ignore (30 phút)
1. `employee-documents-tab.tsx`: Điều tra lý do cần `@ts-ignore` → fix hoặc thay bằng `@ts-expect-error` với comment
2. `smart-table-row.tsx`: Same approach

### Verification
- `grep -rn 'as Record<string, unknown>' src/` → 0 kết quả (trong production code)
- `grep -rn '@ts-ignore' src/` → 0 kết quả
- `npx tsc --noEmit` → 0 errors
- `npx vitest run` → 0 regression

### Ước lượng
- Step 1 (payroll): 1 giờ
- Step 2 (shift editor): 1 giờ
- Step 3 (error handling): 30 phút
- Step 4 (@ts-ignore): 30 phút
- **Tổng: ~3 giờ**

---

## Priority & Execution Order

```
Ngày 1 (6-8h):
├── Debt #2 Step 1: Phân tích bundle composition (15 phút)
├── Debt #2 Step 2-3: Lazy-load + tree-shaking (1h)
├── Debt #1 Phase 1A: Fix ContractFormModal crash (30 phút)
├── Debt #1 Phase 1B: Fix EmployeesPage tests (2h)
├── Debt #1 Phase 1C: Fix remaining test failures (2-3h)

Ngày 2 (3-4h):
├── Debt #3 Step 1: Fix Payroll Rules Manager type safety (1h)
├── Debt #3 Step 2: Fix Shift Editor type safety (1h)
├── Debt #3 Step 3-4: Error handling + @ts-ignore cleanup (1h)
├── Debt #1 Phase 1D: Flaky test detection + prevention (30 phút)
└── Final verify: lint + tsc + test + build + check-budget
```

**Lý do ưu tiên Debt #2 trước:** Bundle budget fix (lazy-loading) có thể tự động
fix một phần test failures (employees page + code splitting tests), giảm effort
cho Debt #1.

---

## Rủi ro & Mitigation

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Lazy-loading sub-components gây layout shift | Trung bình | Dùng `<Suspense>` + skeleton loader |
| Type narrowing (#3) lộ bug ẩn trong payroll | Cao | Test kỹ payroll rules CRUD; audit runtime behavior |
| Test fix (#1) chỉ là patch, không fix root cause | Trung bình | Mỗi test fix phải kèm theo hiểu biết về root cause |
| Bundle vẫn vượt sau khi lazy-load | Thấp | Sub-routes là giải pháp backup đã proven |

---

## Definition of Done

- [ ] `npx vitest run` → 0 failing tests (7/7 suites, 54/54 tests passing)
- [ ] `npx vitest run` lần 2 → 0 flaky tests
- [ ] `node scripts/check-budget.mjs` → exit 0, `employees.lazy` < 280 kB
- [ ] `grep -rn 'as Record<string, unknown>' src/` → 0 (production code)
- [ ] `grep -rn '@ts-ignore' src/` → 0
- [ ] `npx eslint src` → 0 errors, 0 warnings (giữ nguyên)
- [ ] `npx tsc --noEmit` → 0 errors (giữ nguyên)
- [ ] UI manual smoke test: employees CRUD, payroll rules, contract form, shifts

---

## Audit Evidence (baseline)

| Metric | Hiện tại | Target |
|--------|---------|--------|
| Test failures | 54 tests / 7 suites | **0** |
| Bundle budget | 1 chunk over (employees.lazy 300.3 kB) | **0** over |
| `as Record<string, unknown>` | 17 | **0** |
| `@ts-ignore` | 3 | **0** |
| Lint errors + warnings | 0 | **0** (giữ nguyên) |
| TSC errors | 0 | **0** (giữ nguyên) |
