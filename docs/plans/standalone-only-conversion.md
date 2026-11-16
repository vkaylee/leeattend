# Standalone-Only Conversion

Status: Approved

## Goal
Retire SaaS runtime and deployment topology. LeeAttend runs one local organization in one PostgreSQL database while preserving `workspace_id` in routes, JWTs, RBAC, audit records, and tenant-table predicates.

## Decisions
- **Data:** no SaaS data exists. No multi-database data migrator or compatibility shim.
- **API contract:** retain `/api/v1/workspaces/{ws_id}/...`; the configured singleton ID remains required and any other ID is rejected before repository access.
- **Cutover:** greenfield, clean removal. Remove SaaS deployment, provisioning, dynamic pool, BYOD, and tenant-pooler paths rather than deprecating them.
- **Authoritative deployment:** `docker-compose.yml` becomes standalone deployment. Delete `docker-compose.standalone.yml`; do not add Compose profiles for a topology no longer supported.
- **Retained data model:** retain `workspaces`, membership/role tables, `workspace_id` columns, and every tenant query filter. One database is not permission to remove logical authorization boundaries.

## Constraints and Interfaces
- Rule IDs: `database-rules.md §1, §3-4`; `access-control.md §1-2`; `configuration-management.md §2-4`; `change-management.md §3, §5`; `data-governance.md §1-5`.
- No destructive migration or table/column drop without explicit human sign-off. Existing migrations stay immutable; new migrations need verified `down.sql`.
- No external API consumer exists. Route removal remains documented as a clean cutover in release notes.
- Startup still fails closed for UUID, database URLs, TLS, JWT, encryption keys, license, and bootstrap credentials. No secret values in Compose, docs, logs, or tests.
- Existing system/tenant migration sequence remains: apply `migrations/system/`, then `migrations/tenant/`, to one database before bootstrap. Preserve globally unique migration versions.

## Tasks
- [ ] 1. Record accepted ADR and release cutover contract in `docs/decisions/`; update `docs/architecture/database-architecture.md` to one-database topology and explicitly retain logical workspace/RBAC boundaries. → Verify: review identifies one database, one organization, no SaaS/BYOD/tenant-pooler contract.
- [ ] 2. Make standalone mandatory in `apiserver/internal/config/config.go` and `main.go`: remove `STANDALONE_ENABLED` branching and SaaS-only tenant-pooler configuration; retain validated singleton workspace, bootstrap admin, and license inputs. Update `apiserver/.env.example`. → Verify: missing singleton/license configuration fails startup; no tenant-pooler environment key is read.
- [ ] 3. Simplify `apiserver/internal/db/pool/manager.go`: remove `BuildTenantURL`, tenant URL generation, `tenantTx`/`tenantSess` caches, eviction, and dynamic-open code. `GetTenantTxPool` always returns API/system pool and `GetTenantSessPool` always returns worker/session pool only after singleton-ID validation. → Verify: pool tests prove exactly two owned pools, foreign workspace rejection, close safety, and absence of dynamic URL/pool behavior.
- [ ] 4. Remove SaaS control-plane behavior: delete workspace listing/creation/provisioning and tenant database/BYOD configuration paths, their route registration, repository/service dependencies, OpenAPI entries, and client calls. Replace standalone-only route middleware with the smallest permanent singleton workspace guard needed by preserved `{ws_id}` routes. → Verify: provisioning routes return 404 because unregistered; valid singleton tenant requests authorize and run; foreign `ws_id` is rejected before repository execution.
- [ ] 5. Preserve one-database schema and bootstrap: confirm all system and tenant migrations apply to a fresh shared database, then bootstrap creates or validates exactly one workspace/admin transactionally. Add only additive, rollback-tested migration changes needed to remove SaaS-only persisted configuration. → Verify: fresh database migration + bootstrap succeeds; second matching bootstrap is idempotent; conflicting workspace state fails closed; no destructive SQL is introduced without signed approval.
- [ ] 6. Replace deployment manifests with one standalone `docker-compose.yml`: one PostgreSQL service, Redis, MinIO, API, dashboard; shared DB URLs; TLS, secret, license, healthcheck, volume, and dependency contracts. Delete `docker-compose.standalone.yml`, SaaS DB/pooler services, legacy migration profile, and tenant pooler configuration. → Verify: `./scripts/_compose-validate.sh` passes with required placeholder environment; rendered config contains one PostgreSQL service and no `DB_TENANT_POOLER_*`, `db_workspace`, or PgBouncer service.
- [ ] 7. Convert development, CI, and test fixtures in `.compose/`, `leedevkit.toml`, DevKit scripts, and test seeds to the one-database contract. Keep isolated ephemeral test containers, not production Compose secrets. → Verify: test fixture migrates system and tenant schemas into one test database; API test target uses canonical `apiserver`; no test fixture starts tenant DB or tenant pooler.
- [ ] 8. Remove SaaS-only docs, examples, release/deployment instructions, migration evidence, and frontend workspace-provisioning surfaces; regenerate OpenAPI/client types if route schema changes. Update data inventory and operator backup/restore runbook for one encrypted appliance database. → Verify: repository search has no active SaaS, BYOD, dynamic tenant database, `db_workspace`, or tenant-pooler references outside archived historical ADR/evidence explicitly marked historical.
- [ ] 9. Add regression coverage for singleton authorization, bootstrap, one-pool data access, migration order, Compose contract, and dashboard behavior without workspace creation/listing. Run security, API, web, and deployment smoke verification. → Verify: `./leedevkit test all`; `./scripts/_compose-validate.sh`; disposable Compose deployment reaches `/health/ready`, supports bootstrap admin login and one attendance write/read, rejects a foreign workspace ID, and cleanly tears down.

## Done When
- [ ] Production, development, CI, and tests use one PostgreSQL database and no tenant pooler/database exists.
- [ ] Runtime has no feature flag or executable branch for SaaS.
- [ ] Every retained tenant request remains workspace-scoped and RBAC-authorized against configured singleton ID.
- [ ] Workspace provisioning/BYOD/multi-workspace routes, APIs, client code, config, and docs are gone.
- [ ] Database backup/restore, migration, license, TLS, secrets, and data-retention operations have standalone runbooks.
- [ ] Full isolated suite and disposable deployment smoke pass.

## Review Focus
- Removing physical tenant isolation must not weaken `workspace_id` query predicates or handler authorization.
- System and tenant migrations must coexist in one migration history without version collision.
- Removal of provisioning must not leave reachable unauthenticated routes or stale OpenAPI/client calls.
- Shared pool ownership must not introduce double-close or transaction/session semantic regressions.
- Appliance backups must cover T1/T2 data encryption, object storage, and restore drills.

## Rollback
Before deployment, take encrypted PostgreSQL and MinIO backups plus rendered Compose/environment-key inventory. Since this is greenfield and no SaaS data exists, rollback means redeploying prior image and prior Compose revision; do not run destructive schema changes in this cutover. If data exists before implementation begins, stop before Task 4 and replace this plan with a workspace-selected export/import and validation plan.
