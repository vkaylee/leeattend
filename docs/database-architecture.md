# Database Architecture

LeeAttend is a single-organization appliance with one PostgreSQL database and one `public` schema. The ordered baseline migration installs the schema, and `apiserver/migrations/20261108000000_single_organization_cutover/` converts the compatibility tables/columns to the final organization-scoped model.

## Topology

| Component | Contract |
|---|---|
| PostgreSQL | One database, one `public` schema |
| Baseline migration | `apiserver/migrations/20261103000000_init/` |
| Runtime pools | API and worker pools connect to the same database; no tenant routing |
| Final schema migration | `apiserver/migrations/20261108000000_single_organization_cutover/` |

Physical database split is gone. Legacy platform tables (`system_roles`, `user_system_roles`, and `system_db_clusters`) are removed by `apiserver/migrations/20261106000000_remove_system_roles/`. The cutover migration then renames `workspaces` to `organization_settings`, `workspace_roles` to `roles`, `workspace_role_permissions` to `role_permissions`, and `workspace_members` to `user_roles`, and drops every `workspace_id` column.

## Schema groups

The unified baseline contains all existing system and business objects:

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
apiserver/migrations/
├── 20261103000000_init/
├── 20261106000000_remove_system_roles/
├── 20261107000000_add_window_layout_to_dashboard_preferences/
└── 20261108000000_single_organization_cutover/

Apply the baseline once against the target database. `up.sql` is ordered system-first then business-schema. `down.sql` drops and recreates `public`; it is a destructive recovery operation and requires a verified backup before use.

Migration directory versions MUST remain unique. New post-baseline changes MUST use a new migration directory under `apiserver/migrations/`; do not recreate `system/` or `tenant/` directories.

## Deployment and rollback

1. Take an encrypted PostgreSQL backup and record the application/image version.
2. Apply `20261103000000_init` to a clean or approved target database.
3. Verify migration history, expected table/partition counts, constraints, triggers, and bootstrap compatibility.
4. For rollback testing only, run `down.sql` against a disposable database, then re-apply `up.sql`.
5. In a failed production migration, restore the backup or roll forward with a corrective migration; do not run `down.sql` against the live database without explicit approval.

## Target architecture after all phases

- One database and one schema.
- No `workspaces` table, `workspace_id` columns, or workspace-prefixed business tables.
- No `/workspaces/{id}` API path or browser-held organization state.
- Direct organization-level roles and permissions.
- Repository queries use one DB handle without tenant routing predicates.

These properties are implemented in source; verification runs on disposable infrastructure during integration.
