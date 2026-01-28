# LeeAttend Source Code Structure

## Repository Root

Location: `/home/elt1541/leeattend/`

### Key Files at Root Level

- `README.md` - Project overview and testing guide
- `CLAUDE.md` - AI agent orchestration instructions
- `AGENTS.md` - Agent delegation protocols
- `CHANGELOG.md` - Version history
- `.gitignore` - VCS exclusions
- `run-tests.sh` - Unified test orchestrator (executable)
- `setup-test-env.sh` - Docker-based test database provisioning
- `docker-compose.test.yml` - Test environment configuration

## Component Directory Structure

### 1. API Server

**Location**: `/home/elt1541/leeattend/apiserver/`

**Entry Point**: `/home/elt1541/leeattend/apiserver/src/main.rs` (182 lines)

**Purpose**: Central REST API server (Rust + Axum + PostgreSQL)

**Directory Structure**:
```
apiserver/
├── src/
│   ├── main.rs                  (182 lines) - Server entry point
│   ├── lib.rs                   - App configuration, middleware
│   ├── models.rs                - Domain data structures
│   ├── handlers/                (18 modules) - HTTP request handlers
│   │   ├── auth.rs             - Login, register, JWT
│   │   ├── agents.rs           - Agent management
│   │   ├── employees.rs        - Employee CRUD
│   │   ├── attendance.rs       - Log ingestion
│   │   ├── payroll.rs          - Payroll operations
│   │   ├── leaves.rs           - Leave workflows
│   │   ├── shifts.rs           - Shift management
│   │   ├── working_patterns.rs - Schedule templates
│   │   ├── assignment_rules.rs - Auto-scheduling
│   │   ├── departments.rs      - Org structure
│   │   ├── leave_types.rs      - Leave policy
│   │   ├── overtime.rs         - Overtime policy
│   │   ├── workspaces.rs       - Tenant management
│   │   ├── ess.rs              - Employee self-service
│   │   ├── devices.rs          - Device management
│   │   ├── debug.rs            - Debug endpoints (dev-only)
│   │   └── error.rs            - Error handling
│   ├── services/                (19 modules) - Business logic
│   │   ├── auth_service.rs     - JWT, password handling
│   │   ├── employee_service.rs - Employee operations
│   │   ├── attendance_service.rs - Pairing logic
│   │   ├── payroll_service.rs  - Payroll calculation
│   │   ├── leave_service.rs    - Leave accrual
│   │   ├── shift_service.rs    - Shift assignment
│   │   ├── device_service.rs   - Device coordination
│   │   └── ... (13 more services)
│   ├── repositories/            (21 modules) - Data access
│   │   ├── employee_repository.rs
│   │   ├── attendance_repository.rs
│   │   ├── payroll_repository.rs
│   │   ├── leave_repository.rs
│   │   ├── shift_repository.rs
│   │   ├── device_repository.rs
│   │   └── ... (15 more repositories)
│   ├── domain/
│   │   ├── policy_resolver.rs  - Payroll tier resolution
│   │   └── pairing_engine.rs   - Attendance pairing
│   └── middleware/
│       ├── auth.rs             - JWT validation
│       ├── workspace.rs        - Tenant isolation
│       └── error_handling.rs   - Error responses
├── migrations/                  (83 SQL files)
│   ├── 20240101000000_initial_schema.sql
│   ├── 20240101000001_users_auth.sql
│   ├── ... (81 more migration files)
│   └── 20260128000000_latest_schema.sql
├── tests/                       - Integration tests
│   ├── common/
│   │   └── mod.rs              - Test utilities
│   ├── auth_tests.rs
│   ├── employee_tests.rs
│   ├── attendance_tests.rs
│   ├── payroll_tests.rs
│   ├── schema_isolation_tests.rs
│   └── ... (more integration tests)
├── Cargo.toml                   - Rust dependencies
├── Cargo.lock                   - Dependency lock file
├── .env                         - Environment variables (local)
├── .env.example                 - Template (checked in)
├── .sqlx/                       - SQLx compile-time metadata
└── rust-toolchain.toml          - Rust version

**Key Rust Dependencies**:
- `axum` 0.7 - Web framework
- `tokio` - Async runtime (full features)
- `sqlx` 0.8.6 - Database driver
- `tower-http` - CORS middleware
- `jsonwebtoken` - JWT handling
- `bcrypt` - Password hashing
- `serde`/`serde_json` - Serialization
- `chrono`/`chrono-tz` - DateTime
- `uuid` - ID generation
- `tracing`/`tracing-subscriber` - Logging

**Configuration**:
- Database URL: `DATABASE_URL` environment variable
- JWT secret: `JWT_SECRET` environment variable
- Server port: Dynamic (3000+), written to `.api_port`
- Graceful shutdown: Catches SIGINT/SIGTERM

**Compile-Time Checks**:
- SQLx queries verified against live PostgreSQL
- Metadata cache in `.sqlx/` directory
- Update via: `cargo sqlx prepare`

### 2. Edge Agent - Main

**Location**: `/home/elt1541/leeattend/agent-main/`

**Entry Point**: `/home/elt1541/leeattend/agent-main/src/main.rs` (452 lines)

**Purpose**: Edge orchestrator (Rust + SQLite + WebSocket)

**Directory Structure**:
```
agent-main/
├── src/
│   ├── main.rs                 (452 lines) - Main orchestration
│   ├── lib.rs                  - Library exports
│   ├── models.rs               - Data structures
│   ├── database.rs             - SQLite operations
│   ├── central_client.rs       - WebSocket to server
│   ├── worker_client.rs        - JSON-RPC to sidecars
│   ├── retry.rs                - Exponential backoff
│   └── tests/                  - Unit tests
│       ├── retry_tests.rs
│       ├── database_tests.rs
│       └── ...
├── migrations/                  (6 SQL files)
│   ├── 20240101000000_initial_schema.sql
│   ├── 20240101000001_attendance_logs.sql
│   ├── ... (4 more migrations)
│   └── 20260128000000_latest_schema.sql
├── tests/                       - Integration tests
│   ├── integration_tests.rs
│   └── agent_orchestration_tests.rs
├── Cargo.toml                   - Rust dependencies
├── Cargo.lock                   - Dependency lock file
├── .env                         - Environment variables
├── .env.example                 - Template (checked in)
├── rust-toolchain.toml          - Rust version specification
└── test-agent.db               - SQLite database (generated at runtime)

