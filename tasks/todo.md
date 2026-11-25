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
- [ ] **T5.5** Run `leedevkit test web </dev/null`.
- [ ] **T5.6** Run `leedevkit test api </dev/null` and `leedevkit test all </dev/null` when backend/OpenAPI/DB contract changed.
- [ ] **T5.7** Perform route × role browser audit at 1920×1080, then mobile smoke; record direct URL, refresh, validation, success, failure, dirty, auth, console, and network evidence.
- [ ] **T5.8** Read per-phase `.test_logs/`; report pass/fail/skip/warnings and remaining risks.

### Final checkpoint

- [ ] Employee create is dedicated page, one page, no user selector.
- [ ] User linking is User/Member-owned, explicit, permissioned, atomic, and audited.
- [ ] Existing links and employee non-create flows preserved.
- [ ] Full applicable test and UI evidence recorded.

---

# Duplicate employee detection after employee_tenures

## Phase D0 — Policy and baseline

- [x] **D0.1** Human approved normalization, hard/soft match, global employee-code, permission, privacy, audit, 30/actor/60s rate limit, and feature-flag default/shadow/pilot policy on 2026-11-24. Flag owner/removal date remains required before rollout.
- [x] **D0.2** Run aggregate duplicate report on isolated disposable schema; all ten counters zero on empty-schema baseline. No PII output/artifact. Populated-fixture/target prevalence not measured.

### Checkpoint D-A

- [x] Human approved identity and code policy before schema/API work.
- [ ] Decide/provision populated disposable fixture if data-prevalence report is required; current schema-only baseline is not prevalence evidence.

## Phase D1 — Lookup foundation

- [x] **D1.1** Add reversible nullable phone/national-ID HMAC lookup columns and non-unique partial indexes (`20261125000000_employee_identity_matching`, `phases/hr_core/up.sql`, schema contracts).
- [x] **D1.2** Add bounded idempotent backfill/reconciliation (`internal/domain/identity`, `config.IdentityLookupHMACSecret`, `repository/employee_identity_backfill.go`); fail closed without secret; block unique indexes on unresolved collisions.

### Checkpoint D-B

- [x] Migration forward/down (numeric + fresh phased) and backfill dry-run/first-run/rerun/collision/secret-missing integration tests pass.
- [x] No raw PII, secret, or lookup hash in reports/logs/artifacts.

## Phase D2 — Match API

- [x] **D2.1** Add `POST /api/v1/employees/match` with `employees:identity:match`, exact hard matching, soft candidates, redacted response, audit, and rate limit.
- [x] **D2.2** Add API/repository/OpenAPI tests for auth, validation, hard/soft/none, redaction, and rate limit.

## Phase D3 — Atomic write paths

- [x] **D3.1** Harden create-new: employee + profile + first tenure + employment in one transaction; recheck identity/code under lock.
- [x] **D3.2** Add `POST /api/v1/employees/{employee_id}/tenures`; lock/revalidate, keep employee identity/code, insert tenure only.
- [x] **D3.3** Add integration tests for rollback, duplicate/race, stale candidate, overlap, terminated reuse, and active rejection.

### Checkpoint D-C

- [x] New person creates one employee and first tenure.
- [x] Existing person creates tenure only.
- [x] ESS/attendance/reports resolve correct identity.

## Phase D4 — UX integration

- [x] **D4.1** Add phone/DOB/national-ID fields and debounced preflight to `/employees/new`.
- [x] **D4.2** Implement explicit hard reuse/create and soft review states; never auto-reuse; keep PII out of URL/artifacts; localize/a11y.
- [x] **D4.3** Route attendance staging through same preflight; raw user ID is not identity proof.
- [x] **D4.4** Add page/component/staging tests for match states, stale/error, reuse/create, keyboard/focus, and duplicate submit.

### Checkpoint D-D

- [x] Search-before-create works from employee page and attendance staging.
- [x] Hard match cannot accidentally duplicate; soft match never auto-reuses.

## Phase D5 — Rollout and final verification

- [ ] **D5.1** Feature flag defaults off; pilot admin/HR; aggregate metrics only; rollback documented.
- [ ] **D5.2** Regenerate API types, run focused tests then full web/API/all gates, inspect `.test_logs/`, and audit all five roles at 1920x1080 then mobile.

### Final duplicate-detection checkpoint

- [ ] Search existing → explicit reuse/create works.
- [ ] New employee has profile + first tenure atomically.
- [ ] Reuse adds tenure only.
- [ ] Concurrent identity/code races are safe.
- [ ] No PII leaks; rollback evidence recorded.

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

- [ ] **U4.1** Run `leedevkit test api --unit-only` for focused iteration.
- [ ] **U4.2** Run `leedevkit test api` with integration checks.
- [ ] **U4.3** Run `leedevkit test all` when shared backend/API behavior is affected.
- [ ] **U4.4** Inspect `.test_logs/` per phase; report pass/fail/skip/warnings and remaining risks.
- [ ] **U4.5** Confirm unrelated worktree paths remain preserved and unstaged.

### Final checkpoint

- [ ] `ids.UserID` protects internal authenticated-user boundaries.
- [ ] JWT, HTTP, JSON, PostgreSQL, authorization, audit, and revocation contracts remain compatible.
- [ ] Negative, validation, persistence, and regression evidence recorded.

