# Codebase Summary

Overview of the LeeAttend project structure, core technologies, and recent major architectural improvements.

## 🏗️ Project Architecture

LeeAttend is a high-performance attendance and payroll system split into three primary components:

1.  **apiserver (Central Server)**
    - **Language**: Rust (Axum framework)
    - **Database**: PostgreSQL (SQLx)
    - **Core Responsibility**: Manages workspaces, employees, complex pairing algorithms (FSM), hierarchical temporal payroll calculation, and serves as the WebSocket hub for Edge Agents.
2.  **agent-main (Edge Orchestrator)**
    - **Language**: Rust
    - **Database**: SQLite (SQLx)
    - **Core Responsibility**: Lightweight process running at physical locations. It manages a Python sidecar (`agent-zk-worker`), buffers attendance logs in a local SQLite DB, and syncs data to the Central Server via a persistent WebSocket connection.
    - **Robust Config**: Features a centralized configuration system with smart fallbacks, environment template support, and configurable RPC timeouts.
    - **Graceful Termination**: Implements `CancellationToken` (Rust) and `asyncio.Event` (Python) to ensure clean exits, sidecar cleanup, and hardware connection closing on termination signals.
3.  **webdashboard (Admin & ESS Portal)**
    - **Framework**: React 18 (Vite, TypeScript)
    - **Styling**: Tailwind CSS v4 (Metro-style aesthetic)
    - **Key Features**: Page Object Model (POM) based E2E testing, hierarchical priority legends for schedule and payroll decision-making, and full Employee Self-Service (ESS) portal.

## 🧪 Testing Infrastructure

The project uses a **Unified Test Orchestrator (`run-tests.sh`)** to provide a consistent development experience across all components.

- **Selective Testing**: Supports `--api`, `--web`, and `--all` flags.
- **Pattern Matching**: Can run specific test files or cases (e.g., `./run-tests.sh --api auth`).
- **Auto-Environment**: Automatically handles Docker-based Postgres setup, migration runs, and isolated database provisioning for parallel workers.
- **Lint Enforcement**: Phase 0 (Linting) is mandatory for every run to maintain code quality.

## 📅 Attendance & Pairing Engine

- **Unified Ingestion**: The `AttendanceLogService` serves as the single source of truth for processing logs from both real-time Edge Agents and manual XLSX/CSV imports.
- **Timezone-Aware Parsing**: Naive timestamps from physical devices are automatically converted to UTC based on workspace-specific configuration (Timezone, Date/Time formats).
- **Robust Import**: Supports "Partial Success" during bulk uploads. Duplicate logs and invalid employee IDs are automatically skipped (`Silent Skip`) while valid records are committed.
- **Data Integrity**: Enforced by unique constraints on `(workspace_id, employee_id, punch_time)`.
- **Traceability**: All pairing decisions (via FSM or Anchor-First algorithms) are documented in `generation_metadata` for auditing.

- **Schedule Traceability Verification**: Dedicated integration tests (`schedule_traceability_tests.rs`) verify that every schedule assignment (Manual, Rule-based, or Default) accurately records its origin and reasoning metadata.
## 🌐 Localization (i18n)

- **Parity**: Verified key parity across English (`en`), Vietnamese (`vi`), and Chinese (`zh`).
- **Verification**: Automated tests ensure no hardcoded strings exist in JSX and that all keys are synchronized across locale files.
- **Automated Synchronization**: Continuous testing validates that all supported languages (English, Vietnamese, Chinese) remain in perfect sync, preventing runtime translation errors.
