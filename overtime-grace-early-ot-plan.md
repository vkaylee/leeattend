# Overtime, Grace Period, and Early OT

Status: Implementation blocked — partial WIP remains

Latest state:
- Shift API/repository/schedule responses no longer own moved policy fields.
- Work Policy contracts/repository/migration now carry moved fields.
- Shift editor controls removed; Work Policy form payload controls added.
- Calculation orchestration, full UI contract regeneration, remaining test fixtures, and integration verification are not complete.
- Do not merge or deploy partial WIP.

Approved decisions:
- Reset development data; no legacy Shift-rule mapping.
- Return `policy_missing`; no fallback calculation for employment without Work Policy.
- Move all identified Shift-owned policy rules: grace, Early OT, cap, aggregation, and break deduction/overtime.

Approved decisions:
- Reset development data; no legacy Shift-rule mapping.
- Return `policy_missing`; no fallback calculation for employment without Work Policy.
- Move all identified Shift-owned policy rules: grace, Early OT, cap, aggregation, and break deduction/overtime.

## Goal
Make `Shift` a pure schedule/parser input and move every grace/Early OT eligibility decision into effective-dated `Work Policy`, with no legacy compatibility layer because LeeAttend is pre-production.

## Decision
- `Shift` owns **schedule facts only**: interval start/end, day offsets, UTC offsets, interval order, break boundaries, and other facts needed to parse attendance logs. It answers: **when was work scheduled?**
- `Shift` does not own eligibility, grace, OT, rounding, deduction, approval, quota, or multiplier rules. Remove `early_punch_in_grace_period_minutes`, `allow_early_ot`, `max_early_ot_minutes`, and `ot_aggregation_mode` from Shift contracts/storage/UI. Derive `is_night_shift` from schedule facts instead of persisting it as a rule input where feasible.
- `Work Policy` owns applicability and calculation decisions: late-arrival grace, Early OT enabled/disabled, Early OT qualification grace, Early OT cap, rounding/aggregation, break treatment, approval/payability, quota, and policy provenance. Resolve by `employment_id + work_date` through immutable effective-dated policy versions.
- Keep `OvertimePolicy` as the policy component for multiplier, approval, quota, and OT lifecycle. Extend the existing Work Policy version contract to carry or reference Early OT/attendance rules; do not create a second OT configuration source.
- Split ambiguous `Grace Period` into explicit fields: `late_arrival_grace_minutes` and `early_ot_qualification_grace_minutes`. Do not reuse one number for two meanings.
- Calculation flow: parse raw punches against Shift facts, resolve Work Policy, then run pure calculation. Early OT remains separate from detected, claimed, approved, and payable OT; no early time becomes payable without normal approval rules.
- Pre-production permits a clean contract/schema cutover. No dual-read, legacy Shift fallback, mixed-version rollout, or deprecation adapter. Migration remains reversible and requires approval before destructive column removal.
- No generic formula engine, microservice, or duplicate settings page.

## Ownership table
| Concern | Owner |
|---|---|
| Scheduled start/end and overnight boundaries | `Shift` |
| Punch parsing and interval matching | Attendance parser using `Shift` facts |
| Late-arrival grace | `Work Policy` |
| Early OT eligibility and cap | `Work Policy` |
| OT multiplier, approval, quota, payable lifecycle | `OvertimePolicy` referenced by `Work Policy` |
| Employee/date applicability | `LaborPolicyGroup` membership/version |
| Historical calculation provenance | Calculation snapshot |

## Calculation boundary
```text
raw punch logs
  -> Shift resolver: schedule facts
  -> attendance parser: normalized intervals
  -> Work Policy resolver: effective rules
  -> pure calculation engine: eligibility and minutes
  -> detected / claimed / approved / payable results
```

```go
CalculateAttendance(schedule ScheduleFacts, punches []PunchInterval, policy PolicyRules) Result
```

Calculation code receives resolved data only. No DB or repository calls inside pure domain logic.

Example:

```text
scheduled start: 08:00
clock-in:        07:40
early OT grace:  15 minutes
early OT:        enabled
eligible Early OT: 5 minutes
```

