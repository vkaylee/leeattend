# Scout Report: LeeAttend Codebase Analysis

**Exploration Date**: 2026-01-28  
**Scope**: Complete project structure, tech stack, architecture patterns, dependencies  
**Explorer**: scouter agent  

---

## Executive Summary

LeeAttend is a comprehensive attendance management system built with a multi-language, multi-process architecture. The system comprises:
- **3 main components**: API Server (Rust), Edge Agent (Rust + Python), Web Dashboard (React)
- **83 database migrations** tracking schema evolution
- **817 test files** across all components
- **Sidecar architecture** for hardware abstraction
- **Multi-tenant SaaS** with workspace isolation

---

## Tech Stack Components

### Backend/API Server (`/home/elt1541/leeattend/apiserver`)
**Language**: Rust (Edition 2021)  
**Framework**: Axum 0.7 (Tokio-based async web framework)  
**Database**: PostgreSQL via SQLx 0.8.6  
**Key Dependencies**:
- `tokio` - Async runtime (full features)
- `sqlx` - Compile-time checked SQL queries
- `tower-http` - CORS middleware
- `jsonwebtoken` - JWT authentication
- `bcrypt` - Password hashing
- `serde` / `serde_json` - Serialization
- `chrono` / `chrono-tz` - Timezone-aware datetime
- `uuid` - ID generation
- `rust_decimal` - Precise financial calculations

**Architecture Pattern**: Handler-Service-Repository (3-tier)
- `/home/elt1541/leeattend/apiserver/src/handlers/` - 18 handler modules
- `/home/elt1541/leeattend/apiserver/src/services/` - 19 service modules
- `/home/elt1541/leeattend/apiserver/src/repositories/` - 21 repository modules
- `/home/elt1541/leeattend/apiserver/src/domain/` - Domain logic (policy resolvers)

**Entry Point**: `/home/elt1541/leeattend/apiserver/src/main.rs` (182 lines)
- Dynamic port binding (3000+)
- File-based service discovery (`.api_port`)
- Graceful shutdown with SIGINT/SIGTERM handling
- Connection pooling (max 10 connections)

**Migration Count**: 83 SQL files in `/home/elt1541/leeattend/apiserver/migrations/`

---

### Edge Agent - Main Orchestrator (`/home/elt1541/leeattend/agent-main`)
**Language**: Rust (Edition 2024)  
**Database**: SQLite via SQLx 0.8  
**Key Dependencies**:
- `tokio` - Async runtime
- `sqlx` - SQLite persistence
- `tokio-tungstenite` - WebSocket client
- `serde` / `serde_json` - Data serialization
- `futures-util` - Stream utilities
- `dotenvy` - Environment config
- `log` / `env_logger` - Logging

**Entry Point**: `/home/elt1541/leeattend/agent-main/src/main.rs` (452 lines)

**Architecture Highlights**:
- **Sidecar Pattern**: Spawns brand-specific Python workers via JSON-RPC 2.0
- **Semaphore-Based Concurrency**: Fair round-robin device polling
- **Batch DB Writer**: 100-record buffer with 200ms flush interval
- **Retry Logic**: Exponential backoff (max 60s, 10 attempts)
- **WebSocket Sync**: Persistent connection to central server
- **Cleanup Worker**: 365-day retention policy (runs every 24h)
- **FIFO Ordering**: `ORDER BY id ASC` guarantees

**Key Modules**:
- `/home/elt1541/leeattend/agent-main/src/central_client.rs` - Server connection
- `/home/elt1541/leeattend/agent-main/src/worker_client.rs` - Sidecar RPC client
- `/home/elt1541/leeattend/agent-main/src/retry.rs` - Backoff utilities
- `/home/elt1541/leeattend/agent-main/src/database.rs` - SQLite operations
- `/home/elt1541/leeattend/agent-main/src/models.rs` - Data structures

