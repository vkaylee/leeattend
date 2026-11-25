# Overtime, Grace Period, Early OT & Break Ownership Plan

Status: Completed — verified across unit, PostgreSQL integration, migration rehearsal, 5-role browser acceptance, and full LeeDevKit gates

## Approved Architecture (2-Layer HRM Standard)

Separate **schedule & capture facts** from **effective-dated calculation rules** with zero duplicate ownership and zero legacy fallback:

| Concern | Owner | Notes |
|---|---|---|
| Shift hours (`start_time`, `end_time`, day offsets, UTC offsets, sort order) | `Shift` (`shift_default_intervals`) | Answers: *when is work scheduled?* |
| Break window & duration (`break_start`, `break_end`, `break_duration_minutes`) | `Shift` (`shift_default_intervals`) | Answers: *when is the break window?* |
| Break punch capture (`break_punch_mode`: `none` \| `required`) | `Shift` (`shift_default_intervals`) | Operational punch behavior enforced by `ESSAttendanceAction` |
| Break deduction & break OT (`break_auto_deduct`, `break_is_overtime`) | `Work Policy` (`work_credit_policies`) | Effective-dated calculation rule; removed from `Shift` |
| Late-arrival grace (`late_arrival_grace_minutes`) | `Work Policy` (`work_credit_policies`) | Reuse `matcher.ComputeLateMinutes` (<= grace => 0, > grace => full tardiness) |
| Early OT eligibility, grace, cap (`early_ot_enabled`, `early_ot_qualification_grace_minutes`, `early_ot_max_minutes`) | `Work Policy` (`work_credit_policies`) | Distinct from late grace; never auto-promoted to payable OT |
| Multi-segment OT aggregation (`ot_aggregation_mode`: `aggregate` \| `per_segment`) | `Work Policy` (`work_credit_policies`) | Governs rounding/unitizing & cap across multi-segment days |
| Rounding & work unit (`work_unit_minutes`, `rounding_policy`, `early_arrival_strategy`) | `AttendancePolicy` (`attendance_policy_versions`) | Resolved via `work_credit_policies.attendance_policy_id` effective on `business_date` |
| OT multipliers, quotas, approval lifecycle (`detected -> claimed -> approved -> payable`) | `OvertimePolicy` | Referenced by `labor_policy_group_versions.overtime_policy_id` |
| Employee/date applicability & provenance | `LaborPolicyGroup` + `attendance_daily_summaries` | Missing effective policy returns `policy_missing`; no fallback |

## Break Calculation Contract (Shift Capture × Work Policy × Overrides)
- **`break_punch_mode = 'required'`**:
  - Deduct actual break minutes from `time_entry_breaks` (`actual_break_minutes`), NOT scheduled auto-deduct.
  - Missing/unpaired break punch sets `break_provisional = true`, `break_exception_code = 'missing_break_pair'`.
- **`break_punch_mode = 'none'`**:
  - Apply `Work Policy` (`break_auto_deduct`, `break_is_overtime`) against merged scheduled break windows (`parseBreakWindowsSnapshot`).
  - Subtract approved `schedule_day_overrides` (`override_type = 'break_worked'`, `status = 'approved'`) from deducted break minutes.
  - Unexpected `time_entry_breaks` rows set `break_exception_code = 'unexpected_break_punch'`.

## Calculation Boundary
```text
raw_attendance_logs + time_entry_breaks + schedule_day_overrides
  -> Shift / work_segments snapshot (schedule & break_punch_mode facts)
  -> Effective Work Policy + AttendancePolicy resolver (employment_id + business_date; fail with policy_missing)
  -> Pure calculation engine (matcher.ComputeLateMinutes + buckets.CalculateBuckets)
  -> Single-TX upsert into attendance_daily_summaries with work_policy_version_id & shift_version_id
```

## Rule IDs
- `ARCH-01`: Single source of truth — `Shift` = schedule + punch capture; `Work Policy` = effective-dated calculation rules.
- `DB-03`: Transactional integrity (`service.WithTx`) and non-negative CHECK constraints on summaries.
- `MIG-01`: Clean pre-prod cutover with reversible `up.sql` and `down.sql` + phased schema parity.
- `API-01`: Contract-first — update Go handlers/repos, `apiserver/openapi.yaml`, and generated TypeScript together.
- `UI-01`: i18n across EN/VI/ZH, client validation, accessible labels/errors, no dead controls.
- `TEST-01`: Unit + PostgreSQL integration + migration rehearsal + 5-role 1920×1080 Playwright + full `leedevkit test api/web/all`.

