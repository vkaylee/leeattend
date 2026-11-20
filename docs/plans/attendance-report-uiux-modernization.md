# Modernize Attendance Report UI/UX

Status: Approved

## Goal
Make `/reports` feel like a modern enterprise attendance workspace: clear hierarchy, one predictable filter system, responsive behavior from 390px to 1920px, and WCAG 2.1 AA interaction without changing report API behavior.

## Rule IDs
- `DESIGN-UX-01`: `.agent/rules/design-rules.md` §2–4 — semantic controls, focus, responsive layout, loading/empty states.
- `WF-MATRIX-01`: `.agent/rules/development-workflow.md` §2–3 — test impact matrix and implementation loop.
- `TEST-GATE-01`: `.agent/rules/testing-standards.md` §1–4 — changed behavior tests plus full applicable target.
- `CODE-FRONTEND-01`: `.agent/rules/coding-standards.md` §6–7 — production UI states and no unnecessary abstractions.

## Product decisions
- Keep existing report data/API contracts.
- Keep two view modes: `Bảng công tháng` and `Tổng hợp giờ`.
- Keep `Xem nhật ký thô` as navigation and `Xuất Excel/CSV` as the single page action.
- Keep current default last-month behavior and existing URL date-range compatibility.
- Use existing `ResponsiveToolbar`, `ReportFilters`, `FilterPanel`, `FilterTrigger`, `SearchInput`, `Button`, and Lucide icons; add no dependency.
- Use one compact month control with quick choices (`Tháng này`, `Tháng trước`, `Chọn tháng`) rather than presenting overlapping month controls.
- Use department/employer selectors as scalable combobox/dropdown controls; retain chips only for active-filter summary.

## Tasks

- [x] 1. Rework `webdashboard/src/pages/reports/components/reports-header.tsx` into a responsive page header: title/subtitle, view-mode tabs, raw-log navigation, and separated export action. Preserve `aria-pressed`, route search params, and matrix-vs-summary export behavior. Verify desktop hierarchy has one active tab and one visually subordinate export action.

- [ ] 2. Recompose `webdashboard/src/components/features/reports/report-filters.tsx`: month selector first, employee-status selector second, department/employer selectors third, search last; remove horizontal clipping and fixed-width label truncation. Add explicit labels/accessible names, selected state, clearable controls, and active-filter summary with `Xóa bộ lọc`. Verify keyboard operation and long department names.

- [ ] 3. Adapt `webdashboard/src/components/ui/responsive-toolbar.tsx` only where required: desktop toolbar remains inline; tablet wraps into stable rows; mobile opens existing bottom sheet with focus trap, Escape close, safe footer, and Apply action. Preserve existing callers and avoid duplicate filter state. Verify no horizontal page scroll at 390px/768px.

- [ ] 4. Update `webdashboard/src/pages/reports-page.tsx` composition: place filters in the new hierarchy, expose result context (`Đang hiển thị ...`), preserve matrix/table rendering, and add loading, error/retry, and empty-result presentation at the report-content boundary using existing components/patterns. Do not alter query semantics or employee drawer behavior.

- [ ] 5. Update locale keys in existing `webdashboard/src/i18n/locales/{vi,en}/...` report/common files for new labels, filter summary, clear action, empty state, retry state, and export loading state. Run locale reconciliation; no hardcoded user-facing strings in new UI.

- [ ] 6. Extend colocated tests: `webdashboard/src/pages/__tests__/reports-page.test.tsx`, `webdashboard/src/components/features/reports/__tests__/report-filters.test.tsx`, and add/update header tests if an existing test location exists. Cover view switching, export action, month boundary labels, active-filter clear, search behavior, empty/error states, mobile sheet open/close, and accessible names. Preserve existing date arithmetic and API-call assertions.

- [ ] 7. Verify visual behavior at `1920x1080`, `1440x900`, `1024x768`, `768x1024`, `390x844`, and `360x800`; check console/network errors, focus order, contrast, reduced motion, and no clipped labels. Use least-privileged documented account for authenticated UI testing and mask all account data in artifacts.

- [ ] 8. Run `leedevkit test web` without `--unit-only`; inspect per-phase logs, then report pass/fail/skipped checks and remaining risks. Run focused web tests first only for iteration, never as final proof.

## Test impact matrix

- Changed behavior: filter presentation, view/export affordances, responsive filter sheet, visible result/empty/error context.
- Preserved behavior: API params, default last-month range, URL date range, search debounce, infinite scroll, employee drawer, export routing.
- Boundaries: month/year rollover, custom range, empty departments/employers, long labels, zero results, 360px width.
- Failures: report query error, export loading/error, missing metadata, mobile overlay close/reopen.
- Security: no new API or authorization boundary; authenticated UI test uses least privilege.
- Compatibility: existing translations, existing toolbar callers, existing query hooks, light/dark themes.

## Done when

- Header hierarchy separates navigation from actions.
- Filters never clip or force page-level horizontal scrolling at required viewports.
- All controls have visible/semantic labels, keyboard focus, and selected state.
- Mobile filter sheet traps focus and restores focus to trigger.
- Loading, empty, and error states are visible and actionable.
- Existing report behavior tests remain green; new UI behavior tests pass.
- `leedevkit test web` completes successfully with integration/E2E phase executed.