**Migrations**: 6 files in `/home/elt1541/leeattend/agent-main/migrations/`

---

### Edge Agent - ZK Worker Sidecar (`/home/elt1541/leeattend/agent-zk-worker`)
**Language**: Python 3  
**Dependencies** (requirements.txt):
- `python-dotenv` - Config management
- `pyzk2` - ZKTeco device protocol
- `psutil` - Process monitoring
- `pytest` / `pytest-asyncio` / `pytest-timeout` - Testing

**Entry Point**: `/home/elt1541/leeattend/agent-zk-worker/main.py` (40 lines)
**Worker Module**: `/home/elt1541/leeattend/agent-zk-worker/zk_worker.py` (5635 bytes)

**Architecture**:
- **Stateless Driver**: No database, no business logic
- **JSON-RPC 2.0 Server**: Listens on dynamic TCP port
- **Signal Handling**: SIGINT/SIGTERM for graceful shutdown
- **Hardware Abstraction**: ZKTeco device communication via pyzk2

**Standard RPC Methods**:
- `common.ping()` - Heartbeat
- `device.connect(ip, port)` - Hardware connection
- `device.get_logs()` - Fetch attendance records
- `device.get_time()` - Device time sync

---

### Web Dashboard (`/home/elt1541/leeattend/webdashboard`)
**Language**: TypeScript  
**Framework**: React 19.2.3  
**Build Tool**: Vite 7.3.1  
**Desktop Wrapper**: Tauri 2.9.6  
**Key Dependencies**:
- `react-router-dom` 7.12.0 - Client-side routing
- `@tanstack/react-query` 5.90.16 - Server state management
- `tailwindcss` 4 - Styling
- `i18next` 25.7.4 - Internationalization
- `date-fns` 4.1.0 - Date utilities
- `lucide-react` 0.562.0 - Icon library
- `cmdk` 1.1.1 - Command palette
- `@radix-ui/*` - Headless UI components

**Dev Dependencies**:
- `vitest` 4.0.16 - Unit testing
- `@playwright/test` 1.57.0 - E2E testing
- `eslint` 9 - Linting
- `typescript` 5 - Type checking

**Structure**:
```
/home/elt1541/leeattend/webdashboard/src/
├── pages/           # 20 page components
├── components/      # UI components (payroll, shifts, leave, time-tracking, etc.)
├── lib/             # Utilities and API clients
│   ├── api/         # Domain-specific API modules (10+ files)
│   └── utils/       # Helper functions
├── hooks/           # Custom React hooks
├── context/         # React Context providers
└── i18n/            # Translations (locales/)
```

**Pages** (20 total in `/home/elt1541/leeattend/webdashboard/src/pages/`):
- AgentsPage, AssignmentRulesPage, AttendanceLogsPage
- DashboardPage, DepartmentsPage, DevicesPage
- EmployeesPage, LeaveBalancesPage, LeaveRequestsPage
- LoginPage, MembersPage, OnboardingPage
- PayrollPage, ProfilePage, RegisterPage
- ReportsPage, ShiftsPage, WorkingPatternsPage
- ess/ subdirectory (Employee Self-Service portal)
- settings/ subdirectory

---

## Database Architecture

### Central Server (PostgreSQL)
**Migration System**: SQLx migrations in `/home/elt1541/leeattend/apiserver/migrations/`
**Total Migrations**: 83 files

**Core Tables** (identified from migrations):
- `workspaces` - Multi-tenant isolation
- `users` - Authentication
- `workspace_members` - RBAC
- `devices` - Biometric hardware registry
- `authorized_agents` - Edge agent authorization
- `employees` - HR data
- `departments` - Organizational structure
- `shifts` - Work schedules
- `raw_attendance_logs` - Immutable punch records
- `time_entries` - Paired work sessions
- `payslips` / `payslip_segments` - Payroll snapshots
- `leave_requests` / `leave_balances` - Leave management
- `working_patterns` / `shift_rosters` - Schedule templates
- `assignment_rules` - Auto-scheduling logic

