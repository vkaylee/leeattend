# Database Architecture

## Overview

LeeAttend uses a **dual-database model** separated by responsibility:

| Database | Purpose | Location | Management |
|----------|---------|----------|------------|
| **System DB** | Global platform tables | Same Postgres cluster (usually) | Migrations in `migrations/system/` |
| **Tenant DBs** | Per-workspace business data | One database per workspace, named `tenant_{workspace_id}` | Migrations in `migrations/tenant/` |

All runtime queries are routed through **Supavisor** (connection pooler), which acts as the database orchestrator and provides connection pooling, tenant-aware routing, and SSL enforcement.

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

Created by tenant migrations (`migrations/tenant/`). Each workspace gets an independent copy.

- `employees`, `departments`, `shifts`
- `time_entries`, `raw_attendance_logs`, `attendance_daily_summaries`
- `leave_requests`, `leave_balances`, `leave_policies`
- `payroll_*`, `payslips`, `salary_*`
- `devices`, `feature_flags`, `dashboard_preferences`
- And 40+ other business tables

### Why the Split?

- **Data isolation**: A workspace's business data is physically separate from other workspaces.
- **Scaling**: Different databases can be on different Postgres servers.
- **Backup granularity**: Per-tenant backup/restore without affecting others.
- **BYOD**: Workspaces can use their own external database (`custom_db_url_encrypted`).

---

## Connection Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    APPLICATION STARTUP                   │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  1. run_system_migrations()  ───→ Direct (DATABASE_URL) │
│  2. Supavisor init_tenant()   ───→ HTTP to Supavisor    │
│  3. DieselPools::connect()    ───→ DATABASE_POOLER_URL  │
│  4. register_workspace_tenants_and_migrate()              │
│     ├── Query workspaces from system DB                  │
│     ├── Register each with Supavisor (HTTP PUT)          │
│     └── run_tenant_migrations()  ───→ DATABASE_POOLER_URL│
│                                                          │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│                    RUNTIME QUERIES                       │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  System queries  → api pool  → Supavisor → System DB    │
│  Tenant queries  → ten pool  → Supavisor → Tenant DB    │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Pool Architecture

```rust
pub struct DieselPools {
    pub api: DieselPool,        // System DB — through Supavisor
    pub worker: DieselPool,     // System DB — through Supavisor (unused)
    pub tenants: Arc<dyn TenantPoolProvider>,  // Tenant DBs — through Supavisor
}
```

**TenantPoolProvider** trait:

```rust
pub trait TenantPoolProvider: Send + Sync {
    async fn get_pool(&self, workspace_id: &Uuid) -> Result<DieselPool, AppError>;
}
```

- **ProductionTenantPoolManager**: Caches pools per workspace. Constructs pooler-aware URLs.
- **TestTenantPoolManager**: Returns a single shared pool for all workspaces (used in tests).

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
├── system/          # Applied to the system DB
│   ├── 20260101000000_initial_schema/
│   ├── 20260505010000_phase1_triggers_and_gin_indexes/
│   └── ...
└── tenant/          # Applied to each tenant DB
    ├── 20260101000001_initial_schema/
    ├── 20260505010001_phase1_triggers_and_gin_indexes/
    └── ...
```

### Startup Migration Order

1. **`run_system_migrations()`** — Direct connection via `DATABASE_URL`. Runs before Supavisor is available.
2. **Supavisor initialization** — Registers the system tenant with Supavisor.
3. **Pool connection** — API and tenant pools connect through Supavisor.
4. **`register_workspace_tenants_and_migrate()`**:
   a. Query all workspaces from system DB (via direct connection, before pools are needed).
   b. For each workspace, register an idempotent tenant config with Supavisor (HTTP PUT).
   c. `run_tenant_migrations()` — Through Supavisor (fallback to direct if no pooler configured).

### Tenant Provision Flow

When a new workspace is created:

1. **CREATE DATABASE** — Direct connection (DDL cannot go through a pooler).
2. **Run tenant migrations** — Direct connection (new DB is not yet registered with Supavisor).
3. **Build Supavisor payload** — Extract host/port/database/user/password from the direct URL.
4. **`register_tenant_dynamic()`** — HTTP PUT to Supavisor API to register the new tenant.
5. After registration, all subsequent tenant queries go through Supavisor.

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
