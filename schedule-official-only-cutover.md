# Schedule Official-Only Cutover Plan

Status: Approved

## Goal
Cut over operational schedule reads, attendance/payroll consumers, and frontend schedule/ESS surfaces to assigned official records (`schedule_day_revisions` + `work_segments`), removing silent `activation-aware` whole-query fallback while preserving `resolveEffectiveShift` for preview and publication.

## Constraints and Interfaces
- **Applicable rules:** `migration-and-rollback.md` (expand/migrate/verify/contract, roll-forward once official evidence exists), `database-rules.md` (tenant/scope isolation, transaction atomicity), `access-control.md` (server-side fail-closed authorization before data lookup), `api-governance.md` (contract compatibility & OpenAPI sync), `testing-standards.md` (`leedevkit test` full targets, never unit-only).
- **Global constraints:**
  - Never delete or mutate published/superseded revisions, closure snapshots, raw punches, or `legacy_unverified` provenance.
  - Keep `resolveEffectiveShift` / `resolveScheduleData` as the single engine for preview and office/batch publication.
  - Do not run `scripts/_go-gates.sh --integration` directly (mutates formatting and drops DB); use `leedevkit test <target>`.
  - After official/closure evidence exists, recovery is roll-forward only (no destructive down-migration or silent fallback to legacy writes).
- **Global interfaces:**
  - `GET /api/v1/schedules` (`apiserver/internal/api/schedules.go`, `apiserver/openapi.yaml`, `webdashboard/src/types/api-generated.ts`): target default `view=official`; explicit `view=legacy` deprecated only during transition if needed.
  - Day states: `published` (work), `explicit_off`, `missing` (unassigned), `provisional` (preview only).

## Tasks

- [x] **Task 1: Filter official revisions in backend consumers** (`apiserver/internal/repository/payroll_loader.go:268`, `apiserver/internal/repository/schedule_shadow.go:185`, `apiserver/internal/repository/attendance.go:336`)
  - `loadBatchWorkSegments`: filter `ScheduledWorkdayCounts` by `schedule_day_revisions.state = 'published'` while preserving `CompletedShiftCounts` across replans and legacy days with no official revision.
  - `loadLegacyShadowDays`: restrict legacy segment scan to `schedule_revision_id IS NULL AND segment_status <> 'cancelled'`.
  - `resolvePunchSegment`: join `schedule_day_revisions` (`state = 'published'`), prefer exact in-shift match before 2h early check-in window; unified per-employment advisory lock order across `PunchIn`, `PunchOut`, `ESSAttendanceAction`, `ReprocessAttendance`, and `ResolveUnscheduled`.
  - **Verify:** DB integration tests covering mixed legacy + superseded + published segments, shadow mismatch, early punch-in, and concurrent goroutine races (`leedevkit test api` passed 3/3).

- [x] **Task 2: Harden activation gate and schedule read authorization** (`apiserver/internal/api/organization.go:575`, `apiserver/internal/api/schedules.go:710`, `apiserver/internal/repository/schedule_evidence.go:265`)
  - Blocked `PutConfig` from writing `schedule.lifecycle.enabled` directly (must use readiness-gated activation endpoint).
  - Enforced fail-closed `schedules:read` + `HasGlobalScheduleRead` (role-level global+permission check preventing dual-role escalation) + `ResolveAuthorizedScheduleEmployments` on `GET /api/v1/schedules`; enforced `AuthorizeScheduleMutation` on lifecycle write endpoints; tightened `AuthorizeScheduleEmployment` to global role or managed department.
  - **Verify:** Unit + DB integration tests for employee, managed-department manager, global reader, dual-role escalation, write-scope denial, and `PutConfig` rejection (`leedevkit test api` passed 3/3).

- [x] **Task 3: Fix frontend official/missing/provisional rendering and coverage filters** (`webdashboard/src/components/shifts/weekly-roster-grid.tsx:141`, `schedule-lifecycle-panel.tsx:77`, `roster/roster-utils.ts:51`, `hooks/use-monthly-schedules.ts:61`, `pages/shifts/hooks/use-monthly-cache-patch.ts:28`)
  - Weekly grid, lifecycle panel, and monthly cells distinguish `missing` / `explicit_off` / `provisional` / `published` / `conflict`.
  - Fixed monthly coverage filter (no client-side fabricated rows when filtering `missing`/`complete`) and fixed monthly cache invalidation key index (`queryKey[2] === "monthly"`).
  - Fixed weekly copy (employment-scoped fetch, `source === "roster"`, calendar date shift) and monthly copy (`explicit_off` preservation + date deduplication).
  - **Verify:** Vitest + E2E + lint (`leedevkit test web` passed 3/3).