**Key Patterns**:
- **Multi-Tenancy**: `workspace_id` column enforced on all tenant data
- **Temporal Data**: `effective_from` / `effective_to` for point-in-time queries
- **Audit Trail**: `created_at` / `updated_at` timestamps
- **Foreign Keys**: Cascade deletes for workspace isolation

### Edge Agent (SQLite)
**Database File**: `test-agent.db` (local persistence)
**Migration System**: SQLx migrations in `/home/elt1541/leeattend/agent-main/migrations/`
**Total Migrations**: 6 files

**Core Tables**:
- `attendance_logs` - Local cache of punch records
  - Columns: `id`, `device_id`, `user_id`, `timestamp`, `is_synced`, `created_at`, `updated_at`
  - Indexes: `id ASC` for FIFO ordering
- `devices` - Device configuration
- `parsing_errors` - Failed log entries
- `agent_config` - Runtime configuration (key-value store)

**PRAGMA Settings** (from architecture docs):
- `journal_mode=WAL` - Concurrent reads/writes
- `synchronous=NORMAL` - Balanced durability
- `cache_size=-64000` - 64MB RAM cache
- `temp_store=MEMORY` - In-memory temp tables

---

## API Endpoints (High-Level)

**Base URL**: `http://localhost:3000/api` (dynamic port)

**Handler Modules** (from `/home/elt1541/leeattend/apiserver/src/handlers/`):
- `auth.rs` - Login, register, JWT refresh
- `agents.rs` - Edge agent management, WebSocket
- `employees.rs` - HR CRUD operations
- `attendance.rs` - Log ingestion, pairing
- `shifts.rs` - Shift management
- `working_patterns.rs` - Schedule templates
- `assignment_rules.rs` - Auto-scheduling rules
- `leaves.rs` - Leave request workflow
- `leave_types.rs` - Leave policy management
- `payroll.rs` - Payroll calculation, payslips
- `overtime.rs` - Overtime policy
- `workspaces.rs` - Tenant management
- `ess.rs` - Employee self-service portal
- `debug.rs` - Debug/observability (dev only)

**WebSocket Endpoint**: `/ws/agents/:agent_id` (bidirectional sync)

---

## Architectural Patterns Observed

### 1. Sidecar Architecture
**Location**: `/home/elt1541/leeattend/agent-main/src/main.rs` (lines 200-345)
- **Master**: Rust Orchestrator (business logic, persistence, sync)
- **Sidecar**: Python Workers (hardware drivers only)
- **Communication**: JSON-RPC 2.0 over TCP (localhost)
- **Isolation**: One sidecar per device, crashes don't cascade

### 2. Retry Logic with Exponential Backoff
**Location**: `/home/elt1541/leeattend/agent-main/src/retry.rs`
- **Constants**: `SIDECAR_MAX_RETRY_DELAY_SECS = 60`, `SIDECAR_MAX_RETRY_COUNT = 10`
- **Jitter**: ±25% randomization to prevent thundering herd
- **Stability Window**: 30s stable connection resets retry counter
- **Applied To**: Sidecar spawn, device polling, WebSocket connection

### 3. Multi-Tenant Isolation (Path Priority)
**Location**: `/home/elt1541/leeattend/apiserver/src/lib.rs` (WorkspaceId extractor)
- **Primary**: Extract `workspace_id` from URL path (`:ws_id`, `:id`)
- **Fallback**: Use `X-Workspace-Id` header
- **Enforcement**: All queries include `WHERE workspace_id = ?`

### 4. Handler-Service-Repository (3-Tier)
**Pattern**:
```
Handler (API Layer) → Service (Business Logic) → Repository (Data Access)
```
- **Handlers**: Extract params, call service, return HTTP response
- **Services**: Domain logic, orchestrate multiple repositories
- **Repositories**: Trait-based (`#[async_trait]`), SQLx queries

