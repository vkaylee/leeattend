# LeeAttend Project Context

> **Purpose:** Bridge generic Antigravity skills to LeeAttend's actual codebase. Read this AFTER loading any skill to ground it in project reality.
> **Rule:** This file overrides any conflicting advice from external skills. Project context is king.

---

## 🏗️ Architecture Overview

**LeeAttend** is a multi-tenant Enterprise HRM/Attendance system:

| Layer | Technology | Location |
|-------|-----------|----------|
| API Server | Rust + Axum 0.7 + Tokio 1.36 | `apiserver/` |
| Background Workers | Rust (same binary, worker pool) | `apiserver/src/worker/` |
| Agent | Rust (ZK worker) | `agent-main/` |
| Dashboard | React + TypeScript SPA | `webdashboard/` |
| Database | PostgreSQL (multi-tenant) | via Diesel 2.2 async |
| Cache/Queue | Redis | via `redis` crate |
| Object Storage | AWS S3 | via `aws-sdk-s3` |

### Request Flow
```
Request → Middleware (RateLimitLayer → UserId → WorkspaceId → MemberContext)
        → Router (18 domain modules) → Handler → Service → Repository → DieselPools → PostgreSQL
```

### State Tree
`Arc<AppState>` → sub-states (`employees`, `attendance`, `shifts`, `leaves`, `payroll`, `reports`, `core`, `agents`, `infra`, `storage`, `file_service`, `scanner`) — each exposes `.service()` and `.repo()` accessors.

---

## 🦀 Rust & Backend Patterns

### Error Handling
- **Type:** `AppResult<T>` = `Result<T, AppError>` (defined in `src/errors.rs`)
- **Constructors (ALWAYS use I18nKey):**
  - `AppError::not_found(I18nKey::NotFound)`
  - `AppError::auth(I18nKey::InvalidToken)`
  - `AppError::validation(I18nKey::Validation)`
  - `AppError::forbidden(I18nKey::Forbidden)`
  - `AppError::conflict(I18nKey::Conflict)`
  - `AppError::internal("db: {details}")`
- **NEVER:** `unwrap()`, `expect()`, raw error strings — denied at crate level via `#[deny(clippy::unwrap_used, clippy::expect_used)]`

### Handler Signature Standard
```rust
#[utoipa::path(...)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn handler_name(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Query(params): Query<ParamsType>,
) -> AppResult<Json<ResponseType>>
```

### Service & Repository Layering
- **Handlers** (`src/handlers/`) → call services, never repositories directly
- **Services** (`src/services/`) → business logic, depend on repository TRAITS
- **Repositories** (`src/repositories/`) → `#[async_trait]` trait definitions + `diesel_*` implementations
- **DieselPools** (`src/diesel_pool/`) → PostgreSQL connection pools

### Model Architecture (Two-Layer)
- **Domain Model** (`src/models/`): private fields, public getters, `Serialize/Deserialize/ToSchema`
- **Diesel Entity** (`src/models/diesel/`): `XxxEntity` with `Queryable/Insertable/AsChangeset`, all fields public

### Async Safety Rules
- NEVER hold `MutexGuard` or `RwLockReadGuard` across `.await`
- NEVER use blocking I/O (`std::fs`, `std::thread::sleep`) in async code — use `tokio` equivalents
- Use `tokio::task::spawn_blocking` for CPU-intensive work

---

## 🌐 API Conventions

### Routing
- **Base:** `/workspaces/:ws_id/...` (tenant), `/system/...` (system)
- **18 route modules** in `src/routes/`, each exported as `pub fn router() -> Router<Arc<AppState>>`
- Routes map to handlers in `src/handlers/` — route modules only wire paths, never contain logic

### Response Format
- **List:** `PaginatedResponse<T>` = `{ "data": [...], "total": N, "page": N, "limit": N }`
- **Mutation:** `GenericResponse` = `{ "message": "..." }`
- **Create:** `(StatusCode::CREATED, Json<T>)`
- **Delete:** `StatusCode::NO_CONTENT`
- **Error:** `{ "error": { "code": "NOT_FOUND", "message": "..." } }` via `AppError::into_response()`

