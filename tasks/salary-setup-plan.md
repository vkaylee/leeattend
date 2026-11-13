# Implementation Plan: Salary Setup Page

## Context

`/salary-management` now shows only configured `employee_salary_components`, but no clear primary UI exists for creating assignments. Existing assignment UI lives inside `EmployeeSalaryConfig`, mounted from employee detail/drawer and currently infers `activeEmployment`. HR needs a dedicated full page tied to explicit employment: review current/history/future salary components, add/change an effective-dated assignment, and return to list with filters preserved.

Approved source spec: `salary-administration-plan.md`. It requires `/salary-management/$employmentId/edit`, single-employment editing, no bulk edit/import/export, history preservation, employment boundaries, approved T2 salary access, and one canonical write path.

## Architecture

```text
T2/grant policy + schema
  -> employment-scoped API contract
  -> repository transaction/validation/audit
  -> API client/query keys
  -> guarded route/navigation
  -> full-page setup UI
  -> role/browser/integration verification
```

Use full page, not drawer. Reuse `EmployeeSearch`, `useEmployeeSearch`, `fetchEmployee`, salary API wrappers, shared UI primitives, and employee-detail visual patterns. Treat `EmployeeSalaryConfig` as reusable UI/query reference, not complete write contract: it lacks stop/reason/history pagination/409/concurrency handling.

## Scope decisions

- Route: `/salary-management/$employmentId/edit`.
- One explicit employment per workflow; no bulk/import/export/payroll calculation.
- Salary policy here means `component_definition_id + amount + effective_date`; payroll rule/incentive policy stays separate.
- Current/future/stopped/unconfigured and zero-amount states are explicit. Unconfigured employees never appear as fake component rows.
- No new dependency/localStorage. Migration only if schema cannot represent required stop/reason/concurrency; migration requires down script.
- TanStack route files follow repository convention: `salary-management.$employmentId.edit.tsx` plus lazy sibling; regenerate route tree.

## Dependency graph and vertical slices

### Phase 1: Contract and backend

#### Task 1 — Permission, scope, and API contract gate

Acceptance:
- [ ] Separate `payroll:read` from approved salary write grant (`payroll:manage` or `salary:manage`).
- [ ] Contract defines selected employment identity, current/future/stopped/unconfigured/zero states, effective date, reason, errors, no-store, audit, compatibility, and organization-local `as_of`.
- [ ] Schema decision proves whether stop/history/concurrency needs migration.

Verification:
- [ ] Review access-control, data-governance, API, database rules and current handlers/tests.
- [ ] Add contract tests only after grant/T2 decision.
- [ ] Check cited paths with codebase-memory and direct source reads where coverage is incomplete.

Dependencies: None.

#### Task 2 — Employment-scoped read/write backend path

Acceptance:
- [ ] Detail/history returns only selected employment rows with stable current/future/stopped semantics.
- [ ] Write validates employment/component/date/amount/reason, preserves history, and prevents cross-employment writes.
- [ ] Duplicate/conflicting writes return structured 409/validation errors with no partial side effects.
- [ ] Unauthorized/forbidden/not-found paths stop before salary query/mutation.
- [ ] Existing list contract remains intact.

Verification:
- [ ] API unit + PostgreSQL integration tests cover success, empty, future, stopped, zero, duplicate, invalid date/employment, 401/403/404/409, retry/concurrency, rollback, no-store/audit.
- [ ] Run `./leedevkit test api`.

Dependencies: Task 1.

### Checkpoint 1 — Backend contract

- [ ] T2/grant gate approved or blocker recorded.
- [ ] Contract tests define statuses and error shapes.
- [ ] Integration tests prove employment/history boundaries.
- [ ] No frontend write UI targets unstable APIs.

### Phase 2: Route and setup flow

#### Task 3 — Guarded edit route and list entry links

Acceptance:
- [ ] Add `salary-management.$employmentId.edit.tsx` and lazy sibling; regenerate route tree.
- [ ] Direct URL/refresh handle valid, invalid, and missing employment.
- [ ] Write route requires approved write grant; server remains authority.
- [ ] List row action uses `employment_id`; row key uses `employment_id + component_id`.
- [ ] Return search preserves `as_of`, search, employer, department, status, and page; never amount/PII.

Verification:
- [ ] Route/guard tests cover HR/admin allowed and employee/team-lead/shift-manager denied behavior.
- [ ] Navigation tests cover URL and search restoration.
- [ ] Run web typecheck and focused route/page tests.

Dependencies: Tasks 1–2.

#### Task 4 — Employment context and setup page shell

Acceptance:
- [ ] Header identifies employee, selected employment, employer, department, job title, and dates.
- [ ] Skeleton, retry/error, missing-employment, and no-record states exist.
- [ ] Queries never run with empty/invalid employment ID.
- [ ] Effective date follows organization-local API contract, not browser UTC.

Verification:
- [ ] Component tests cover loading/error/missing/valid/no-record contexts.
- [ ] Browser-check valid and invalid routes.

Dependencies: Task 3.

### Checkpoint 2 — Route/context

