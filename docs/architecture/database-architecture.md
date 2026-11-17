# Database Architecture

LeeAttend is a single-organization appliance with one PostgreSQL database and one `public` schema. Fresh installation uses five readable migration phases that create the final organization-scoped model directly: `foundation`, `hr_core`, `attendance_schedule`, `payroll`, and `runtime_seeds`.

## Topology

| Component | Contract |
|---|---|
| PostgreSQL | One database, one `public` schema |
| Fresh-install phases | `docs/plans/migration-phase-manifest.yaml` |
| Historical schema oracle | `apiserver/migrations/20261110000000_single_organization_baseline/` plus current numeric migrations |
| Runtime pools | API and worker pools connect to the same database; no tenant routing |
| Migration runner | `apiserver/migrations/run-dev.sh` |

Physical database split is gone. The baseline creates final organization tables directly, including `organization_settings`, `roles`, `role_permissions`, and `user_roles`; it does not create legacy platform or workspace objects.

## Schema groups

The phased fresh-install schema contains all existing system and business objects. Historical baseline currently owns most of these objects and remains schema oracle until phased catalog comparison passes:

- Authentication and platform objects: `users`, `sessions`, `organization_settings`, `roles`, `role_permissions`, `user_roles`, `user_extra_roles`, `user_extra_permissions`, notifications, storage, and infrastructure ledger tables.
- Business objects: employees, departments, shifts, attendance, leave, payroll, contracts, workflows, files, reports, and outbox tables.
- Partitioned objects: audit and staging-log parents plus their monthly/default partitions.

Business tables carry no organization or tenant key: the one organization is implicit, so `organization_settings` is a singleton row and authorization is stored directly per user. `files`, `compliance_rules`, and `config_entries` are the organization-wide replacements for their former workspace-prefixed names.

## Connection architecture

```
System/API      ──→ API pool     ──→ PgBouncer (transaction mode) ──→ one PostgreSQL database
Business TX     ──→ same API pool ──→ PgBouncer                   ──→ one PostgreSQL database
Worker/session  ──→ worker pool  ──→ same PgBouncer endpoint      ──→ one PostgreSQL database
```

All application, migration, and operator-tool connections use PgBouncer. PostgreSQL's own healthcheck and PgBouncer's upstream connection are the only direct connections. Transaction pooling is enabled on the shared endpoint. PgBouncer prepared-statement tracking is enabled with `max_prepared_statements=250`; application code must not rely on session state outside a transaction.

No tenant database is created, selected, cached, evicted, or provisioned. Runtime pool management uses one database and no repository resolves a client-supplied organization ID.

## Migrations

```text
foundation
  -> hr_core
  -> attendance_schedule
  -> payroll
  -> runtime_seeds
```

Fresh install target is described by `docs/plans/migration-phase-manifest.yaml`. Active fresh phases live under `apiserver/migrations/phases/`. Current numeric migrations remain schema oracle and historical upgrade context; do not apply them through active runner. Apply historical chain only on disposable verification databases. `down.sql` files are rollback/recovery artifacts and require verified backup before use.

Migration phase versions MUST remain unique. New post-production changes MUST append to the active phase runner; do not rewrite an installed phase.

`config_entries` uses typed values, explicit business-key registry in Go, effective dates, and NULL-safe uniqueness for organization scope. `system_settings` remains JSON key/value storage for system/integration documents such as `smtp`; it is not business configuration.

## Deployment and rollback

1. Take an encrypted PostgreSQL backup and record the application/image version before any non-disposable operation.
2. Apply the five fresh-install phases only to a fresh target database.
3. Verify phase markers, expected table/partition counts, constraints, triggers, views, seeds, and bootstrap compatibility.
4. For disposable recovery testing only, run phase `down.sql` artifacts or recreate the database, then re-apply phases.
5. For an existing database, restore a compatible backup or recreate the disposable database; do not attempt an old-chain upgrade.

## Target architecture after all phases

- One database and one schema.
- No `workspaces` table, `workspace_id` columns, or workspace-prefixed business tables.
- No `/workspaces/{id}` API path or browser-held organization state.
- Direct organization-level roles and permissions.
- Repository queries use one DB handle without tenant routing predicates.

These properties are implemented in source; verification runs on disposable infrastructure during integration.
