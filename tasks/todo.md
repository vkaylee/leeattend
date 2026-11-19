# Employee create page and user-side linking — task list

## Phase 0 — Baseline and contract freeze

- [ ] **T0.1** Capture current employee/member tests, route tree, generated API contract, create/edit call sites, and preserve unrelated worktree changes.
- [ ] **T0.2** Write route × role matrix for `employee`, `team_lead`, `shift_manager`, `hr_manager`, `admin`; include allowed/denied route/actions, expected 403/redirect, console, network, and untested reasons.
- [ ] **T0.3** Freeze `/employees/new` route, allowlisted return context, `enrollment_id` prefill behavior, and no PII/form state in URL.
- [x] **T0.4** Choose `members:employee:link`; reject link conflicts with `409` (no force-transfer v1); reject removed employee mutation fields explicitly; keep create active.

### Checkpoint A

- [ ] Human reviews API ownership, permissions, conflicts, return context, and compatibility before implementation.

## Phase 1 — User-side identity contract

- [x] **T1.1** Add member-scoped link/unlink API and route.
- [x] **T1.2** Add organization/member/employee scope checks and deterministic 401/403/404/409 errors.
- [x] **T1.3** Make link/unlink plus employee-tenure identity update atomic; add audit evidence; preserve existing links.
- [ ] **T1.4** Add API/repository integration tests for success, unlink, idempotency, conflict, cross-scope, rollback, and concurrency (handler coverage added; DB integration pending).
- [x] **T1.5** Update OpenAPI and frontend API wrapper; generated type regeneration blocked by denied `bunx openapi-typescript` permission.

### Checkpoint B

- [ ] User-side link/unlink contract passes API and integration tests.
- [ ] Existing link records remain unchanged.

## Phase 2 — Remove employee-side identity ownership

- [ ] **T2.1** Remove employee mutation `user_id`/`force_transfer` behavior after User-side API exists.
- [ ] **T2.2** Preserve read-only `EmployeeRecord.user_id` for existing consumers.
- [ ] **T2.3** Implement and test chosen stale-client compatibility behavior; no silent dropped identity fields.
- [ ] **T2.4** Regenerate TypeScript API types and update employee API tests.

### Checkpoint C

- [ ] Employee HR create/update cannot link, unlink, or steal accounts.
- [ ] ESS/attendance/read consumers still resolve existing links.

## Phase 3 — Employee create page

- [x] **T3.1** Extract page-level HR form state/mutation from `EmployeeFormModal`; reuse existing form inputs, queries, mutation, toast, and query keys.
- [x] **T3.2** Remove organization-member fetching, user combobox, and `user_id` from employee create payload.
- [x] **T3.3** Add `employees.new.tsx` and `.lazy.tsx` route pair with `employees:create` guard and error boundary.
- [x] **T3.4** Move Add Employee action from modal open to `/employees/new` navigation.
- [x] **T3.5** Add responsive page header/back/cancel/save, validation, loading/error, focus, duplicate-submit prevention, and dirty blocker.
- [x] **T3.6** Preserve safe `enrollment_id` prefill without user linking.
- [ ] **T3.7** Add page, route, navigation, and regression tests (written; wrapper execution blocked by container lifecycle readiness).

### Checkpoint D

- [ ] Authorized direct and header entry reaches `/employees/new`.
- [ ] No create modal opens; submit success/failure behavior verified.
- [ ] Employee list/detail and non-create dialogs regressions pass.

## Phase 4 — User/Member linking UI

- [x] **T4.1** Add Members-side link/unlink control with `members:employee:link` permission gate.
- [x] **T4.2** Add linked/unlinked/loading/empty/error/conflict/pending states and explicit unlink confirmation; no force-transfer in v1.
- [x] **T4.3** Use bounded employee candidate search; invalidate member and employee queries on success.
- [ ] **T4.4** Add member UI/API contract tests and PII-safe artifact checks (existing member regression passes; dedicated link UI tests pending).
- [ ] **T4.5** Add read-only linked-account status on employee detail if absent.

### Checkpoint E

- [ ] Members is sole link/unlink UI entry point.
- [ ] Conflict never silently reassigns; downstream identity behavior verified.

## Phase 5 — Cleanup and release verification

