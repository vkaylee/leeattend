# Remove salary base groups

## Scope and authorization
User approved removing the entire salary base-group model, including custom groups and memberships. Preserve salary component definitions, employee amounts, historical payroll, and unrelated working-tree changes. Do not build the future formula engine. Do not execute destructive changes against production.

## Rules
- Coding Standards §2: remove replaced code and verify consumers.
- Database Rules §4: forward and rollback migration; user authorized dropping the two retired tables.
- Migration and Rollback §2, §4: protect non-reconstructable data and verify recovery.
- Development Workflow §2, §7: test impact and verified evidence.
- API Governance §2: user-approved breaking removal; backend and dashboard must deploy together.

## Test impact
| Dimension | Verification |
| --- | --- |
| Changed behavior | No base-group tab, selector, fetch, endpoints or bootstrap seeds. |
| Preserved behavior | Component create/list and employee salary amounts; historical payslip fields unchanged. |
| Inputs | Component request contains name/code without memberships; malformed input still rejected. |
| State | Fresh and existing databases, including custom groups and memberships. |
| Failures/recovery | Forward/rollback migration protects existing data and fails atomically. |
| Security | Retained salary permissions unchanged; removed endpoints inaccessible. |
| Compatibility | Remove client wrappers/generated contracts; coordinated deployment required. Existing rule consumers reviewed before removal. |
| Operations | No production migration execution; hermetic verification only. |

## Steps
- [x] Remove backend group API/repository/seed and add reversible migration tests.
- [x] Remove dashboard group UI/client/contracts and update regression tests.
- [ ] Run focused checks, then `leedevkit test all`; inspect individual phases.
- [ ] Record exact results and any remaining blockers.

## Migration operations
- Deploy the API and dashboard together; the retired `base_group_ids` request field is rejected and salary-base routes return 404.
- Forward migration snapshots exact group/member rows into restricted `salary_base_retirement_backup`, then drops both public tables without CASCADE. Unexpected dependencies abort the transaction.
- Rollback uses `20261113020000_retire_salary_base_groups/down.sql`; it restores rows, constraints and trigger, then removes snapshots. If referenced component definitions were deleted after retirement, rollback fails atomically: retain the snapshot and roll forward instead of synthesizing data.
- Owner: payroll maintainer. Snapshot is rollback-only; review after the deployment rollback window, no later than 2026-12-13. Remove only after explicit acceptance and durable backup. No production or active development database was migrated by this task.
- Existing gross/net fields on historical payslips and the current calculation engine remain unchanged. Rule form no longer offers the removed DB-group selector; its `rule_config.base` was not consumed by the current evaluator.

## Verification evidence
- First focused frontend run used file names as a name filter and skipped all 2,996 tests; not counted as a pass.
- Corrected command: `leedevkit test web --unit-only --pattern 'SalaryComponentManager|PayrollRulesManager|PayrollSettings Page|payroll API' --json --quiet`: 48 passed across 4 files; 2,948 unrelated tests skipped. Log: `.test_logs/leedevkit-test-18bfb50b/Unit_Tests_webdashboard.log`.
- PostgreSQL 18 disposable database: baseline plus `apiserver/migrations/testdata/salary_base_retirement.sql` passed with `psql -v ON_ERROR_STOP=1`, through repository Compose wrappers. Covers exact custom/system group snapshots, assigned amount history, retained writes, and up/down/up/down restoration. Disposable database container removed afterward. Container engine emitted a warning about an unrelated existing network JSON file; SQL execution exited 0.
- Full regression command in progress: `leedevkit test all --json --quiet`. Backend unit packages passed (`.test_logs/leedevkit-test-3056d21a/Unit_Tests_go-unit.log`). Inspection found configured Go integration runner has no Go integration task; explicit PostgreSQL fixture above supplies migration integration evidence, not full API E2E coverage.
- Additional `leedevkit test web --json --quiet` in progress. Unit phase: 2,976 passed, 25 failed across 5 files. Failures involve concurrent salary setup translations/router Link mock and schedule lifecycle/shift tests, not the retired-group regression suites. Log: `.test_logs/leedevkit-test-0092f5db/Unit_Tests_webdashboard.log`. No full regression pass claimed.
- Standalone `./scripts/_npm.sh run type-check` blocked: wrapper selects `/usr/bin/python3.14` rather than required `.leedevkit/.venv/bin/python3`. `leedevkit run` is unsupported. Full web target includes typecheck; no wrapper/devkit edits made.
- Generated TypeScript contract edited narrowly: configured `gen-api` expects missing `webdashboard/openapi.json`; full regeneration unavailable. Types/lint must verify the reduced contract.
