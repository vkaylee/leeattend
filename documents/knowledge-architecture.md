# LeeAttend System Architecture

## Architecture Overview

LeeAttend employs a distributed, multi-component architecture designed for real-time attendance tracking with edge processing capability. The system separates concerns across three runtime components: a central API server, edge agents, and web clients.

## Core Components

### 1. API Server (Central Authority)

**Location**: `/home/elt1541/leeattend/apiserver/src/main.rs`

**Technology Stack**:
- Language: Rust Edition 2021
- Web Framework: Axum 0.7 (Tokio async)
- Database: PostgreSQL
- Database Driver: SQLx (compile-time checked queries)

**Responsibilities**:
- REST API endpoint for all client operations
- WebSocket gateway for real-time agent communication
- Database schema management (83 migrations)
- Authentication and authorization (JWT)
- Multi-tenant workspace isolation
- Business logic for payroll, leave, shifts

**Architecture Pattern**: 3-Tier (Handler-Service-Repository)

```
HTTP/WebSocket Request
    ↓
Handler Layer (18 modules)
    ↓ Extract & Validate
Service Layer (19 modules)
    ↓ Business Logic
Repository Layer (21 modules)
    ↓ Database Access
PostgreSQL
```

**Key Modules**:
- `handlers/auth.rs` - Login, register, JWT refresh
- `handlers/agents.rs` - Agent management, WebSocket
- `handlers/employees.rs` - HR CRUD operations
- `handlers/attendance.rs` - Log ingestion, pairing
- `handlers/payroll.rs` - Payroll operations
- `handlers/leaves.rs` - Leave workflows
- `services/` - Business logic orchestration
- `repositories/` - Database trait implementations
- `domain/` - Policy resolvers (multi-tier payroll)

**Port Binding**:
- Dynamic allocation (3000+)
- Written to `/home/elt1541/leeattend/apiserver/.api_port`
- Graceful shutdown on SIGINT/SIGTERM

### 2. Edge Agent - Main Orchestrator

**Location**: `/home/elt1541/leeattend/agent-main/src/main.rs` (452 lines)

**Technology Stack**:
- Language: Rust Edition 2024
- Database: SQLite (WAL mode)
- Database Driver: SQLx
- WebSocket Client: tokio-tungstenite
- Concurrency: Tokio + Semaphore

**Responsibilities**:
- Orchestrate worker processes (sidecars)
- Poll biometric devices on fair round-robin schedule
- Manage local SQLite cache
- Sync logs to central server via WebSocket
- Handle retry logic with exponential backoff
- Enforce FIFO ordering of log records
- Execute daily cleanup tasks (365-day retention)

**Architecture Highlights**:

**Sidecar Pattern**:
```
Agent Main (Rust, Orchestrator)
    ↕ JSON-RPC 2.0 (TCP)
Worker Process (Python, Stateless Driver)
    ↓
Biometric Device
```

- One sidecar spawned per device brand
- Stateless worker (no persistence, no business logic)
- Hardware abstraction: Workers handle all device protocol
- Isolation: Worker crashes don't cascade to orchestrator

**Concurrency Model**:
- Semaphore-based fair scheduling
- Per-cycle permit acquisition/release
- No starvation: Sequential iteration with `join_all`
- Configurable: `max_concurrent_workers` parameter

**Batch Processing**:
- Producer-consumer via `mpsc::channel(10000)`
- Buffer size: 100 records or 200ms timeout
- Single transaction per batch (reduces lock contention)
- Backpressure via 10ms yield after flush

**Retry Logic**:
- Location: `/home/elt1541/leeattend/agent-main/src/retry.rs`
- Max delay: 60 seconds
- Max attempts: 10
- Jitter: ±25% randomization
- Stability window: 30 seconds stable connection resets counter
- Applied to: Sidecar spawn, device polling, WebSocket

