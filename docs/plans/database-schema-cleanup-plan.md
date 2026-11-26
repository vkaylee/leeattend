# Database Schema Evaluation & Cleanup Plan (Pre-Go-Live)

Status: Track 1 complete & synced to local dev DB; Track 2 (`employee_shifts`) audit complete — **not redundant** (awaiting keep/rename vs product-retirement decision).

## Goal
Evaluate and remove verified legacy database tables remaining from the 2-DB / Rust-agent / pre-lifecycle eras before go-live, while preserving confirmed roadmap tables and active domain boundaries.

## Constraints and Interfaces
- **Global constraints:**
  - Pre-go-live disposable DB recreation is permitted; every destructive removal still requires explicit approval on this plan and verified fresh-install + integration tests.
  - Keep confirmed roadmap tables untouched: `attendance_sources`, `attendance_source_pull_states`, `external_attendance_events`, `payroll_incentive_assignments`.
  - Keep `employee_documents` separate from `files` (T2/T3 legal retention and `(tenure_id, employee_id)` ESS isolation per `docs/operations/data-inventory.md`).
  - No public API route, JSON field, or permission contract breaks without explicit approval.
  - Applicable rulebooks: `database-rules.md`, `migration-and-rollback.md`, `data-governance.md`, `access-control.md`, `coding-standards.md` (Dead Code Policy), `testing-standards.md`.
- **Global interfaces:**
  - Phased schema DDL: `apiserver/migrations/phases/{foundation,hr_core,attendance_schedule,payroll}/{up,down}.sql`
  - Schema contract & integrity: `apiserver/migrations/testdata/phased_schema_contract.sql`, `employment_relationship_integrity.sql`
  - Go repositories & tests: `apiserver/internal/repository/`
  - Verification gate: `leedevkit test api --json` (unit + integration + lint) and fresh local dev DB rebuild (`COMPOSE_PROJECT_NAME=leedevkit-dev`).

## Candidate Classification (Decision Gate)

| Table(s) | Current Evidence | Proposed Action | Gate / Blocker |
|---|---|---|---|
| `authorized_agents` | Legacy agent-related token/worker schema, with no Go references found; historical pull plan does not establish current implementation status. | **Evaluate** | Verify current integrations and intended ownership before deletion. |
| `system_audit_logs` | Legacy system-DB audit subset from 2-DB split; `audit_logs` is sole active compliance log (T2, 3y retention per `data-inventory.md`); 0 Go refs; 1 FK to `users`. | **Drop** | Confirm no separate retention/isolation policy requires a second audit table. |
| `system_files` | Legacy system-DB file table; `employee_profiles.avatar_file_id` already points to `files(id)` (`hr_core/up.sql:872`), only `users.avatar_file_id` still FKs `system_files(id)` (`foundation/up.sql:2917`); 0 Go queries on `system_files`. | **Retarget FK → Drop** | Verify `users.avatar_file_id` upload/read paths work against `files(id)` with proper access control. |
| `policy_templates` | 0 FKs, 0 Go/web/doc/script refs. | **Decide: Drop or Keep** | Product owner confirmation: planned reusable policy template library or dead schema? |
| `employee_shifts` | **Not redundant** despite stale `DEPRECATED` comment: stores open-ended (`end_date IS NULL`) fixed shift assignments feeding preview (`schedule_lifecycle_preview.go:69`), publication (`schedule_lifecycle_publication.go:95`), compliance (`pattern_compliance.go:240`), swaps (`shift_swaps.go:595`), and history (`assignment_history.go:32`). Cannot be replaced by day-level `schedule_day_revisions` or higher-precedence `employee_working_patterns` without losing semantics. | **Keep & Rename** (`employment_fixed_shifts`) or retire feature | Decide whether fixed shift assignment stays a product feature (recommended: rename & remove misleading `DEPRECATED` comment) or is retired across API/UI/seed. |
| `employee_documents`, `files` | Distinct domain lifecycles & legal retention (`employee_documents` linked to contracts & tenures). | **Keep Separate** | None. |

## Tasks

