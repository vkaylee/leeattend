# Database Architecture

LeeAttend is being converted to a single-tenant appliance. Phase 1 establishes one PostgreSQL database and one ordered baseline migration. Later phases remove the remaining workspace-compatible API and data-access seams.

## Phase 1 topology

| Component | Phase 1 contract |
|---|---|
| PostgreSQL | One database, one `public` schema |
| Migration directory | `apiserver/migrations/20261103000000_init/` |
| Runtime pools | API and worker pools connect to the same database; no tenant routing |
| Workspace compatibility | `workspace_id` columns and workspace tables remain temporarily in schema only |

Physical database split is gone. Logical system/tenant ordering remains inside baseline only so existing repository/API data contracts can migrate incrementally. Legacy platform tables (`system_roles`, `user_system_roles`, and `system_db_clusters`) are removed by `apiserver/migrations/20261106000000_remove_system_roles/`; standalone authorization uses the singleton organization's global `workspace_roles` and `workspace_members` records.

## Schema groups

The unified baseline contains all existing system and business objects:

- Authentication and platform objects: `users`, `sessions`, workspace-compatible role/member tables, notifications, storage, and infrastructure ledger tables. Legacy platform tables exist only during pre-migration compatibility and are removed by the post-baseline migration.
- Business objects: employees, departments, shifts, attendance, leave, payroll, contracts, workflows, files, reports, and outbox tables.
- Partitioned objects: audit and staging-log parents plus their monthly/default partitions.

`workspace_id` remains present in this phase because repository SQL, response contracts, and tests still consume it. Phase 2 removed runtime pool/configuration dependence; Phase 3 removes repository predicates and API data dependencies; the final schema contract removes the column and obsolete workspace tables only after those consumers are migrated.

## Connection architecture

```
System/API      ──→ API pool     ──→ PgBouncer (transaction mode) ──→ one PostgreSQL database
Business TX     ──→ same API pool ──→ PgBouncer                   ──→ one PostgreSQL database
Worker/session  ──→ worker pool  ──→ same PgBouncer endpoint      ──→ one PostgreSQL database
```

All application, migration, and operator-tool connections use PgBouncer. PostgreSQL's own healthcheck and PgBouncer's upstream connection are the only direct connections. Transaction pooling is enabled on the shared endpoint. PgBouncer prepared-statement tracking is enabled with `max_prepared_statements=250`; application code must not rely on session state outside a transaction.

No tenant database is created, selected, cached, evicted, or provisioned. Runtime pool management uses one database without workspace-ID validation; repository/API compatibility fields remain until later phases.

## Migrations

```text
apiserver/migrations/
├── 20261103000000_init/
│   ├── up.sql
│   └── down.sql
└── 20261106000000_remove_system_roles/
    ├── up.sql
    └── down.sql

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
- No `workspaces` table or `workspace_id` columns.
- No `/workspaces/{ws_id}` API path or `active_workspace_id` browser state.
- Direct organization-level roles and permissions.
- Repository queries use one DB handle without tenant routing predicates.

These target properties are not claimed as complete until the later backend, repository, API, frontend, and script phases pass their verification gates.
