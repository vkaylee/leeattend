# LeeAttend Project Overview

## Project Purpose

LeeAttend is a comprehensive, enterprise-grade attendance management system designed for multi-tenant SaaS deployment. It provides biometric device integration, real-time attendance tracking, leave management, payroll integration, and employee self-service capabilities.

## Core Objectives

- Automate attendance tracking via biometric devices (ZKTeco)
- Real-time data synchronization between edge agents and central server
- Multi-tenant workspace isolation with role-based access control
- Payroll integration with flexible policy configurations
- Leave management with approval workflows
- Employee self-service portal for time-off requests
- Comprehensive reporting and audit trails

## Technology Stack Overview

### Backend Components

**API Server** (Rust + Axum)
- Language: Rust Edition 2021
- Framework: Axum 0.7 (Tokio async runtime)
- Database: PostgreSQL via SQLx 0.8.6
- Authentication: JWT (jsonwebtoken)
- Password Security: bcrypt hashing
- File: `/home/elt1541/leeattend/apiserver/src/main.rs`

**Edge Agent - Main** (Rust + SQLite)
- Language: Rust Edition 2024
- Database: SQLite via SQLx 0.8
- Concurrency: Tokio async runtime with semaphore-based scheduling
- File: `/home/elt1541/leeattend/agent-main/src/main.rs`

**Edge Agent - Worker** (Python)
- Language: Python 3
- Device Protocol: pyzk2 (ZKTeco device communication)
- RPC: JSON-RPC 2.0 server
- File: `/home/elt1541/leeattend/agent-zk-worker/main.py`

**Web Dashboard** (TypeScript + React)
- Language: TypeScript with React 19.2.3
- Build: Vite 7.3.1
- Desktop: Tauri 2.9.6 wrapper
- Styling: Tailwind CSS 4
- Routing: React Router DOM 7.12.0
- File: `/home/elt1541/leeattend/webdashboard/src/App.tsx`

## Key Features

### Attendance Management
- Real-time biometric device integration (ZKTeco brand)
- Automatic log synchronization via WebSocket
- Attendance pairing engine (converts raw logs to work sessions)
- Multi-device support with device isolation
- Device health monitoring and error tracking

### Edge Agent Architecture
- Sidecar pattern with Python workers for hardware abstraction
- Fair round-robin device polling with exponential backoff
- Local SQLite persistence with 365-day retention cleanup
- Batch database writer (100 records or 200ms flush)
- FIFO ordering guarantees for log consistency
- Graceful shutdown handling (SIGINT/SIGTERM)

### Multi-Tenancy
- Workspace-level isolation
- Role-based access control (workspace members)
- Tenant data enforcement on all database queries
- Separate workspace configurations

### Employee Management
- Employee profile management
- Department-based organization
- Shift assignments and rosters
- Leave type and balance management
- Payroll configuration (employee, department, workspace levels)

### Leave Management
- Leave request workflow with approval process
- Multiple leave types (PTO, sick, etc.)
- Leave balance tracking
- Temporal policies for accrual rules
- Employee self-service request portal

### Payroll Integration
- Multi-tier policy resolution (employee > department > workspace)
- Payroll segment calculations
- Payslip generation and history
- Overtime management with rate overrides
- Temporal effective dating for policy changes

### Reporting
- Attendance reports
- Leave reports
- Payroll reports
- Custom report builder
- Data export capabilities

## Architecture Summary

### Three-Tier Design Pattern

```
Frontend (React)
    ↓
API Server (Axum + PostgreSQL)
    ↕ WebSocket
Edge Agent (Rust + SQLite)
    ↕ JSON-RPC 2.0
Sidecar Worker (Python)
    ↓
Biometric Device
```

### 83 Database Migrations

The API server manages schema evolution through 83 SQL migrations covering:
- Multi-tenant workspace structure
- User authentication and authorization
- Employee and department management
- Device registry and configuration
- Attendance logging and pairing
- Shift management and rosters
- Leave request workflows
- Payroll calculations and payslips
- Working pattern templates

### 6 Edge Agent Migrations

SQLite schema for local persistence:
- Attendance logs (local cache)
- Device configuration
- Parsing error tracking
- Agent configuration (key-value store)

## Database Architecture

### PostgreSQL (Central Server)
**Size**: 83 migrations
**Primary Tables**: workspaces, users, workspace_members, devices, employees, departments, shifts, raw_attendance_logs, time_entries, payslips, leave_requests, working_patterns, assignment_rules

**Key Features**:
- Multi-tenancy with `workspace_id` enforcement
- Temporal data with `effective_from`/`effective_to`
- Immutable raw logs with generated time entries
- Audit trail with `created_at`/`updated_at`

### SQLite (Edge Agent)
**Size**: 6 migrations
**Primary Tables**: attendance_logs, devices, parsing_errors, agent_config

**Configuration**:
- WAL mode for concurrent reads
- 64MB in-memory cache
- NORMAL synchronous setting

## Getting Started

### Prerequisites

- Rust 1.70+ (for apiserver and agent-main)
- Python 3.9+ (for agent-zk-worker)
- Node.js 18+ (for webdashboard)
- PostgreSQL 13+ (central server)
- Docker (for test environment)

### Quick Start

