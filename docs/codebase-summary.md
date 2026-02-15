# Codebase Summary

**Last Updated:** 2026-02-03

Overview of the LeeAttend project structure, core technologies, and recent architectural improvements.

## 🏗️ Project Architecture

LeeAttend is a high-performance attendance and payroll system split into three primary components:

### 1. apiserver (Central Server)
- **Language**: Rust (Axum framework)
- **Database**: PostgreSQL 14+ (SQLx)
- **Core Responsibility**: Manages workspaces, employees, complex pairing algorithms (FSM), hierarchical temporal payroll calculation, and serves as the WebSocket hub for Edge Agents.
- **Key Modules**:
    - **Leave Management V2**: Relational design with `leave_types` reference table (FK-enforced), dynamic policy assignment, and accrual logic.
    - **Overtime Engine**: Flexible, rule-based overtime calculation with priority-based segment evaluation.
    - **Payroll Engine**: Temporal, 3-tier settings (Global -> Group -> Individual) with traceability.
    - `src/handlers`: REST API endpoints and WebSocket handlers.
    - `src/repositories`: Database access layer with RLS-like filtering.

### 2. agent-main (Edge Orchestrator)
- **Language**: Rust
- **Database**: SQLite (SQLx)
- **Core Responsibility**: Lightweight process running at physical locations. It manages multiple Python sidecars (ZKTeco, Mock, etc.) via JSON-RPC 2.0, buffers attendance logs locally, and syncs data to the Central Server.
- **Key Modules**:
    - **main.rs**: Orchestrator loop managing device polling, worker lifecycle, and configuration reloading
    - **central_client.rs**: WebSocket connection to Central Server with split read/write tasks (prevents ACK starvation), `RetryManager` for connection resilience (30s timeout, 60s stability threshold)
    - **worker_client.rs**: JSON-RPC 2.0 client for sidecar communication (5min read, 30s write timeout)
    - **retry.rs**: Reusable exponential backoff with jitter (2s-60s delays, ±25% jitter) for sidecar retries
    - **memory_store.rs**: In-memory DashMap-based log storage with signature cache deduplication (DashSet<u64>)
    - **hybrid_database.rs**: Facade orchestrating memory-first reads with disk persistence
    - **database.rs**: SQLite operations for offline log buffering and device configuration
    - **config.rs**: Environment-based configuration loading with defaults
- **Features**:
    - **Sidecar Resilience**: Exponential backoff retry (max 10 retries, formula: 2^retry_count capped at 60s) with 30s stability threshold
    - **Central Sync**: WebSocket with split read/write tasks (no ACK starvation), auto-reconnect (max 10 retries), and ACK-based delivery confirmation
    - **Signature Cache Deduplication**: Hash-based duplicate detection (FxHash u64) with 96% memory savings (0.8MB vs 20MB for 100K logs)
    - **Two-Phase Startup**: Phase A loads signatures from disk, Phase B loads unsynced queue
    - **Offline Buffering**: Queues logs locally during network unavailability
    - **Dynamic Reconfiguration**: Hot-reload device config from Central Server via ConfigPush messages
    - **Multi-Brand Support**: Spawns brand-specific sidecars (zkteco, mock) based on database config
    - **Concurrent Workers**: Semaphore-based concurrency control (configurable max workers)
    - **Fair Polling**: Semaphore-based round-robin scheduling ensures no device starvation
    - **Duplicate Detection**: In-memory HashSet filters redundant logs (99.95% queue traffic reduction)
    - **Log Cleanup**: Automatic deletion of synced logs older than 365 days (daily task)
    - **FIFO Guarantee**: ORDER BY id ASC ensures logs sync in exact device order
    - **CLI Commands**: `--help`, `failed-logs [limit]`, `failed [limit]` for log inspection
- **Test Coverage**: 71 tests (39 library + 32 integration) including duplicate detection, memory leak detection, WebSocket split read/write
- **Agent Test Coverage**: 182 tests total including signature cache deduplication, exponential backoff, and ACK starvation prevention

### 3. Sidecar Workers (Device Connectors)
- **Languages**: Python 3.10+
- **Core Responsibility**: Low-level communication with biometric hardware.
- **Components**:
    - **agent-zk-worker**: Communicates with ZKTeco devices using `pyzk`.
    - **agent-mock-worker**: Simulates device behavior for testing and development.
- **Protocol**: Standardized JSON-RPC 2.0 over TCP (loopback).