**Key Modules**:

**main.rs** (452 lines):
- Async event loop using Tokio
- Semaphore-based concurrency control
- Batch database writer (100 records or 200ms)
- Device polling orchestration
- WebSocket connection maintenance
- Signal handling (SIGINT/SIGTERM)

**central_client.rs**:
- WebSocket connection to central server
- Message serialization (serde_json)
- Retry logic with exponential backoff
- Heartbeat/ping mechanism
- Configuration reload handler

**worker_client.rs**:
- JSON-RPC 2.0 client for sidecars
- Method calls: `device.connect`, `device.get_logs`
- Timeout handling (5min read, 30s write)
- Process spawning and monitoring

**database.rs**:
- SQLite connection pool
- PRAGMA configuration (WAL mode, cache)
- Table schema: attendance_logs, devices, parsing_errors, agent_config
- Batch INSERT operations
- FIFO ordering (ORDER BY id ASC)

**retry.rs**:
- Exponential backoff calculation
- Constants: MAX_DELAY=60s, MAX_ATTEMPTS=10
- Jitter ±25% randomization
- Stability window (30s resets counter)

**Key Rust Dependencies**:
- `tokio` - Async runtime
- `sqlx` - SQLite driver
- `tokio-tungstenite` - WebSocket client
- `serde`/`serde_json` - Serialization
- `futures-util` - Stream utilities
- `dotenvy` - Environment config
- `log`/`env_logger` - Logging

**Configuration**:
- Central server URL: `CENTRAL_SERVER_URL` env var
- Agent ID: `AGENT_ID` env var
- Logging level: `RUST_LOG` env var
- Database: `test-agent.db` (SQLite, auto-created)

### 3. Edge Agent - ZK Worker

**Location**: `/home/elt1541/leeattend/agent-zk-worker/`

