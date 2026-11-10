# ADR-013: Single-organization persistence boundary

- Status: Accepted
- Date: 2026-11-10
- Scope: PostgreSQL schema, migration lifecycle, API organization context

## Context

LeeAttend is an on-premise single-organization appliance, not a SaaS multi-tenant product. The previous schema retained `workspaces`, `workspace_*` tables, and `workspace_id` columns from the earlier architecture. That compatibility layer added tenant concepts that the active runtime no longer supports.

The application has not entered production. No old production database upgrade path is required.

## Decision

Use one PostgreSQL database and one implicit organization scope. The fresh baseline schema uses `organization_settings` as the singleton organization settings table and has no `workspaces`, `workspace_*` tables, or `workspace_id` columns.

Squash the historical migration chain into one fresh-install baseline. Development and test databases are reset/recreated. The migration runner rejects stale or partially migrated databases instead of silently dropping data or attempting an old-schema upgrade.

Organization scope comes from trusted server bootstrap. Clients cannot select organization scope through URL, headers, request data, or tokens.

## Alternatives rejected

1. Keep legacy workspace tables: preserves obsolete SaaS concepts and permits schema/runtime drift.
2. Add a destructive `DROP TABLE workspaces` migration: retains unnecessary migration complexity and makes fresh installation depend on legacy schema.
3. Maintain an old-database upgrade path: unnecessary before production and increases irreversible migration surface.

## Consequences

- Fresh installation is the supported database lifecycle.
- Existing development databases must be reset.
- Databases created by the old chain fail with an explicit reset/recovery message.
- No automatic rollback can reconstruct former tenant boundaries after schema contraction.
- Historical Rust-to-Go migration material may mention workspace terminology but is not active runtime input.

## Security and privacy

Removing client-selectable tenant scope reduces accidental cross-organization routing. Authorization remains explicit through user identity, roles, permissions, ownership, and department boundaries. Logs and API payloads must not expose obsolete workspace identifiers.

## Operational and recovery impact

The baseline runs atomically. Before any destructive development reset, confirm database/volume is disposable. Recovery means recreating a disposable database or restoring a backup/snapshot. Do not treat `down.sql` as a production recovery mechanism.

The migration acceptance gate compares final schema catalogs for tables, columns, types, defaults, nullability, constraints, indexes, triggers, functions, comments, and extensions before legacy migration files are removed.

## Migration and ownership

- Baseline: `apiserver/migrations/20261110000000_single_organization_baseline/`
- Runner: `apiserver/migrations/run-dev.sh`
- Owner: LeeAttend repository maintainer
- Removal criterion: no active runtime, migration runner, or schema object depends on workspace terminology