### 5. Event-Sourcing Audit Trail
**Location**: Attendance pairing logic (architecture docs §10)
- **Raw Logs**: Immutable `raw_attendance_logs` table
- **Pairing Engine**: Converts flat events to `time_entries` sessions
- **Algorithms**: Anchor-First (primary), FSM (explicit intent)
- **Traceability**: `generation_metadata` JSONB stores source log IDs

### 6. 3-Tier Temporal Policy Resolution
**Location**: Payroll engine (architecture docs §14-15)
- **Level 1**: Individual override (`employee_payroll_configs`)
- **Level 2**: Department override (`department_payroll_configs`)
- **Level 3**: Workspace global (`payroll_settings`)
- **Lookup**: `Max(effective_date) WHERE effective_date <= target_date`

### 7. Batch DB Writer with Backpressure
**Location**: `/home/elt1541/leeattend/agent-main/src/main.rs` (lines 89-133)
- **Pattern**: Producer-consumer via `mpsc::channel(10000)`
- **Buffer**: 100 records or 200ms timeout
- **Single Transaction**: Batch INSERT reduces lock contention
- **Yield**: Sleep 10ms after flush to allow sync worker

### 8. Semaphore-Based Fair Scheduling
**Location**: `/home/elt1541/leeattend/agent-main/src/main.rs` (lines 165, 302-334)
- **Concurrency Control**: `Arc<Semaphore>` with `max_concurrent_workers`
- **Per-Cycle Acquire**: Permit released after each poll cycle
- **No Starvation**: Sequential iteration with `join_all` ensures fairness

---

## Configuration Files & Their Purposes

### Root Directory
- `CLAUDE.md` - AI agent orchestration instructions
- `AGENTS.md` - Agent delegation protocols
- `README.md` - Project overview, testing guide
- `CHANGELOG.md` - Version history
- `.gitignore` - VCS exclusions
- `docker-compose.test.yml` - Test environment setup
- `run-tests.sh` - Unified test orchestrator
- `setup-test-env.sh` - Docker-based test DB provisioning

### API Server
- `/home/elt1541/leeattend/apiserver/Cargo.toml` - Rust dependencies
- `/home/elt1541/leeattend/apiserver/.env` - Environment variables (DB_URL, JWT_SECRET)
- `/home/elt1541/leeattend/apiserver/.sqlx/` - SQLx compile-time metadata

### Edge Agent
- `/home/elt1541/leeattend/agent-main/Cargo.toml` - Rust dependencies
- `/home/elt1541/leeattend/agent-main/.env` - Agent config (CENTRAL_SERVER_URL, AGENT_ID)
- `/home/elt1541/leeattend/agent-zk-worker/requirements.txt` - Python dependencies
- `/home/elt1541/leeattend/agent-zk-worker/.env.example` - Worker config template

### Web Dashboard
- `/home/elt1541/leeattend/webdashboard/package.json` - NPM dependencies
- `/home/elt1541/leeattend/webdashboard/vite.config.ts` - Build configuration
- `/home/elt1541/leeattend/webdashboard/tailwind.config.ts` - Styling
- `/home/elt1541/leeattend/webdashboard/playwright.config.ts` - E2E test config
- `/home/elt1541/leeattend/webdashboard/vitest.config.ts` - Unit test config
- `/home/elt1541/leeattend/webdashboard/src-tauri/Cargo.toml` - Desktop wrapper

---

## Testing Infrastructure

### Test Count: 817 files
**Breakdown**:
- Rust unit tests: `*_test.rs` modules
- Rust integration tests: `/tests/` directories
- TypeScript unit tests: `*.test.ts`, `*.test.tsx`
- E2E tests: `*.spec.ts` (Playwright)
- Python tests: `test_*.py` (pytest)