`07:40` is parsed from attendance data; policy decides whether 5 minutes qualifies.

## Migration stance
Because app is not in production, prefer clean cutover over compatibility scaffolding. Existing development values on Shift must not be silently reinterpreted. During migration, either reset disposable dev data or run an explicit, reviewed mapping only after business semantics are confirmed. Never add a hidden fallback.

No new production rollout compatibility is needed, but `up.sql` and `down.sql` remain mandatory and destructive removal still needs explicit approval.

## No implementation yet
This file only updates architecture and implementation plan. Existing working-tree changes remain untouched.

## Rule IDs
- `ARCH-01`: preserve explicit ownership and avoid duplicate configuration sources.
- `DB-03`: preserve data integrity and transaction boundaries.
- `MIG-01`: clean, reversible schema cutover with rollback boundary.
- `API-01`: contract-first validation and stable internal ownership.
- `TEST-01`: unit, integration, browser, and full LeeDevKit verification.
- `UI-01`: i18n, accessibility, loading/error states.

## Constraints and interfaces
- `ARCH-01`: `Shift` exposes schedule facts; `Work Policy` exposes effective rules; calculation consumes resolved value objects.
- `DB-03` / `MIG-01`: remove obsolete Shift columns only through reviewed migration with working `down.sql`; do not silently reinterpret dev data.
- `API-01`: contract-first clean break; update Go structs, OpenAPI/generated TypeScript, repository queries, and consumers together.
- `TEST-01`: focused tests, integration tests, browser acceptance, then full `api`, `web`, and `all` gates.
- `UI-01`: configure rules under Work Policy settings, not Shift editor; keep i18n, loading/error states, keyboard focus, and accessible controls.
- Preserve unrelated user changes in `webdashboard/src/lib/__tests__/api.test.ts`, `webdashboard/src/lib/api/shifts.ts`, and `webdashboard/src/pages/shifts/hooks/use-shift-data.ts`.

## Constraints and interfaces
- `ARCH-01`: preserve current domain boundaries and reuse existing shift/work-policy objects.
- `DB-03` / `MIG-01`: no destructive rename; any migration must be additive, reversible, and preserve historical versions.
- `API-01`: contract-first, additive fields only; UUID/date/minute validation and backend authorization before business logic.
- `TEST-01`: focused tests, integration tests, then full `api`, `web`, and `all` gates.
- `UI-01`: existing UI components, i18n, labels, loading/error states, keyboard focus, and accessible toggle labeling.
- Preserve unrelated user changes in `webdashboard/src/lib/__tests__/api.test.ts`, `webdashboard/src/lib/api/shifts.ts`, and `webdashboard/src/pages/shifts/hooks/use-shift-data.ts`.