## Completed Baseline
- [x] **B1. Shift grace/Early OT removal:** Dropped `early_punch_in_grace_period_minutes`, `allow_early_ot`, `max_early_ot_minutes`, `ot_aggregation_mode`, `is_night_shift` from `shift_default_intervals` (`20261119000000_work_policy_attendance_rules`), Shift API/Repo, and Shift editor (`grace-ot-section.tsx`, `ot-aggregation-section.tsx` deleted).
- [x] **B2. Work Policy rule storage & API:** Added the 7 rule fields to `work_credit_policies`, domain validation, API, `apiserver/openapi.yaml`, and basic inputs in `payroll-settings-section.tsx`.
- [x] **B3. Provenance, H6 pairing & closure guard:** Added `work_policy_version_id` / `shift_version_id` (`20261119010000_attendance_policy_provenance`), `ensureReprocessPeriodsOpen`, `pairRawLogsForReprocess`, multi-break overlap merge, `ErrAttendancePolicyMissing` / `ErrPayrollPolicyMissing`, and `h6_pairing_reprocess_payroll_integration_test.go`.

## Remaining Refactor Plan

### Phase 1 — Clean Cutover: Remove Break Policy Rules from Shift
- [x] **1.1 Migration (`20261126000000_remove_shift_break_policy_rules/{up,down}.sql`)**
  - Drop `is_break_auto_deduct`, `break_is_overtime`, `break_credit_mode` and their CHECK constraints (`shift_default_intervals_break_credit_mode_check`, `shift_default_intervals_break_policy_check`) from `shift_default_intervals`.
  - Keep `break_punch_mode` (`none` | `required`) and `shift_default_intervals_break_punch_mode_check`.
  - Keep phased schema `apiserver/migrations/phases/attendance_schedule/up.sql` aligned.
  - Verify: `up -> down -> up` cleanly applies and rolls back.
- [x] **1.2 Backend Shift & Schedule Contract Cleanup**
  - Remove `is_break_auto_deduct`, `break_is_overtime`, `break_credit_mode` from:
    - `apiserver/internal/api/shifts.go`, `apiserver/internal/api/shifts_test.go`
    - `apiserver/internal/api/schedules.go`
    - `apiserver/internal/repository/shifts.go`
    - `apiserver/internal/repository/schedules.go`
    - `apiserver/internal/repository/schedule_lifecycle.go` (preview/replan/breaks snapshot keep `start`, `end`, `start_at`, `end_at`, `duration_minutes`, `break_punch_mode` only)
    - `apiserver/internal/repository/schedule_break_override_integration_test.go`
  - Update `apiserver/openapi.yaml` and regenerate `webdashboard/src/types/api-generated.ts`.
  - Verify: no Shift or Schedule endpoint reads, writes, or documents removed policy fields.

### Phase 2 — Complete Pure Calculation Engine & Reprocess Pipeline
- [x] **2.1 Pure Calculation (`apiserver/internal/domain/payroll/buckets/buckets.go`)**
  - Wire `OTAggregationMode` (`aggregate` vs `per_segment`) cleanly for multi-segment daily calculation (unit rounding and `MaxEarlyOTMinutes` cap behavior).
  - Support `BreakPunchMode` (`required` uses `ActualBreakMinutes` + provisional/exception flags; `none` uses `Work Policy` `IsBreakAutoDeduct` / `IsBreakOvertime` minus approved `break_worked` minutes).
  - Keep all calculation functions pure (no DB/IO).
  - Verify: unit tests in `buckets_test.go` cover grace boundaries, caps, `aggregate` vs `per_segment`, `break_punch_mode = required` (complete + missing pair), `break_punch_mode = none` (auto-deduct, paid, break OT, unexpected punch, approved `break_worked` override).
- [x] **2.2 Orchestration & Transaction (`apiserver/internal/repository/attendance.go`)**
  - Select `wc.late_arrival_grace_minutes`, `wc.ot_aggregation_mode`, and effective `attendance_policy_versions.rules` (`work_unit_minutes`, `rounding_policy`, `early_arrival_strategy`) for `wc.attendance_policy_id` on `business_date`.
  - Compute daily `late_minutes` using existing `matcher.ComputeLateMinutes(punchIn, shiftStart, lateGrace, isHoliday)` — do not duplicate logic.
  - Load `time_entry_breaks` per `time_entry` and approved `schedule_day_overrides` (`break_worked`) per `(employment_id, business_date)`.
  - Persist `late_minutes`, `actual_break_minutes`, `deducted_break_minutes`, `break_provisional`, `break_exception_code`, `net_work_minutes`, `net_paid_minutes`, `work_policy_version_id`, `shift_version_id`.
  - Wrap `ReprocessAttendance` (period check + pairing + daily upserts) in a single DB transaction (`service.WithTx`).
  - Verify: rollback leaves zero partial summaries; `policy_missing` aborts cleanly; closed period returns `ErrAttendanceLockedPeriod`.