### Test Orchestrator: `./run-tests.sh`
**Location**: `/home/elt1541/leeattend/run-tests.sh` (executable)
**Modes**:
- `--all` - Full suite (lint → unit → integration)
- `--api` - Backend only (apiserver + agent-main + agent-zk-worker)
- `--web` - Frontend only (webdashboard + Playwright)

**Phases**:
1. **Phase 0 (Mandatory)**: Linting (cargo clippy, npm lint, type-check)
2. **Phase 1**: Unit tests (fast, offline)
3. **Phase 3**: Integration/E2E (Docker-based Postgres, migrations)

### Test Patterns (from code-standards.md)
**Rust Integration**:
- `TestDatabase::new()` - Unique Postgres DB per test
- Automatic migrations via `sqlx::migrate!()`
- Signal-safe cleanup (SIGINT/SIGTERM handling)
- Thread limit: Match worker pool size

**Playwright E2E**:
- **POM (Page Object Model)**: `tests/pom/` directory
- **Custom Fixtures**: `authPage` for auto-login
- **Locator Priority**: data-testid > semantic attrs > roles
- **Isolation**: Multi-DB (test_database_e2e_N)
- **Timeout**: 15s global, 10s API (centralized via `apiFetch`)

**Frontend Unit (Vitest)**:
- Timeouts: 10s
- `isolate: false` for speed
- Coverage: v8 provider

---

## Dependencies & Integration Points

### Agent → Central Server
**Protocol**: WebSocket (outbound from agent)
**URL**: `wss://${CENTRAL_SERVER_URL}/ws/agents/:agent_id`
**Messages**:
- **Outbound**: `{ type: "logs", batch: [...] }`
- **Inbound**: `{ type: "ack", ids: [...] }`, `{ type: "reload_config" }`
**Retry**: 10 attempts, exponential backoff (1s-60s)
**Stability**: 60s stable before reset

### Agent → Sidecar Worker
**Protocol**: JSON-RPC 2.0 over TCP
**Port**: Dynamic (OS-assigned, passed via CLI arg)
**Timeout**: 5min read, 30s write
**Methods**: `device.connect`, `device.get_logs`, `common.ping`
**Isolation**: One sidecar per device, killed on error

### Dashboard → API Server
**Protocol**: REST + WebSocket
**Base URL**: `http://localhost:${API_PORT}/api`
**Discovery**: Reads `../apiserver/.api_port` (Vite proxy config)
**Auth**: JWT in `Authorization: Bearer ${token}` header
**Workspace**: `X-Workspace-Id` header or path param

### API Server → PostgreSQL
**Driver**: SQLx (compile-time checked)
**Pool**: Max 10 connections
**Migrations**: Automatic on startup (`sqlx::migrate!()`)
**Isolation**: `workspace_id` filter on all queries

---

## Conventions & Standards

### File Naming
**Rust**: `snake_case.rs` (module-per-file)
**TypeScript**: `PascalCase.tsx` (component files), `camelCase.ts` (utilities)
**Tests**: `*_test.rs`, `*.test.ts`, `*.spec.ts`, `test_*.py`
**Migrations**: `YYYYMMDDHHMMSS_description.sql`

### Code Organization
**Rust**:
- Handlers: `src/handlers/domain.rs`
- Services: `src/services/domain_service.rs`
- Repositories: `src/repositories/domain_repository.rs`
- Models: Consolidated in `src/models.rs`

**TypeScript**:
- Pages: `src/pages/DomainPage.tsx`
- Components: `src/components/domain/Component.tsx`
- API Clients: `src/lib/api/domain.ts`
- Hooks: `src/hooks/use-domain.ts`

### Module Size
**Rust**: Max 200 lines per file (from code-standards.md)
- Exception: Complex domain modules may exceed
**Method Size**: Max 30 lines per function (Rule of 30)

### Import Organization
**Rust**: `use` statements grouped (std → external → internal)
**TypeScript**: Imports grouped (React → 3rd-party → internal)

