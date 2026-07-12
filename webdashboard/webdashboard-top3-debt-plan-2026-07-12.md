# Kế hoạch Top 3 Nợ Kỹ thuật Webdashboard (2026-07-12)

> **Đo đạc:** 2026-07-12 10:57
> **TSC:** ✅ 0 errors | **Tests:** ✅ 236 files / 2223 tests / 0 fail | **Bundle:** ✅ All chunks under budget
> **Lint:** 🔴 **111 errors** | 🟡 **521 warnings**

---

## Executive Summary

Plan cũ (2026-07-11) + 2 commits `ca3d7311d`, `426c17637` đã giải quyết xong:
type-safety `SafeClient`, bundle budgets, top-5 monoliths.

**Sau đo đạc:** TSC, test, bundle đều sạch. **100% nợ còn lại nằm ở 111 lint errors.**
Không còn debt "nghi ngờ" — số liệu rõ ràng:

| # | Debt | Count | Severity | Có thể auto-fix? |
|---|------|-------|----------|-------------------|
| **#1** | **Complexity + max-lines-per-function** | 62 errors | 🔴 Hard error (block CI) | ❌ Cần refactor thủ công |
| **#2** | **Type-safety residue** | ~15 errors + 114 warnings | 🔴 error 🟡 warn | ❌ Cần phân tích logic |
| **#3** | **Code style consistency** | ~34 errors + 400+ warnings | 🔴 error 🟡 warn | ✅ ~90% auto-fix qua `--fix` |

---

## DEBT #1: Complexity + Long Functions (62 errors)

**Tác động:** Cao nhất. Mỗi error là 1 hàm có complexity >10 hoặc >150 dòng — rủi ro
bug cao, không test được, maintainer sau khó hiểu.

### Top targets (từ lint output, xếp theo severity):

| File | Function | Problem |
|------|----------|---------|
| `leave-type-manager.tsx` | — | Arrow fn complexity **24** |
| `attendance-import-dialog.tsx` | — | Arrow fn complexity **24** |
| `shift-editor-page.tsx` | — | Arrow fn complexity **20** |
| `payroll-table.tsx` | `PayrollDetailDrawer` | complexity **15** |
| `use-assign-policy.ts` | `useAssignPolicy` | complexity **15** |
| `monthly-roster-grid.tsx` | `PatternCard` | complexity **20** |
| `employee-form-modal.tsx` | `EmployeeFormModal` | complexity **29** |
| `leave-type-manager.tsx` | `EditLeaveTypeModal` | **180 dòng** |
| `e-contract-details-drawer.tsx` | `EContractDetailsDrawer` | **212 dòng** |
| `smart-table.tsx` | `SmartTable` | **254 dòng** |
| `chart-catalog-sidebar.tsx` | `ChartCatalogSidebar` | **203 dòng** |
| `use-smart-table-keyboard.ts` | `useSmartTableKeyboard` | **161 dòng** |
| `create-adjustment-modal.tsx` | `CreateAdjustmentModal` | **161 dòng** |
| `use-monthly-roster.ts` | `useMonthlyRoster` | **238 dòng** |
| `policy-assignments-section.tsx` | `PolicyAssignmentsSectionContent` | **202 dòng** |

### Kế hoạch

**Cách tiếp cận:** Pattern đã chuẩn từ commit 426c — tách logic thành:
- `useXxx` hook (state + handlers)
- `XxxSubComponent` (render từng phần)
- Component chính giữ layout + orchestration

**Ưu tiên 3 file có impact lớn nhất:**
1. `employee-form-modal.tsx` (complexity **29**) — ảnh hưởng employee CRUD
2. `smart-table.tsx` (**254 dòng**) — generic component, ảnh hưởng toàn bộ table trong app
3. `shift-editor-page.tsx` (complexity **20**) — core feature

**Phần còn lại:** Sau khi sửa top-3, gỡ từng file khỏi lint `ignores` allowlist,
xử lý dần 15+ file còn lại trong các sprint tiếp.

### Kiểm tra
- `npx eslint .` → 0 `complexity` / `max-lines-per-function` error ở file đã refactor.
- `npx vitest run` → 0 regression.
- UI test thủ công từng component.

---

## DEBT #2: Type-safety Residue (~15 errors + key warnings)

