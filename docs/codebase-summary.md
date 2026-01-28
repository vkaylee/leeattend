# Codebase Summary

**Last Updated:** 2026-01-27

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
    - **central_client.rs**: WebSocket connection to Central Server with `RetryManager` for connection resilience (30s timeout, 60s stability threshold)
    - **worker_client.rs**: JSON-RPC 2.0 client for sidecar communication (5min read, 30s write timeout)
    - **retry.rs**: Reusable exponential backoff with jitter (2s-60s delays, ±25% jitter) for sidecar retries
    - **database.rs**: SQLite operations for offline log buffering and device configuration
    - **config.rs**: Environment-based configuration loading with defaults
- **Features**:
    - **Sidecar Resilience**: Exponential backoff retry (max 10 retries) with 30s stability threshold
    - **Central Sync**: WebSocket with auto-reconnect (max 10 retries) and ACK-based delivery confirmation
    - **Offline Buffering**: Queues logs locally during network unavailability
    - **Dynamic Reconfiguration**: Hot-reload device config from Central Server via ConfigPush messages
    - **Multi-Brand Support**: Spawns brand-specific sidecars (zkteco, mock) based on database config
    - **Concurrent Workers**: Semaphore-based concurrency control (configurable max workers)
    - **Fair Polling**: Semaphore-based round-robin scheduling ensures no device starvation
    - **Duplicate Detection**: In-memory HashSet filters redundant logs (99.95% queue traffic reduction)
    - **Log Cleanup**: Automatic deletion of synced logs older than 365 days (daily task)
    - **FIFO Guarantee**: ORDER BY id ASC ensures logs sync in exact device order
    - **CLI Commands**: `--help`, `failed-logs [limit]`, `failed [limit]` for log inspection
- **Test Coverage**: 67 tests (39 library + 28 integration) including duplicate detection, memory leak detection

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
- **Key Features**:
    - **Leave Policy Manager**: Centralized management of leave types and policies.
    - **Overtime Policy Manager**: Visual editor for complex overtime rules.
    - **Official Payslip UI**: High-fidelity, print-ready payslip generation using Radix-UI.
    - **ESS Portal**: Self-service for employees to view attendance, payslips, and request leave.
    - **Performance**: Server-side pagination and fuzzy search for large employee datasets (5000+ records).

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

- **Unified Orchestrator**: `./run-tests.sh` handles linting, unit tests, and E2E tests.
- **Auto-Environment**: Automatically provisions isolated Docker Postgres instances for parallel testing.
- **Scope**:
    - **Unit**: Rust (`cargo test`), Frontend (`vitest`).
    - **E2E**: Playwright for web flows.
    - **Integration**: API tests with real database backing.

## 📂 Directory Structure Highlights

```
.
├── agent-main/              # Rust Edge Orchestrator
├── agent-zk-worker/         # ZKTeco Python Sidecar
├── agent-mock-worker/       # Simulation Sidecar (New)
├── apiserver/               # Rust Backend
│   ├── migrations/          # SQLx Migrations
│   ├── src/                 # Source Code
│   └── tests/               # Integration Tests
├── agent-main/              # Rust Edge Orchestrator
├── agent-zk-worker/         # Python Hardware Interface
├── webdashboard/            # React Frontend
│   ├── src/components/      # UI Components
│   ├── src/pages/           # Route Pages
│   └── tests/               # Playwright E2E Tests
├── docs/                    # Documentation
└── plans/                   # Feature Plans & Reports
```