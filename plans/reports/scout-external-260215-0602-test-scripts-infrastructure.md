# Test Scripts & Infrastructure Scout Report

**Generated:** 2026-02-15 06:02 | **Branch:** main | **Status:** Complete

---

## Test Orchestration (Root Level)

| File | Purpose |
|------|---------|
| `run-tests.sh` | Main unified orchestrator. Modes: `--all`, `--api`, `--web`, `--clean`. Pattern filter support. 30min global timeout watchdog. |
| `docker-compose.test.yml` | 4 services: `test_db` (PG16 tmpfs), `apiserver_test`, `agent_test`, `web_test`. Shared volumes for sccache, api sync, logs. |
| `setup-test-env.sh` | DB template creation, migration, parallel test isolation, ephemeral E2E databases. |

## Phase Scripts (`scripts/`)

| File | Phase | What it runs |
|------|-------|-------------|
| `scripts/test-lint.sh` | Phase 0 | api-sync → apiserver clippy → agent clippy → web lint → web typecheck |
| `scripts/test-unit.sh` | Phase 1 | apiserver `nextest --lib` → agent `nextest --lib` → agent-zk-worker `pytest` → web `vitest` |
| `scripts/test-integration.sh` | Phase 3 | Pre-compile → start API server → wait /health → apiserver integration → Playwright E2E |
| `scripts/test-utils.sh` | Shared | Colors, `run_with_timeout()`, WSL detection, worker auto-tuning, `run_parallel_ordered()`, summary table |

## API Server Test Files (`apiserver/`)

| File | Purpose |
|------|---------|
| `apiserver/Dockerfile.test` | Multi-stage: Rust 1.88 + cargo-chef + nextest + sccache |
| `apiserver/test-runner.sh` | DB init, migration, test execution, coverage mode (`--coverage`) |
| `apiserver/test.sh` | Legacy test runner (superseded) |
| `apiserver/fix_all_tests.sh` | Bulk-fix storage fields in test AppState instances |
| `apiserver/migrate-tests.sh` | Migrate test files to integration test structure |
| `apiserver/tests/integration/` | Integration test modules (auth, attendance, etc.) |

## Agent Test Files (`agent-main/`, `agent-zk-worker/`)

| File | Purpose |
|------|---------|
| `agent-main/Dockerfile.test` | Multi-stage: Rust 1.88 + Python3 + nextest + sccache |
| `agent-main/src/**/*_test*.rs` | Rust unit tests (inline `#[cfg(test)]` modules) |
| `agent-zk-worker/tests/test_zk_worker.py` | Python pytest: ZKService connect/disconnect, attendance logs, worker client |
| `agent-zk-worker/conftest.py` | Pytest fixtures |

## Web Test Files (`webdashboard/`)

### Config
| File | Purpose |
|------|---------|
| `webdashboard/vitest.config.ts` | jsdom env, setup: `./src/test/setup.ts`, 60s timeout |
| `webdashboard/playwright.config.ts` | `./tests` dir, 90s timeout, single worker on CI |
| `webdashboard/package.json` | `test`: vitest, `test:coverage`: vitest+coverage, `test:e2e`: playwright |

### Test Utilities
- `webdashboard/src/test/setup.ts` / `setup.tsx` — Vitest setup
- `webdashboard/src/test/utils.tsx` — `createTestQueryClient()`, `renderWithClient()`
- `webdashboard/src/test/factories.ts` — Test data factory

### Unit Tests (~50+ files)
- **Hooks (9):** use-datetime, use-debounce, use-report-stream, use-dark-mode, use-workspace, use-theme, use-i18n-sync, use-agents, use-language
- **Lib (8):** datetime, standardization, api, excel-builder, apiFetch, utils, attendance, pagination-response
- **i18n (2):** translation-usage, locales
- **Components:** payroll (12), time-tracking (3), leave (3), common (2), shifts (2), core (3), UI (3), features (2), layout (1)
- **Pages (16):** Departments, Shifts, PageRendering, DateFormat, Adjustments, Devices, Payroll, Reports, AttendanceLogs, LeaveRequests, Pagination, Agents, Members, WorkingPatterns, MyAttendance, LeaveSettings

### E2E Tests (21 Playwright specs)
- route-regression, sidebar-navigation, device-crud, register-flow, debug-login, report-export-stream, stability-regression, leave-policies, leave-policy-assignment, build-smoke, attendance-logs, z-index-stacking, employee-pagination, change-password, login-success, debug-metadata, report-export-realtime, EmployeeSearchIntegration, members-ui

### E2E Support
- **Mocks:** `tests/mocks/mock-data.ts`, `tests/mocks/api-intercept.ts`
- **POMs:** BasePage, LoginPage, ConnectivityPage, Sidebar, EmployeesPage
- **Utils:** `tests/worker-orchestrator.ts`

## Quick Reference

```bash
./run-tests.sh --all              # Full suite
./run-tests.sh --api              # API (Rust + Python)
./run-tests.sh --web              # Web (Vitest + Playwright)
./run-tests.sh --api auth         # Filter by "auth"
./run-tests.sh --clean            # Cleanup containers
TIMEOUT_GLOBAL=600 ./run-tests.sh --api  # Custom timeout
```

## Timeout Hierarchy

| Variable | Default | Scope |
|----------|---------|-------|
| TIMEOUT_GLOBAL | 1800s | Entire suite |
| TIMEOUT_LINT | 600s | Per lint task |
| TIMEOUT_UNIT | 300s | Per unit task |
| TIMEOUT_INTEGRATION | 600s | Per integration task |
| TIMEOUT_BUILD | 600s | Pre-compilation |
| TIMEOUT_DB_WAIT | 120s | DB readiness |

## Totals

| Category | Count |
|----------|-------|
| Orchestration scripts | 7 |
| Docker configs | 3 |
| API server test files | 5+ scripts, multiple integration modules |
| Agent test files | Rust inline tests + 1 Python test file |
| Web unit tests | ~50+ |
| Web E2E tests | 21 |
| Web test support | 8 (setup, utils, factories, POMs, mocks) |

---

No unresolved questions.