### Authentication & Authorization
- **Auth:** JWT Bearer token → `UserId` extractor → `MemberContext` (with roles, permissions, workspace_id)
- **Permission checks:** `ctx.require_permission(perm)`, `ctx.require_scoped_permission(perm, dept_id)`
- **Tenant isolation:** `workspace_id` extracted from `:ws_id` path param or `X-Workspace-Id` header

### Middleware Stack
1. CORS
2. `TraceLayer` (HTTP tracing)
3. `locale_middleware` (i18n: `?lang=` query → `Accept-Language` header → fallback `"vi"`)
4. `RateLimitLayer` (read: 120/60s, write: 30/60s — configurable via env)
5. `request_id_middleware` (UUID v7 correlation)

---

## 🗄️ Database Conventions

### Stack
PostgreSQL + Diesel 2.2 + `diesel-async` + `deadpool` connection pools

### Connection Pools (`DieselPools`)
| Pool | Purpose |
|------|---------|
| `api` | HTTP handlers, CRUD, auth |
| `worker` | Background jobs, import, accrual |
| `tenants_tx` | Per-tenant pgbouncer transaction mode (recommended) |
| `tenants_sess` | Per-tenant pgbouncer session mode |

### Multi-Tenancy
- `TenantPoolManager` maps `workspace_id` → per-tenant DB pool
- Pools cached via **moka** (50 max capacity, 30-min idle TTL)
- **HARD RULE:** Every tenant query MUST include `.filter(workspace_id.eq(...))` — no exceptions

### Migrations
- **Location:** `apiserver/migrations/`
- **Naming:** `YYYYMMDDHHMMSS_description/` with `up.sql` + `down.sql`
- **Categories:** `system/` (system-wide), `tenant/` (per-tenant schema)

### Schema Conventions
- **Primary keys:** `UUID PRIMARY KEY DEFAULT gen_random_uuid()`
- **Timestamps:** `TIMESTAMPTZ NOT NULL DEFAULT NOW()` for `created_at`, `updated_at`
- **Soft deletes:** `deleted_at TIMESTAMPTZ`, filter with `WHERE deleted_at IS NULL`
- **Column names:** `snake_case`

### Repository Access Pattern
```rust
let workspace_id = auth_token.workspace_id();
let mut conn = TransactionConn(
    diesel_pools.tenants_tx.get_connection(&workspace_id).await
        .map_err(|e| AppError::internal(format!("db: {e}")))?,
);
```

### Transaction Pattern
```rust
conn.transaction(|conn| {
    Box::pin(async move {
        // Multiple operations — all or nothing
        diesel::insert_into(table).values(...).execute(conn).await?;
        diesel::update(other_table).set(...).execute(conn).await?;
        Ok(())
    })
}).await
```

---

## 🧪 Testing Conventions

- **Unit tests:** `#[tokio::test]` with mock repositories (`MockXxxRepository` via `mockall`)
- **Integration tests:** `apiserver/tests/integration/` — uses `TestFactory` + `TestDatabase` + Tower `oneshot()`
- **Test runner:** `./test.sh <target>` — NEVER `cargo test` directly
- **Coverage threshold:** 80%
- **Verification pipeline:** Security → Lint → Schema → Tests → UX → SEO → Performance

---

## 🔗 Cross-References

When working on specific concerns, also load:

| Concern | Rulebook |
|---------|----------|
| Code quality, errors, security | `@[.agent/rules/coding-standards.md]` |
| Database, migrations, N+1, tenants | `@[.agent/rules/database-rules.md]` |
| Tests, coverage, test patterns | `@[.agent/rules/testing-standards.md]` |
| Project structure, commits | `@[.agent/rules/project-structure.md]` |
| UI/UX, a11y, design system | `@[.agent/rules/design-rules.md]` |
| RBAC, JWT, permissions | `@[.agent/rules/access-control.md]` |
| PII handling, data retention | `@[.agent/rules/data-governance.md]` |
| TLS, encryption, secrets | `@[.agent/rules/encryption-rules.md]` |
| Logging, tracing, metrics | `@[.agent/rules/observability-rules.md]` |
| Build scripts, hermetic wrappers | `@[.agent/rules/execution-wrappers.md]` |
| Config, feature flags, env vars | `@[.agent/rules/configuration-management.md]` |
