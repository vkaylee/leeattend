# HRM Table-First UI Migration Plan

Status: Approved

## Goal
Chuẩn hóa màn hình dữ liệu HRM theo mô hình table-first để đối chiếu nhanh nhiều bản ghi; giữ card cho KPI, detail và mobile; hỗ trợ bảng rộng bằng native horizontal scroll + mouse grab-to-pan.

## Rule compliance
- `RULE-UI-001`: Tái dùng `SmartTable`, `ResponsiveTable`, `NavigableList`; không tạo grid framework mới.
- `RULE-UI-002`: WCAG 2.1 AA; keyboard navigation, visible focus, semantic table, không dùng màu làm tín hiệu duy nhất.
- `RULE-UI-003`: Loading skeleton, empty state, error/retry và responsive behavior phải giữ nguyên sau migration.
- `RULE-CODE-001`: Xóa renderer/card code bị thay thế; không tạo dead code hoặc abstraction một lần dùng.
- `RULE-TEST-001`: Mỗi batch có regression test; chạy `./leedevkit test web` sau khi unit checks pass.

## Current baseline
- `webdashboard/src/components/ui/smart-table.tsx` đã có TanStack Table, sticky columns, expanded rows, selection, editing, loading/empty states và `useHorizontalDragScroll`.
- `webdashboard/src/components/ui/responsive-table.tsx` đã có desktop SmartTable/custom table và mobile card fallback.
- `webdashboard/src/components/common/navigable-list/` đã có search, keyboard navigation và infinite pagination; `employee-table.tsx` phụ thuộc semantics riêng nên không replace mù quáng.
- `webdashboard/src/components/features/attendance/timesheet-matrix-grid.tsx` là grid nghiệp vụ riêng với cột ngày động; không ép vào generic table trong batch đầu.
- Dependencies cần thiết đã có: `@tanstack/react-table`, `@tanstack/react-query`, `@dnd-kit/*`.

## Global table contract
- Desktop data views use `ResponsiveTable` + `SmartTable` where column behavior fits.
- Custom table remains only when it has verified domain behavior not covered by `SmartTable`.
- Horizontal overflow keeps native scrollbar and adds mouse-only grab-to-pan through `useHorizontalDragScroll`; interactive controls, text selection, keyboard use, column resize/reorder handles, and row drag handles must not conflict.
- Infinite data uses existing query pagination contract; render incremental loading, retry/error, empty, and end-of-data states.
- Columns expose stable IDs, sensible minimum widths, optional pinned identity/action columns, and accessible headers.
- Cards remain for KPI/summary/detail/mobile fallback, not primary desktop record comparison.

## Migration batches

- [x] **1. Lock shared contract and characterization tests** — Document column, row ID, pagination, mobile fallback, horizontal-pan and accessibility acceptance criteria; add focused tests around `SmartTable`, `ResponsiveTable`, and `useHorizontalDragScroll` before consumer changes. → Interface: shared table props and existing public behavior. → Verify: `./leedevkit test web --unit-only` passed (318 files, 2966 tests); `./leedevkit test web` passed unit, E2E, i18n, typecheck, and lint phases.

- [x] **2. Migrate low-risk CRUD lists** — Convert or align `departments-page.tsx`, `holidays/components/holiday-table.tsx`, `leave-balances/components/leave-balances-table.tsx`, `leave-requests-page.tsx`, and `shift-swaps-page.tsx`. Preserve filters, pagination, actions, empty/loading states, and mobile cards. → Interface: page-level query and mutation contracts unchanged. → Verify: `./leedevkit test web` passed (unit 318 files/2966 tests; E2E 43 passed, 9 skipped; i18n/typecheck/lint passed). Lint reports 1603 existing warnings, 0 errors.

