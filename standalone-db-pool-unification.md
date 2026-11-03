# Standalone DB pool unification

Status: Verified with full-suite infrastructure blocker

## Goal
In standalone mode, transaction tenant requests reuse `systemPool`; session tenant requests reuse `workerPool`; no dynamic tenant pool is created. SaaS dynamic database-per-workspace isolation remains unchanged.

## Rule compliance
- DB-1: Tenant queries retain explicit `workspace_id` predicates.
- DB-3: Repository transaction ownership stays at existing `service.WithTx`/`pgxpool.Pool.Begin` boundaries.
- CFG-2: Only configured standalone workspace resolves; nil or other IDs fail closed.
- CM-1: Review required before merge.

## Decision
Choose Option A. `systemPool` and `workerPool` already have separate transaction/session pooler configuration and owned close lifecycle. Option B removes those semantics without evidence they are interchangeable.

## Tasks
- [x] Map pool manager, callsites, bootstrap, repository/service contract, Compose, migration inventory, docs, and existing tests.
- [x] Confirm deployment contract: release pre-migrates one database, applies `migrations/system/` then `migrations/tenant/`, and shares migration history. Migration directory versions are globally unique.
- [x] Add manager regression tests: configured workspace returns system/worker pools, makes no third open or cache entry, rejects other IDs, and eviction leaves shared-pool ownership intact.
- [x] Preserve and extend SaaS tests: URL derives `tenant_<workspace_id>` and distinct workspace requests open isolated pools. Existing `service.WithTx` tests continue to cover commit/rollback behavior unchanged.
- [x] Add standalone bootstrap regression coverage with a transaction-compatible pool test double; it proves idempotent singleton initialization commits against shared-schema tables.
- [x] Cut standalone path before dynamic cache/URL construction; preserve SaaS path unchanged.
- [x] Update standalone topology and release-pre-migration deployment documentation; run required wrappers and actual standalone smoke.

## Behavior
- Standalone before: validates configured workspace, then opens/cache a tenant pool with tenant limits against shared database URL.
- Standalone after: validates configured workspace, returns pre-owned system/API or worker/session pool, never derives `tenant_*`, opens no tenant pool, and has no tenant cache entry to evict/close.
- SaaS before/after: derives `tenant_<workspace_id>` from tenant base URL; each transaction/session workspace remains independently cached and evictable.

## Migration and rollback
No schema SQL change. Deployment applies `migrations/system/` then `migrations/tenant/` once to standalone's shared database, with globally unique directory versions verified before implementation. Both initial migrations define `fn_set_updated_at` with `CREATE OR REPLACE`, so order is compatible. Roll back by restoring previous application image; no data migration is required for pool ownership change.

## Verification
- Passed: `./scripts/_go-gates.sh --fix`, `./scripts/_go-gates.sh`, targeted pool and bootstrap race tests, and `./leedevkit test web` (3014/3014).
- Passed: standalone Compose structural smoke with non-secret placeholders.
- Earlier blocker: `./leedevkit test all` used stale `api-go`/`apiserver` service names; active Compose, devkit targets, lifecycle health checks, CI override, and Go gate now use canonical `apiserver`. CLI target remains `api`.
- Active configuration/docs audit: `apiserver/.env.example` and `docs/database-architecture.md` describe standalone shared DB and SaaS-only dynamic tenant URLs.