### Track 1 — Quick-Win Legacy Cleanup (Pre-Go-Live)
- [x] Task 1: Verify each candidate's runtime consumers, intended product use, FK dependencies, retention, and security boundaries → Constraints: absence of literal references is not proof of redundancy; retain unresolved candidates → Interface: table disposition table above → Evidence: application has not gone live; `files` includes the `system_files` metadata columns; template gallery uses frontend constants. External consumers and other environments have not been independently verified.
- [x] Task 2: Verify `users.avatar_file_id` writers/readers (`apiserver/internal/repository/auth.go`, `files.go`, `api/auth.go`) and retarget `users_avatar_file_id_fkey` from `public.system_files(id)` to `public.files(id)` (retaining existing `NO ACTION` delete behavior) → Constraints: preserve avatar authorization & T2/T3 access checks (`access-control.md`, `data-governance.md`) → Interface: `foundation/up.sql` FK + `files` table → Verify: `git grep -n system_files apiserver/` clean outside negative contract; FK verified in `phased_schema_contract.sql`.
- [x] Task 3: Drop approved Track 1 tables (`authorized_agents`, `system_audit_logs`, `system_files`, `policy_templates`) from `foundation/{up,down}.sql`, including their PKs, indexes, triggers, and FKs → Constraints: do not touch roadmap tables or `audit_logs` partitions → Interface: `apiserver/migrations/phases/foundation/{up,down}.sql` → Verify: `git grep -n -E 'authorized_agents|system_audit_logs|system_files|policy_templates'` clean in `apiserver/migrations/phases/`.
- [x] Task 4: Add negative table assertions (`to_regclass(...) IS NULL`) for dropped Track 1 tables and verify `users_avatar_file_id_fkey -> public.files` in `phased_schema_contract.sql`; update `docs/architecture/database-architecture.md` → Constraints: keep historical plans unchanged → Interface: `phased_schema_contract.sql`, architecture docs → Verify: `git diff --check` clean.

### Track 2 — Resolve `employee_shifts` (Post-Audit Options)
- **Verified finding:** `employee_shifts` is **not** replaceable by `schedule_day_revisions` (day-level published snapshots) or `employee_working_patterns` (higher precedence `roster > pattern > rule > fixed > none`, different history shape). Its legacy `DEPRECATED` comment (`attendance_schedule/up.sql:1477`) and `employee_shifts_new_*` PK/FK names are misleading artifacts.
- [ ] Option A (Recommended — Keep & Rename): Rename `employee_shifts` → `employment_fixed_shifts`, normalize `employee_shifts_new_*` constraint names, update `guard_legacy_schedule_mutation()` `TG_TABLE_NAME`, Go SQL queries, seed, tests, and replace stale `DEPRECATED` comment. Preserves 100% API/UI/lifecycle semantics.
- [ ] Option B (Feature Retirement — Breaking): Explicitly retire fixed shift assignment (`POST /api/v1/shifts/assign`, `"fixed"` resolver/history source, `scripts/seed_schedule_lifecycle_dev.sql` fixed seed) across backend + frontend, then drop table. Requires product sign-off.
- [ ] Option C (Minimal): Keep `employee_shifts` table name as-is; update misleading comment only.

### Final Verification (Per Track)
- [ ] Task 7: Run full verification gate (`leedevkit test api --json`), recreate disposable local dev DB (`leedevkit-dev`), run `phased_schema_contract.sql` and `employment_relationship_integrity.sql`, and verify `apiserver` health → Constraints: never report complete on `--unit-only`; confirm dev target before DB reset → Interface: disposable test stack + local dev stack → Verify: `leedevkit test api --json` passes `unit`, `integration`, `lint` (`failed=0`); live dev DB passes both SQL contracts and `apiserver` is `healthy`.

## Done When
- [ ] Approved Track 1 legacy tables are absent from fresh-install phases and enforced absent in `phased_schema_contract.sql`.
- [ ] `users.avatar_file_id` references `public.files(id)` consistently with `employee_profiles.avatar_file_id`.
- [ ] `employee_shifts` is retired only after all runtime writers/readers are cut over and verified (Track 2).
- [ ] Roadmap tables (`attendance_sources*`, `external_attendance_events`, `payroll_incentive_assignments`) and `employee_documents` remain intact.
- [ ] Full `leedevkit test api --json` suite and local dev DB contract checks pass.

## Review Focus
- `users.avatar_file_id` deletion semantics (`ON DELETE SET NULL` vs `NO ACTION`) when an avatar file in `files` is deleted or deactivated.
- Any implicit `audit_logs` expectation for system/bootstrap events previously intended for `system_audit_logs`.
- Whether `webdashboard` still calls `POST /api/v1/shifts/assign` in Track 2 before changing its persistence target.
- `scripts/seed_schedule_lifecycle_dev.sql` compatibility when `employee_shifts` is removed in Track 2.
