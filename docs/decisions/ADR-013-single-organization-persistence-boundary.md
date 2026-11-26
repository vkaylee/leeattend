# ADR-013: Single-organization persistence boundary

- Status: Accepted — phased fresh-install amendment
- Date: 2026-11-10; amendment 2026-11-17
- Scope: PostgreSQL schema, migration lifecycle, API organization context

## Context

LeeAttend is an on-premise single-organization appliance, not a SaaS multi-tenant product. The previous schema retained `workspaces`, `workspace_*` tables, and `workspace_id` columns from the earlier architecture. That compatibility layer added tenant concepts that the active runtime no longer supports.

The application has not entered production. No old production database upgrade path is required.

## Decision

Use one PostgreSQL database and one implicit organization scope. The fresh baseline schema uses `organization_settings` as the singleton organization settings table and has no `workspaces`, `workspace_*` tables, or `workspace_id` columns.

Squash the historical migration chain into a fresh-install schema organized into five readable phases: `foundation`, `hr_core`, `attendance_schedule`, `payroll`, and `runtime_seeds`. Development and test databases are reset/recreated. The migration runner rejects stale or partially migrated databases instead of silently dropping data or attempting an old-schema upgrade. Historical numeric migrations remain the schema oracle until catalog comparison and runtime verification pass.

Existing-data backfills are not run on fresh databases. Destructive compatibility cleanup is represented directly in final schema state. Runtime seed data is idempotent; user-dependent formula seeds run only after standalone bootstrap creates its administrator.

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
- Historical migration material may mention workspace terminology but is not active runtime input.

## Security and privacy

Removing client-selectable tenant scope reduces accidental cross-organization routing. Authorization remains explicit through user identity, roles, permissions, ownership, and department boundaries. Logs and API payloads must not expose obsolete workspace identifiers.

## Operational and recovery impact

The baseline runs atomically. Before any destructive development reset, confirm database/volume is disposable. Recovery means recreating a disposable database or restoring a backup/snapshot. Do not treat `down.sql` as a production recovery mechanism.

Phase SQL must add final constraints after referenced tables exist, without weakening them. Historical phase ownership and cross-phase dependency records remain available in git history.

## Migration and ownership

- Active schema source: `apiserver/migrations/phases/` (`foundation`, `hr_core`, `attendance_schedule`, `payroll`, `runtime_seeds`)
- Schema contracts: `apiserver/migrations/testdata/phased_schema_contract.sql`, `apiserver/migrations/testdata/employment_relationship_integrity.sql`
- Runner: `apiserver/migrations/run-dev.sh`
- Owner: LeeAttend repository maintainer
- The numeric chain was removed from the working tree on 2026-11-26; it remains available in git history. A PostgreSQL 18 comparison checked selected catalog properties and found differences in CHECK constraints, including a stricter `time_entry_snapshot` check and consolidated minute checks. This comparison did not establish full catalog equivalence or explain the origin of those differences. Fresh-phase contract, disposable reverse/reapply, and `leedevkit test all --json` passed; review remaining catalog differences before relying on parity.