**Key Modules**:
- `central_client.rs` - WebSocket connection to server
- `worker_client.rs` - JSON-RPC client for sidecars
- `database.rs` - SQLite operations
- `retry.rs` - Backoff utilities
- `models.rs` - Data structures
- `main.rs` - Orchestration loop

**Database** (`test-agent.db`):
- Connection pool: SQLite default
- PRAGMA `journal_mode=WAL` - Concurrent reads/writes
- PRAGMA `synchronous=NORMAL` - Balanced durability
- PRAGMA `cache_size=-64000` - 64MB in-memory cache
- PRAGMA `temp_store=MEMORY` - In-memory temp tables

**Tables**:
- `attendance_logs` - Local punch cache (indexed by id for FIFO)
- `devices` - Device configuration and status
- `parsing_errors` - Failed log entries for troubleshooting
- `agent_config` - Runtime configuration (key-value)

### 3. Edge Agent - ZK Worker Sidecar

**Location**: `/home/elt1541/leeattend/agent-zk-worker/main.py` (40 lines)

**Technology Stack**:
- Language: Python 3
- Device Protocol: pyzk2 (ZKTeco library)
- RPC: JSON-RPC 2.0 server (aiohttp)
- Monitoring: psutil (process info)

**Responsibilities**:
- Hardware abstraction for ZKTeco biometric devices
- Device connection and protocol handling
- Log retrieval from device memory
- Device time synchronization
- Stateless operation (no persistence, no business logic)

**Architecture**:
- Entry point spawned by agent-main via `spawn_child`
- Listens on dynamic TCP port (passed via CLI argument)
- Handles JSON-RPC 2.0 method calls
- Graceful shutdown on SIGINT/SIGTERM

**Standard RPC Methods**:
```
common.ping() → boolean
  Heartbeat check

device.connect(ip, port) → {status, device_info}
  Connect to biometric device

device.get_logs() → [{timestamp, user_id, log_type}]
  Fetch attendance records from device

device.get_time() → {device_time, server_time}
  Sync device time
```

**Read/Write Timeouts**:
- Read: 5 minutes
- Write: 30 seconds

**Implementation** (`zk_worker.py`): 5635 bytes
- Device connection pooling
- Log parsing with error handling
- Protocol-specific command formatting

### 4. Web Dashboard (Frontend)

**Location**: `/home/elt1541/leeattend/webdashboard/src/App.tsx`

**Technology Stack**:
- Language: TypeScript
- Framework: React 19.2.3
- Build: Vite 7.3.1
- Desktop: Tauri 2.9.6
- Styling: Tailwind CSS 4
- Routing: React Router DOM 7.12.0
- State Management: TanStack React Query 5.90.16
- Internationalization: i18next 25.7.4
- Components: Radix UI (headless)
- Icons: Lucide React 0.562.0
- Commands: cmdk 1.1.1 (command palette)

**Pages** (20 total):
- Attendance: AttendanceLogsPage
- Employees: EmployeesPage, ProfilePage
- Departments: DepartmentsPage
- Shifts: ShiftsPage, WorkingPatternsPage
- Leave: LeaveRequestsPage, LeaveBalancesPage
- Payroll: PayrollPage
- Admin: AgentsPage, DevicesPage, AssignmentRulesPage
- Setup: LoginPage, RegisterPage, OnboardingPage
- Reports: ReportsPage
- Dashboard: DashboardPage
- ESS Portal: ess/ subdirectory
- Settings: settings/ subdirectory
- Members: MembersPage (workspace collaboration)

**Directory Structure**:
```
/home/elt1541/leeattend/webdashboard/src/
├── pages/              (20 page components)
├── components/         (UI components by domain)
│   ├── payroll/
│   ├── shifts/
│   ├── leave/
│   ├── time-tracking/
│   └── ...
├── lib/
│   ├── api/            (10+ API client modules)
│   │   ├── agents.ts
│   │   ├── employees.ts
│   │   ├── attendance.ts
│   │   ├── payroll.ts
│   │   └── ...
│   └── utils/          (Helpers)
├── hooks/              (Custom React hooks)
├── context/            (React Context providers)
├── i18n/
│   └── locales/        (Translations)
└── App.tsx             (Root component)
```

