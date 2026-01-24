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
    - **Official Payslip Document UI**: Professional, high-fidelity A4-ready preview modal using Radix-UI and Tailwind `print:` modifiers for precise PDF and physical output generation.
    - **Rendering Stability**: Implements robust defensive coding and explicit numeric parsing for high-reliability financial data visualization.


## 🛡️ Multi-Tenancy & Data Isolation

The system enforces strict data isolation between workspaces using a Shared-Database, Row-Level Isolation pattern:

- **Mandatory Workspace ID**: All tenant-specific tables (Time Entries, Employees, Shifts, etc.) are required to have a `workspace_id` column.
- **Automated Compliance Testing**: The `apiserver/tests/schema_isolation_tests.rs` test suite automatically scans the PostgreSQL schema and fails if any new base table (excluding global whitelisted ones like `users` or `workspaces`) is created without a `workspace_id`.
- **Constraint Enforcement**: `workspace_id` columns are configured with `NOT NULL` and `FOREIGN KEY` constraints referencing the `workspaces` table with `ON DELETE CASCADE`.

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

## 💰 Payroll System Architecture

### Multi-Layered Design
The payroll engine is built on three complementary layers working in tandem:

1. **Temporal Configuration Layer**:
   - Hierarchical 3-tier policy resolution: Individual > Department > Workspace
   - Point-in-time historical lookups preserve calculation accuracy
   - Supports mid-period organizational changes (salary increases, transfers)

2. **Segmentation & Proration Engine**:
   - Automatically detects change points within pay periods
   - Slices months into independent segments (e.g., 01-15 old salary, 16-31 new salary)
   - Each segment maintains immutable snapshots in `payslip_segments`
   - Ensures exact prorating when configuration changes occur mid-month

3. **Granular Component Model** (New):
   - **Component Definitions**: Workspace-level catalog of salary types (Basic, Housing Allowance, Meal Allowance, etc.)
   - **Component Assignment**: Admins assign components to employees with effective dates for retroactive corrections
   - **Flexible Calculation**: Components flagged with `is_base_for_ot` participate in overtime rate derivation
   - **Payslip Detail**: Each payslip includes detailed `salary_components` breakdown showing prorated amounts per component
   - **Backward Compatible**: Defaults to legacy segment-based display when components are absent

### Key Features
- **Incentive Tiers**: Complex metric-driven bonuses (Diligence Bonus) evaluated against leave/late metrics
- **Immutable Audit Trail**: All calculation snapshots frozen in database (future policy changes don't alter historical payslips)
- **Batch Processing**: Parallel payroll calculation for multiple employees
- **Debug Context**: Time-travel debugging via `X-Debug-Mock-Date` header for testing historical scenarios

## 🌐 Localization (i18n)

- **Parity**: Verified key parity across English (`en`), Vietnamese (`vi`), and Chinese (`zh`).
- **Verification**: Automated tests ensure no hardcoded strings exist in JSX and that all keys are synchronized across locale files.
- **Automated Synchronization**: Continuous testing validates that all supported languages (English, Vietnamese, Chinese) remain in perfect sync, preventing runtime translation errors.

## 🚀 Recent Major Improvements (Jan 2026)

### Search-as-you-type Performance Optimization
- **Backend Search Limit**: Enforced a `LIMIT 50` on employee searches at the repository level (`employee_repository.rs`) to prevent client-side lag and database overhead in large workspaces (3000+ employees).
- **Asynchronous UI Patterns**: Refactored major lists (Employees Page, Overtime Assignment) to use a debounced server-side search pattern via the reusable `EmployeeSearch` and `Combobox` components.
- **Test Reliability Standards**: Adopted `data-testid` based locators as the primary standard for all new and refactored UI components to ensure E2E test resilience against i18n changes.

## ⚡ Payroll & Policy Engine

The payroll system has been upgraded with a **Flexible Overtime Policy Engine**:

- **Granular Control**: Define overtime rules based on:
  - **Daily Thresholds**: E.g., >8 hours triggers x1.5 rate.
  - **Weekly Thresholds**: E.g., >40 hours triggers x2.0 rate.
  - **Day Types**: Specific multipliers for Rest Days and Holidays.
- **Priority Logic**: Rules are evaluated in a specific order, allowing complex tiered compensation structures.
- **Visual Management**: A new **Overtime Policy Manager** in the web dashboard allows HR administrators to create, edit, and assign policies visually.

## 🚀 Performance Optimizations

- **Denormalized Access**: The `time_entries` table now includes a direct `workspace_id` column (migrated via `20260124000000_add_workspace_id_to_time_entries.sql`). This removes the need for costly joins with the `employees` table during high-volume workspace-level reporting and simplifies Row-Level Security (RLS) policies.
- **Optimized Pagination**: Employee lists now support server-side search and pagination with optimized SQL queries to handle large workforces efficiently.
