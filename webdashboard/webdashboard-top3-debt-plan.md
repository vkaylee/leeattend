# Kế hoạch giải quyết Top 3 Nợ Kỹ thuật Webdashboard

> Ngày: 2026-07-11
> Tổng quan: 163k LOC, 1065 files TS, 2214 tests
> Trạng thái hiện tại: 143 TS errors (136x TS2322 + 7x TS18048), 109 ESLint errors, 396 ESLint warnings

---

## Mục lục

1. [Debt #1: OpenAPI Client Type Unsafe](#debt-1-openapi-client-type-unsafe)
2. [Debt #2: Bundle Size Bloat](#debt-2-bundle-size-bloat)
3. [Debt #3: Monolithic Components](#debt-3-monolithic-components)

---

## Debt #1: OpenAPI Client Type Unsafe

**Priority: 🔴 Cao nhất — Systemic**

### Vấn đề

`openapi-fetch` trả về `{ data: T | undefined }` cho mọi response. 136 function trong 22 file API module không xử lý `undefined` này, dẫn đến:

```
src/lib/api/attendance.ts:         24 lỗi TS2322
src/lib/api/payroll.ts:            21 lỗi
src/lib/api/workspaces.ts:          13 lỗi
src/lib/api/lifecycle.ts:           13 lỗi
src/lib/api/leaves.ts:              12 lỗi
src/lib/api/ess.ts:                 11 lỗi
src/lib/api/employees.ts:           10 lỗi
... (15 file khác)
```

Hai pattern khác nhau tồn tại song song:
- **Pattern A** (~70 functions): dùng `client.GET/POST/PUT/DELETE` — destructure `{ data: resData }` từ openapi-fetch, gán thẳng cho kiểu `T` thay vì `T | undefined`
- **Pattern B** (~20 functions): dùng `fetch<T>()` wrapper (từ client.ts) — wrapper vẫn giữ nguyên chữ ký `Promise<{ data: T }>` (không bao gồm `undefined`), nên bản thân wrapper đã sai kiểu

Ngoài ra, **41+** `throw new Error("No response data")` còn sót lại trong api/ layer. Commit `ca3d7311` đã xóa 251 cái, nhưng chưa hết.

### Kế hoạch xử lý

**Cách tiếp cận: Chỉ sửa 1 file — `client.ts` — không touch 22 file API module**

#### Bước 1: Thêm utility type vào `client.ts`

```typescript
// Thêm vào src/lib/api/client.ts

/**
 * Unwraps openapi-fetch's base response by stripping | undefined from data.
 * The server always returns data on 2xx; undefined only arrives on network
 * errors which customFetch already converts to thrown ApiError.
 * This one-liner eliminates 136 TS2322 errors across 22 API modules
 * without touching a single call site.
 */
export type SafeData<T> = T extends (infer U)[] ? U[] : T;
```

Wait, mục tiêu là làm cho `client.GET/POST/etc` trả về `{ data: T }` thay vì `{ data: T | undefined }`.

Cách chính xác:

```typescript
// src/lib/api/client.ts — thêm helper

/**
 * Safe version of openapi-fetch's client methods.
 * Wraps every method so response.data is T, never T | undefined.
 */
export const safeClient = new Proxy(client, {
  get(target, prop: string) {
    const original = target[prop as keyof typeof target];
    if (typeof original !== 'function') return original;
    return async (...args: Parameters<typeof original>) => {
      const res = await original(...args);
      return res as { data: NonNullable<(typeof res)['data']> };
    };
  },
});
```

Nhưng Proxy có thể chậm. Cách đơn giản hơn:

```typescript
// Override chữ ký trả về — type-level assertion
type SafeClient = {
  [M in 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE']: <P extends keyof paths, R>(
    ...args: Parameters<typeof client[M]>
  ) => Promise<{ data: NonNullable<Awaited<ReturnType<typeof client[M]>> extends { data: infer D } ? D : never> }>;
};
```

Thực tế, đơn giản nhất là tạo type wrapper quanh phương thức:

```typescript
// Trong client.ts

type ClientResponse<R> = R extends Promise<{ data: infer D }> 
  ? Promise<{ data: Exclude<D, undefined> }>
  : R;

// Sau đó tạo typed client wrapper
export const typedClient = {
  GET: <P extends Parameters<typeof client.GET>[0]>(
    url: P, 
    init?: Parameters<typeof client.GET>[1]
  ): ClientResponse<ReturnType<typeof client.GET>> => {
    return client.GET(url as never, init as never) as never;
  },
  POST: ...,
  PUT: ...,
  PATCH: ...,
  DELETE: ...,
};
```

Nhưng cách này verbose. Cách gọn nhất — type helper:

```typescript
/**
 * Strip `| undefined` from the `data` field of openapi-fetch's return type.
 * Server always returns data on 2xx; undefined would be a network error
 * which customFetch already converts to ApiError. This removes 136 TS2322
 * errors from the API layer at the type level.
 *
 * Usage: replace `client.GET(...)` calls with `s.GET(...)` or add `satisfies`
 * to the return type.
 */
type SafeResponse<T> = T extends Promise<{ data: infer D }>
  ? Promise<{ data: Exclude<D, undefined> }>
  : T;

// Export utility — caller-opt-in
export function safe<R>(p: Promise<{ data: R }>): Promise<{ data: Exclude<R, undefined> }> {
  return p as Promise<{ data: Exclude<R, undefined> }>;
}
```

Hmm, nhưng nếu mỗi call site phải thêm `safe()` thì vẫn phải sửa 22 file.

**Cách tiếp cận tối ưu: Sửa chữ ký return type của `customFetch` + dùng mapped type cho toàn bộ client object.**

Thực tế, vì `customFetch` đã xử lý lỗi HTTP (throw `ApiError`), nên `data` sẽ không bao giờ là `undefined` nếu promise resolve. Vấn đề là openapi-fetch's type vẫn để `| undefined`.

Giải pháp: Tạo wrapper type quanh client với chữ ký override:

```typescript
// Trong client.ts — 1 block type, không cần sửa call site nào

type SafeClient = {
  [M in 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE']: <
    P extends Parameters<(typeof client)[M]>[0],
    O extends Parameters<(typeof client)[M]>[1],
  >(
    url: P,
    init?: O,
  ) => Promise<
    { data: Exclude<Awaited<ReturnType<(typeof client)[M]>>['data'], undefined> }
  >;
};

export const typedClient: SafeClient = client as never;
```

Wait, còn cả `fetch<T>()` helper nữa. Nó đã có chữ ký `Promise<{ data: T }>` rồi — nhưng sai ở chỗ nó ép kiểu `client[method]` thành `(p, i) => Promise<{ data: T }>`. Vì openapi-fetch thực tế trả về `{ data: T | undefined }`, assertion này là unsound nhưng TS chấp nhận vì có `as`. Các call site dùng `fetch<T>()` sẽ không bị lỗi.

Vậy **chỉ cần sửa type cho `client.GET/POST/...`**.

#### Bước 2: Thêm type wrapper và xóa fallback guards

Sau khi sửa xong type, dùng codemod (hoặc thủ công) xóa toàn bộ `if (!resData) throw new Error("No response data")` vì chúng không còn cần thiết:

```
File                           | Guards cần xóa
-------------------------------|-----------------
attendance.ts                  | 7 guards
employees.ts                   | 4 guards
working-patterns.ts            | 4 guards
employee-phases.ts             | 4 guards
contracts.ts                   | 2 guards
shift-swaps.ts                 | 3 guards
holidays.ts                    | 2 guards
payroll.ts                     | 1 guard
system.ts                      | 1 guard
departments.ts                 | 1 guard
... (các file khác)            | ~12 guards còn lại
Tổng                           | ~41 guards
```

Các guard không liên quan đến `No response data` (VD: `throw new Error("Workspace not selected")`, `throw new Error("Login failed")`) — giữ nguyên vì chúng là validation logic, không phải type fix.

#### Bước 3: Xử lý `as never` escape hatches

Có **8** `as never` trong production API code phục vụ FormData upload và override type:

```typescript
// attendance.ts (4 cái) — FormData upload, staging-logs
body: formData as never,   // Lý do: openapi-fetch không có type cho multipart
} as never);               // Lý do: params shape không khớp schema

// employees.ts (2 cái) — FormData upload, employee update
body: employee as never,

// shifts.ts (1 cái)
body: data as never,

// agents.ts (1 cái)
body: command as never,
```

Các `as never` cho FormData giữ nguyên — đó là limitation của openapi-fetch với multipart. Các `as never` khác cần kiểm tra từng trường hợp và thay bằng typed alternative.

**Không ưu tiên xử lý `as never` trong đợt này vì chúng là những trường hợp đặc biệt có lý do chính đáng.**

### Files cần sửa

| File | Thay đổi |
|------|----------|
| `src/lib/api/client.ts` | +15 dòng type wrapper `SafeClient` |
| `src/lib/api/attendance.ts` | Xóa ~7 guards `if (!resData)` |
| `src/lib/api/employees.ts` | Xóa ~4 guards |
| `src/lib/api/working-patterns.ts` | Xóa ~4 guards |
| `src/lib/api/employee-phases.ts` | Xóa ~4 guards |
| `src/lib/api/contracts.ts` | Xóa ~2 guards |
| `src/lib/api/shift-swaps.ts` | Xóa ~3 guards |
| `src/lib/api/holidays.ts` | Xóa ~2 guards |
| `src/lib/api/payroll.ts` | Xóa ~1 guard |
| `src/lib/api/system.ts` | Xóa ~1 guard |
| `src/lib/api/departments.ts` | Xóa ~1 guard |
| ~5 file khác | Xóa ~12 guards |

### Kiểm tra

1. `npx tsc --noEmit` — 136 TS2322 giảm xuống 0
2. `npm test` — 2214 tests pass, 0 regressions

---

## Debt #2: Bundle Size Bloat

**Priority: 🟠 Cao — Ảnh hưởng trực tiếp đến end-user load time**

### Vấn đề

Phân tích `dist/assets/` cho thấy 2 vi phạm budget + 1 chunk quá lớn:

| Chunk | Raw size | Budget | Gzip | Vượt? |
|-------|----------|--------|------|-------|
| `vendor-exceljs` | **937 KB** | — | — | ❌ Quá lớn (Riêng chunk) |
| `vendor-query` | **279 KB** | ≤200 KB | ≤60 KB | ❌ |
| `reports.lazy` | **158 KB** | ≤300 KB | ≤60 KB | ✅ (trong lazy budget) |
| `locale-vi` | **190 KB** | ≤150 KB | ≤50 KB | ❌ |

Total JS: **6.5 MB** (7.6 MB dist/ tổng cộng bao gồm fonts, svg, etc.)

### Kế hoạch xử lý

#### Step 1: Tối ưu `vendor-query` (279KB → ~150KB)

**Nguyên nhân:** `@tanstack/react-router` + `@tanstack/react-query` + router-vite-plugin đang gộp vào `vendor-query` (dòng 223 eslint.config: `normalized.includes("node_modules/@tanstack/")`).

**Giải pháp:** Tách router ra chunk riêng hoặc main bundle:
```typescript
// vite.config.ts — sửa manualChunks
if (normalized.includes("node_modules/@tanstack/")) {
  if (normalized.includes("react-router") || normalized.includes("router-devtools")) {
    return "vendor-router";  // ~120KB riêng
  }
  return "vendor-query";     // còn ~150KB
}
```

#### Step 2: Giảm `locale-vi` (190KB → ~120KB)

**Nguyên nhân:** File JSON locale tiếng Việt có nhiều key không dùng đến.

**Giải pháp:**
1. Chạy `scripts/locales/reconcile_locales.js` để detect keys không dùng:
   ```
   node scripts/check-i18n.mjs src
   ```
2. Xóa keys không dùng khỏi locale JSON files
3. Kiểm tra bundle lại

Nếu không đủ, cân nhắc nén JSON keys (ngắn hơn) hoặc code-split locale theo page.

#### Step 3: Xác nhận `vendor-exceljs` (937KB) đã được dynamic import

Kiểm tra: `exceljs` đã dynamic import (`await import("exceljs")`) trong:
- `src/lib/excel-builder.ts` — ✅ dynamic
- `src/lib/excel-no-show.ts` — ✅ dynamic
- `src/pages/attendance/hooks/use-attendance-adjustments-page.ts` — ✅ dynamic
- `src/components/features/employees/employee-termination-import-dialog.tsx` — ✅ dynamic
- `scripts/check-budget.mjs` — **Không có budget cho exceljs trong BUDGETS array**

**Hành động:**
1. Thêm budget entry cho `vendor-exceljs` vào `scripts/check-budget.mjs`:
   ```javascript
   { name: "ExcelJS", prefix: "vendor-exceljs-", suffix: ".js", maxSize: 950_000, maxGzip: 280_000 },
   ```
2. Nếu muốn giảm dung lượng, cân nhắc thay thế bằng thư viện nhẹ hơn (`xlsx` ~200KB) — **ưu tiên thấp** vì có thể break API

#### Step 4: Tối ưu locale chunks với code splitting theo page

Nếu locale-vi vẫn >150KB sau cleanup, implement lazy locale loading:
```typescript
// Thay vì import toàn bộ locale JSON cùng lúc
// Đảm bảo mỗi page chỉ load locale keys cho page đó
```

**Đây là ưu tiên thấp — chỉ làm nếu bước 2 không đủ.**

### Files cần sửa

| File | Thay đổi |
|------|----------|
| `vite.config.ts` | Tách `@tanstack/react-router` khỏi `vendor-query` |
| `scripts/check-budget.mjs` | Thêm budget cho `vendor-exceljs` |
| Locale JSON files (vi.json, zh.json, en.json) | Xóa keys không dùng |

### Kiểm tra

1. `npm run build && node scripts/check-budget.mjs` — budget pass
2. `npm run analyze` — visualizer kiểm tra chunk sizes
3. Kiểm tra thủ công: app load không mất chunk nào quan trọng

---

## Debt #3: Monolithic Components

**Priority: 🟠 Trung bình-Cao — Ảnh hưởng maintainability & testability**

### Vấn đề

ESLint report: **109 errors** (chủ yếu `complexity` + `max-lines-per-function`).

Top components cần refactor gấp (theo mức độ nghiêm trọng):

| Component | File | Lines | Complexity | Lý do cần refactor |
|-----------|------|-------|------------|-------------------|
| EmployeeTerminationImportDialog | employee-termination-import-dialog.tsx | 595 | 32 | Nhập/xuất Excel + validation + state management lẫn nhau |
| DailyReportTL (Timeline) | attendance/ | 197 | 26 | Timeline rendering + filter + interaction |
| RosterToolbar | roster-toolbar.tsx | 251 | 27 | Quá nhiều filter + view mode + action buttons |
| Sidebar | sidebar.tsx | — | 30 | Navigation items + permissions + collapse state |
| SmartTable | smart-table.tsx | 254 | 14 | Generic table complexity |
| DailyReportTableRow | attendance/ | 151 | 23 | Conditional rendering quá nhiều |
| DailyReportTimeline | attendance/ | 197 | 26 | Timeline rendering + interactions |
| EmployeeFormModal | employee/ | — | 29 | Form validation + multi-step |
| EContractDetailsDrawer | employee/ | 212 | — | Chi tiết hợp đồng + actions |
| getActiveNavGroup | navigation/ | — | 11 | Logic điều hướng phức tạp |

### Kế hoạch xử lý

**Cách tiếp cận: Refactor incremental — top 5 component có complexity cao nhất**

#### Component 1: EmployeeTerminationImportDialog (595 dòng, complex=32)

**Vấn đề:** Component này chứa:
- File upload + drag-drop
- Excel parsing (dynamic import exceljs)
- Data validation (columns, rows)
- Preview table
- Bulk confirmation dialog
- Error display
- Tất cả trong 1 component

**Refactor:**
1. Tách `useTerminationImport` hook — xử lý file upload, parsing, validation
2. Tách `ImportPreviewTable` component — hiển thị preview data
3. Tách `ImportErrorList` component — hiển thị lỗi validation
4. Component chính chỉ còn layout + orchestration (~150 dòng)

#### Component 2: Sidebar (complex=30)

**Vấn đề:** Navigation rendering + permission checking + collapse state.

**Refactor:**
1. Tách `useSidebarNavigation` hook — lọc items theo permissions
2. Tách `SidebarItem` component — render 1 nav item + children
3. Tách `SidebarCollapseButton` component

#### Component 3: RosterToolbar (251 dòng, complex=27)

**Vấn đề:** Filter bar với quá nhiều state + handlers.

**Refactor:**
1. Tách `useRosterFilters` hook — quản lý filter state
2. Tách `DateRangePicker` component
3. Tách `EmployeeFilter` component
4. Tách `ViewModeSwitch` component

#### Component 4: DailyReportTimeline (197 dòng, complex=26)

**Vấn đề:** Timeline rendering + interactions + conditional logic.

**Refactor:**
1. Tách `TimelineBar` component — render 1 timeline bar
2. Tách `useTimelineInteraction` hook — click/hover/scroll
3. Tách `TimelineLegend` component

#### Component 5: DailyReportTableRow (151 dòng, complex=23)

**Vấn đề:** Quá nhiều cell types + conditional rendering + actions.

**Refactor:**
1. Tách `AttendanceCell` component — render 1 cell dựa trên loại dữ liệu
2. Tách `RowActions` component — menu actions
3. Tách `useRowData` hook — transform row data

### Files cần sửa

| # | File | Thay đổi |
|---|------|----------|
| 1 | `src/components/features/employees/employee-termination-import-dialog.tsx` | Tách → 3 components + 1 hook |
| 2 | `src/components/layout/sidebar.tsx` | Tách → 2 components + 1 hook |
| 3 | `src/components/shifts/roster-toolbar.tsx` | Tách → 3 components + 1 hook |
| 4 | `src/pages/attendance/components/daily-report-timeline.tsx` | Tách → 2 components + 1 hook |
| 5 | `src/pages/attendance/components/daily-report-table-row.tsx` | Tách → 2 components + 1 hook |

### Kiểm tra

1. `npm run lint` — complexity errors cho 5 component này giảm về 0
2. `npm test` — 2214 tests pass
3. Render test thủ công: các chức năng không thay đổi

---

## Lịch trình ưu tiên

```
Sprint 1 (Debt #1): Sửa type safety — ~2-3 giờ
  ├── client.ts: +SafeClient type wrapper (15 dòng)
  └── Xóa ~41 guards `if(!resData)` trong 22 file API
  
Sprint 2 (Debt #2): Bundle size — ~1-2 giờ
  ├── vite.config.ts: tách vendor-router
  ├── check-budget.mjs: thêm budget exceljs
  └── Locales: cleanup keys không dùng
  
Sprint 3 (Debt #3): Monolithic components — ~4-6 giờ
  ├── Top 5 components refactor
  └── Kiểm tra từng component
```

## Rủi ro

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Type change ở client.ts ảnh hưởng runtime | Thấp | Chỉ type-level change, runtime không đổi |
| Bundle refactor làm lệch code-splitting | Trung bình | Kiểm tra với `npm run build` + analyze |
| Component refactor break UI | Cao | Test từng component, không refactor hàng loạt |

---

## Kết luận

1. **Debt #1 (Type Safety)** — Fix ngay vì systemic, 1 block type wrapper xóa 136 lỗi
2. **Debt #2 (Bundle Size)** — Fix nhanh vì ảnh hưởng user, tách router chunk + cleanup locales
3. **Debt #3 (Monolithic Components)** — Fix có chọn lọc top 5 component, để dần trong các sprint

Tổng thời gian ước lượng: **~8-11 giờ** (1-2 ngày làm việc).