---

# `/shifts/new` HR UX — actionable checklist

## Phase 0 — Baseline

- [ ] Capture `git status --short`; preserve unrelated worktree changes.
- [ ] Run `leedevkit --help` and `leedevkit test --help`.
- [ ] Confirm inline interval payload and frontend-only scope.

## Phase 1 — Metrics

- [ ] Reuse `getShiftCardMetrics` for all-period aggregation.
- [ ] Add expected-punch summary without duplicating existing arithmetic.
- [ ] Test normal, split, duration/fixed break, overnight, midnight break, empty/incomplete drafts.

## Phase 2 — Hook contract

- [ ] Preserve default/add/remove/edit hydration/overnight behavior.
- [ ] Test all intervals serialize with HH:MM:SS.
- [ ] Test duration mode sends null clock fields.
- [ ] Test fixed mode sends both clock fields and rejects incomplete values.
- [ ] Checkpoint: focused hook tests + typecheck.

## Phase 3 — Editor UX

- [ ] Rename visible interval copy to `Khoảng làm việc` and locale equivalents.
- [ ] Hide optional interval name in normal flow; expose accessible advanced control.
- [ ] Explain one normal period and when to add split period.
- [ ] Replace ambiguous fixed-break toggle with duration/fixed-time choice.
- [ ] Add labels, helper text, inline validation, keyboard/a11y coverage.

## Phase 4 — Preview

- [ ] Remove first-period-only preview calculation.
- [ ] Show presence, break, net work, expected punches.
- [ ] Test split/overnight/empty summaries and preserve timeline behavior.
- [ ] Checkpoint: component tests + 1920×1080 visual verification.

## Phase 5 — i18n and page regression

- [ ] Add matching keys to en/vi/zh `attendance.json`.
- [ ] Remove new hardcoded labels/placeholders.
- [ ] Preserve save/cancel, dirty blocker, edit, history, loading/error behavior.
- [ ] Run locale parity, translation usage, typecheck, lint, and build.

## Phase 6 — E2E and full verification

- [ ] Extend `/shifts/new` E2E: basic, split, duration break, fixed break, overnight, payload, cancel/dirty.
- [ ] Read `docs/dev-accounts.md`; use least-privileged documented account.
- [ ] Create route × role acceptance matrix before browser testing.
- [ ] Announce/use 1920×1080 viewport; capture console/network evidence without PII/secrets.
- [ ] Run focused E2E through project wrapper.
- [ ] Run applicable full web/integration suite.
- [ ] Inspect `.test_logs/`; record pass/fail/skip/warnings and remaining risks.
- [ ] Final checkpoint: unrelated worktree changes preserved; no backend/schema changes.

---

# HRM break capture and credit lifecycle — task list

## Phase 0 — Contract and baseline

- [ ] Verify actual latest migration, schema markers, UUID/FK conventions, current generated contracts, and all time-entry scan sites.
- [ ] Freeze explicit `break_punch_mode` and `break_credit_mode`; remove ambiguous legacy semantics in same cutover.
- [ ] Record required-mode rule: fixed break boundaries only; required + automatic deduction rejected.

## Phase 1 — Schema and resolved policy

- [x] Add `break_punch_mode` and `break_credit_mode` to shift intervals and future `work_segments.breaks_snapshot` values.
- [x] Add child `time_entry_breaks`, action idempotency storage, unique/open-state indexes, and rollback migration.
- [x] Add scheduled/actual/deducted/net/provisional/exception fields to daily summaries and closure snapshots.
- [ ] Add API/generated type round-trip and migration invariant tests.

## Phase 2 — Transactional actions

- [x] Add explicit ESS action/state endpoints: `work_in`, `work_out`, `break_start`, `break_end`.
- [x] Replace implicit toggle behavior; malformed/missing action returns validation error.
- [x] Implement employment/open-entry locking, request hash replay/conflict, duplicate protection, and audit evidence.
- [x] Implement state machine, system-implied work-out break close, and blocking missing-break exception.

## Phase 3 — Calculation and summaries

- [ ] Implement one pure calculation path for scheduled overlap, paid break, and actual required break.
- [ ] Keep time-entry duration gross; recompute daily summary deterministically without double deduction.
- [ ] Handle overnight business date/timezone and split interval ownership.

## Phase 4 — Reports, payroll, closure

- [ ] Make reports/payroll read daily summary net fields only.
- [ ] Pre-aggregate punch display data; guarantee one report row per employment/business date.
- [ ] Extend closure locking/snapshot and block open/provisional break exceptions.
- [ ] Update Go/TS/docs report contracts and consumer tests.

## Phase 5 — Frontend rollout

- [x] Add explicit capture/credit controls to shift editor and payload.
- [x] Add ESS state/action UI, timers, invalid-action disabling, idempotency key, and exception state.
- [ ] Update en/vi/zh translations and accessibility tests.
- [ ] Enable `required` per shift after shadow comparison; default new policies to `none`.

## Verification

- [ ] Run focused API/web tests through wrappers.
- [ ] Run migration up/down and invariant queries on disposable DB.
- [ ] Run `leedevkit test api </dev/null`, `leedevkit test web </dev/null`, then `leedevkit test all </dev/null`.
- [ ] Inspect `.test_logs/`; preserve unrelated worktree changes and avoid `.leedevkit/`.