**Entry Point**: `/home/elt1541/leeattend/agent-zk-worker/main.py` (40 lines)

**Purpose**: Stateless hardware driver for ZKTeco devices (Python)

**Directory Structure**:
```
agent-zk-worker/
├── main.py                      (40 lines) - Entry point
├── zk_worker.py                 (5635 bytes) - Implementation
├── tests/
│   ├── test_device_connect.py
│   ├── test_log_retrieval.py
│   └── ...
├── requirements.txt             - Python dependencies
├── .env.example                 - Template
├── pytest.ini                   - Test configuration
└── pyproject.toml               - Project metadata

**Key Modules**:

**main.py** (40 lines):
- Entry point for sidecar process
- Argument parsing (port, device config)
- Signal handling (SIGINT/SIGTERM)
- JSON-RPC server initialization

**zk_worker.py** (5635 bytes):
- JSON-RPC 2.0 server implementation
- Device connection logic
- Log retrieval (parsing ZKTeco protocol)
- Device time synchronization
- Error handling and recovery

**Standard RPC Methods**:
```
common.ping() → true
device.connect(ip, port) → {status, info}
device.get_logs() → [{timestamp, user_id, log_type}]
device.get_time() → {device_time, server_time}
```

**Python Dependencies**:
- `pyzk2` - ZKTeco device protocol
- `python-dotenv` - Config management
- `psutil` - Process monitoring
- `pytest` - Testing framework
- `pytest-asyncio` - Async test support
- `pytest-timeout` - Test timeout management

**Configuration**:
- Listen port: Passed via CLI argument
- Logging level: `LOG_LEVEL` env var
- Device timeout: 5min read, 30s write

### 4. Web Dashboard

**Location**: `/home/elt1541/leeattend/webdashboard/`

**Entry Point**: `/home/elt1541/leeattend/webdashboard/src/App.tsx`

**Purpose**: React-based web dashboard (TypeScript + Vite + React 19)

**Directory Structure**:
```
webdashboard/
├── src/
│   ├── App.tsx                  - Root component
│   ├── main.tsx                 - Vite entry
│   ├── pages/                   (20 page components)
│   │   ├── DashboardPage.tsx
│   │   ├── AttendanceLogsPage.tsx
│   │   ├── EmployeesPage.tsx
│   │   ├── DepartmentsPage.tsx
│   │   ├── ShiftsPage.tsx
│   │   ├── WorkingPatternsPage.tsx
│   │   ├── LeaveRequestsPage.tsx
│   │   ├── LeaveBalancesPage.tsx
│   │   ├── PayrollPage.tsx
│   │   ├── AgentsPage.tsx
│   │   ├── DevicesPage.tsx
│   │   ├── AssignmentRulesPage.tsx
│   │   ├── LoginPage.tsx
│   │   ├── RegisterPage.tsx
│   │   ├── OnboardingPage.tsx
│   │   ├── ProfilePage.tsx
│   │   ├── ReportsPage.tsx
│   │   ├── MembersPage.tsx
│   │   ├── ess/                 - Employee self-service
│   │   │   ├── ESSPage.tsx
│   │   │   ├── LeaveRequestForm.tsx
│   │   │   └── MyAttendance.tsx
│   │   └── settings/
│   │       ├── SettingsPage.tsx
│   │       └── PreferencesPage.tsx
│   ├── components/              - Reusable UI components
│   │   ├── payroll/
│   │   │   ├── PayrollTable.tsx
│   │   │   ├── PayslipViewer.tsx
│   │   │   └── ...
│   │   ├── shifts/
│   │   │   ├── ShiftForm.tsx
│   │   │   ├── RosterGrid.tsx
│   │   │   └── ...
│   │   ├── leave/
│   │   │   ├── LeaveRequestForm.tsx
│   │   │   ├── BalanceWidget.tsx
│   │   │   └── ...
│   │   ├── attendance/
│   │   │   ├── LogViewer.tsx
│   │   │   ├── DeviceStatus.tsx
│   │   │   └── ...
│   │   ├── common/
│   │   │   ├── Header.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   ├── DataTable.tsx
│   │   │   └── ...
│   │   └── ...
│   ├── lib/
│   │   ├── api/                 (10+ API client modules)
│   │   │   ├── auth.ts
│   │   │   ├── employees.ts
│   │   │   ├── attendance.ts
│   │   │   ├── payroll.ts
│   │   │   ├── leaves.ts
│   │   │   ├── shifts.ts
│   │   │   ├── devices.ts
│   │   │   ├── agents.ts
│   │   │   └── ...
│   │   ├── utils/
│   │   │   ├── format.ts
│   │   │   ├── validation.ts
│   │   │   ├── constants.ts
│   │   │   └── ...
│   │   └── types/
│   │       ├── api.ts
│   │       ├── models.ts
│   │       └── ...
│   ├── hooks/                   - Custom React hooks
│   │   ├── use-auth.ts
│   │   ├── use-employees.ts
│   │   ├── use-attendance.ts
│   │   ├── use-payroll.ts
│   │   ├── use-leaves.ts
│   │   └── ...
│   ├── context/                 - React Context providers
│   │   ├── AuthContext.tsx
│   │   ├── WorkspaceContext.tsx
│   │   └── NotificationContext.tsx
│   ├── i18n/
│   │   ├── i18n.ts
│   │   └── locales/
│   │       ├── en.json
│   │       ├── vi.json          (Vietnamese)
│   │       └── ...
│   └── styles/
│       ├── globals.css
│       └── components/
├── tests/
│   ├── unit/                    - Vitest unit tests
│   │   ├── components/
│   │   ├── utils/
│   │   └── hooks/
│   └── e2e/                     - Playwright E2E tests
│       ├── tests/
│       │   ├── auth.spec.ts
│       │   ├── employees.spec.ts
│       │   ├── attendance.spec.ts
│       │   └── ...
│       ├── pom/                 - Page Object Model
│       │   ├── base-page.ts
│       │   ├── login-page.ts
│       │   ├── employees-page.ts
│       │   └── ...
│       └── fixtures/
│           └── auth.ts
├── src-tauri/                   - Tauri desktop wrapper
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
│   └── tauri.conf.json
├── public/                      - Static assets
│   └── favicon.ico
├── package.json                 - NPM dependencies
├── package-lock.json            - Dependency lock
├── tsconfig.json                - TypeScript config
├── vite.config.ts               - Build configuration
├── vitest.config.ts             - Unit test config
├── playwright.config.ts         - E2E test config
├── tailwind.config.ts           - Styling config
├── postcss.config.js            - CSS processing
├── eslint.config.js             - Linting rules
└── .env.example                 - Template

