# Phase SQL generation plan

Status: generator preview executes successfully on disposable PostgreSQL; generated migration SQL is not committed.

## Source

Use schema-only PostgreSQL dump captured in `migration-catalog-oracle.md` as source. Do not hand-copy table definitions. Keep historical numeric migration chain unchanged until generated phases pass catalog comparison.

## Generator and validator

```bash
./scripts/_python.sh scripts/validate-migration-phase-map.py \
  --dump /tmp/leeattend-schema-oracle.sql \
  --map docs/plans/migration-phase-source-map.tsv \
  --cross-dependencies docs/plans/migration-phase-cross-dependencies.tsv

./scripts/_python.sh scripts/generate-migration-phases.py \
  --dump /tmp/leeattend-schema-oracle.sql \
  --map docs/plans/migration-phase-source-map.tsv \
  --output /tmp/migration-phase-preview-20261117-v12
```

Validator result:

```text
migration phase map: PASS objects=162 cross_phase_fks=67 attendance_schedule=50, foundation=58, hr_core=31, payroll=23
```

Preview section inventory matches oracle after intentional omission of upgrade-only retirement snapshot objects. Preview v12 executes on disposable PostgreSQL after fixing table ownership for normal/partitioned indexes and stripping pg_dump `\\restrict`/`\\unrestrict` directives:

```text
foundation applied
hr_core applied
attendance_schedule applied
payroll applied
runtime_seeds applied
```

Disposable catalog:

```text
tables:    161
views:       1
functions: 258
triggers:   78
indexes:   537
```

All five markers and phased schema contract pass. Reverse rollback pass also verified: runtime seeds remove only owned rows, reverse phase `down.sql` files commit successfully, and public schema object count reaches zero. Disposable database was removed after verification.

Verified preview was promoted to `apiserver/migrations/phases/`. Active runner now applies only these five fresh phases and rejects historical/partial marker state. Historical numeric migrations remain in repository as oracle and are not scanned by active runner.

Preview section inventory matches oracle after intentional omission of upgrade-only retirement snapshot objects:

```text
oracle sections:    1168
preview sections:   1162
omitted:            salary_base_retirement_backup schema, groups/members tables,
                    two snapshot PK constraints, and groups_slug_idx
```

All remaining section counts match for extensions, types, functions, public tables, partition attachments, views, constraints, indexes, triggers, FK constraints, and comments. The omitted objects belong to non-public `salary_base_retirement_backup`, created only by historical destructive retirement migration; fresh install must not recreate them.

Generator tests:

```bash
./scripts/_python.sh -m pytest \
  scripts/tests/test_migration_phase_map.py \
  scripts/tests/test_migration_phase_generator.py -q
```

Result: `9 passed` (including runtime seed contracts).

## Dump section classes

The oracle contains:

```text
EXTENSION: 2
TYPE: 17
FUNCTION: 15
TABLE: 163 dump sections
TABLE ATTACH: 26
VIEW: 1
CONSTRAINT: 222
INDEX: 325
INDEX ATTACH: 91
TRIGGER: 78
FK CONSTRAINT: 184
COMMENT: 43
```

`TABLE` count includes parent and child partition definitions; source map records 162 table/view objects because the dump inventory treats the view separately. Preserve all partition attach and index attach sections in `attendance_schedule` or `foundation` according to parent ownership.

## Generation order

Generate each phase in this order:

1. `BEGIN;`
2. shared extensions/types/functions in `foundation`
3. phase-owned table definitions
4. partition child definitions and `ATTACH PARTITION`
5. local primary/unique/check/exclusion constraints
6. local indexes
7. functions/triggers whose referenced tables now exist
8. cross-phase foreign keys assigned by install-order rule
9. views
10. required phase contract checks
11. insert phase marker
12. `COMMIT;`

Do not copy dump `SET`, owner, ACL, `\restrict`, `\unrestrict`, `ALTER DEFAULT PRIVILEGES`, or data sections into application migrations.

## Phase marker contract

```text
fresh_foundation_v1
fresh_hr_core_v1
fresh_attendance_schedule_v1
fresh_payroll_v1
fresh_runtime_seeds_v1
```

Marker insertion must happen only after all phase objects and required constraints succeed. Failure must roll back marker and phase SQL together.

## Cross-phase FK rule

Use `migration-phase-cross-dependencies.tsv`. For each edge, create FK in later install phase. This handles 67 cross-phase edges without a sixth phase. Same-phase cycles create after all tables in that phase exist.

## Partition rule

Partition parents and children stay together:

- `audit_logs*`: foundation
- `attendance_staging_logs*`: attendance_schedule

Create parent, child tables, partition bounds, parent constraints/indexes, child attachments, and child index attachments in same phase transaction.

## Seed rule

Generated DDL currently contains no seed rows because dump is schema-only. Runtime seed ownership stays separate. Seed files:

```text
docs/plans/runtime-seeds.sql
docs/plans/runtime-seeds.down.sql
```

Run seed SQL after the four DDL phases with an explicit `seed_effective_date` psql variable. Runtime seed contract tests cover idempotency, no workspace leakage, no arbitrary user dependency, and bounded rollback.

Runtime seed ownership stays separate:

- `ff_econtracts`, `ff_ess`: runtime seed
- production payroll metric rows: runtime seed, extracted from historical `20261113030000_payroll_input_registry`
- KPI config defaults: runtime seed, extracted from historical `20261114000000_payroll_policy_parameters`
- `kpi.bonus` formula: standalone bootstrap after administrator creation; do not use arbitrary first user
- `schedule_lifecycle`: created/updated by runtime activation flow; do not silently seed enabled state without product decision

All runtime seeds must be idempotent. Avoid `CURRENT_DATE` as seed identity; use explicit effective date policy or bootstrap-controlled current date.

## Generator acceptance checks

Before committing generated phase SQL:

- every oracle table/view maps exactly once
- every oracle extension/type/function/trigger maps exactly once
- every FK maps exactly once
- every partition attach maps with its parent phase
- every phase has `BEGIN`, marker insert, `COMMIT`, and `down.sql`
- no `workspace`, old migration marker, owner, ACL, data, secret, or PII enters fresh phase SQL
- no historical backfill or destructive retirement DML enters fresh phase SQL
- generated SQL is deterministic on repeated generation
- fresh install catalog equals oracle after intentional fresh-only differences are listed

## Deliberate simplification

Do not build a generic SQL parser. PostgreSQL dump section boundaries are stable and already marked by `-- Name: ...; Type:`. Use small repository-local generator only after confirming no existing generator covers this path. Keep generator code separate from generated migration files and add runnable self-checks for object coverage and determinism.