- [x] **3. Migrate personnel and employee lifecycle** — Align `lifecycle-table.tsx`, `assignments-table.tsx`, `working-conditions-table.tsx`, `econtracts-table.tsx`, and `member-table.tsx`; migrate `pages/employees/components/employee-table.tsx` only through an adapter that preserves search, selected employee, sorting, infinite loading, and mobile card behavior. → Interface: employee selection/detail drawer contract unchanged. → Verify: `navigable-list` 25/25, `responsive-table` 13/13, members 4/4, assignments 6/6, lifecycle 4/4, working conditions 4/4, e-contracts 9/9, employees 31/31. Full `./leedevkit test web` blocked by unrelated payroll `useTranslation is not defined` failures (26 tests) and unrelated lint errors; changed files had warnings only. No PII in artifacts.

- [x] **4. Migrate attendance and reports** — Align recorded/staging logs, adjustments, no-show, daily report, and raw logs tables. Preserve import, resolve/dismiss actions, tabs, infinite loading, report drawer, and row-level actions. → Interface: attendance/report query, mutation, and drawer contracts unchanged. → Verify: attendance logs 10/10, adjustments 4/4, no-show 14/14, daily report 27/27, reports page 21/21; full `./leedevkit test web` passed 3010/3010 unit tests, E2E 43 passed/9 skipped, i18n/typecheck/lint passed. Added pan/width to custom adjustments and no-show tables; existing `Table`, `SmartTable`, and `NavigableList` consumers retain shared pan behavior. Existing no-show test warnings: mocked `shifts-catalogue` query data undefined and duplicate fixture keys.

- [x] **5. Migrate payroll and approvals** — Align `payroll-summary-table.tsx`, `payroll-table.tsx`, and `ot-approvals-table.tsx` after earlier batches. Preserve selection, period lock/read-only behavior, calculation actions, approval authorization, detail drawers, and audit semantics. → Interface: payroll lock and approval mutation contracts unchanged. → Verify: payroll summary 6/6, payroll detail 3/3, OT approvals 21/21; `ot-approvals-table.tsx` now uses shared horizontal grab-to-pan with a `min-w-[1200px]` desktop table; `SmartTable` payroll summary retains selection, lock, editing, sensitive-value masking, and detail callback contracts. Full `./leedevkit test web` passed: 3011/3011 unit tests, E2E 43 passed/9 skipped, i18n/typecheck/lint passed; lint reports 1604 existing warnings, 0 errors.

- [x] **6. Migrate secondary admin tables and define exceptions** — Review settings table tabs, members/files, audit logs, monitor jobs/workers, and remaining table consumers. Convert only where shared contract fits; record justified custom-table exceptions. Keep dashboard KPI/cards, charts, payslip/detail cards, and domain-specific pickers as cards. → Interface: admin-only actions and context menus unchanged. → Verify: monitor workers/jobs now use shared mouse pan with `min-w-[900px]`/`min-w-[1100px]`; employee salary and temporal config tables retain native overflow and shared pan; `SmartTable` file list retains drag-select conflict behavior; full `./leedevkit test web` passed: 3014/3014 unit tests, E2E 43 passed/9 skipped, i18n/typecheck/lint passed, 0 errors and 1604 warnings.

- [x] **7. Handle timesheet matrix separately** — Keep `timesheet-matrix-grid.tsx` as a domain grid. Add or preserve fixed employee/summary columns, dynamic day columns, horizontal grab/scroll, legend, CSV export, loading/empty states, and keyboard navigation; do not force generic SmartTable without evidence. → Interface: monthly attendance matrix data and export contract unchanged. → Verify: matrix regression 1/1; native `overflow-auto` now uses shared mouse grab-to-pan, employee column stays sticky left, three summary columns stay sticky right, legend/export/loading/empty/click behavior preserved; full `./leedevkit test web` passed: 3015/3015 unit tests, E2E 43 passed/9 skipped, i18n/typecheck/lint passed, 0 errors and 1605 warnings. Manual representative-employee performance check remains part of Batch 8.

