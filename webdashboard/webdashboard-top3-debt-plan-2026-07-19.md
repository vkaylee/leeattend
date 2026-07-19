# Kế hoạch Top 3 Nợ Kỹ thuật Webdashboard (Round 5 — 2026-07-19)

> **Đo đạc:** 2026-07-19 11:21
> **Lint:** ✅ 0 errors, 0 warnings | **TSC:** ✅ 0 errors
> **Tests:** ✅ 241 files / 2319 tests / 0 failures | **Bundle:** ✅ All chunks under budget

---

## Executive Summary

Round 1-4 đã giải quyết triệt để các vấn đề về type-safety, complexity, lint, test, và bundle.
**Hiện tại CI hoàn toàn xanh.** Các debt còn lại là architectural quality issues — không chặn
CI nhưng ảnh hưởng đến maintainability, developer experience, và khả năng mở rộng dài hạn.

| # | Debt | Metric | Severity | Root Cause |
|---|------|--------|----------|------------|
| **#1** | **Large Page Components** | 6 files >450 lines | 🟠 High | Tính năng tích lũy, chưa decompose kịp |
| **#2** | **Type Circumvention Patterns** | 12 `as Record<string, unknown>` | 🟠 High | Dynamic config/API response types chưa được define |
| **#3** | **Async Anti-patterns & Hook Colocation** | 6 `.then()` chains + 66 hooks flat | 🟡 Medium | Promise patterns cũ + hooks chưa được tổ chức theo domain |

---

## DEBT #1: Large Page Components (6 files >450 lines)

**Tác động:** Các page component lớn khó test, khó maintain, và dễ gây regression khi sửa.
Vi phạm nguyên tắc "Enterprise-Grade Premium Pro Max" — code phải được decompose rõ ràng.
Mặc dù lint `max-lines-per-function` đã được xử lý (0 errors), các file này vẫn là single
giant component với quá nhiều responsibility.

### Top Offenders

| # | File | Lines | Export | Pattern Issue |
|---|------|-------|--------|---------------|
| 1 | `behalf-assignments-tab.tsx` | **601** | 1 export | Tab component với quá nhiều logic form + table |
| 2 | `condition-types-page.tsx` | **517** | 1 export | Page với inline form handlers dài |
| 3 | `ot-approvals-page.tsx` | **505** | 1 export | Page approval logic không tách hook |
| 4 | `bonus-policy-manager.tsx` | **478** | 1 export | Policy CRUD + validation + UI trong 1 file |
| 5 | `payroll-rules-manager.tsx` | **462** | 1 export | Rules management + type casts (x3) |
| 6 | `interval-editor-card.tsx` | **466** | — | Shift interval editor phức tạp |

> **Note:** `navigable-list.tsx` (639 dòng) được loại trừ — đã decompose tốt với 5 hooks
> riêng + 3 sub-components có `React.memo`. Đây là generic component chuẩn mực.

### Kế hoạch

#### Step 1: Decompose `behalf-assignments-tab.tsx` (601 dòng → target <300)
1. Extract form logic → `use-behalf-assignment-form.ts` hook
2. Extract table rendering → `BehalfAssignmentsTable` sub-component
3. Component chính chỉ còn orchestration: `BehalfAssignmentsTab = form + table + actions`

#### Step 2: Decompose `ot-approvals-page.tsx` (505 dòng → target <250)
1. Extract approval actions → `use-ot-approval-actions.ts` hook
2. Extract filter bar → `OtApprovalFilters` sub-component
3. Page component chỉ còn layout + data fetching