### Phase 3 — Frontend UI Cleanup, i18n & Stale Type Removal
- [x] **3.1 Shift Editor & Shift Metrics Cleanup**
  - Update `webdashboard/src/pages/shifts/components/break-config-section.tsx` and `webdashboard/src/pages/shifts/hooks/use-shift-editor.ts`:
    - Keep `breakPunchMode` (`none` | `required`), break schedule (`fixed` vs `duration`), `breakStart`, `breakEnd`, `breakDuration`.
    - Remove `isAutoDeduct`, `isBreakOvertime`, `breakCreditMode` checkboxes, state, and payload fields.
    - Show concise helper text noting that break deduction & break OT rules are configured in **Work Policy**.
  - Update `webdashboard/src/components/shifts/shift-timeline-preview.tsx` and `webdashboard/src/lib/shift-card-metrics.ts` (+ `__tests__/shift-card-metrics.test.ts`).
- [x] **3.2 Work Policy Settings UX, Validation & i18n**
  - `webdashboard/src/components/payroll/payroll-settings-section.tsx`:
    - Replace all hardcoded English labels/options (lines 344–371) with i18n keys.
    - Add matching translations in `webdashboard/src/i18n/locales/{en,vi,zh}/settings.json` (and `shifts.json` / `attendance.json` where updated).
    - Add client validation (non-negative integers for `late_arrival_grace_minutes`, `early_ot_qualification_grace_minutes`, `early_ot_max_minutes`) with accessible `aria-invalid` / `aria-describedby` error messages and clear grouping (Grace & Early OT vs Break Treatment).
  - Add unit tests in `payroll-settings-section.test.tsx` covering render, validation, and payload submission for all 7 fields.
- [x] **3.3 Clean Stale Legacy Types & Test Fixtures**
  - Remove obsolete Shift fields (`early_punch_in_grace_period_minutes`, `allow_early_ot`, `max_early_ot_minutes`, `ot_aggregation_mode`, `is_break_auto_deduct`, `break_credit_mode`, `break_is_overtime`) from:
    - `webdashboard/src/lib/api/types-work-segments.ts`, `types-employment.ts`, `types-attendance.ts`, `condition-types.ts`
    - `webdashboard/src/pages/settings/components/condition-type-form.tsx`
    - `webdashboard/src/hooks/reports/use-row-meta.ts`, `daily-report-columns.tsx`
    - `webdashboard/src/__tests__/factories/test-factories.ts` and affected unit test fixtures (`shifts-page.test.tsx`, `shift-picker-*.test.tsx`, `roster-toolbar-parts.test.tsx`, `shift-revision-history-dialog.test.tsx`, `daily-report-*.test.tsx`, `reports-page.test.tsx`, `unrecorded-shifts-tab.test.tsx`).

### Phase 4 — Verification Gates
- [x] **4.1 Integration Tests (PostgreSQL)**
  - Extend `h6_pairing_reprocess_payroll_integration_test.go` and add targeted integration tests covering:
    - `late_arrival_grace_minutes` boundary (<= grace vs > grace)
    - `early_ot_qualification_grace_minutes` + `early_ot_max_minutes` + `early_ot_enabled = false`
    - `ot_aggregation_mode` (`aggregate` vs `per_segment`)
    - `break_punch_mode = 'required'` with `time_entry_breaks` (paired + missing pair exception)
    - `break_punch_mode = 'none'` with `Work Policy` auto-deduct, break OT, and approved `schedule_day_overrides` (`break_worked`)
    - `ErrAttendancePolicyMissing` when membership is missing
    - Transaction rollback atomicity
- [x] **4.2 Migration Rehearsal**
  - Verify `up -> down -> up` on `20261126000000_remove_shift_break_policy_rules`, `20261119000000_work_policy_attendance_rules`, and `20261119010000_attendance_policy_provenance`.
- [x] **4.3 Browser Acceptance by Role (1920×1080)**
  - Read `docs/dev-accounts.md`, build route × role acceptance matrix for all 5 roles (`employee`, `team_lead`, `shift_manager`, `hr_manager`, `admin`).
  - Verify Shift editor (no break deduction/OT or grace/Early OT toggles; `break_punch_mode` + window work cleanly) and Work Policy settings (all 7 fields localized, validated, persisted) with zero console/5xx errors; mask all PII/credentials.
- [x] **4.4 Full LeeDevKit Gates**
  - Run `leedevkit test api`, `leedevkit test web`, `leedevkit test all`; inspect every phase in `.test_logs/` (never rely on trailing summary alone); run `git diff --check`.

## Done When
- `Shift` owns schedule facts + `break_punch_mode` only; zero calculation rules remain on `Shift` DB, API, OpenAPI, TS types, or UI.
- `Work Policy` is the sole effective-dated source for late grace, Early OT, OT aggregation, and break deduction/OT rules.
- `ReprocessAttendance` resolves `Work Policy` + `AttendancePolicy`, reuses `matcher.ComputeLateMinutes`, applies `time_entry_breaks` + `schedule_day_overrides`, runs in a single TX, and records provenance.
- Work Policy UI is localized (EN/VI/ZH), validated, accessible, and tested.
- Migration rehearsal, PostgreSQL integration tests, 5-role 1920×1080 browser acceptance, and full `leedevkit test api/web/all` pass.
