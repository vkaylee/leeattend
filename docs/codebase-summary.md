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
    - **Robust Config**: Features a centralized configuration system with smart fallbacks and environment template support.
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

- **Robust Import**: Supports "Partial Success" during bulk XLSX/CSV uploads. Duplicate logs and invalid employee IDs are automatically skipped (`Silent Skip`) while valid records are committed.
- **Data Integrity**: Enforced by unique constraints on `(workspace_id, employee_id, punch_time)`.
- **Traceability**: All pairing decisions (via FSM or Anchor-First algorithms) are documented in `generation_metadata` for auditing.

## 🌐 Localization (i18n)

- **Parity**: 100% key parity across English (`en`), Vietnamese (`vi`), and Chinese (`zh`).
- **Verification**: Automated tests ensure no hardcoded strings exist in JSX and that all keys are synchronized across locale files.