**API Communication**:
- Base URL: `http://localhost:${API_PORT}/api`
- Discovery: Reads `../apiserver/.api_port` via Vite proxy
- Authentication: JWT in `Authorization: Bearer` header
- Workspace Context: `X-Workspace-Id` header

**State Management**:
- React Query for server state
- React Context for UI state
- Local storage for preferences

**Testing**:
- Unit tests: Vitest 4.0.16
- E2E tests: Playwright 1.57.0
- Page Object Model (POM) pattern
- Custom fixtures for authentication

## Design Patterns

### 1. Sidecar Architecture

**Implementation Location**: `/home/elt1541/leeattend/agent-main/src/main.rs` (lines 200-345)

**Pattern Description**:
- Master process (Rust) orchestrates sidecar processes (Python)
- Sidecars are stateless, single-purpose drivers
- Communication via JSON-RPC 2.0 over localhost TCP
- Master controls lifecycle: spawn, monitor, kill

**Benefits**:
- Hardware abstraction layer
- Language flexibility (Python for device protocols, Rust for orchestration)
- Process isolation prevents driver crashes from cascading
- Easy to add new device brands

**Communication Flow**:
```
Agent Main → JSON-RPC Method Call → ZK Worker → Device
Agent Main ← JSON-RPC Response ← ZK Worker ← Device
```

### 2. Retry Logic with Exponential Backoff

**Implementation Location**: `/home/elt1541/leeattend/agent-main/src/retry.rs`

**Pattern Description**:
- Configurable max delay (default: 60 seconds)
- Configurable max attempts (default: 10)
- Jitter ±25% to prevent thundering herd
- Stability window (30s) resets retry counter
- Applied to: Sidecar spawn, device polling, WebSocket

**Backoff Formula**:
```
delay = min(2^attempt, MAX_DELAY) ± 25% random
```

**Applied Contexts**:
1. Sidecar process spawn failures
2. Device polling unavailability
3. WebSocket reconnection after network loss

### 3. Multi-Tenant Isolation (Path Priority)

**Implementation Location**: `/home/elt1541/leeattend/apiserver/src/lib.rs`

**Pattern Description**:
- Extract `workspace_id` from URL path (e.g., `/api/workspaces/:ws_id/employees`)
- Fallback to `X-Workspace-Id` header if path not present
- Enforce on all queries: `WHERE workspace_id = $1`

**Benefits**:
- URL-first design prevents accidental tenant leakage
- Header fallback for edge cases
- Compile-time enforcement via extractor traits

**Validation Level**:
- Path parameter binding in route definition
- Repository query filtering
- Schema-level foreign keys (optional cascade)

### 4. Handler-Service-Repository (3-Tier)

**Pattern Description**:
```
Handler (API Layer)
  → Extract params, validate input
  → Call service method
  → Return HTTP response

Service (Business Logic)
  → Orchestrate multiple repositories
  → Apply domain rules
  → Calculate derived values

Repository (Data Access)
  → Trait-based design
  → SQLx async queries
  → Transaction management
```

**Module Organization**:
- Handlers: `/home/elt1541/leeattend/apiserver/src/handlers/` (18 modules)
- Services: `/home/elt1541/leeattend/apiserver/src/services/` (19 modules)
- Repositories: `/home/elt1541/leeattend/apiserver/src/repositories/` (21 modules)

**Benefits**:
- Testability: Each layer can be unit tested independently
- Reusability: Services and repositories used by multiple handlers
- Separation of concerns: Clear responsibility boundaries

### 5. Event-Sourcing Audit Trail

**Implementation Location**: Attendance pairing logic

**Pattern Description**:
- Immutable `raw_attendance_logs` table stores all device events
- Pairing engine converts flat events to `time_entries` (work sessions)
- `generation_metadata` JSONB tracks source log IDs
- Two pairing algorithms: Anchor-First (primary), FSM (explicit intent)

