# LeeAttend — Coding Standards

## Rust (Backend)

### Language Version & Lints
- **Edition**: Rust 2021
- **Warnings**: Denied (`warnings = "deny"`)
- **Clippy**: `unwrap_used = "deny"` — all `unwrap()` calls forbidden in production code (test code exempted via `#![cfg_attr(test, allow(clippy::unwrap_used))]`)
- Use `.map_err()`, `?` operator, or `.expect()` with descriptive messages instead

### Naming Conventions
| Item | Convention | Example |
|---|---|---|
| Files | `snake_case.rs` | `attendance_summary_service.rs` |
| Structs/Enums | `PascalCase` | `AttendanceDailySummary` |
| Functions | `snake_case` | `recalculate_daily_summary()` |
| Constants | `SCREAMING_SNAKE_CASE` | `EXCEL_CONTENT_TYPE` |
| Module names | `snake_case` | `pub mod holiday_repository;` |

### Architecture Patterns
- **Handler → Service → Repository** layering
- Handlers handle HTTP concerns only (parsing, auth, response formatting)
- Services contain business logic, orchestrate repos and domain policies
- Repositories contain raw SQL via `sqlx::query!` / `sqlx::query_as!`
- Domain policies are pure functions without I/O

### SQL & Database
- All queries use `sqlx::query!` or `sqlx::query_as!` macros for compile-time verification
- Queries require a live database connection during compilation (`DATABASE_URL`)
- Migrations use timestamp-based naming: `YYYYMMDDHHMMSS_description.sql`
- All tables include `workspace_id` for multi-tenant isolation
- Use `gen_random_uuid()` for primary keys
- Include `created_at` / `updated_at` timestamps

### Error Handling
- Return `Result<T, (StatusCode, String)>` from handlers
- Use `?` operator with `.map_err()` for error propagation
- Never use `unwrap()` in production code

### Testing
- Unit tests in `#[cfg(test)] mod tests` blocks within source files
- Integration tests in `tests/integration/` directory
- Test execution via `./run-tests.sh` (Docker-based, never `cargo test` directly)
- Use `serial_test` for DB-dependent integration tests
- Test app helper: `common::app::TestApp`

## TypeScript/React (Frontend)

### File Structure
| Type | Convention | Example |
|---|---|---|
| Pages | `PascalCase.tsx` | `ReportsPage.tsx` |
| Components | `PascalCase.tsx` | `DailyReportTable.tsx` |
| API modules | `kebab-case.ts` or `camelCase.ts` | `holidays.ts` |
| Types | `types.ts` (central) | `lib/api/types.ts` |
| Hooks | `use*.ts` | `useWorkspace.ts` |
| Tests | `__tests__/Component.test.tsx` | Co-located |

### Component Patterns
- Functional components with hooks (no class components)
- `useTranslation()` for all user-facing text (i18next)
- `@tanstack/react-query` for server state (queries, mutations, invalidation)
- `getCurrentWorkspaceId()` for workspace context (not a hook)
- Tailwind CSS for styling (no CSS modules)
- Lucide React for icons

### API Client Pattern
```typescript
// lib/api/holidays.ts
import { API_BASE_URL } from './config';
import { apiFetch } from './helpers';
import { Holiday } from './types';

export async function fetchHolidays(wsId: string): Promise<Holiday[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/holidays`);
  if (!response.ok) throw new Error('Failed to fetch holidays');
  return response.json();
}
```

### Styling
- TailwindCSS 4 utility classes
- Dark mode support via `dark:` prefix
- Design tokens: consistent color palette, typography (Inter font)
- Component patterns: rounded corners (`rounded-xl`/`rounded-2xl`), shadow effects
- Status badges: color-coded with `bg-{color}-50 text-{color}-600 border-{color}-100`

## Git & Version Control

### Commit Format
Conventional Commits:
```
feat(holidays): add holiday column to excel writer
fix(payroll): exclude holidays from late count
test(holiday): add summary recalculation test
docs: update architecture documentation
chore(deps): bump sqlx to 0.8.6
```

### Branch Strategy
- Feature branches from `main`
- PR-based merge workflow

## Docker & Testing

### Docker Compose Profiles
| Profile | File | Purpose |
|---|---|---|
| Base | `docker-compose.yml` | Shared service definitions |
| Dev | `docker-compose.dev.yml` | Hot-reload, host volume mounts |
| Test | `docker-compose.test.yml` | Isolated test environment |
| CI | `docker-compose.ci.yml` | GitHub Actions pipeline |

### Test Commands
```bash
./run-tests.sh --all              # Full suite
./run-tests.sh --api              # Backend (clippy + unit + integration)
./run-tests.sh --api --smoke      # Quick compilation check
./run-tests.sh --web              # Frontend (vitest + playwright)
./run-tests.sh --docker=dev up -d # Start dev environment
```

### WSL2 Note
Direct `cargo test` on WSL2 can freeze the system. Always use `./run-tests.sh` which runs inside Docker containers.

## Internationalization (i18n)

- Framework: `i18next` + `react-i18next`
- Languages: English (`en`), Vietnamese (`vi`)
- Translation files in `webdashboard/src/i18n/`
- Use `t("key")` for all user-visible strings
- Document files always in English (per CORE.md Law 6)
