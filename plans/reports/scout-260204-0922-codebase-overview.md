# Scout Report — Codebase Overview

**Date:** 2026-02-04 09:22 | **Branch:** main

---

## Repo Root

```
leeattend/
├── agent-main/          # Edge agent (Rust, SQLite, WebSocket)
├── agent-mock-worker/   # Mock worker for testing
├── agent-zk-worker/     # Zero-knowledge worker
├── apiserver/           # Central API server (Rust, Postgres, Redis)
├── webdashboard/        # Frontend SPA (React 19, Vite, TailwindCSS 4)
├── docs/                # 10 doc files (architecture, standards, API ref)
├── plans/               # 21 plan dirs + reports/
├── run-tests.sh         # Unified test runner (WSL RAM-disk optimized)
└── CLAUDE.md            # Project instructions
```

---

## apiserver/ (Central API — Rust + Axum)

**Stack:** axum 0.7, tokio, sqlx 0.8.6 (Postgres), Redis, serde, chrono-tz, rust_xlsxwriter, aws-sdk-s3

### Layout
| Path | Contents |
|------|----------|
| `src/` | lib.rs (AppState), main.rs (routes), models.rs (43KB, all entities) |
| `src/handlers/` | 16 route handlers (agents, auth, attendance, employees, payroll, reports, shifts, leaves, etc.) |
| `src/services/` | 23 service modules (attendance calc, payroll eval, report coord, excel writer, etc.) |
| `src/repositories/` | 21 data-access modules |
| `src/worker/` | runner.rs (semaphore concurrency), handlers/ (attendance.rs, merge.rs) |
| `src/domain/` | policy resolver, overtime rules |
| `src/cache/` | Redis cache (shift, workspace, manager) |
| `src/queue/` | Redis queue (report jobs, pubsub) |
| `src/utils/` | datetime, debug, serde helpers |
| `migrations/` | 100+ Postgres migrations |
| `tests/` | report_calculation_tests.rs, work_unit_integration_tests.rs |

---

## agent-main/ (Edge Agent — Rust + SQLite)

**Stack:** tokio, sqlx (sqlite), tokio-tungstenite (WebSocket), dashmap, chrono-tz

| File | Size | Role |
|------|------|------|
| main.rs | 45KB | Entry point |
| memory_store.rs | 87KB | Core in-memory storage ⚠️ large |
| database.rs | 47KB | SQLite operations ⚠️ large |
| central_client.rs | 30KB | API server communication |
| hybrid_database.rs | 21KB | Hybrid storage layer |
| config.rs | 9KB | Configuration |
| crash_recovery.rs | 5KB | Recovery |
| retry.rs / worker_client.rs / utils.rs / zk_client.rs | <10KB each | Support modules |

---

## webdashboard/ (Frontend — React 19 + Vite)

**Stack:** React 19, react-router-dom 7, @tanstack/react-query 5, TailwindCSS 4, Radix UI, Vitest, Playwright, date-fns, i18next, exceljs

### Layout
| Path | Contents |
|------|----------|
| `src/pages/` | 20 page components (Dashboard, Attendance, Payroll, Reports, Shifts, Employees, Leaves, etc.) |
| `src/components/` | 14 subdirs, ~59 files. Largest: payroll (16), ui (9), common (7), layout (5), leave (5) |
| `src/lib/api/` | 17 API client modules |
| `src/lib/` | datetime.ts, excel-builder.ts, utils.ts |
| `src/hooks/` | Custom React hooks |
| `src/context/` | React context providers |
| `src/i18n/` | i18next localization |
| `src/types/` | TS type defs (report-stream.ts) |

---

## Current Working State (git status)

**Modified files (uncommitted):**
- `apiserver/src/models.rs` — model changes
- `apiserver/src/services/excel_writer.rs` — Excel export updates
- `apiserver/src/services/report_service.rs` — report logic changes
- `apiserver/src/worker/handlers/attendance.rs` — attendance handler
- `apiserver/src/worker/handlers/merge.rs` — merge handler
- `apiserver/tests/report_calculation_tests.rs` — test updates
- `apiserver/tests/work_unit_integration_tests.rs` — test updates
- `webdashboard/src/lib/__tests__/excel-builder.test.ts` — Excel builder tests
- `webdashboard/src/lib/api/types.ts` — API type updates
- `webdashboard/src/lib/excel-builder.ts` — Excel builder logic
- `webdashboard/src/pages/ReportsPage.tsx` — Reports page
- `webdashboard/src/types/report-stream.ts` — Report streaming types

**Deleted (staged):** All migration files in both agent-main/ and apiserver/ (moved to RAM — WSL optimization)

**Untracked:** migrations/ dirs, migrations_backup/, plans/reports/, docs/TODO_reports_ui_update.md

---

## Key Architecture Notes

- **Report pipeline:** Frontend → WebSocket → Redis queue → Worker (semaphore) → S3 → Excel
- **Auth:** JWT bearer tokens; debug impersonation mode
- **Testing:** `./run-tests.sh` moves migrations to /dev/shm to avoid WSL VirtioFS freeze
- **Active work:** Reports feature (Excel export, streaming, WebSocket) based on recent commits + modified files