**Workflow**:
```
Device → raw_attendance_logs (immutable)
         ↓
         Pairing Engine
         ↓
         time_entries (sessions)
         ↓
         Reports & Payroll
```

**Benefits**:
- Full audit trail: Original raw data preserved
- Recalculation: Pairing can be re-run with new rules
- Data lineage: Track which raw logs generated which sessions

### 6. 3-Tier Temporal Policy Resolution

**Implementation Location**: Payroll engine

**Pattern Description**:
Resolves policies with highest specificity wins:
1. Individual override: `employee_payroll_configs`
2. Department override: `department_payroll_configs`
3. Workspace global: `payroll_settings`

**Lookup Query**:
```sql
SELECT config FROM employee_payroll_configs
WHERE employee_id = ? AND effective_date <= target_date
ORDER BY effective_date DESC LIMIT 1
UNION ALL
SELECT config FROM department_payroll_configs
WHERE department_id = ? AND effective_date <= target_date
ORDER BY effective_date DESC LIMIT 1
UNION ALL
SELECT config FROM payroll_settings
WHERE workspace_id = ? AND effective_date <= target_date
ORDER BY effective_date DESC LIMIT 1
```

**Benefits**:
- Flexible overrides at multiple levels
- Time-aware (supports policy changes)
- Clear precedence rules

### 7. Batch DB Writer with Backpressure

**Implementation Location**: `/home/elt1541/leeattend/agent-main/src/main.rs` (lines 89-133)

**Pattern Description**:
- Producer (worker threads) send logs to channel
- Consumer (DB writer) buffers 100 records or 200ms timeout
- Single transaction per batch
- Sleep 10ms after flush (yield to sync worker)

**Channel Configuration**:
- Type: `mpsc::channel(10000)`
- Capacity: 10,000 messages
- Producer: Multiple worker tasks
- Consumer: Single database writer

**Benefits**:
- Reduces database lock contention
- Controlled memory usage via channel bounds
- Backpressure: Producers wait when channel full

### 8. Semaphore-Based Fair Scheduling

**Implementation Location**: `/home/elt1541/leeattend/agent-main/src/main.rs` (lines 165, 302-334)

**Pattern Description**:
- `Arc<Semaphore>` with `max_concurrent_workers` permits
- Per-cycle acquire: One permit per polling cycle
- Release after each poll completes
- Sequential iteration with `join_all` ensures FIFO fairness

**Scheduling Guarantees**:
- No starvation: Devices polled in order
- Fair concurrency: Limited simultaneous polls
- Bounded resources: Configurable max workers

**Pseudocode**:
```rust
for device in devices {
    let permit = semaphore.acquire().await;
    spawn_task(async move {
        poll_device(device).await;
        drop(permit);
    });
}
join_all(tasks).await;
```

## Data Flow Diagrams

### Attendance Log Sync Flow

```
Biometric Device
    ↓ (pull every N minutes)
ZK Worker Sidecar (Python)
    ↓ (JSON-RPC response)
Agent Main (Rust)
    ↓ (batch buffer: 100 or 200ms)
SQLite Local Cache (test-agent.db)
    ↓ (WebSocket message: logs batch)
API Server (Rust)
    ↓ (validate & transform)
PostgreSQL raw_attendance_logs (immutable)
    ↓ (pairing engine)
PostgreSQL time_entries (sessions)
    ↓ (payroll calculation)
PostgreSQL payslips (frozen snapshots)
```

### Request Flow (UI → Server)

```
Web Browser / Tauri App
    ↓ (HTTPS/WebSocket)
Vite Dev Server (proxy) / Reverse Proxy
    ↓ (http://localhost:${PORT}/api)
API Server Handler (Axum)
    ↓ (extract WorkspaceId)
Service Layer (business logic)
    ↓ (validate workspace_id)
Repository Layer (SQLx)
    ↓ (parameterized query: WHERE workspace_id = $1)
PostgreSQL Connection Pool
    ↓
PostgreSQL Database
    ↓ (response)
API Server Handler
    ↓ (HTTP 200 + JSON)
Web Browser / Tauri App
```