**Key NPM Dependencies**:
- `react` 19.2.3 - UI framework
- `react-dom` 19.2.3 - DOM rendering
- `react-router-dom` 7.12.0 - Routing
- `@tanstack/react-query` 5.90.16 - Server state
- `typescript` 5 - Type checking
- `vite` 7.3.1 - Build tool
- `tailwindcss` 4 - Styling
- `i18next` 25.7.4 - Internationalization
- `date-fns` 4.1.0 - Date utilities
- `lucide-react` 0.562.0 - Icons
- `cmdk` 1.1.1 - Command palette
- `@radix-ui/*` - Headless components

**Dev Dependencies**:
- `vitest` 4.0.16 - Unit testing
- `@playwright/test` 1.57.0 - E2E testing
- `eslint` 9 - Linting
- `@types/react` - Type definitions
- `@types/node` - Node types

**API Communication**:
- Base URL: `http://localhost:${API_PORT}/api`
- Discovery: Vite proxy reads `../apiserver/.api_port`
- Auth: JWT in Authorization header
- Workspace: `X-Workspace-Id` header

**Build Modes**:
- Development: `npm run dev` (Vite dev server)
- Production: `npm run build` (optimized bundle)
- Desktop: `cargo tauri build` (Tauri wrapper)
- Tests: `npm run test` (Vitest), `npm run test:e2e` (Playwright)

## Documentation Directory

**Location**: `/home/elt1541/leeattend/docs/`