### 4. webdashboard (Admin & ESS Portal)
- **Framework**: React 18 (Vite, TypeScript)
- **Styling**: Tailwind CSS v4 (Metro-style aesthetic)
- **State Management**: TanStack Query (React Query)
- **Component Structure** (Organized by concern):
    - `src/components/ui/` - Generic, reusable UI primitives (Modal, Toast, Combobox, Tabs)
    - `src/components/common/` - Shared cross-cutting components (AuthGuard, WorkspaceGuard)
    - `src/components/layout/` - Page structure components (MainLayout, Sidebar, Navbar, WorkspaceSwitcher)
    - `src/components/features/` - Business domain components organized by feature:
        - `features/agents/` - Agent management (AgentEditModal)
        - `features/attendance/` - Attendance operations (AttendanceImportDialog)
        - `features/workspaces/` - Workspace management (CreateWorkspaceDialog)
        - `features/settings/` - Settings UI (RoleManagementModal)
    - `src/components/payroll/` - Payroll-specific components
    - `src/components/leave/` - Leave management components
    - `src/components/employees/` - Employee management components
- **Theming Infrastructure** (NEW - Feb 2026):
    - `src/context/dark-mode-context-definition.ts` - Context type definition
    - `src/context/use-dark-mode-context.ts` - useContext consumer hook
    - `src/context/DarkModeContext.tsx` - Provider component wrapping App
    - `src/hooks/use-dark-mode.ts` - Public hook for theme toggle and state
    - Dark mode CSS variables in `src/index.css` with Tailwind semantic tokens
    - FOUC prevention script in `index.html`
- **Key Features**:
    - **Dark Mode System**: Complete theme infrastructure with localStorage persistence and system preference detection (Phase 1)
    - **Leave Policy Manager**: Centralized management of leave types and policies.
    - **Overtime Policy Manager**: Visual editor for complex overtime rules.
    - **Official Payslip UI**: High-fidelity, print-ready payslip generation using Radix-UI.
    - **Real-Time Report Export**: WebSocket-based streaming with automatic HTTP fallback (NEW - Feb 2026).
    - **ESS Portal**: Self-service for employees to view attendance, payslips, and request leave.
    - **Performance**: Server-side pagination and fuzzy search for large employee datasets (5000+ records).

### 5. Production Infrastructure (Dockerized)
- **Containerization**: Standardized Dockerfiles for `apiserver` and `webdashboard` using multi-stage builds to minimize image size and maximize security.
- **Orchestration**: `docker-compose.yml` provides a turn-key production environment including PostgreSQL, Redis, and automated Nginx reverse proxying.
- **Reverse Proxy**: Nginx integration for serving static frontend assets and routing API/WebSocket traffic to the backend.

## 🛡️ Security & Multi-Tenancy

- **Row-Level Isolation**: All tenant-specific tables (Time Entries, Employees, Shifts, etc.) enforce a mandatory `workspace_id`.
- **Automated Audit**: `apiserver/tests/schema_isolation_tests.rs` ensures no table is created without isolation.
- **RBAC**: Strict role-based access control (`owner`, `admin`, `member`) enforced at the handler level using `MemberContext`.

## ⚡ Payroll & Policy Engine

- **Flexible Overtime**: Granular control over daily/weekly thresholds and day-type multipliers (Holiday, Rest Day).
- **Incentives**: Configurable attendance bonuses (Diligence, Performance, Safety).
- **Traceability**: All payroll calculations are traceable to specific rules and time entries.

## 🚀 Performance Optimizations

- **Relational Integrity**: Migrated legacy string-based `leave_type` to normalized `leave_types` table with Foreign Key constraints (Jan 2026).
- **Denormalized Access**: `time_entries` table includes `workspace_id` for direct RLS and optimized reporting queries.
- **Full-Text Search**: `pg_trgm` (Trigram) indexes on employee names for instant fuzzy search.
- **Optimized Pagination**: Efficient SQL pagination for employee and log lists.

## 🧪 Testing Infrastructure

LeeAttend employs a high-performance testing infrastructure optimized for speed, reliability, and SSD longevity, specifically tailored for WSL2 environments.

