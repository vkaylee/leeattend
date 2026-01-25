# Codebase Summary

**Last Updated:** 2026-01-25

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
- **Core Responsibility**: Lightweight process running at physical locations. It manages a Python sidecar (`agent-zk-worker`), buffers attendance logs locally, and syncs data to the Central Server.
- **Features**:
    - **Resilience**: Offline buffering and automatic retry.
    - **Process Management**: Spawns and monitors the Python worker.
    - **Configuration**: Environment-based config with robust defaults.

### 3. webdashboard (Admin & ESS Portal)
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