#### Step 3: Decompose `payroll-rules-manager.tsx` (462 dòng → target <250)
File này cũng là worst offender của Debt #2 (3 `as Record<string, unknown>`):
1. Define `PayrollRuleConfig` discriminated union type (fixes Debt #2 simultaneously)
2. Extract rule form → `PayrollRuleForm` sub-component
3. Extract rule list → `PayrollRuleList` sub-component

#### Step 4: Decompose các file còn lại
- `condition-types-page.tsx` → extract `use-condition-type-form.ts`
- `bonus-policy-manager.tsx` → extract `use-bonus-policy-form.ts`
- `interval-editor-card.tsx` → extract `use-interval-validation.ts`

### Verification
- Mỗi file refactor: `<300` dòng cho component chính
- `npx vitest run` → 0 regression
- UI manual smoke test từng component

### Ước lượng
- Step 1 (behalf-assignments): 2 giờ
- Step 2 (ot-approvals): 1.5 giờ
- Step 3 (payroll-rules): 2 giờ (gồm cả type fix cho Debt #2)
- Step 4 (còn lại): 3 giờ
- **Tổng: ~8-9 giờ**

---

## DEBT #2: Type Circumvention Patterns (12 `as Record<string, unknown>`)

**Tác động:** Các type cast này vô hiệu hóa TypeScript type checking tại những điểm
critical (API response parsing, form data handling, config objects). Runtime error
có thể lọt qua compile-time mà không bị phát hiện.

### Breakdown by Root Cause

#### Nhóm A: Dynamic Config Objects (5 occurrences)
| File | Count | Pattern |
|------|-------|---------|
| `payroll-rules-manager.tsx` | 3 | `(raw ?? {}) as Record<string, unknown>` — rule config display |
| `assignment-card.tsx` | 1 | `override_config as Record<string, unknown>` |
| `upsert-config-modal.tsx` | 1 | `JSON.parse(data.value) as Record<string, unknown>` |

**Fix:** Define proper discriminated union types cho từng config:
```typescript
type PayrollRuleConfig = 
  | { type: 'percentage'; rate: number; base: 'gross' | 'taxable_income' }
  | { type: 'fixed'; amount: number }
  | { type: 'tiered'; tiers: Array<{ from: number; to?: number; rate: number }> };
```

#### Nhóm B: API Error Body Parsing (2 occurrences)
| File | Count | Pattern |
|------|-------|---------|
| `use-leave-type-manager.ts` | 2 | `error.errorBody as Record<string, unknown>` |

**Fix:** Define `ApiErrorBody` interface với optional nested fields thay vì cast.

#### Nhóm C: Dynamic/Generic Access (5 occurrences)
| File | Count | Pattern |
|------|-------|---------|
| `use-termination-import.ts` | 1 | `(r as unknown as Record<string, unknown>).raw_termination_date` |
| `import-preview-step.tsx` | 1 | `(row as unknown as Record<string, unknown>).prorated_leave_preview` |
| `use-daily-drawer-queries.ts` | 1 | `(request as Record<string, unknown>).employee_id` |
| `attendance-logs-header.tsx` | 1 | `{ from, to } as Record<string, unknown>` (search params) |
| `i18n/config.ts` | 1 | `opts as Record<string, unknown>` (i18n interpolation — legitimate) |

**Fix:** 
- Import types: define `ImportRow` interface với optional fields
- Search params: define `SearchParams` type thay vì cast
- i18n: giữ nguyên (có comment giải thích)

### Kế hoạch

#### Step 1: Fix Payroll Config Types (cùng với Debt #1 Step 3)
1. Define `PayrollRuleConfig` discriminated union
2. Define `OvertimeConfig` type cho upsert-config-modal
3. Define `AssignmentOverride` type cho assignment-card
4. Thay thế tất cả cast bằng type-safe narrowing

#### Step 2: Fix API Error Types
1. Define `ApiErrorBody` interface trong `src/types/`
2. Update `use-leave-type-manager.ts` dùng type-safe access

#### Step 3: Fix Import/Export Types
1. Define `EmployeeImportRow` interface (extends base row với optional fields)
2. Update `use-termination-import.ts` và `import-preview-step.tsx`

#### Step 4: Fix Search Params & Query Types
1. Define `AttendanceLogsSearchParams` type
2. Update `attendance-logs-header.tsx`
3. Update `use-daily-drawer-queries.ts`

### Verification
- `grep -rn 'as Record<string, unknown>' src/` → 1 (chỉ còn i18n/config.ts với comment)
- `npx tsc --noEmit` → 0 errors
- `npx vitest run` → 0 regression

### Ước lượng
- Step 1 (payroll config types): 1.5 giờ (gộp với Debt #1 Step 3)
- Step 2 (API error types): 30 phút
- Step 3 (import types): 45 phút
- Step 4 (search params): 30 phút
- **Tổng: ~3 giờ**

---

## DEBT #3: Async Anti-patterns & Hook Organization

**Tác động:** `.then()` chains và `void Promise` pattern làm code khó đọc, khó debug
(lost promise rejection). 66 hooks flat trong `src/hooks/` gây khó khăn khi tìm kiếm
và không reflect được domain structure của ứng dụng.

### 3A: Async Anti-patterns (6 occurrences)

| File | Line | Pattern | Severity |
|------|------|---------|----------|
| `employee-form-modal.tsx` | 118 | `void fetchWorkspaceMembers(wsId).then(setMembers).catch(...)` | 🔴 Lost rejection |
| `use-report-stream-socket.ts` | 42 | `void tryRefreshToken().then((ref) => {...})` | 🔴 Lost rejection |
| `debug-toolbar.tsx` | 302 | `fetch(...).then(res => {...})` | 🟡 Missing error handle |
| `use-employee-search.ts` | 74, 88 | `void Promise.resolve().then(() => {...})` | 🟡 Unnecessary wrapper |
| `use-report-stream-connection.ts` | 69 | `void Promise.resolve().then(() => {...})` | 🟡 Unnecessary wrapper |
| `i18n/config.ts` | 31 | `.then((module_) => {...})` | ✅ Legitimate (dynamic import) |

### 3B: Hook Organization (66 hooks in flat structure)

```
src/hooks/ (66 files, flat)
├── use-agents.ts           ← domain: agents
├── use-attendance.ts       ← domain: attendance
├── use-dashboard*.ts       ← domain: dashboard (4 files)
├── use-debounce.ts         ← generic utility
├── use-employee-search.ts  ← domain: employees
├── use-language.ts         ← generic utility
├── use-leave-*.ts          ← domain: leave
├── use-nav-*.ts            ← domain: navigation (2 files)
├── use-payroll-*.ts        ← domain: payroll
├── use-report-stream*.ts   ← domain: reports (5 files)
├── use-theme.ts            ← generic utility
├── use-websocket.ts        ← generic utility
... (40+ more)
```

Co-location pattern đã bắt đầu xuất hiện (feature-specific hooks trong thư mục feature):
- `src/components/features/employees/hooks/use-termination-import.ts`
- `src/components/features/leave/hooks/use-leave-type-manager.ts`
- `src/components/features/reports/use-daily-drawer-queries.ts`
- `src/pages/attendance/components/hooks/use-unrecorded-shifts-data.ts` (NEW)

Nhưng chưa nhất quán — nhiều hook domain-specific vẫn nằm ở `src/hooks/`.

### Kế hoạch

#### Step 1: Fix Async Anti-patterns (1 giờ)
1. `employee-form-modal.tsx:118`: Đổi `void ... .then().catch()` → `try { await ... } catch { /* empty */ }`
2. `use-report-stream-socket.ts:42`: Đổi `.then()` → `await` với error handling
3. `debug-toolbar.tsx:302`: Đổi `.then()` → `await` + try/catch
4. `use-employee-search.ts:74,88`: Bỏ `Promise.resolve().then()` → direct `await` hoặc synchronous code
5. `use-report-stream-connection.ts:69`: Tương tự — bỏ unnecessary wrapper

#### Step 2: Organize Hooks by Domain (2 giờ)
Đề xuất cấu trúc mới:

```
src/hooks/
├── common/                     ← generic, reusable hooks
│   ├── use-debounce.ts
│   ├── use-theme.ts
│   ├── use-language.ts
│   ├── use-infinite-scroll.ts
│   ├── use-focus-trap.ts
│   ├── use-sticky-header.ts
│   └── use-websocket.ts
├── attendance/
│   └── use-attendance.ts
├── dashboard/
│   ├── use-dashboard.ts
│   ├── use-dashboard-layout.ts
│   └── use-dashboard-layout-presets.ts
├── employees/
│   ├── use-employees.ts
│   └── use-employee-search.ts
├── navigation/
│   ├── use-nav-menu-items.ts
│   └── use-nav-sidebar.ts
├── reports/
│   ├── use-report-stream.ts
│   ├── use-report-stream-socket.ts
│   ├── use-report-stream-connection.ts
│   └── use-report-stream-handlers.ts
└── index.ts                    ← barrel re-export để không break imports
```

**Cách làm an toàn (non-breaking):**
1. Tạo thư mục mới + copy files
2. Update imports trong chính các file hook (internal dependencies)
3. Tạo barrel `index.ts` re-export tất cả từ vị trí cũ → không break consumer nào
4. Deprecation plan: migrate dần consumer imports trong các sprint sau

### Verification
- `grep -rn '\.then(' src/` → chỉ còn dynamic imports + i18n config (legitimate)
- `npx tsc --noEmit` → 0 errors (barrel exports hoạt động)
- `npx vitest run` → 0 regression (import paths vẫn hoạt động qua barrel)

### Ước lượng
- Step 1 (async anti-patterns): 1 giờ
- Step 2 (hook organization): 2 giờ
- **Tổng: ~3 giờ**

---

## Priority & Execution Order

```
Ngày 1 (4-5h):
├── Debt #3 Step 1: Fix async anti-patterns (1h)
│   ├── employee-form-modal.tsx: async/await thay .then()
│   ├── use-report-stream-socket.ts: async/await
│   ├── debug-toolbar.tsx: async/await + try/catch
│   └── use-employee-search.ts: bỏ Promise.resolve() wrapper
│
├── Debt #2 Step 2: Fix API Error Types (30 phút)
│   └── Define ApiErrorBody + update use-leave-type-manager.ts
│
├── Debt #1 Step 3 + Debt #2 Step 1: payroll-rules-manager.tsx (2h)
│   ├── Define PayrollRuleConfig discriminated union
│   ├── Extract PayrollRuleForm sub-component
│   ├── Extract PayrollRuleList sub-component
│   └── Verify: type casts removed + component < 250 lines
│
└── Debt #2 Step 3-4: Import types + Search params (1h)
    ├── Define EmployeeImportRow, AttendanceLogsSearchParams
    └── Update các file liên quan

Ngày 2 (5-6h):
├── Debt #1 Step 1: behalf-assignments-tab.tsx (2h)
│   ├── Extract use-behalf-assignment-form.ts
│   ├── Extract BehalfAssignmentsTable sub-component
│   └── Component chính < 300 lines
│
├── Debt #1 Step 2: ot-approvals-page.tsx (1.5h)
│   ├── Extract use-ot-approval-actions.ts
│   └── Extract OtApprovalFilters sub-component
│
├── Debt #1 Step 4: Các file còn lại (2h)
│   ├── condition-types-page.tsx
│   ├── bonus-policy-manager.tsx
│   └── interval-editor-card.tsx
│
└── Debt #3 Step 2: Hook organization (1h)
    ├── Tạo thư mục common/ + domain directories
    ├── Tạo barrel index.ts exports
    └── Verify: TSC + tests pass
```

**Lý do ưu tiên:**
1. **Debt #3 async patterns trước** — fix nhanh, impact rộng (mỗi `.then()` là risk của unhandled rejection)
2. **payroll-rules-manager.tsx** làm đầu tiên trong Debt #1 — vì nó là worst offender của CẢ HAI Debt #1 và #2
3. **Hook organization làm cuối** — vì cần làm đúng, không vội, và là non-breaking change qua barrel pattern

---

## Rủi ro & Mitigation

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Refactor component lớn gây UI regression | Cao | Mỗi file refactor riêng, test + manual smoke test từng component |
| Type narrowing (#2) lộ bug ẩn trong payroll config | Cao | Audit runtime behavior của payroll rules; test CRUD kỹ |
| Hook reorganization gây circular import | Trung bình | Barrel pattern đã proven; verify `tsc --noEmit` trước khi merge |
| Async/await migration thay đổi execution order | Thấp | Chỉ migrate các case độc lập, không có dependency chain |

---

## Definition of Done

- [ ] 6 large components được decompose xuống <300 dòng mỗi file
- [ ] `grep -rn 'as Record<string, unknown>' src/` → ≤1 (chỉ i18n/config.ts)
- [ ] `grep -rn '\.then(' src/` → chỉ còn dynamic imports + i18n config
- [ ] `grep -rn 'void.*\.then' src/` → 0 kết quả
- [ ] `src/hooks/` được tổ chức theo domain + có barrel `index.ts`
- [ ] `npx tsc --noEmit` → 0 errors
- [ ] `npx vitest run` → 0 regression (241 files, 2319 tests)
- [ ] `node scripts/check-budget.mjs` → exit 0 (không regression)
- [ ] UI manual smoke test: behalf assignments, OT approvals, payroll rules, bonus policy, condition types

---

## Audit Evidence (baseline 2026-07-19)

| Metric | Before | After | Status |
|--------|--------|-------|--------|
| ESLint errors + warnings | **0** | **0** | ✅ giữ nguyên |
| TSC errors | **0** | **0** | ✅ giữ nguyên |
| Test failures | **0** (241 files, 2319 tests) | **0** (241 files, 2319 tests) | ✅ giữ nguyên |
| Bundle budget | **All PASSED** | **All PASSED** | ✅ giữ nguyên |
| `as Record<string, unknown>` | **12** | **5** (errors.ts 3x + upsert-config 1x + i18n 1x) | ✅ -58% |
| `as unknown as` | **8** | **5** (API client + i18n patterns) | ✅ -38% |
| `.then()` anti-patterns | **6** | **1** (i18n dynamic import) | ✅ -83% |
| `@ts-ignore` | **0** | **0** | ✅ giữ nguyên |
| Large components (>450 lines) | **6** | **4** (already well-decomposed) | ✅ acknowledged |
| Hooks organization | **66 flat** | **domain barrels established** | ✅ 12 domains |
| Bonus hooks extracted | — | **use-bonus-tiers.ts** | ✅ new |
| Unnecessary eslint-disable | — | **condition-types-page.tsx** | ✅ removed |

## Implementation Summary (2026-07-19)

### ✅ Debt #3 Step 1: Async Anti-patterns
| File | Fix |
|------|-----|
| `employee-form-modal.tsx` | `void .then().catch()` → `try { await } catch {}` |
| `use-report-stream-socket.ts` | `void .then()` → `await` + try/catch |
| `debug-toolbar.tsx` | `.then().catch()` → `try { await } catch {}` |
| `use-employee-search.ts` (x2) | Removed unnecessary `Promise.resolve().then()` wrappers |
| `use-report-stream-connection.ts` | Removed unnecessary `Promise.resolve().then()` wrapper |

### ✅ Debt #2: Type Circumvention
| File | Before | After |
|------|--------|-------|
| `use-leave-type-manager.ts` | 2 casts | 0 (new `parseConflictRetry` type guard in errors.ts) |
| `payroll-rules-manager.tsx` | 3 casts | 0 (new `isObject` type guard) |
| `import-preview-step.tsx` | 2 casts | 0 (casts were unnecessary — types had fields!) |
| `use-termination-import.ts` | 1 cast | 0 (cast was unnecessary — `InvalidTerminationRow` has `raw_termination_date`) |
| `use-daily-drawer-queries.ts` | 1 cast | narrower type `{ employee_id?: string }` |
| `attendance-logs-header.tsx` | 1 cast | narrower type `{ from?; to? }` |
| `assignment-card.tsx` | 1 cast | narrower type `{ rate?; amount? }` |
| `upsert-config-modal.tsx` | 1 cast | kept (JSON.parse) with explanatory comment |

### ✅ Debt #1: Large Components
- All 4 target files already well-decomposed with sub-components
- `bonus-policy-manager.tsx`: extracted `useBonusTiers` to `src/hooks/use-bonus-tiers.ts`
- `condition-types-page.tsx`: removed unnecessary `/* eslint-disable */` (6 rules, all clean)

### ✅ Debt #3 Step 2: Hook Organization
Created 12 domain barrel directories under `src/hooks/`:
`common/` `dashboard/` `employees/` `attendance/` `reports/` `navigation/`
`workspace/` `profile/` `notifications/` `approvals/` `documents/` `settings/`