## Integration Points

### Agent ↔ Server

**Protocol**: WebSocket persistent connection
**Endpoint**: `/ws/agents/:agent_id`
**URL**: `wss://${CENTRAL_SERVER_URL}/ws/agents/:agent_id`

**Outbound Message** (Agent → Server):
```json
{
  "type": "logs",
  "batch": [
    {"device_id": "...", "user_id": "...", "timestamp": "...", ...}
  ]
}
```

**Inbound Message** (Server → Agent):
```json
{
  "type": "ack",
  "ids": [1, 2, 3]
}
```

Or:
```json
{
  "type": "reload_config",
  "config": {...}
}
```

**Retry Logic**:
- Max attempts: 10
- Backoff: Exponential (1s to 60s)
- Stability: 60s without error resets retry counter

### Agent ↔ Sidecar

**Protocol**: JSON-RPC 2.0 over TCP
**Port**: Dynamic (OS-assigned, passed via CLI)
**Timeout**: 5 minutes read, 30 seconds write

**Standard Methods**:
1. `common.ping()` → `true`
2. `device.connect(ip, port)` → `{status, info}`
3. `device.get_logs()` → `[{timestamp, user_id}]`
4. `device.get_time()` → `{device_time, server_time}`

### Dashboard ↔ API

**Protocol**: REST + WebSocket
**Base URL**: `http://localhost:${API_PORT}/api`
**Discovery**: Vite proxy reads `../apiserver/.api_port`
**Auth**: `Authorization: Bearer ${jwt_token}`
**Workspace**: Path param or `X-Workspace-Id` header

## Dependencies & External Services

### Internal Dependencies

**API Server**:
- PostgreSQL 13+ (required)
- Redis (optional, for session caching)

**Edge Agent**:
- SQLite 3.37+ (bundled)
- Python 3.9+ (for sidecars)
- Network access to API server (WebSocket)

**Web Dashboard**:
- Node.js 18+ (build-time)
- API server (runtime)

### External Services

**Biometric Devices**:
- ZKTeco T&A system
- Protocol: Proprietary (implemented via pyzk2)
- Default port: 5005

**PostgreSQL**:
- Version: 13+
- Pool size: Max 10 connections (API server)
- Extensions: None required currently

## Database Schema Highlights

### PostgreSQL Schema (83 migrations)

**Tenant Isolation Tables**:
- `workspaces` - Tenant root
- `workspace_members` - RBAC users per tenant
- `users` - Global user accounts

**Core Business Tables**:
- `devices` - Biometric hardware registry
- `employees` - HR data with relationships
- `departments` - Org structure
- `shifts` - Work schedules
- `raw_attendance_logs` - Immutable device events
- `time_entries` - Paired work sessions

**Temporal Tables**:
- `working_patterns` - Schedule templates (effective_from/to)
- `shift_rosters` - Shift assignments (temporal)
- `employee_payroll_configs` - Override policies (temporal)
- `department_payroll_configs` - Department policies (temporal)

**Workflow Tables**:
- `leave_requests` - Leave application with status
- `leave_balances` - Accrual tracking
- `leave_types` - Policy definitions

**Reporting Tables**:
- `payslips` - Frozen payroll snapshots
- `payslip_segments` - Line items per payslip
- `assignment_rules` - Auto-scheduling logic

### SQLite Schema (6 migrations)

**Local Cache**:
- `attendance_logs` - Punch records (synced to server)
- `devices` - Device config
- `parsing_errors` - Failed log entries
- `agent_config` - Runtime settings

## Security Architecture

### Authentication

- Method: JWT (jsonwebtoken crate)
- Secret: `JWT_SECRET` environment variable
- Signed algorithm: HS256 (default) or RS256
- Expiry: Configurable, typically 24 hours
- Refresh: Supported via token endpoint

### Authorization