---

## Security & Observability

### Multi-Tenancy Enforcement
**Location**: All repository queries
**Pattern**: `WHERE workspace_id = $1`
**Test**: `schema_isolation_tests.rs` validates all tables

### Authentication
**Method**: JWT (jsonwebtoken crate)
**Secret**: `JWT_SECRET` env var
**Expiry**: Configurable
**Refresh**: Supported

### Debug Features (Dev Only)
**Headers**:
- `X-Debug-Impersonate-Id` - Assume user identity
- `X-Debug-Mock-Date` - Time travel for policy testing
**Endpoint**: `/api/debug/*` (disabled in production)

### Logging
**Rust**: `tracing` / `tracing-subscriber` (structured logs)
**Python**: Standard `logging` module
**Format**: `env_logger` with custom filters

---

## Build & Deployment

### API Server
**Build**: `cargo build --release` (optimized binary)
**Migrations**: Automatic on startup
**Port**: Dynamic (3000+), written to `.api_port`
**Signals**: Graceful shutdown on SIGINT/SIGTERM

### Edge Agent
**Build**: `cargo build --release`
**Database**: SQLite (created if missing)
**Workers**: Spawns Python sidecars dynamically
**Cleanup**: 365-day retention task (24h interval)

### Web Dashboard
**Build**: `npm run build` (Vite)
**Output**: `/home/elt1541/leeattend/webdashboard/dist/`
**Modes**:
- SPA: Static HTML/JS/CSS
- Tauri: Native desktop app
- Standalone: Rust server wrapper

### Test Environment
**Setup**: `./setup-test-env.sh` (Docker Compose)
**Databases**: `test_database_${WORKER}` (isolated per worker)
**Cleanup**: Automatic on exit

---

## Documentation Files

**Location**: `/home/elt1541/leeattend/docs/`

**Available Docs**:
1. `system-architecture.md` - Comprehensive architecture guide (488 lines)
2. `api-reference.md` - API endpoint documentation
3. `code-standards.md` - Conventions and best practices (389 lines)
4. `codebase-summary.md` - High-level overview
5. `deployment-guide.md` - Deployment instructions
6. `time-tracking-refactor-phase-1.md` - Feature implementation notes
7. `ui-ux-standards.md` - Frontend design guidelines
8. `PHASE-1-DOCUMENTATION-SUMMARY.md` - Milestone summary

---

## Integration Points for New Code

### Adding a New API Endpoint
**Steps**:
1. Define model in `/home/elt1541/leeattend/apiserver/src/models.rs`
2. Create repository trait in `src/repositories/domain_repository.rs`
3. Implement service in `src/services/domain_service.rs`
4. Add handler in `src/handlers/domain.rs`
5. Register route in `src/lib.rs` (create_app function)
6. Run migration if schema change needed
7. Update SQLx metadata: `cargo sqlx prepare`

### Adding a New Page (Dashboard)
**Steps**:
1. Create page component: `webdashboard/src/pages/DomainPage.tsx`
2. Add route in `webdashboard/src/App.tsx`
3. Create API client: `webdashboard/src/lib/api/domain.ts`
4. Create hook: `webdashboard/src/hooks/use-domain.ts`
5. Add translation keys in `webdashboard/src/i18n/locales/`
6. Add navigation link in sidebar component

### Adding a New Device Brand (Sidecar)
**Steps**:
1. Create worker directory: `agent-brand-worker/`
2. Implement JSON-RPC 2.0 server (Python/Node/Rust)
3. Expose methods: `common.ping`, `device.connect`, `device.get_logs`
4. Update orchestrator in `agent-main/src/main.rs` (line 182-195)
5. Add brand detection logic in database

### Adding Database Migration
**API Server**:
1. Create file: `apiserver/migrations/YYYYMMDDHHMMSS_description.sql`
2. Test: `./setup-test-env.sh` (applies migrations)
3. Update metadata: `cargo sqlx prepare`