- [ ] **T5.1** Migrate attendance prefill caller and detail/list edit navigation to page route.
- [ ] **T5.2** Remove obsolete create modal state/imports/member lookup after exhaustive call-site search; retain non-create dialogs.
- [ ] **T5.3** Update i18n, tests, generated route tree, and generated API types through wrappers.
- [ ] **T5.4** Run focused web/API tests during iteration.
- [ ] **T5.5** Run `./leedevkit test web </dev/null`.
- [ ] **T5.6** Run `./leedevkit test api </dev/null` and `./leedevkit test all </dev/null` when backend/OpenAPI/DB contract changed.
- [ ] **T5.7** Perform route × role browser audit at 1920×1080, then mobile smoke; record direct URL, refresh, validation, success, failure, dirty, auth, console, and network evidence.
- [ ] **T5.8** Read per-phase `.test_logs/`; report pass/fail/skip/warnings and remaining risks.

### Final checkpoint

- [ ] Employee create is dedicated page, one page, no user selector.
- [ ] User linking is User/Member-owned, explicit, permissioned, atomic, and audited.
- [ ] Existing links and employee non-create flows preserved.
- [ ] Full applicable test and UI evidence recorded.

---

# Typed UserID Internal Boundary — task list

## Phase 1 — Foundation and authentication

- [x] **U1.1** Add minimal `ids.UserID` parse/string/UUID conversion behavior only where needed; test valid, malformed, zero, and round-trip values.
- [x] **U1.2** Migrate JWT generation and verification to `ids.UserID`; preserve UUID subject text, errors, lifetimes, and revocation key behavior.
- [x] **U1.3** Store HTTP auth context as `ids.UserID` while keeping `UserIDFromContext` UUID compatibility until handler boundaries migrate; preserve 401, actor tracing, and revocation behavior.

### Checkpoint A

- [ ] Auth and middleware focused tests pass.
- [ ] No handler receives raw UUID from `UserIDFromContext`.

## Phase 2 — Complete work-policy/payroll actor path

- [x] **U2.1** Update work-policy and payroll handler actor values to use typed context values.
- [ ] **U2.2** Update authorization interfaces/mocks and work-policy/payroll domain/repository actor parameters. `MemberContext.UserID` now uses `ids.UserID`; API `Authorizer` interface remains UUID-compatible pending full mock/caller migration.
- [x] **U2.3** Keep actor IDs typed through work-policy/payroll repository calls; existing SQL adapters retain explicit UUID boundaries.
- [x] **U2.4** Existing API/repository tests compile and focused API tests pass; full repository run remains blocked by pre-existing `work policy is missing` failures.

### Checkpoint B

- [ ] HTTP-to-SQL work-policy/payroll actor path compiles and tests pass.
- [ ] JSON and DB UUID contracts remain unchanged.

## Phase 3 — Remaining authenticated-user consumers

- [x] **U3.1** Migrate bounded authenticated actor path for attendance adjustments; full Authorizer caller migration remains pending.
- [ ] **U3.2** Leave DTOs, path IDs, unrelated IDs, map keys, SQL locals, and infrastructure storage fields unchanged unless boundary analysis proves otherwise.
- [ ] **U3.3** Classify remaining raw user UUID uses and remove accidental semantic mixing.

### Checkpoint C

- [ ] Remaining raw user UUIDs have documented transport/SQL/infrastructure reasons.
- [ ] No broad UUID replacement or unrelated worktree change appears.

## Phase 4 — Verification

- [ ] **U4.1** Run `./leedevkit test api --unit-only` for focused iteration.
- [ ] **U4.2** Run `./leedevkit test api` with integration checks.
- [ ] **U4.3** Run `./leedevkit test all` when shared backend/API behavior is affected.
- [ ] **U4.4** Inspect `.test_logs/` per phase; report pass/fail/skip/warnings and remaining risks.
- [ ] **U4.5** Confirm unrelated worktree paths remain preserved and unstaged.

### Final checkpoint

- [ ] `ids.UserID` protects internal authenticated-user boundaries.
- [ ] JWT, HTTP, JSON, PostgreSQL, authorization, audit, and revocation contracts remain compatible.
- [ ] Negative, validation, persistence, and regression evidence recorded.