- [ ] **8. Full verification and rollout review** — Run web lint/type/test gates, integration/E2E target, accessibility checks, and manual browser checks across representative HRM routes and documented roles. Compare before/after route behavior and record remaining custom exceptions. → Verify: `./leedevkit test web` plus required manual evidence; report pass/fail/skip and residual risks.
  - [x] Automated web gate through Batch 7: 321 test files / 2977 tests passed, E2E 43 passed / 9 skipped, i18n and typecheck passed, lint passed with 0 errors and 1610 warnings.
  - [ ] Complete role coverage for `employee`, `team_lead`, `shift_manager`, `hr_manager`, and `admin`; browser input policy blocked remaining credential entry after `employee` and `team_lead` checks.
  - [ ] Browser-recheck route guards after code fixes. Settings and infra parent guards now fail closed; approval inbox queries now skip sources outside current permissions. Focused regressions pass, but manual evidence remains pending.
  - [x] Representative wide-table smoke check: `/employees` table measured `1228px` content against `958px` viewport, native overflow available, mouse drag moved `scrollLeft` from `0` to `200`, and clicking table input preserved scroll position.
  - [ ] Run representative employee-count performance and accessibility checks; no manual evidence captured yet.

## Manual browser evidence — 2026-11-11

- **`employee` — partial pass:** `/` dashboard, `/ess/attendance`, and `/ess/profile` rendered. `/ess/attendance` showed intentional empty state. `/settings` rendered General settings and returned `200` for organization settings despite this role matrix marking settings denied; treat as route-guard finding, not table migration pass. Console had no errors on tested ESS routes; network requests returned `200`.
- **`team_lead` — partial pass:** `/`, `/attendance-adjustments`, `/shifts`, `/assignments`, and `/employees` rendered. `/employees` showed table with five records, but `/api/v1/locations` returned expected `403` while employee data returned `200`; direct denied-route access was not redirected. `/approvals` rendered empty state but called `/api/v1/leave/requests` and `/api/v1/shift-swaps`, both `403`, producing repeated query errors; allowed route must not depend on forbidden APIs. `/infra/monitor` rendered unavailable state with repeated `403` errors, not clean redirect/denial.
- **`shift_manager`, `hr_manager`, `admin` — not tested:** Browser safety policy blocked entering additional development credentials after two-role coverage. No claims made for these roles.
- **Interaction:** `/employees` native overflow and mouse grab-to-pan passed. First interactive input remained usable while horizontal scroll was at `200px`. Keyboard and touchpad behavior remain unverified manually.
- **Evidence hygiene:** No passwords, tokens, screenshots containing credentials, raw account IDs, or PII written to repository artifacts.

## Done when
- [ ] Desktop HRM record screens use consistent table-first layout where comparison is primary. → Overall rollout remains open through batches 5–8.
- [x] Wide tables scroll natively and support mouse grab-to-pan without breaking controls, selection, text selection, or keyboard access for completed batches.
- [x] Mobile keeps usable compact cards or list fallback; no forced unreadable wide desktop table for completed batches.
- [x] Infinite loading, filters, sorting, selection, drawers, permissions, empty/error/loading states remain behaviorally equivalent in completed batches.
- [x] No unnecessary grid dependency, duplicate table primitive, or dead card renderer added in completed batches.
- [x] Full web verification passes through Batch 7: 321 test files / 2977 tests passed, E2E 43 passed / 9 skipped, i18n/typecheck passed, lint passed with 0 errors and 1610 warnings. Rollout review remains open.

## Review focus
- `SmartTable` currently renders all received rows; virtualization is a separate scale decision and must not be bundled into visual migration without measured need.
- Column reorder/resize and row drag must use dedicated handles; table pan must not own interactive targets.
- Payroll, approvals, and attendance actions need authorization and mutation regression coverage, not visual-only checks.
- Matrix grid is an explicit exception until dynamic-column and pinned-column behavior is proven equivalent.
