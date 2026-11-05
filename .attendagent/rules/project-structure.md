# 🏗️ Project Structure & Commits

## 1. Component Map
| Component | Language | Path |
|-----------|----------|------|
| API Server | Go (`net/http`) | `apiserver/` |
| Web Dashboard | React + TypeScript | `webdashboard/` |
| Containers | Compose | `docker-compose*.yml` |
| Tests | Go + Playwright/Bun | `./leedevkit test` |

## 2. Commit Convention
Use conventional commits for all changes: 
`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`

## 3. AI Logging & Workspace Memory
- **Memory Retention:** Document major architectural shifts in `.ai/ADRs/` to retain long-term AI context.
- **Clean Workspace:** Do NOT save `.txt` or temporary planning files in project root. Use `/tmp/` or prefix `ai_` (which is added to `.gitignore`).

## 4. Directory Map
| Directory | Purpose |
| `apiserver/main.go` | API entry point and startup |
| `apiserver/internal/api/` | HTTP handlers and API contracts |
| `apiserver/internal/service/` | Business logic and orchestration |
| `apiserver/internal/repository/` | PostgreSQL repositories and SQL access |
| `apiserver/internal/dbpool/` | `pgx/v5` pool management |
| `apiserver/internal/config/` | Environment and runtime configuration |
| `apiserver/migrations/` | PostgreSQL migrations |
| `webdashboard/` | React + TypeScript SPA frontend |
| `scripts/` | Hermetic build, test, migration, and validation wrappers |

## 5. Dependency Flow
```
HTTP request → API middleware → Go handler → service → repository → pgx/PostgreSQL
                                  └──────→ Redis / MinIO as required
Web dashboard → REST/WebSocket API
```

**Rules:**
- Handler → Service only; handlers must not access repositories directly.
- Service → Repository only; keep PostgreSQL access behind repository boundaries.
- Infrastructure clients (Redis, MinIO) stay behind the service/repository boundary that owns their operation.
- NEVER skip a layer without documenting the exception.

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