**Tác động:** Type-safety là ưu tiên cao trong project (CLAUDE.md: "zero hardcoded
secrets, strict AppError handling"). 15 errors + 114 `no-unnecessary-condition` warnings
cho thấy còn sót type escape hatches.

### Error breakdown:

| Rule | Count | Ví dụ |
|------|-------|-------|
| `@typescript-eslint/no-explicit-any` | ~10 | `as any`, `: any` type annotation |
| `@typescript-eslint/consistent-type-definitions` | ~5 | Dùng `type` thay vì `interface` |
| `@typescript-eslint/no-empty-object-type` | ~1 | `{}` empty object type |
| **Warning:** `no-unnecessary-condition` | 114 | `if (x !== undefined)` khi x không thể undefined |
| **Warning:** `prefer-nullish-coalescing` | 44 | `a || b` thay vì `a ?? b` |

### Kế hoạch

**Step 1: Dọn `no-explicit-any` (10 errors)** — review từng case, thay bằng:
- `unknown` + type guard nếu dynamic
- Type cụ thể từ `api-generated.ts` nếu là API response
- `Record<string, unknown>` cho config objects

**Step 2: Sửa type definition consistency (5 errors)** — thay `type Foo = { ... }` thành
`interface Foo { ... }` ở các non-union types.

**Step 3: Xử lý `no-unnecessary-condition` top-offenders** — 114 warnings, tập trung
vào các file có nhiều nhất. Dùng `tsc --noUnusedLocals` để tìm dead branches.

### Kiểm tra
- `npx eslint .` → 0 `no-explicit-any`, `consistent-type-definitions`, `no-empty-object-type`.
- `npx vitest run` → 0 regression.

---

## DEBT #3: Code Style Consistency (~34 errors + auto-fixable warnings)

**Tác động:** Thấp nhất về correctness, nhưng cao về số lượng (400+ warnings).
Cản trở maintainer vì lint noise che các lỗi thực sự. Rất nhiều trong số này có thể
**auto-fix**.

### Error + warning mix:

| Rule | Count | Auto-fix? |
|------|-------|-----------|
| `perfectionist/sort-imports` | 300+ | ✅ `--fix` |
| `@typescript-eslint/array-type` | ~20 | ✅ `--fix` |
| `no-console` | ~20 | ✅ `--fix` (xóa) |
| `import-x/no-named-as-default` | ~20 | ❌ manual (import rename) |
| Misc (`no-unused-imports`, `perfectionist/sort-jsx-props`, etc.) | ~50 | ✅ `--fix` |

### Kế hoạch

**Step 1: Auto-fix hàng loạt** (an toàn vì là style-only):
```bash
cd webdashboard
npx eslint . --fix
```
Lệnh này fix được: `sort-imports`, `array-type`, `sort-jsx-props`, `sort-interfaces`,
`no-console`, `no-unused-imports`. Ước tính **~300+ warnings** và **~15 errors**
biến mất.

**Step 2: Manual fix phần còn lại** — khoảng 5-10 errors + 20 warnings:
- `import-x/no-named-as-default`: đổi `import i18n from "i18next"` thành
  `import { t } from "i18next"` hoặc dùng `default` import đúng cách
- `@typescript-eslint/only-throw-error`: wrap string errors thành `new Error(...)`
- `unicorn/filename-case`: rename file nếu cần

**Step 3: Verify** — chạy `npx eslint .` sau auto-fix để thấy số liệu sạch hơn,
từ đó xác định manual-fix list chính xác.

### Kiểm tra
- `npx eslint .` → warnings giảm ~50% (từ 521 xuống ~200), errors giảm ~30% (từ 111 xuống ~75).
- Run full test suite → 0 regression.
- Auto-fix thuần túy style (import sort, array syntax, JSX prop order) → zero logic change → zero risk.

---

## Priority & Execution Order

```
Phase 1: Debt #3 — Auto-fix hàng loạt (15 phút)
  ├── npx eslint . --fix
  ├── Verify: test pass + lint count drops
  └── Commit: "chore(webdashboard): auto-fix 300+ style violations"

Phase 2: Debt #2 — Type-safety residue (2-3 giờ)
  ├── Sửa ~10 no-explicit-any
  ├── Sửa ~5 type-definition inconsistency
  ├── Commit từng nhóm rule
  └── Verify: test pass

Phase 3: Debt #1 — Complexity reduction (1-2 ngày, incremental)
  ├── Top-3 files trước (employee-form-modal, smart-table, shift-editor-page)
  ├── Refactor pattern: tách hook + sub-component
  ├── Gỡ ignore từng file khỏi eslint.config.mjs
  └── Các file còn lại trong sprint tiếp theo
```

**Lý do làm #3 trước:** Auto-fix an toàn, giảm noise ngay lập tức → dễ nhìn thấy các
lỗi thực sự cần sửa ở #1 và #2.

---

## Rủi ro

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Refactor complexity (#1) break UI | Cao | Từng file, test từng component |
| Thay `any` (#2) lộ bug ẩn | Trung bình | Review kỹ từng case, không thay hàng loạt |
| `--fix` thay đổi import order (#3) gây circular import | Thấp | Commit riêng, revert dễ |
| Auto-fix chạm vào file test (disabled type-check) | Thấp | Test files được config riêng, --fix chỉ chạm style rules |

## Kết luận

1. **Debt #1 (Complexity):** 62 errors, nghiêm trọng nhất — refactor incremental top-3 file trước.
2. **Debt #2 (Type-safety):** 15 errors + 114 warnings — dọn dẹp `any` + unnecessary-condition.
3. **Debt #3 (Style):** 400+ auto-fixable — làm trước để giảm noise.

Tổng ước lượng: **~3 ngày** (auto-fix 15 phút, #2 ~3h, #1 ~2 ngày).