**Key Files**:
- `system-architecture.md` (488 lines) - Architecture documentation
- `api-reference.md` - API endpoint documentation
- `code-standards.md` (389 lines) - Coding conventions
- `codebase-summary.md` - High-level overview
- `deployment-guide.md` - Deployment instructions
- `ui-ux-standards.md` - Frontend design guidelines
- `PHASE-1-DOCUMENTATION-SUMMARY.md` - Milestone notes

## Configuration Files

### Root Level
- `docker-compose.test.yml` - Test environment (PostgreSQL, Redis)
- `run-tests.sh` - Test orchestrator
- `setup-test-env.sh` - Docker setup script
- `.gitignore` - Git exclusions
- `CLAUDE.md` - AI orchestration
- `AGENTS.md` - Agent protocols
- `README.md` - Project overview
- `CHANGELOG.md` - Version history

### Per Component
- `Cargo.toml` - Rust dependencies (apiserver, agent-main)
- `Cargo.lock` - Dependency lock (Rust)
- `package.json` - NPM dependencies (webdashboard)
- `package-lock.json` - Dependency lock (npm)
- `.env` - Local environment variables (never commit)
- `.env.example` - Template (checked in, committed)
- `vite.config.ts` - Vite build config (webdashboard)
- `tailwind.config.ts` - Tailwind CSS config (webdashboard)
- `playwright.config.ts` - Playwright E2E config (webdashboard)
- `vitest.config.ts` - Vitest unit config (webdashboard)

## Entry Points

### API Server
- **Absolute Path**: `/home/elt1541/leeattend/apiserver/src/main.rs`
- **Lines**: 182
- **Primary Function**: Server initialization, route setup, signal handling
- **Port**: Dynamic (3000+), written to `.api_port`

### Edge Agent (Main)
- **Absolute Path**: `/home/elt1541/leeattend/agent-main/src/main.rs`
- **Lines**: 452
- **Primary Function**: Orchestration loop, device polling, WebSocket sync
- **Config**: `.env` file (CENTRAL_SERVER_URL, AGENT_ID)

### Edge Worker (ZK)
- **Absolute Path**: `/home/elt1541/leeattend/agent-zk-worker/main.py`
- **Lines**: 40
- **Primary Function**: Process entry, JSON-RPC server setup
- **Port**: Dynamic (passed via CLI argument)

### Web Dashboard
- **Absolute Path**: `/home/elt1541/leeattend/webdashboard/src/App.tsx`
- **Routing**: React Router v7
- **Build**: Vite (dev: 5173, prod: static files)

## Module Size Guidelines

**Target**: Max 200 lines per file (from code-standards.md)

**Rust Modules**:
- Handlers: Single domain per file (auth.rs, employees.rs)
- Services: Business logic, orchestrate repos
- Repositories: Data access layer only
- Models: Consolidated in models.rs

**TypeScript Components**:
- Pages: One page per file
- Components: Reusable UI units, grouped by domain
- Hooks: One hook per file (use-domain.ts)
- API clients: Domain-specific modules (employees.ts, payroll.ts)

**Exception**: Complex modules may exceed 200 lines with justification

## Code Organization Patterns

### Rust 3-Tier Pattern
```
handlers/domain.rs
  ↓
services/domain_service.rs
  ↓
repositories/domain_repository.rs
  ↓
models.rs (shared types)
```

### TypeScript Component Structure
```
pages/DomainPage.tsx
  ├── imports: components, hooks, api
  ├── useDomain() custom hook
  ├── components/domain/
  ├── lib/api/domain.ts
  └── i18n keys
```

## Navigation Guide

### To Add API Endpoint
1. Define model: `apiserver/src/models.rs`
2. Create repository: `apiserver/src/repositories/domain_repository.rs`
3. Create service: `apiserver/src/services/domain_service.rs`
4. Create handler: `apiserver/src/handlers/domain.rs`
5. Register route: `apiserver/src/lib.rs` (create_app)
6. Database migration: `apiserver/migrations/YYYYMMDDHHMMSS_*.sql`

### To Add Dashboard Page
1. Create page: `webdashboard/src/pages/DomainPage.tsx`
2. Add route: `webdashboard/src/App.tsx`
3. Create API client: `webdashboard/src/lib/api/domain.ts`
4. Create hook: `webdashboard/src/hooks/use-domain.ts`
5. Add i18n keys: `webdashboard/src/i18n/locales/*.json`
6. Update sidebar: `webdashboard/src/components/common/Sidebar.tsx`