- [x] **Task 4: Cut over ESS clock timeline and unscheduled resolution UI** (`webdashboard/src/pages/ess/my-clock-page.tsx`, `shift-24h-timeline.tsx`, `apiserver/internal/repository/schedule_lifecycle_resolve_unscheduled.go`, `webdashboard/src/components/features/reports/resolve-unscheduled-modal.tsx`, `use-daily-drawer-mutations.ts`)
  - Wired `MyClockPage` to today's official schedule (`view=official`, org-timezone `todayStr`, interval break extraction, overnight punch rendering), removing hardcoded `08:30–17:30`.
  - Implemented atomic backend endpoint `POST /api/v1/schedules/{employment_id}/{business_date}/resolve-unscheduled` (OpenAPI documented; requires `schedules:amend` + `schedules:publish:rotating` + target scope; deadlock-free advisory lock + row lock order; requires unscheduled attendance evidence; cross-employer shift check; approved/pending OT guard; `revisionID`-scoped overnight reconciliation; employment-scoped pairing/reprocess in one transaction) and wired `ResolveUnscheduledModal` with required `reason`, composite modal key, drawer state reset, and stable retry `Idempotency-Key`.
  - **Verify:** DB integration tests (including overnight, multi-employment isolation, cross-employer shift, protected OT, and concurrent goroutine race tests) + web unit/E2E (`leedevkit test api` & `leedevkit test web` passed 6/6).

- [x] **Task 5: Add bulk/office publish UX and clear draft-vs-published separation** (`webdashboard/src/components/shifts/schedule-lifecycle-panel.tsx`, `webdashboard/src/pages/shifts/content.tsx`, `webdashboard/src/pages/attendance/hooks/use-working-patterns.ts`, `webdashboard/src/pages/assignment-rules/hooks/use-assignment-rule-form.ts`)
  - Added lazy-loaded `BatchSchedulePublishBar` on monthly roster view (local-date `formatDate`, payload-fingerprinted preview auto-invalidation, stable retry `Idempotency-Key`, backend `revision_number` populated in `GetOfficialSchedules`, `<60 kB` gzip lazy-chunk budget verified).
  - Surfaced partial-failure errors without closing bulk pattern modal when failures remain; invalidated `schedules` cache on assignment rule create/update.
  - **Verify:** `bun run build && node scripts/check-budget.mjs` + `leedevkit test web` passed 3/3.

- [x] **Task 6: Verify runtime data, office worker, and shadow parity before switch** (local dev DB `leedevkit-dev-db-1` inspected read-only)
  - Verified `feature_flags.schedule_lifecycle = true`, 1 employer (5 active employments), 143 `published` `schedule_day_revisions`, 0 legacy `work_segments`, 0 open `schedule_shadow_mismatches`, 0 dead-letter `ScheduleOfficeCoverageRefresh` jobs. Noted `config_entries` only covered `2026-11-01..2026-11-30` and 0 `schedule_office_policies` exist yet (manual/rotating mode active).

- [x] **Task 7: Switch `GET /api/v1/schedules` default from `activation-aware` to `official`** (`apiserver/internal/api/schedules.go`, `apiserver/internal/repository/schedules.go`, `webdashboard/src/lib/api/shifts.ts`)
  - Default `view` switched to `official` (server-side `coverage=missing|complete` now active by default); removed `activation-aware` all-or-nothing `bool_and` branch and `GetActivationAwareSchedules`; kept explicit `view=legacy` only for transition until Task 8.
  - Added cross-consumer parity DB integration test (`TestIntegrationOfficialScheduleParityAcrossReaderNoShowPunchAndPayroll`).
  - **Verify:** `leedevkit test api --json` (3/3 passed) and `leedevkit test web --json` (3/3 passed).

- [x] **Task 8: Contract & end-to-end verification**
  - Retired deprecated HTTP `view=legacy` / `GetOrganizationSchedules` external read path (`apiserver/internal/api/schedules.go`, `apiserver/internal/repository/schedules.go`, `webdashboard/src/lib/api/shifts.ts`); kept internal `resolveScheduleData` / `resolveEffectiveShift` for preview & office publisher.
  - Fixed Boy Scout regression found during 5-role Playwright audit: `shift_manager` / `team_lead` (who have `schedules:read` without `employees:read`) now load authorized monthly official schedules directly (`use-shift-data.ts`, `use-monthly-schedules.ts`, `monthly-roster-grid.tsx`).
  - Completed desktop-large (`1920x1080`) Playwright route × role audit across all 5 documented roles (`employee`, `team_lead`, `shift_manager`, `hr_manager`, `admin`) with zero unexpected 4xx/5xx/console/server errors, and verified full `leedevkit test api --json` + `leedevkit test web --json` + bundle budget.

## Done When
- [x] All operational schedule views, ESS clock, reports, no-show, punch matching, and payroll read only assigned official records (`schedule_day_revisions` + `work_segments`).
- [x] `activation-aware` whole-query fallback is removed; `missing`, `explicit_off`, `published`, and `provisional` are distinct across API and UI.
- [x] `leedevkit test api --json` and `leedevkit test web --json` pass all phases (including `int-api` and `e2e-web`) and 5-role Playwright audit passes.

## Review Focus
- `payroll_loader.go` denominator parity on historical periods that have only legacy unlinked segments (`schedule_revision_id IS NULL`) vs activated official periods.
- Early punch-in matching buffer in `resolvePunchSegment` vs overnight/adjacent split shifts (avoid ambiguous double-match).
- `GetNoShowReport` (`apiserver/internal/repository/reports.go:639`) behavior when `os.state IS NULL` (`missing` must not surface as false absent).
- Shift version creation path vs `guardLifecycleShiftMutation` (`lifecycle_guards.go:241`) to confirm creating a new `shift_version` is not blocked when old versions are published.
- Unscheduled resolution authorization: ensure roles resolving unscheduled punches have required late-publish/amend permission without privilege escalation.
