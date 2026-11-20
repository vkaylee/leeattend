# Migration phase inventory

Status: Research complete; no migration source edited.

## Decision boundary

Fresh-install-only. Development and test databases may be recreated. Existing migration directories remain the schema oracle until the phased schema passes catalog and runtime verification.

## Rule IDs

- `database-rules.md §4`: every migration keeps rollback artifact; no production-destructive drop without approval.
- `migration-and-rollback.md §1-6`: separate schema, data backfill, behavior switch, and cleanup; verify recovery boundary.
- `development-workflow.md §2, §6`: persistence changes require migration, compatibility, recovery, and reconciliation evidence.
- `testing-standards.md §1-2`: run applicable full LeeDevKit targets, not only unit tests.

## Current facts

- Current baseline: `apiserver/migrations/20261110000000_single_organization_baseline/up.sql`.
- Current runner applies baseline, then every numeric directory in lexical order: `apiserver/migrations/run-dev.sh:7-64`.
- Current chain has 1 baseline plus 29 post-baseline migrations through `20261117000000_schedule_day_offsets`.
- Baseline already creates most legacy/core HR, attendance, schedule, and payroll tables. Moving post-baseline directories alone will not create readable domain phases.
- Current schema contract checks both baseline and later migration markers: `apiserver/migrations/testdata/schema_contract.sql:50-428`.
- `20261115010000_department_payroll_reconciliation/up.sql` creates a runtime view; it must not be omitted from the final catalog.

## Target install order

```text
foundation
  -> hr_core
  -> attendance_schedule
  -> payroll
  -> runtime_seeds
```

All phases run inside one migration-controlled fresh install before API startup. `runtime_seeds` needs a bootstrap boundary for user-dependent formula data; see seed policy below.

## Phase inventory

### 00 foundation

Own final shared schema and objects used by every later phase:

- PostgreSQL extensions: `btree_gist`, `pg_trgm`.
- Shared enum/types, UUID/default functions, `fn_set_updated_at`, common validation functions and triggers.
- `schema_migrations`.
- Single-organization/auth objects: `organization_settings`, users, roles, permissions, sessions, files, audit/outbox foundations.
- Common HR entities required by foreign keys: employers, employees, departments, employments, employee contracts.
- Existing attendance/schedule/payroll base tables required by later foreign keys, unless regenerated into their owning phase.
- Final `config_entries` constraints from `20261112000000_config_entries_integrity`: NULL-safe uniqueness and effective-date validity.
- Final shared indexes, foreign keys, partitions, and constraints.

Fresh-only changes:

- Do not reproduce duplicate-row deletion from `20261112000000_config_entries_integrity`; fresh DB has no duplicates.
- Do not create retired `salary_base_groups` or `salary_component_base_members` only to drop them later. Preserve their final absence. Existing upgrade retirement remains historical behavior until chain replacement.
- Keep compatibility columns only when active runtime still reads/writes them; verify each against Go consumers before removal.

### 10 hr_core

Own HR relationship and tenure boundaries:

- `employment_relationships` table and its exclusion constraint, indexes, validators, and triggers from `20261112010000_employment_relationships`.
- Relationship links on `employments` and `employee_contracts`.
- `employee_tenures` and `employee_tenure_reconciliation` from `20261115030000_employee_tenures`.
- `employment_relationships.tenure_id` foreign key and index.

Fresh-only changes:

- Omit `20261112020000_employment_relationship_backfill` DML. No historical rows exist on fresh install.
- Omit the `employee_tenure_reconciliation` insert for existing employees. Keep table and constraints; runtime/bootstrap creates explicit reconciliation rows when needed.
- Do not infer tenure boundaries from legacy dates.

Dependencies:

- Foundation employers, employees, users, employments, employee contracts, and `fn_set_updated_at`.
- Must complete before any future non-null enforcement of relationship or tenure links.

### 20 attendance_schedule

Own attendance closure, schedule lifecycle, publication, official revisions, and legacy-source guards:

- `payroll_periods` from `20261112030000_payroll_periods`. It belongs here because schedule and attendance guards depend on payroll lock state, while payroll periods also depend on attendance closure state.
- Schedule lifecycle tables, official work-segment fields, revision/publication/closure triggers, and outbox fields from `20261113000000_schedule_lifecycle`.
- Schedule activation-range exclusion constraint from `20261114030000_schedule_activation_ranges`.
- Legacy schedule backfill result schema from `20261114040000_schedule_legacy_backfill`: final revision state type and provenance columns. Omit existing-data backfill DML on fresh install.
- `schedule_shadow_mismatches` from `20261114050000_schedule_shadow_reconciliation`.
- `schedule_legacy_mutation_evidence` from `20261114070000_schedule_legacy_mutation_evidence`.
- `schedule_office_policies` from `20261114100000_schedule_office_policies`.
- Legacy mutation guard function/triggers from `20261115000000_schedule_legacy_mutation_guards`.
- Official shift-swap revision columns, constraints, and index from `20261116000000_atomic_official_shift_swaps`.
- Day-offset and UTC-offset columns/checks from `20261117000000_schedule_day_offsets`.