- [ ] HR can navigate list → explicit employment edit → back without losing filters.
- [ ] Read-only roles are denied before write UI/API use.
- [ ] No historical/multi-employer employment is inferred from active employee employment.

#### Task 5 — Canonical assignment/edit/stop workflow

Acceptance:
- [ ] Render current, future, stopped, history, and zero-amount states explicitly.
- [ ] Validate component, decimal amount, effective date, employment bounds, reason, and conflicts.
- [ ] Pending submit disabled; failed save preserves draft; success invalidates detail/list queries.
- [ ] 409 offers reload/review and never overwrites newer data.
- [ ] Existing employee payroll-config behavior remains covered; no duplicate salary write path.

Verification:
- [ ] Web tests cover success, validation, API failure, conflict, temporal states, 409, and refresh.
- [ ] Run `./leedevkit test web --unit-only --pattern 'salary-management|employee-salary-config'`.
- [ ] Browser-check add/change/stop, reload, selected employment/history.

Dependencies: Tasks 2–4.

#### Task 6 — Explicit unconfigured worklist/search

Acceptance:
- [ ] Default table remains assignment-only.
- [ ] HR can search by name/code and navigate to valid employment setup.
- [ ] No valid employment gives actionable blocked state.
- [ ] Existing employer/department/unconfigured search fields are rendered or intentionally removed with tests.

Verification:
- [ ] Test empty search, selection, no employment, navigation, denied write permission.
- [ ] Browser-check worklist and post-save refresh.

Dependencies: Tasks 3–5.

### Checkpoint 3 — Complete setup flow

- [ ] HR can select explicit employment, add/change/stop, refresh, and see correct history/list.
- [ ] Failed save preserves draft; 409 cannot overwrite newer data.
- [ ] Existing history/payroll-config remains intact.

### Phase 3: Consolidation and polish

#### Task 7 — Employee-detail summary/link consolidation

Acceptance:
- [ ] Employee detail has one canonical link using explicit employment context.
- [ ] Compensation history and separate standard-days/hours config remain available.
- [ ] Duplicate salary form/unimplemented delete actions removed only after call-site verification.
- [ ] Return link restores employee detail context.

Verification:
- [ ] Update drawer/detail tests.
- [ ] Run focused/full web tests and browser flow.

Dependencies: Tasks 4–6.

#### Task 8 — i18n/accessibility/responsive/operational polish

Acceptance:
- [ ] No new hardcoded strings; en/vi/zh complete.
- [ ] Labels, focus, keyboard, Escape, skeleton, empty, error, retry, pending, success, and 409 states work.
- [ ] Desktop/mobile/dark mode usable.
- [ ] Amounts absent from URL/storage/logs/unauthorized responses.

Verification:
- [ ] Run i18n, typecheck, lint, accessibility tests, and web E2E.
- [ ] Browser-check keyboard/mobile/dark mode and console/network.

Dependencies: Tasks 5–7.

### Checkpoint 4 — Final acceptance

- [ ] `./leedevkit test api`
- [ ] `./leedevkit test web`
- [ ] `./leedevkit test all`
- [ ] `git diff --check`
- [ ] Route × role browser matrix covers allowed HR/admin and denied employee/team-lead/shift-manager behavior.
- [ ] No credentials, tokens, raw salary amounts, or raw PII in artifacts/logs.

## Risks

| Risk | Mitigation |
|---|---|
| T2/grant approval missing | Stop before frontend write UI; document blocker. |
| Employee vs employment ID mismatch | Resolve explicit employment before editor; always submit employment ID. |
| Existing component UI append-only | Define backend contract first; reuse only presentation/query pieces. |
| Partial multi-row save | Keep one-row atomic submit unless new transactional batch API is approved. |
| Rehire/transfer/multi-employer errors | Use selected employment ID; never infer active employment. |
| UTC/org timezone mismatch | API owns `as_of`; frontend displays returned date. |
| Generated route drift | Use route generator and typecheck; never hand-edit generated tree. |
| Unrelated dirty worktree | Inspect and commit only scoped files. |

## Critical files

- `salary-administration-plan.md`
- `webdashboard/src/pages/salary-management-page.tsx`
- `webdashboard/src/lib/api/salary-management.ts`
- `webdashboard/src/components/payroll/employee-salary-config.tsx`
- `webdashboard/src/components/common/employee-search.tsx`
- `webdashboard/src/hooks/use-employee-search.ts`
- `webdashboard/src/lib/api/payroll.ts`
- `webdashboard/src/lib/api/employees.ts`
- `webdashboard/src/routes/_authenticated/_organization/salary-management.tsx`
- `webdashboard/src/routes/_authenticated/_organization/salary-management.lazy.tsx`
- `webdashboard/src/pages/employees/employee-detail-page.tsx`
- `webdashboard/src/components/features/employees/employee-detail-drawer.tsx`
- `apiserver/internal/api/payroll.go`
- `apiserver/internal/repository/payroll.go`

## Verification commands

- `./leedevkit test web --unit-only --pattern 'salary-management|employee-salary-config'`
- `./leedevkit test api`
- `./leedevkit test web`
- `./leedevkit test all`
- `git diff --check`
