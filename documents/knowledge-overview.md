# LeeAttend — Knowledge Overview

## Purpose

LeeAttend is a comprehensive **Attendance Management System** designed for enterprises using ZKTeco biometric devices. It provides end-to-end workforce management covering attendance tracking, shift scheduling, leave management, payroll processing, holiday policies, and overtime calculations.

## Goals

- **Automated Attendance**: Seamlessly collect punch-in/punch-out data from ZKTeco fingerprint/face recognition devices
- **Accurate Payroll**: Calculate salary, overtime pay, and incentive bonuses with configurable work-unit granularity
- **Holiday-Aware Processing**: Full holiday calendar integration affecting OT multipliers, status reporting, and incentive calculations
- **Self-Service Portal**: Employee Self-Service (ESS) for viewing attendance, submitting leave requests, and managing profiles
- **Multi-Workspace**: SaaS-ready architecture supporting multiple workspaces with isolated data
- **Desktop App**: Tauri-based desktop application for local deployment scenarios

## Tech Stack

### Backend — `apiserver/`
| Technology | Version | Purpose |
|---|---|---|
| Rust | 2021 edition | Systems language for performance and safety |
| Axum | 0.7 | Async web framework with WebSocket support |
| SQLx | 0.8 | Compile-time verified SQL queries with PostgreSQL |
| Tokio | 1.36 | Async runtime |
| PostgreSQL | 15+ | Primary relational database |
| Redis | — | Distributed job processing, presence, caching |
| chrono / chrono-tz | 0.4 / 0.9 | Timezone-aware datetime handling |
| rust_xlsxwriter | 0.79 | Excel report generation |
| Moka | 0.12 | In-memory cache (async) |
| utoipa | 5.0 | OpenAPI/Swagger documentation |

### Agent — `agent-main/`
| Technology | Purpose |
|---|---|
| Rust | Device communication orchestrator |
| rustzk | Native ZKTeco device protocol (TCP/UDP) |
| SQLite | Local agent database |

### Frontend — `webdashboard/`
| Technology | Version | Purpose |
|---|---|---|
| React | 19.2 | UI framework |
| Vite | — | Build tool and dev server |
| TailwindCSS | 4 | Utility-first CSS |
| React Router | 7.12 | Client-side routing |
| TanStack React Query | 5.90 | Server state management |
| i18next | 25.7 | Internationalization (en/vi) |
| Lucide React | 0.562 | Icon library |
| Radix UI | — | Accessible primitives (dialog, dropdown, popover) |
| Tauri | 2.x | Desktop app packaging |
| Vitest + Playwright | — | Unit + E2E testing |

### Infrastructure
| Technology | Purpose |
|---|---|
| Docker Compose | Dev, test, CI environments |
| Nginx | Reverse proxy (dev) |
| GitHub Actions | CI/CD pipeline |

## Key Features

1. **Attendance Tracking** — Biometric device sync, raw log ingestion, punch pairing algorithms
2. **Shift Management** — Configurable shifts, rosters, working patterns, break policies
3. **Leave Management** — Leave types, policies, accrual rules, seniority bonuses, balance tracking
4. **Holiday Calendar** — National/company holidays, OT multipliers, holiday-aware reporting
5. **Payroll Processing** — Salary components, OT pay, incentive tiers, payslip generation
6. **Reports & Export** — Daily/summary reports, Excel export with holiday highlighting
7. **Assignment Rules** — Department-based and rule-based shift assignment
8. **OT Claims** — Employee overtime claim workflow with manager approval
9. **Employee Self-Service** — Profile, attendance history, leave requests, clock-in
10. **Admin Dashboard** — Real-time monitoring, worker pool status, agent management

## Getting Started

### Prerequisites
- Docker & Docker Compose
- Rust toolchain (for host-side compilation)
- Node.js 20+ (for frontend development)

### Development
```bash
# Start entire environment with hot-reloading
./run-tests.sh --docker=dev up -d

# View API server logs
./run-tests.sh --docker=dev logs -f apiserver
```

### Testing
```bash
./run-tests.sh --all             # Full test suite
./run-tests.sh --api [pattern]   # Backend tests (Rust)
./run-tests.sh --web [pattern]   # Frontend + E2E tests
```

### Environment Variables
Configured via `.env` file (see `.env.example`). Key variables:
- `API_DATABASE_URL` — PostgreSQL connection string
- `JWT_SECRET` — Authentication token signing key
- `REDIS_URL` — Redis connection for job processing
