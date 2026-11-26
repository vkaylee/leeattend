# Database Architecture

LeeAttend is a single-organization appliance with one PostgreSQL database and one `public` schema. Fresh installation uses five readable migration phases that create the final organization-scoped model directly: `foundation`, `hr_core`, `attendance_schedule`, `payroll`, and `runtime_seeds`.

## Topology

| Component | Contract |
|---|---|
| PostgreSQL | One database, one `public` schema |
| Active fresh-install phases | `apiserver/migrations/phases/` (`foundation`, `hr_core`, `attendance_schedule`, `payroll`, `runtime_seeds`) |
| Schema contract | `apiserver/migrations/testdata/phased_schema_contract.sql` + `employment_relationship_integrity.sql` |
| Runtime pools | API and worker pools connect to the same database; no tenant routing |
| Migration runner | `apiserver/migrations/run-dev.sh` |

Physical database split is gone. The phased schema creates final organization tables directly, including `organization_settings`, `roles`, `role_permissions`, and `user_roles`; it does not create legacy platform or workspace objects.

## Schema groups

The phased fresh-install schema in `apiserver/migrations/phases/` is the sole active schema source:

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

Active fresh-install phases live under `apiserver/migrations/phases/` and are the schema source before production go-live. The historical numeric chain is available in git history. A PostgreSQL 18 comparison found remaining catalog differences; it did not establish complete parity (see `ADR-013`). `down.sql` files are disposable rollback artifacts and require verified backup before any non-disposable use.

Before production, schema changes edit the owning phase directly and require recreating a confirmed disposable database; existing phase markers do not cause the runner to reapply modified phase SQL. Never reset a database or volume without confirming its target and data recovery requirements.

Once in production, freeze the fresh-install phases and append new incremental migrations to the active runner rather than rewriting installed phases.

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
