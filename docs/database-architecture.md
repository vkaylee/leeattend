# Database Architecture

LeeAttend supports two database topologies:

| Mode | System tables | Tenant tables | Runtime routing |
|---|---|---|---|
| **SaaS** | Shared system database | One database per workspace, named `tenant_{workspace_id}` | System API/worker pools plus dynamic tenant transaction/session pools through pooler |
| **Standalone** | Same PostgreSQL database | Same PostgreSQL database | Tenant transaction requests reuse API pool; tenant session requests reuse worker/session pool; no dynamic tenant pool |

The logical split remains: system migrations define platform/workspace/auth tables, tenant migrations define business tables. Standalone applies both migration directories to one database; SaaS applies tenant migrations per workspace database.

SaaS runtime queries use Supavisor/connection poolers where configured. Standalone Compose uses one PostgreSQL service and one database; pool URLs retain transaction/session semantics but resolve to the same database.

---

## Database Split

### System Tables

Created by system migrations (`migrations/system/`). Stored in a single shared database.

- `users` — User accounts and authentication
- `workspaces` — Workspace/organization entities
- `sessions` — User sessions
- `system_roles` — Platform-level roles (platform_admin, support, billing_admin)
- `user_system_roles` — Cross-workspace role assignments
- `authorized_agents` — Device agent credentials
- `audit_logs` — System-level audit trail (partitioned by month)
- `system_audit_logs` — System operation logs
- `notifications` — User notifications (user-level, not tenant-scoped)
- `workspace_members` — User-to-workspace membership **moved to system DB**
- `workspace_roles` — Role definitions per workspace **moved to system DB**
- `workspace_role_permissions` — Permission-to-role mapping **moved to system DB**
- `member_extra_roles` — Per-user extra role assignments **moved to system DB**
- `workspace_member_extra_permissions` — Per-user extra permission overrides **moved to system DB**

### Tenant Tables

Created by tenant migrations (`migrations/tenant/`). In SaaS, each workspace gets an independent copy. In standalone, the same tables exist in the shared appliance database and remain scoped by `workspace_id`.

- `employees`, `departments`, `shifts`
- `time_entries`, `raw_attendance_logs`, `attendance_daily_summaries`
- `leave_requests`, `leave_balances`, `leave_policies`
- `payroll_*`, `payslips`, `salary_*`
- `attendance_sources`, `feature_flags`, `dashboard_preferences`
- And 40+ other business tables

### Why the Split?

- **SaaS data isolation**: A workspace's business data is physically separate from other workspaces.
- **Standalone compatibility**: One workspace keeps the same logical tenant tables and authorization predicates without a second database.
- **Scaling**: SaaS databases can be placed on different Postgres servers.
- **Backup granularity**: SaaS supports per-tenant backup/restore; standalone backs up one appliance database.
- **BYOD**: SaaS workspaces can use their own external database (`custom_db_url_encrypted`).

---

## Connection Architecture

### Standalone

```
System queries  ──→ system/API pool ──→ shared PostgreSQL database
Tenant TX       ──→ same system/API pool ──→ shared PostgreSQL database
Tenant session  ──→ worker/session pool ──→ shared PostgreSQL database
```

`GetTenantTxPool(standaloneWorkspaceID)` and `GetTenantSessPool(standaloneWorkspaceID)` validate the configured workspace, then return the owned shared pools. Other workspace IDs fail closed. Shared pools never enter tenant cache, eviction, or tenant close paths.

### SaaS

```
System queries  ──→ API/worker pool ──→ system database
Tenant TX       ──→ dynamic tenant TX pool ──→ tenant_<workspace_id>
Tenant session  ──→ dynamic tenant session pool ──→ tenant_<workspace_id>
```

SaaS dynamic pools remain cached per workspace and are independently evictable.

### Pool ownership

- `systemPool`: system and standalone tenant transaction queries.
- `workerPool`: worker/system session queries and standalone tenant session queries.
- `tenantTx`/`tenantSess`: SaaS-only dynamic pool caches.

### URL Resolution

```
DATABASE_URL        = postgres://admin:pass@db:5432/leeattend     # Direct
DATABASE_POOLER_URL = postgres://admin.leeattend:pass@supavisor:5452/leeattend  # Via Supavisor
```

The tenant URL constructor (`get_tenant_url`) adapts based on the connection type:

| Base URL user format | Type | Resulting tenant URL |
|---------------------|------|---------------------|
| `admin` (no dot) | Direct | `postgres://admin:pass@db:5432/tenant_{id}` |
| `admin.leeattend` (has dot) | Supavisor | `postgres://admin.tenant_{id}:pass@supavisor:5452/tenant_{id}` |

The dot `.` in the username signals Supavisor routing. Direct Postgres ignores the `.tenant_{id}` suffix.

When `DATABASE_POOLER_URL` is set, the pooler URL is used for:
- The `api` and `worker` pools (system DB queries)
- The tenant pool base URL pattern (per-tenant connections)

When `DATABASE_POOLER_URL` is not set (dev/test), `DATABASE_URL` is used as fallback.

---

## Migrations