## Tasks
- [ ] 1. Inventory and classify current consumers. Trace Shift interval fields through API, repository, attendance parser, OT detection, payroll, generated schema, and UI. Identify every place where `early_punch_in_grace_period_minutes`, `allow_early_ot`, `max_early_ot_minutes`, `ot_aggregation_mode`, break treatment, and `is_night_shift` are read or written. Verify: each field is marked `Shift fact`, `Policy rule`, derived value, or obsolete; no consumer missed.
- [ ] 2. Freeze clean-break contract. Define `ScheduleFacts`, `PolicyRules`, normalized punch intervals, effective-date precedence, missing-policy behavior, zero values, overnight matching, multiple intervals, cap, rounding, and approval flow. Add explicit fields for late grace versus Early OT grace. Verify: contract examples cover 07:40/08:00, exact grace boundary, cap, disabled flag, overnight, and mid-period policy change.
- [ ] 3. Refactor backend contracts and schema. Remove policy fields from Shift request/response/domain/repository SQL and add them to Work Policy/Overtime Policy version contract. Add clean-cutover migration with `up.sql` and `down.sql`; handle existing dev rows through explicit reset or reviewed mapping, never silent reinterpretation. Update OpenAPI/generated TypeScript together. Verify: schema applies, rollback works, no Shift API exposes removed policy fields, and policy version returns all required rules.
- [ ] 4. Implement resolver and pure calculation boundary. Resolve Shift schedule facts by work date, resolve Work Policy by employment/date, normalize punches, then calculate Early OT and late status using pure value objects. Keep DB out of calculation. Separate detected, claimed, approved, and payable values. Verify: unit cases cover boundary, zero, negative rejection, cap, disabled, overnight, multiple intervals, missing policy, and no implicit fallback.
- [ ] 5. Wire persistence, snapshots, and audit. Persist policy rules in immutable effective-dated versions, record selected Shift/version and Work Policy/Overtime Policy provenance in calculation snapshots, and keep locked payroll history immutable. Add audit events for policy changes without raw PII. Verify: disposable PostgreSQL integration covers effective-date resolution, concurrent overlap rejection, rollback, snapshot provenance, and historical result immutability.
- [ ] 6. Move UI configuration to Work Policy. Remove Grace/Early OT controls from `webdashboard/src/pages/shifts/components/grace-ot-section.tsx`, `interval-draft-editor.tsx`, and `use-shift-editor.ts`. Add the controls to existing Work Policy editor/settings with separate labels: `Late arrival grace`, `Early OT qualification grace`, `Allow Early OT`, and `Maximum Early OT`. Update EN/VI/ZH translations, generated types, validation, empty/loading/error/409 states, keyboard focus, and accessible toggle names. Verify: Shift editor contains schedule facts only; Work Policy editor changes policy fields only.
- [ ] 7. Add contract and integration coverage. Extend API tests for clean-break payloads, validation, 403/400/409 paths, effective-date resolution, and policy persistence. Add regression tests proving Shift changes alter schedule parsing but do not alter eligibility rules, while Work Policy changes alter eligibility and preserve snapshots. Verify: old incorrect field ownership fails before implementation and passes after.
- [ ] 8. Browser acceptance by role. Read `docs/dev-accounts.md`, create route × role matrix, use 1920×1080 viewport, and test Shift editor plus Work Policy settings for `employee`, `team_lead`, `shift_manager`, `hr_manager`, and `admin`. Record expected 403/redirect versus unexpected 404/5xx/crash, console, and network evidence; never record credentials or PII.
- [ ] 9. Final verification. Run `leedevkit test api`, `leedevkit test web`, then `leedevkit test all`; inspect phase logs, run `git diff --check`, and report skipped/failed checks. Do not claim completion from unit-only or lint-only output.

## Test impact matrix
- Changed behavior: grace and Early OT boundary, cap, policy resolution, payable-OT separation.
- Existing behavior: normal schedule parsing, late handling, regular/post-shift OT, attendance normalization, and historical snapshots.
- Inputs: missing, zero, negative, maximum, malformed time/date/UUID, overnight and multiple intervals.
- State: no policy membership, effective policy change, mid-period policy change, closed payroll period, stale preview.
- Failures: missing references, disabled policy, overlap, DB conflict, transaction rollback, dependency timeout.
- Security: unauthenticated, forbidden role, unauthorized Shift/Work Policy mutation, audit event without PII.
- Compatibility: intentional pre-production clean break; migration/reset of dev rows, generated API types, and no legacy fallback.

## Done when
- `Shift` exposes schedule facts only; no grace/Early OT/OT eligibility fields remain in its contract, storage, or editor.
- `Work Policy` exposes explicit attendance and Early OT rules with effective-dated membership/version resolution.
- UI wording matches actual semantics in all supported locales.
- Calculation combines point-in-time Shift facts and Work Policy rules and preserves provenance.
- Negative/forbidden/conflict paths are tested.
- Clean migration/reset path is verified; no silent legacy fallback exists.
- Integration, browser acceptance, migration rehearsal, and full LeeDevKit gates pass.

## Review focus
- Existing Shift columns and UI currently mix schedule facts with policy rules; clean cutover must update all readers/writers.
- `early_punch_in_grace_period_minutes` currently has ambiguous semantics and must not be silently mapped to one new rule.
- `is_night_shift` is derived from schedule boundaries; avoid storing duplicate derived state.
- Early OT must remain distinct from approved/payable OT and must not alter locked payroll history.