Fresh-only changes:

- Omit `employment_relationship_backfill` and `schedule_legacy_backfill` data movement; retain final schema and provenance state.
- Do not retain migration-only markers for backfill completion in final phase state.
- Keep evidence and reconciliation tables because active repositories and triggers use them.

Dependencies:

- Foundation users, employers, employments, work segments, shift source tables, config entries, feature flags, outbox, and common trigger functions.
- HR relationship schema where schedule rows reference employment relationships or tenure-aware entities.
- Schedule lifecycle schema must precede activation constraints, backfill result schema, evidence tables, and legacy guards.

### 30 payroll

Own payroll policy, inputs, formula calculation, snapshots, and reconciliation:

- Payroll input registry tables and indexes from `20261113030000_payroll_input_registry`; move metric seed rows to `runtime_seeds`.
- Formula rules, immutable formula versions, and calculation runs from `20261113040000_payroll_formula_versions`.
- `retired_at`, `payslip_id`, and payslip index from `20261113050000_payroll_calculation_run_links`.
- Formula/payslip idempotency and published-effective exclusion constraint from `20261114010000_payroll_calculation_run_idempotency`.
- Input validation/correction columns and `payroll_input_corrections` from `20261114020000_payroll_input_row_governance`.
- Labor policy tables and effective-range constraints from `20261114080000_labor_policy_groups`.
- Policy snapshot index from `20261114090000_payroll_calculation_run_policy_snapshot`.
- `payslip_segments.labor_policy_group_version_id` and index from `20261114110000_payroll_segment_policy_snapshots`.
- Rate component column and check from `20261115020000_pay_basis_rate_components`.
- Final append-only policy snapshot shape from `20261115050000_append_only_payroll_policy_snapshots`; do not recreate the removed policy-snapshot unique index.
- `department_payroll_reconciliation` view from `20261115010000_department_payroll_reconciliation`.

Fresh-only changes:

- Do not run destructive salary-base retirement. Build final schema without retired tables.
- Do not apply the migration that only drops a superseded index; encode final index state directly.
- Keep all final constraints and indexes even when their historical migration also contained data cleanup.

Dependencies:

- Foundation users, departments, employments, payslips, payslip segments, salary component definitions, attendance policies, overtime policies, and config entries.
- `payroll_periods` and attendance closure tables from `attendance_schedule` must exist before runtime guards and payroll lifecycle use.
- Formula tables must exist before calculation-run links, idempotency constraints, snapshots, and KPI formula seed.

### 40 runtime_seeds

Own deterministic default data, not schema:

- Feature flags from `20261113010000_feature_flags_runtime` and `20261115040000_ess_feature_flag`.
- Payroll metric definitions from `20261113030000_payroll_input_registry`.
- Organization KPI config defaults from `20261114000000_payroll_policy_parameters`.
- Other documented default feature/config rows required by fresh bootstrap.

Seed policy:

- All seed writes idempotent with conflict handling.
- No secrets, user data, test data, environment-specific values, or `CURRENT_DATE`-dependent identity in install seeds.
- `20261114060000_payroll_kpi_formula_seed` is user-dependent: it selects a user ID. Do not let migration marker claim success when no formula row was inserted.
- Preferred ownership: bootstrap creates organization/admin first, then inserts KPI formula using that admin ID; remove formula-seed responsibility from generic pre-bootstrap migration, or split it into an explicit post-bootstrap step.
- If migration runner must own it, runner must have an explicit verified bootstrap-user contract and fail when that user is absent.

## Explicitly not carried into fresh phases

- Existing-data backfills: employment relationship backfill and schedule legacy backfill DML.
- Destructive compatibility retirement: salary-base snapshot/drop sequence.
- Historical migration markers and rollback workarounds.
- Temporary upgrade-only cleanup.
- Seed values that depend on current date or an arbitrary first user.

## Catalog oracle and verification sequence

1. Apply current migration chain to disposable database A.
2. Export schema-only catalog from A. Exclude data, owners, ACLs, secrets, and PII.
3. Build phased fresh schema on disposable database B.
4. Compare tables, columns, types, nullability, defaults, PK/FK/unique/check/exclusion constraints, indexes, functions, triggers, views, extensions, and required seed contracts.
5. Test failure inside each phase: transaction rollback leaves no phase marker or partial phase objects.
6. Test rerun/idempotency and stale/partial marker rejection.
7. Run schema contract tests against B, then:

```bash
leedevkit test api
leedevkit test all
```

8. Inspect per-phase logs. Do not rely on final green summary alone.
9. Only after clean catalog/runtime evidence: replace runner and archive old chain.

## Next implementation slice

Create phase manifests and a catalog comparison check first. Do not delete or rename current migrations yet. Do not reset any database until the target volume is proven disposable.