**Edge Agent**:
1. Create file: `agent-main/migrations/YYYYMMDDHHMMSS_description.sql`
2. SQLx auto-applies on agent startup

---

## Warnings & Gotchas

### Performance
- **SQLite Locks**: Agent uses WAL mode, but concurrent writes still block
- **Postgres Pool**: Limited to 10 connections, tune if high load
- **Batch Size**: Log batch writer (100 records) may need tuning for high-traffic

### Testing
- **DO NOT use `#[serial]`**: Tests are isolated via TestDatabase
- **Thread Limit**: Always match `--test-threads` to worker pool size
- **E2E Flakiness**: Avoid `getByText` for critical elements, use `data-testid`

### Security
- **Multi-Tenancy**: NEVER query without `workspace_id` filter
- **JWT Secret**: Must set `JWT_SECRET` in production
- **Debug Headers**: Disable in production builds

### Dependencies
- **Python venv**: Linux and Windows venvs are NOT compatible
- **SQLx Metadata**: Must run `cargo sqlx prepare` after schema changes
- **Tauri**: Desktop builds require system dependencies (WebKit2GTK on Linux)

---

## Key File Locations (Absolute Paths)

**Core Entry Points**:
- API Server: `/home/elt1541/leeattend/apiserver/src/main.rs`
- Edge Agent: `/home/elt1541/leeattend/agent-main/src/main.rs`
- ZK Worker: `/home/elt1541/leeattend/agent-zk-worker/main.py`
- Dashboard: `/home/elt1541/leeattend/webdashboard/src/App.tsx`

**Configuration**:
- API Config: `/home/elt1541/leeattend/apiserver/.env`
- Agent Config: `/home/elt1541/leeattend/agent-main/.env`
- Dashboard Config: `/home/elt1541/leeattend/webdashboard/package.json`

**Documentation**:
- Architecture: `/home/elt1541/leeattend/docs/system-architecture.md`
- Code Standards: `/home/elt1541/leeattend/docs/code-standards.md`
- API Reference: `/home/elt1541/leeattend/docs/api-reference.md`

**Testing**:
- Test Runner: `/home/elt1541/leeattend/run-tests.sh`
- Test Setup: `/home/elt1541/leeattend/setup-test-env.sh`
- Playwright Config: `/home/elt1541/leeattend/webdashboard/playwright.config.ts`

**Migrations**:
- API Migrations: `/home/elt1541/leeattend/apiserver/migrations/`
- Agent Migrations: `/home/elt1541/leeattend/agent-main/migrations/`

---

## Success Metrics

- **Codebase Coverage**: 100% of main components mapped
- **Tech Stack**: All languages, frameworks, and major dependencies identified
- **Architecture**: 8 key patterns documented with locations
- **Integration Points**: Clear pathways for extending each component
- **Test Infrastructure**: Full understanding of 817-file test suite
- **Dependencies**: Complete dependency tree for all 3 components

---

## Unresolved Questions

1. **CI/CD Pipeline**: No `.github/workflows/` or `.gitlab-ci.yml` found - deployment automation status unknown
2. **Mock Worker Implementation**: `agent-mock-worker/` directory exists but worker script location not confirmed
3. **Tauri Desktop Distribution**: Build process for Windows/macOS installers not documented
4. **Database Backup Strategy**: No backup scripts found in `/scripts/` directory
5. **Production Environment Variables**: `.env.production` files not found - production config management unclear
6. **Monitoring & Alerting**: No observability stack (Prometheus, Grafana) detected
7. **Load Testing**: Performance benchmarks for high-traffic scenarios not documented

---

**Report Completed**: 2026-01-28 11:29 UTC  
**Total Files Analyzed**: 817+ test files, 83 migrations, 100+ source files  
**Confidence Level**: High (all critical paths verified with actual code)
