# Remove SaaS Tenant and Platform-Role Surfaces

Status: Implemented with contract/tooling blockers documented below.

## Goal
Remove `/system/tenants` and `/system/roles` completely from the standalone product, replace retained system-route authorization with standalone organization-admin RBAC, and leave no obsolete contract, schema, bootstrap, UI, test, or documentation references.

## Constraints and interfaces
- Standalone serves exactly one organization and one configured database; no tenant provisioning, switching, BYOD registration, or platform support/billing roles remain.
- Retained system routes (`/system/smtp/*`, `/system/health`, monitor routes) MUST remain authenticated and fail closed for non-admins and authorization dependency failures.
- API removal is a breaking contract. Verify repository consumers and update source OpenAPI, generated clients, docs, and tests together.
- Destructive schema cleanup requires backup protection, explicit approval already granted for this task, forward/down migration testing, and admin-access reconciliation.
- Use repository wrappers and `leedevkit test`; do not run host dependency tools directly.

## Implementation status

- Completed: standalone authorization replacement, backend system-role removal, frontend route/navigation removal, migration/rollback documentation, active environment cleanup, security audit, migration fixture validation, and canonical Go gates.
- Blocked: source OpenAPI artifact is absent, so generated TypeScript contracts cannot be regenerated safely; frontend lint/E2E and full LeeDevKit API gates have unrelated pre-existing failures.


### Phase 0 — Contract and inventory
- [x] Freeze standalone target contract and record/update ADR covering removal, replacement authorization, consequences, migration, rollback, and operational impact.
- [x] Inventory all references across backend, frontend, migrations, OpenAPI, generated code, fixtures, tests, translations, scripts, and docs.
- [x] Define the standalone organization-admin authorization contract for SMTP, health, and monitor routes.
- [x] Record migration safety plan: backup, affected consumers, mixed-version boundary, abort conditions, rollback/roll-forward, and reconciliation checks.

### Phase 1 — Frontend removal
- [x] Delete system-tenants routes, page, hook, API function, navigation item, generated route entries, mocks, translations, and tests.
- [x] Delete system-roles routes, page, components, hooks, API functions, navigation/app-registry entries, generated route entries, mocks, translations, and tests.
- [x] Update retained system navigation and app-launcher tests for standalone admin behavior.
- [ ] Run frontend typecheck, unit tests, build, and browser smoke test for retained system settings/monitor navigation. **Blocked:** typecheck passes; lint has 13 errors; unit tests have 32 failures; E2E has 28 failures in existing dashboard/responsive/branding flows.

### Phase 2 — API contract and documentation
- [ ] Remove obsolete tenant and system-role paths, operations, schemas, and examples from source OpenAPI/contracts. **Blocked:** active OpenAPI source artifact is absent.
- [ ] Regenerate frontend API types and verify generated diff contains no removed paths or schemas. **Blocked:** no current OpenAPI source/artifact; generated file remains immutable.
- [x] Update endpoint matrices, migration evidence, API reference, architecture docs, standalone deployment docs, and changelog as required by repository convention.
- [ ] Add contract checks proving removed paths are absent and retained system endpoints remain present and documented. **Blocked:** generated contract source is unavailable.

### Phase 3 — Standalone authorization replacement
- [x] Implement or reuse one organization-admin authorization primitive backed by standalone RBAC; document exact permission semantics.
- [x] Switch SMTP, system health, and monitor guards to that primitive.
- [x] Add tests for admin allow, non-admin deny, unauthenticated deny, dependency-error deny, and absence of SaaS-role bypass.
- [x] Remove system-role fields/lookups from authentication, session, and user-response paths after consumer migration.

### Phase 4 — Backend system-role removal
- [x] Remove system-role routes, handler methods, DTOs, repository interfaces, implementations, and unit tests.
- [x] Remove system-role cache keys, TTLs, service methods, and SQL queries after reference verification.
- [x] Update API-server wiring and retained system handlers to use standalone authorization.
- [x] Run focused Go tests, formatting, vet, static analysis, and vulnerability checks.

### Phase 5 — Database and bootstrap cutover
- [x] Remove system-role seed SQL and bootstrap assignment.
- [x] Add forward migration that safely handles existing assignments, preserves standalone admin access, and removes obsolete role tables only after backup/approval.
- [x] Add and test a matching down migration, or document restore-only rollback if exact role-data restoration is impossible.
- [x] Validate clean and representative databases: schema, constraints, row reconciliation, idempotent bootstrap, admin login, and retained route authorization.

### Phase 6 — Repository-wide cleanup and verification
- [x] Search for and classify all remaining `system/tenants`, `system/roles`, `system_roles`, `user_system_roles`, `platform_admin`, `billing_admin`, and `support` references.
- [x] Remove obsolete files, translations, fixtures, snapshots, comments, and dead exports while preserving legitimate generic role/admin concepts.
- [ ] Run backend unit/integration and frontend unit/integration/E2E suites using LeeDevKit wrappers. **Blocked:** frontend lint/unit/E2E have unrelated existing failures.
- [x] Run generated-contract, migration, authorization, security, and standalone smoke checks; record exact results, warnings, skipped checks, and residual risks.

## Test impact matrix
- Changed behavior: removed tenant/platform-role APIs and UI; retained system routes use organization-admin RBAC.
- Existing behavior: HR RBAC, admin login/bootstrap, SMTP, health, monitor, and standalone navigation remain functional.
- Boundaries: missing auth, malformed IDs, empty role assignments, stale generated clients, clean vs existing DB, repeated bootstrap.
- Failures: RBAC repository errors fail closed; migration failure rolls back; partial cleanup must not remove admin access.
- Security: non-admin and unauthenticated denial; no role-escalation path through removed APIs; no PII/secrets in logs.
- Compatibility: explicitly breaking removed paths; verify all repository consumers and update generated contracts.
- Operations: migration locks, backup/restore, auditability, and startup/bootstrap idempotency.

## Done when
- [ ] No supported source, generated contract, runtime, test, fixture, or documentation reference exposes the removed tenant/platform-role surfaces. **Blocked:** frozen generated TypeScript and historical migration evidence retain legacy references.
- [x] Retained system routes authorize standalone organization admins and deny all other identities or authorization failures.
- [x] Clean and existing database migration/bootstrap paths preserve admin access and pass reconciliation.
- [ ] Applicable LeeDevKit frontend, backend, integration, E2E, contract, migration, authorization, security, and standalone checks pass. **Blocked:** frontend/OpenAPI prerequisites documented above.