### To Add Device Brand
1. Create worker: `agent-brand-worker/main.py`
2. Implement RPC methods: `common.ping`, `device.connect`, `device.get_logs`
3. Register brand: `agent-main/src/main.rs` (spawn logic)
4. Test: `agent-brand-worker/tests/`

### To Add Database Migration
**API Server**:
1. File: `apiserver/migrations/YYYYMMDDHHMMSS_description.sql`
2. Test: `./setup-test-env.sh` (applies migrations)
3. Update SQLx: `cargo sqlx prepare`

**Edge Agent**:
1. File: `agent-main/migrations/YYYYMMDDHHMMSS_description.sql`
2. Applied auto on agent startup

## Testing Infrastructure Location

- **Test Orchestrator**: `/home/elt1541/leeattend/run-tests.sh`
- **Setup Script**: `/home/elt1541/leeattend/setup-test-env.sh`
- **Config**: `/home/elt1541/leeattend/docker-compose.test.yml`
- **Rust Integration**: `/home/elt1541/leeattend/apiserver/tests/`
- **TypeScript E2E**: `/home/elt1541/leeattend/webdashboard/tests/e2e/`
- **Python Tests**: `/home/elt1541/leeattend/agent-zk-worker/tests/`

## File Size Compliance

Current state (from scout report):
- API Server: 18 handlers, 19 services, 21 repositories (modular)
- Edge Agent: Consolidated in main.rs (452 lines, may benefit from refactoring)
- Web Dashboard: 20 pages, organized by domain
- Documentation: Multiple focused files under `/home/elt1541/leeattend/docs/`

**Strategy**: Each component respects 200-line guideline for new code additions.

## Absolute Path Reference

**Component Entry Points**:
- `/home/elt1541/leeattend/apiserver/src/main.rs` - API server
- `/home/elt1541/leeattend/agent-main/src/main.rs` - Edge orchestrator
- `/home/elt1541/leeattend/agent-zk-worker/main.py` - ZK worker
- `/home/elt1541/leeattend/webdashboard/src/App.tsx` - Dashboard

**Configuration**:
- `/home/elt1541/leeattend/apiserver/.env` - API config
- `/home/elt1541/leeattend/agent-main/.env` - Agent config
- `/home/elt1541/leeattend/agent-zk-worker/.env` - Worker config
- `/home/elt1541/leeattend/webdashboard/.env` - Dashboard config (optional)

**Migrations**:
- `/home/elt1541/leeattend/apiserver/migrations/` - (83 files)
- `/home/elt1541/leeattend/agent-main/migrations/` - (6 files)

**Tests**:
- `/home/elt1541/leeattend/run-tests.sh` - Master orchestrator
- `/home/elt1541/leeattend/setup-test-env.sh` - Environment setup
- `/home/elt1541/leeattend/apiserver/tests/` - Integration tests
- `/home/elt1541/leeattend/webdashboard/tests/` - Unit + E2E

**Documentation**:
- `/home/elt1541/leeattend/docs/system-architecture.md` - Architecture
- `/home/elt1541/leeattend/docs/code-standards.md` - Standards
- `/home/elt1541/leeattend/docs/api-reference.md` - API docs

## Quick Navigation

To find code implementing a feature:
1. **API**: Search `apiserver/src/handlers/` for endpoint
2. **Business Logic**: Check corresponding service file
3. **Data Access**: Review repository implementation
4. **UI**: Check `webdashboard/src/pages/` for page
5. **Integration**: Review component imports

To understand data flow:
1. Start at handler (request entry)
2. Follow service calls (logic)
3. Check repository queries (database)
4. Review domain logic (policy resolution)
5. Trace models through chain

For device integration:
1. Check `agent-main/src/main.rs` (orchestration)
2. Review `agent-main/src/worker_client.rs` (RPC client)
3. Examine worker implementation (e.g., `agent-zk-worker/zk_worker.py`)
4. Verify RPC methods exposed