1. Clone the repository
2. Set environment variables (see `.env.example` files)
3. Run test setup: `./setup-test-env.sh`
4. Run tests: `./run-tests.sh --all`
5. Build components:
   - API: `cd apiserver && cargo build --release`
   - Agent: `cd agent-main && cargo build --release`
   - Dashboard: `cd webdashboard && npm install && npm run build`

### Configuration

**API Server** (`/home/elt1541/leeattend/apiserver/.env`):
- `DATABASE_URL` - PostgreSQL connection string
- `JWT_SECRET` - Secret key for JWT signing
- Port: Dynamically assigned (3000+), written to `.api_port`

**Edge Agent** (`/home/elt1541/leeattend/agent-main/.env`):
- `CENTRAL_SERVER_URL` - WebSocket endpoint
- `AGENT_ID` - Unique agent identifier
- `RUST_LOG` - Logging level

**ZK Worker** (`/home/elt1541/leeattend/agent-zk-worker/.env`):
- `LOG_LEVEL` - Logging verbosity
- Port: Dynamic (passed via command-line argument)

## Test Infrastructure

### Test Suite: 817 Files

Coverage across:
- Rust unit tests (`*_test.rs`)
- Rust integration tests (`/tests/` directories)
- TypeScript unit tests (`*.test.ts`, `*.test.tsx`)
- E2E tests (`*.spec.ts` with Playwright)
- Python tests (`test_*.py` with pytest)

### Test Orchestrator

Run: `/home/elt1541/leeattend/run-tests.sh`

Modes:
- `--all` - Full test suite
- `--api` - Backend only
- `--web` - Frontend only

Phases:
1. Linting (cargo clippy, eslint, type-check)
2. Unit tests
3. Integration/E2E (Docker-based Postgres)

## API Base URL

`http://localhost:${DYNAMIC_PORT}/api`

Port discovery via:
- Frontend: Reads `../apiserver/.api_port`
- Direct calls: Use `http://localhost:3000/api` (default)

## Integration Points

### Agent to Server
- Protocol: WebSocket (persistent connection)
- Endpoint: `/ws/agents/:agent_id`
- Messages: Log batches (type: "logs"), acknowledgments (type: "ack")
- Retry: 10 attempts with exponential backoff (1s to 60s)

### Agent to Sidecar Worker
- Protocol: JSON-RPC 2.0 over TCP
- Port: Dynamic (OS-assigned)
- Methods: `device.connect`, `device.get_logs`, `common.ping`
- Timeout: 5 minutes read, 30 seconds write

### Dashboard to API
- Protocol: REST + WebSocket
- Authentication: JWT bearer token
- Workspace: Path parameter or `X-Workspace-Id` header
- Discovery: File-based (`.api_port`)

## Security Features

- Multi-tenancy enforcement at query level
- JWT authentication with configurable expiry
- Password hashing with bcrypt
- Debug features (impersonation, time travel) disabled in production
- Structured logging for audit trails
- Workspace isolation via automatic filtering

## File Structure Quick Reference

**API Server**: `/home/elt1541/leeattend/apiserver/`
- Entry: `src/main.rs`
- Handlers (18): `src/handlers/`
- Services (19): `src/services/`
- Repositories (21): `src/repositories/`
- Models: `src/models.rs`
- Migrations: `migrations/` (83 files)

**Edge Agent Main**: `/home/elt1541/leeattend/agent-main/`
- Entry: `src/main.rs`
- Core modules: central_client.rs, worker_client.rs, database.rs, retry.rs
- Migrations: `migrations/` (6 files)

**Edge Worker (ZK)**: `/home/elt1541/leeattend/agent-zk-worker/`
- Entry: `main.py`
- Implementation: `zk_worker.py`

**Web Dashboard**: `/home/elt1541/leeattend/webdashboard/`
- Entry: `src/App.tsx`
- Pages (20): `src/pages/`
- Components: `src/components/`
- API clients (10+): `src/lib/api/`
- Hooks: `src/hooks/`
- Translations: `src/i18n/locales/`

## Build & Deployment

### API Server
- Build: `cargo build --release`
- Migrations: Auto-applied on startup
- Signals: Graceful shutdown on SIGINT/SIGTERM
- Port: Dynamically bound (3000+)

### Edge Agent
- Build: `cargo build --release`
- Database: SQLite (auto-created if missing)
- Workers: Python sidecars spawned dynamically
- Cleanup: 365-day retention task every 24 hours

### Web Dashboard
- Build: `npm run build` (Vite)
- Output: `/home/elt1541/leeattend/webdashboard/dist/`
- Modes: SPA, Tauri desktop, or Rust server wrapper

## Documentation Resources

- **System Architecture**: `/home/elt1541/leeattend/docs/system-architecture.md`
- **Code Standards**: `/home/elt1541/leeattend/docs/code-standards.md`
- **API Reference**: `/home/elt1541/leeattend/docs/api-reference.md`
- **Deployment Guide**: `/home/elt1541/leeattend/docs/deployment-guide.md`
- **UI/UX Standards**: `/home/elt1541/leeattend/docs/ui-ux-standards.md`

## Next Steps

1. Review system architecture for deep component understanding
2. Check code standards for contribution guidelines
3. Consult API reference for endpoint specifications
4. Follow deployment guide for production setup
5. Review test infrastructure for quality assurance practices