- Model: Role-based access control (RBAC)
- Scope: Per-workspace
- Roles: Owner, Admin, Manager, Employee
- Enforcement: `workspace_members` table + claims in JWT

### Multi-Tenancy Enforcement

- Level 1: Path parameter extraction
- Level 2: Route-level authorization checks
- Level 3: Query-level filtering (`WHERE workspace_id = $1`)
- Level 4: Schema foreign keys (optional cascade)

**Test Coverage**: `schema_isolation_tests.rs` validates all tables

### Debug Features (Dev Only)

**Headers** (disabled in production):
- `X-Debug-Impersonate-Id` - Assume user identity
- `X-Debug-Mock-Date` - Time travel for policy testing

**Endpoint**: `/api/debug/*` (dev-only, removed in release builds)

## Performance Considerations

### Database Performance

**PostgreSQL**:
- Connection pool: Max 10 (tune for load)
- Query optimization: SQLx compile-time checking
- Indexes: Device ID, workspace ID, timestamps

**SQLite**:
- WAL mode: Concurrent reads + single writer
- Cache: 64MB in-memory
- Journal: NORMAL (balanced write speed/durability)

### Agent Performance

**Batch Writer**:
- Buffer: 100 records or 200ms
- Impact: Reduces lock contention 10-100x
- Trade-off: 200ms latency acceptable for attendance logs

**Semaphore Scheduling**:
- Concurrency: Configurable (default 3)
- Polling interval: Device-specific
- Impact: Fair resource allocation, prevents device starvation

### API Performance

**Connection Pooling**:
- Pool size: 10 (PostgreSQL default)
- Tune for: Concurrent request count
- Monitoring: Check `pg_stat_activity`

**Caching** (optional):
- Redis for session tokens
- HTTP caching headers for read-only endpoints
- In-memory caching (future consideration)

## Monitoring & Observability

### Logging

**Rust Components**:
- Framework: `tracing` + `tracing-subscriber`
- Initialization: `env_logger` with custom filters
- Levels: TRACE, DEBUG, INFO, WARN, ERROR
- Format: Structured JSON (optional)

**Python Components**:
- Framework: Standard `logging` module
- Initialization: `basicConfig`
- Levels: DEBUG, INFO, WARNING, ERROR

**Log Output**:
- stdout/stderr (captured by container/systemd)
- Files: Configurable destination

### Debug Observability

**API Server** (dev only):
- `/api/debug/health` - Liveness check
- `/api/debug/metrics` - Runtime statistics
- Headers: Impersonation, time travel

**Edge Agent**:
- Log output: Device polling, WebSocket events, errors
- Database state: Query counts, error tracking

## Deployment Architecture

### Single-Machine Deployment

```
┌─────────────────────────────────────┐
│ Server Machine                      │
├─────────────────────────────────────┤
│ API Server (port 3000+)             │
│ PostgreSQL (port 5432)              │
│ Web Dashboard (port 5173 dev)       │
└─────────────────────────────────────┘
         ↕ WebSocket
┌─────────────────────────────────────┐
│ Edge Machine (distributed)          │
├─────────────────────────────────────┤
│ Agent Main (Rust)                   │
│ ├─ ZK Worker 1 (Python)             │
│ ├─ ZK Worker 2 (Python)             │
│ └─ SQLite (test-agent.db)           │
│                                     │
│ Network: TCP 5005-5010 to devices   │
└─────────────────────────────────────┘
```

### High-Availability Deployment (Future)

- Multiple API server instances (load balancer)
- PostgreSQL replication (primary-replica)
- Multiple edge agents (per location/branch)
- Redis for shared session state

## Next Steps for Developers

1. **Understanding Data Flow**: Trace a log from device to payslip
2. **Adding API Endpoint**: Review handler-service-repository pattern
3. **Extending Dashboard**: Create new page following existing patterns
4. **Adding New Device Brand**: Implement new sidecar worker
5. **Database Changes**: Write migration, update SQLx metadata
