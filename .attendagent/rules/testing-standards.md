# 🧪 Testing Standards

> [!WARNING]
> **NEVER run `cargo test`, `npm test`, `npx playwright test`, or any test command directly.** 
> This project uses Podman-based test isolation. The ONLY way to run tests is through `./leedevkit test`.

## 1. Test Execution Commands
> [!IMPORTANT]
> **DO NOT use background execution (`&`) for tests.** The `./leedevkit test` orchestrator handles PTY safety internally. Run it in the foreground so you can read the output and verify the results.

| Scope | Command | Purpose |
|---------------|---------|---------|
| **Frontend** | `./leedevkit test web` | Lint, Unit (Vitest), E2E (Playwright) |
| **Backend** | `./leedevkit test api` | Lint, Unit (Rust), Integration (SQLx) |
| **Full Suite** | `./leedevkit test all` | Comprehensive verification |

### Useful Flags
| Flag | Purpose |
|------|---------|
| `--lint-only` | Run only code formatting and linting checks |
| `--unit-only` | Run only unit tests (skips E2E and linting) |
| `--e2e-only` | Run only integration/E2E tests |
| `--skip-lint` | Skip linting phase (faster iterative TDD) |
| `--pattern <regex>` | Filter tests by name (e.g., `--pattern 'auth'`) |
| `--coverage` | Run with coverage reporting |
| `--fix` | Auto-fix formatting issues |
| `--json` | Machine-readable JSON output (for AI agents) |

## 2. Core Testing Principles
- **Testing Pyramid:** Prioritize Unit > Integration > E2E. 
- **Pattern:** Use AAA Pattern (Arrange-Act-Assert).
- **Code Coverage:** Minimum **80%** coverage for all new business logic.
- **Mandatory Pass:** A task is NOT complete until `./leedevkit test <target>` passes successfully.

### 2.1 Coverage Baseline Protection (ANTI-DRIFT)
- **Coverage thresholds are floors, not aspirational targets.** `vitest.config.ts` enforces per-path thresholds (e.g. `./src/lib/api/` ≥ 80/55/90/85 stmts/branch/fn/lines). A PR that drops coverage below the floor fails CI.
- **Boy Scout for coverage:** When you touch a file, check its coverage. If the file is below the floor, add tests in the SAME PR. Never leave a file less tested than you found it.
- **Baseline is the moving floor.** After each coverage-improving PR, the floor may be raised to the new baseline. The floor only moves up, never down.
- **Coverage is part of Definition of Done.** A feature PR without adequate test coverage is incomplete — same weight as a failing typecheck.

## 3. Verification Pipeline
> 🔴 **Execution Priority:** Security → Lint → Schema → Tests → UX → SEO → Performance

**Mandatory Pipeline:**
1. Quick check: `./leedevkit test <target> --lint-only` (format + lint)
2. Unit tests: `./leedevkit test <target> --unit-only`
3. Full suite: `./leedevkit test <target>` (lint + unit + E2E)
4. For machine-readable results: `./leedevkit test <target> --json`