### Directory Structure

```
apiserver/migrations/
├── system/          # System tables; shared DB in both modes
│   └── ...
└── tenant/          # Tenant tables; shared DB in standalone, per-workspace DB in SaaS
    └── ...
```

### Application order

- **Standalone release**: apply `migrations/system/`, then `migrations/tenant/`, to the same PostgreSQL database using shared migration history before API bootstrap.
- **SaaS release**: apply system migrations to the system database; apply tenant migrations independently to each workspace database during provisioning/reconciliation.

Migration directory versions must remain globally unique when both directories share one standalone migration history.

### Tenant Provision Flow (BYOD)

Tenant databases are **BYOD (Bring Your Own Database)**. The system does **not** create databases.
The user (or an external provider like Railway/Neon) is responsible for creating the database
and providing a valid connection string.

When a new workspace is created:

1. **User provides `database_url`** — Connection string to an existing PostgreSQL database.
2. **`TenantMigrator::provision_tenant_db()`** — Connects to the provided URL, runs all pending tenant migrations.
3. **`build_tenant_payload()`** — Extracts host/port/database/user/password from the URL.
4. **`register_tenant_dynamic()`** — HTTP PUT to Supavisor API to register the new tenant.
5. **Encrypt & save** — The `database_url` is encrypted and stored in `workspaces.custom_db_url_encrypted`.
6. After registration, all subsequent tenant queries go through Supavisor.

> **Future:** External providers (Railway API, Neon API) will create databases
> programmatically and return their connection strings to feed into step 1.

---

## Workspace Table Migration (2026-05-19)

The following tables were moved from Tenant DB to System DB because they represent **cross-cutting relationships** (user-to-workspace mappings):

- `workspace_members`
- `workspace_roles`
- `workspace_role_permissions`
- `member_extra_roles`
- `workspace_member_extra_permissions`

`notifications` was also moved because it is user-level reminder data, not tenant business data.

### Motivation

- These tables were previously in Tenant DB but referenced by queries from System DB.
- Cross-database foreign keys were impossible (Postgres limitation).
- Moving them to System DB enables proper FK constraints (`workspace_members → users`, `workspace_members → workspaces`, etc.).

### FK Constraints Now in System DB

```sql
ALTER TABLE workspace_members
    ADD CONSTRAINT workspace_members_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE workspace_members
    ADD CONSTRAINT workspace_members_workspace_id_fkey
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE;
```

---

## Supavisor Orchestration

Supavisor acts as the **central database orchestrator**. All SQL queries at runtime flow through it.

### Registration

- At **workspace creation time**: The provisioner registers the new tenant DB with Supavisor.
- At **application startup**: Existing workspace tenants are re-registered (idempotent, via HTTP PUT).

### Tenant Payload

```json
{
  "tenant": {
    "db_host": "db",
    "db_port": 5432,
    "db_database": "tenant_{workspace_id}",
    "enforce_ssl": false,
    "users": [{
      "db_user": "admin",
      "db_password": "***",
      "pool_size": 5
    }]
  }
}
```

### Connection Lifecycle without Supavisor

In development or test environments where Supavisor is not available:
- `pooler_url` is `None`.
- All pools fall back to `primary_url` (`DATABASE_URL`).
- Tenant URL constructor skips the Supavisor user-format overlay.

---

## Configuration

### Environment Variables

| Variable | Used For | Required |
|----------|----------|----------|
| `DATABASE_URL` | Migrations, tenant provisioning, startup workspace queries | Yes |
| `DATABASE_POOLER_URL` | Runtime API and tenant queries through Supavisor | In production |
| `DB_API_MAX_CONN` | API pool max connections (default: 20) | No |
| `DB_TENANT_MAX_CONN` | Per-tenant pool max connections (default: 5) | No |

### `DATABASE_URL` Still Required

Even with Supavisor, `DATABASE_URL` is necessary for three operations that cannot go through a pooler:

1. **`run_system_migrations()`** — Runs before Supavisor is available.
2. **`provision_tenant_db()`** — `CREATE DATABASE` is a DDL statement, unsupported by poolers.
3. **Register workspaces at startup** — Querying `workspaces` table uses direct connection.

Runtime API queries never use `DATABASE_URL`.

---

## Cross-Database Foreign Keys

**Postgres does not support cross-database foreign keys.** The system/tenant split means:

- FK constraints between system tables are valid (same database).
- FK constraints between tenant tables are valid (same database).
- FK constraints from tenant to system tables are **removed** — referential integrity is enforced at the application level.
- Five workspace-mapping tables were moved to the system DB to keep their FK constraints intact.

---

## References

- `apiserver/src/diesel_pool.rs` — Pool struct, TenantPoolProvider, URL construction
- `apiserver/src/main.rs` — Startup orchestration (migrations → supavisor → pools → tenant registration)
- `apiserver/src/supavisor.rs` — Supavisor tenant management API client
- `apiserver/src/services/workspace_service.rs` — Workspace creation flow with Supavisor registration
- `apiserver/src/services/tenant_provisioner.rs` — Database provisioning for new workspaces
