# 🏗️ Project Structure & Commits

## 1. Component Map
| Component | Language | Path |
|-----------|----------|------|
| API Server | Rust (Axum) | `apiserver/` |
| Agent (ZK Worker) | Rust | `agent-main/` |
| Web Dashboard | React + TypeScript | `webdashboard/` |
| Containers | Compose | `docker-compose*.yml` |
| Tests | Rust + Playwright | `./leedevkit test` |

## 2. Commit Convention
Use conventional commits for all changes: 
`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`

## 3. AI Logging & Workspace Memory
- **Memory Retention:** Document major architectural shifts in `.ai/ADRs/` to retain long-term AI context.
- **Clean Workspace:** Do NOT save `.txt` or temporary planning files in project root. Use `/tmp/` or prefix `ai_` (which is added to `.gitignore`).

## 4. Directory Map
| Directory | Purpose |
|-----------|---------|
| `apiserver/src/main.rs` | Entry point, Tokio runtime bootstrap |
| `apiserver/src/lib.rs` | Crate root, module declarations, crate-level lints |
| `apiserver/src/routes/` | 18 domain route modules, each exports `pub fn router()` |
| `apiserver/src/handlers/` | Request handlers with `#[utoipa::path]` + `#[tracing::instrument]` |
| `apiserver/src/services/` | Business logic layer between handlers and repositories |
| `apiserver/src/repositories/` | `#[async_trait]` traits + `diesel_*` implementations |
| `apiserver/src/diesel_pool/` | `DieselPools`, `TenantPoolManager`, connection wrappers |
| `apiserver/src/models/` | Domain models (private fields), DTOs, `PaginatedResponse<T>` |
| `apiserver/src/models/diesel/` | Diesel entities: `XxxEntity` with `Queryable/Insertable` |
| `apiserver/src/errors.rs` | `AppError` enum, `AppResult<T>`, `IntoResponse` impl |
| `apiserver/src/mw/` | Middleware: `RateLimitLayer`, auth extractors, i18n |
| `apiserver/src/utils/` | Helpers: `I18nKey`, i18n utilities |
| `apiserver/migrations/tenant/` | Tenant migrations (`YYYYMMDDHHMMSS_description/up.sql`+`down.sql`) |
| `apiserver/migrations/system/` | System-wide migrations |
| `agent-main/` | Agent binary (ZK worker) |
| `webdashboard/` | React + TypeScript SPA frontend |
| `scripts/` | Hermetic wrappers: `_cargo.sh`, `_npm.sh`, `_diesel.sh` |

## 5. Dependency Flow
```
Request → Middleware (RateLimitLayer → UserId → WorkspaceId → MemberContext)
        → Router (18 domain modules)
        → Handler (src/handlers/)
        → Service (src/services/ — business logic)
        → Repository trait (src/repositories/)
        → Diesel implementation (src/repositories/diesel_*)
        → DieselPools (api/worker/tenants_tx/tenants_sess)
        → PostgreSQL
```

**Rules:**
- Handler → Service ONLY (never call Repository directly from Handler)
- Service → Repository trait ONLY (never depend on `diesel_*` impl directly)
- Repository → DieselPools ONLY (never access pool from Service or Handler)
- NEVER skip a layer

## 6. Frontend Type Architecture (Web Dashboard)

### 6.1 Type Ownership — Single Source of Truth
| Source | Path | Hand-edit? | Role |
|---|---|---|---|
| Generated OpenAPI | `src/types/api-generated.ts` | ❌ NEVER | Canonical API types |
| Generated barrel | `src/lib/api/api-generated-reexports.ts` | ❌ NEVER | 1:1 aliases of generated schemas |
| Hand-written | `src/lib/api/types.ts` | ✅ ONLY for exceptions | Generic helpers, UI-only composites, temporary schema-gap overrides |

- **Rule:** If a type exists in `api-generated.ts`, it MUST NOT be re-declared in `types.ts`. Duplication = drift.
- **Rule:** `types.ts` MUST carry `export * from "./api-generated-reexports"` so consumer import paths stay stable.
- **Rule:** Temporary overrides in `types.ts` MUST carry a TODO comment pointing to the schema gap to close.

### 6.2 Barrel-File Convention
- Heavy pure-reexport sets (>20 aliases) go in their own barrel file, not inside the hand-written file.
- The hand-written file re-exports the barrel via `export * from "<barrel>"`.
- Anchor aliases (Group-A names referenced by Group-B overrides) remain in the hand-written file to keep local scoping.
- One barrel per concern — do not mix unrelated re-exports.

### 6.3 API Client Escape-Hatch Discipline
- **FORBIDDEN:** `client as any`, `untyped.ts`, module-level escape hatches.
- **PREFERRED:** `fetch<T>()` / `fire()` helpers for paths not yet in the typed schema.
- **TODO obligation:** Every `fetch<T>()` / `fire()` call on an untyped path MUST carry a TODO to add the path to `openapi.raw.json` and migrate to typed `client.METHOD(...)` after regen.