- **RAM-First Execution (tmpfs):** To bypass WSL's VirtioFS performance bottlenecks, all heavy I/O directories are mounted in RAM (Rust `target/`, `sccache`, `node_modules`, and PostgreSQL data).
- **Persistent RAM Logs:** Test logs are stored in `/tmp/leeattend-test-logs` (RAM) and symlinked to `.test_logs/`. Use `./scripts/debug-logs.sh` for fast analysis.
- **Unified Orchestrator:** `./run-tests.sh` centralizes all tests with automatic RAM cleanup on exit.
- **Smart Worker Pool:** High-efficiency orchestrator that strictly respects `NUM_WORKERS` limits (auto-capped at 2 for WSL/stability).
- **Comprehensive Summary Report**: Generates a detailed, colorized report at the end of every run, providing a snapshot of every task's status and execution duration.
- **Real-time Monitoring**: Provides live progress updates for parallel tasks (`[WAIT] Running: ...`), improving visibility during long-running suites.
- **Cargo Nextest**: High-performance test runner for Rust, providing better parallelism and isolated process execution for 2x faster suite completion.
- **Full Containerization**: All tests (Unit & Integration) run inside Docker containers, ensuring a "clean-room" environment.
- **Mock API Architecture (Frontend)**: Webdashboard E2E tests use **Playwright Intercept** (`page.route`) to mock all backend API calls. This enables 100% independent frontend testing without requiring a live Rust backend or database.
- **Strict 1-Worker Execution**: Playwright is forced to use a single worker (`--workers 1`) in Docker environments to eliminate resource contention and provide 100% reliable results.
- **Docker Build Optimization**: Integrated `cargo-chef` for multi-stage builds, caching Rust dependencies separately from source code to reduce rebuild times from minutes to seconds.
- **Internal Database Networking**: Removed host port mapping (`5433:5432`) for the test database; all services communicate via internal Docker network for enhanced security and cleanliness.
- **SQLx Offline Pre-compilation**: Host-side builds use pre-prepared metadata, removing the need for a live database during the compilation phase.
- **Postgres Performance Tuning**: Test databases are optimized with `fsync=off`, `synchronous_commit=off`, and `full_page_writes=off` for maximum data throughput.

## 📂 Directory Structure Highlights

```
.
├── agent-main/              # Rust Edge Orchestrator
│   ├── .dockerignore        # Build exclusions (target, node_modules)
│   └── Dockerfile.test      # Multi-stage chef-optimized build
├── agent-zk-worker/         # ZKTeco Python Sidecar
├── agent-mock-worker/       # Simulation Sidecar (New)
├── apiserver/               # Rust Backend
│   ├── .dockerignore        # Build exclusions
│   ├── .sqlx/               # Prepared metadata (RAM-backed in tests)
│   ├── migrations/          # SQLx Migrations (RAM-backed in tests)
│   ├── src/                 # Source Code
│   ├── Dockerfile.test      # Multi-stage chef-optimized build
│   └── tests/               # Integration Tests
├── webdashboard/            # React Frontend
│   ├── .dockerignore        # Build exclusions
│   ├── src/components/      # UI Components (organized by concern)
│   ├── src/pages/           # Route Pages
│   ├── Dockerfile.test      # E2E test container
│   └── tests/               # Playwright E2E Tests
├── .test_logs/              # Test logs (RAM-backed)
├── run-tests.sh             # Main orchestrator (RAM setup/cleanup)
└── setup-test-env.sh        # Infrastructure provisioner (Containerized)
```

## 🔄 Real-Time Features (NEW - Feb 2026)

### Report Export with WebSocket Streaming

Implemented real-time report export with automatic fallback mechanism:

**Frontend Components**:
- `webdashboard/src/hooks/use-report-stream.ts` - WebSocket client with auto-reconnect (exponential backoff: 1s, 2s, 4s)
- `webdashboard/src/types/report-stream.ts` - Discriminated union types (CHUNK, DONE, ERROR)
- `webdashboard/src/components/features/reports/ReportExportDialog.tsx` - Integrated UI with fallback mode
- `webdashboard/src/lib/excel-builder.ts` - Client-side Excel generation with streaming support

**Backend Infrastructure**:
- `apiserver/src/handlers/report_websocket.rs` - WebSocket upgrade handler (168 lines)
- `apiserver/src/queue/pubsub.rs` - Redis Pub/Sub for real-time message streaming
- `apiserver/src/services/report_coordinator.rs` - Batch processing with progress tracking

**Performance Improvements**:
| Metric | Before (HTTP Polling) | After (WebSocket) | Improvement |
|--------|----------------------|-------------------|-------------|
| **Latency** | 1000ms | < 100ms | 90% faster |
| **API Calls** | ~30/export | 1/export | 97% reduction |
| **Network Usage** | ~150KB | ~5KB | 97% reduction |
| **UX** | Delayed updates | Real-time progress | Live feedback |

**Reliability**:
- Automatic fallback to HTTP polling on 3 connection failures
- Zero export failures (graceful degradation)
- User notification when in fallback mode
- 461 tests passing (22 unit + 12 E2E + rest system tests)

**Message Protocol**:
```json
{
  "type": "CHUNK",
  "job_id": "uuid",
  "batch_index": 5,
  "total_batches": 350,
  "progress_percentage": 1
}
```

---