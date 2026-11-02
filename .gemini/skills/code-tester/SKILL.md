---
name: code-tester
description: |
  Execute tests, verify fixes, measure coverage, and manage the Podman test
  environment for the LeeAttend project. Activate when the user says: "run
  tests", "chạy test", "verify this fix", "check coverage", "unit test", "E2E",
  "Playwright", "cargo nextest", "test fails", "test bị fail", "xem log test",
  "start DB container", "rebuild test env", "smoke check trước khi commit",
  "tại sao test hang", or any variation of testing or test-env management.
  NEVER use cargo test or npm test directly — always use ./run-tests.sh.
risk: safe
source: custom
---

# Goal

Execute tests and manage the LeeAttend test environment exclusively through
`./run-tests.sh`. The orchestrator handles DB isolation, PTY safety, container
lifecycle, and caching automatically. The AI's job is to select the right flags,
read the output, and recover from failures — never to invoke lower-level tools.

# Instructions

## Step 1 — Classify the request

Before running anything, determine the right mode:

```
"run tests" / "verify fix"
  ├── Know which test? → --api --test-only --test=NAME    (fastest)
  ├── API only?        → --api
  ├── Web/E2E only?    → --web
  ├── Quick pre-commit → --all --smoke
  ├── Full suite?      → --all
  └── Coverage only?   → --api --coverage

"environment broken" / "container error"
  ├── First: --docker=test build
  └── Still broken: --clean (destructive — warn user first)

"test is hanging" / "no output"
  └── --logs  (read from persistent log volume, never redirect orchestrator output)
```

## Step 2 — Confirm before destructive ops

Before running `--clean` or anything that deletes volumes: **state the impact to the user and ask for confirmation.** Rebuilding from scratch takes 5–10 minutes.

## Step 3 — Run with `--dry-run` first (when uncertain)

If the scope is unclear (e.g., "run the payment tests"), use `--dry-run` to
preview what would execute, then proceed.

## Step 4 — Read output and recover

After a test run:
- **Pass** → Report which phases passed and total duration.
- **Fail** → Immediately read `./run-tests.sh --logs` to get the actual error message before attempting a fix.
- **Hang / No output** → Run `./run-tests.sh --logs`. Do NOT re-run the same command; logs explain the hang.
- **Docker Compose error** → Run `./run-tests.sh --docker=test build` first.
- **DB migration error** → Run `./run-tests.sh --docker=test up -d test_db` to start the DB, then retry.

## Step 5 — Never bypass the orchestrator

If the user asks to run `cargo test`, `npm test`, or a script directly, explain why it's unsafe (DB isolation breaks, PTY hangs) and substitute the correct `./run-tests.sh` command.

---

# Commands Reference

### Test Modes
```bash
./run-tests.sh --all              # Lint → Unit → Integration → E2E (everything)
./run-tests.sh --api              # Lint → Unit → Integration for apiserver
./run-tests.sh --web              # Lint → Unit → Playwright E2E for webdashboard
./run-tests.sh --integration      # Integration + E2E only (skip lint/unit)
```

### Targeting
```bash
./run-tests.sh --api --test=test_payroll_calc         # Run one test by name (regex)
./run-tests.sh --api --test-only --test=failing_test  # Skip lint/unit, run only this test
./run-tests.sh --all --smoke                          # Lint + unit only, skip integration
```

### Individual Phases (Advanced)
```bash
./run-tests.sh --api --lint              # Lint phase only
./run-tests.sh --api --unit              # Unit phase only
./run-tests.sh --api --integration-only  # Integration phase only
./run-tests.sh --all --no-color          # Disable ANSI colors (CI pipelines)
```

### Container Management
```bash
./run-tests.sh --docker=dev up -d                 # Start dev environment
./run-tests.sh --docker=dev down                  # Stop dev environment
./run-tests.sh --docker=dev logs -f apiserver     # Tail one service log
./run-tests.sh --docker=dev exec apiserver bash   # Shell into container
./run-tests.sh --docker=dev build apiserver       # Rebuild one service
./run-tests.sh --docker=test build                # Rebuild test environment
./run-tests.sh --docker=prod ps                   # Check prod status
```

### Coverage
```bash
./run-tests.sh --api --coverage                         # Full API coverage
./run-tests.sh --api --coverage --component=apiserver   # One crate only
# Outputs → .test_logs/coverage-*.lcov + HTML report in .test_logs/
# Target: ≥80% for apiserver (cargo-llvm-cov)
```

### Cache & Maintenance
```bash
./run-tests.sh --cache-stats   # Show Cargo/NPM cache sizes
./run-tests.sh --clean-cache   # Delete caches only (containers stay running)
./run-tests.sh --no-cache      # Force fresh build for this run only
./run-tests.sh --warm-cache    # Pre-compile deps (run once on fresh machine)
./run-tests.sh --clean         # ⚠️ DESTRUCTIVE: stop containers + delete volumes + cache
```

### Utilities
```bash
./run-tests.sh --list              # List available tests without running
./run-tests.sh --dry-run           # Preview what would execute
./run-tests.sh --shard=1/3         # CI: run shard 1 of 3
./run-tests.sh --timeout=600       # Override global timeout (default: 1800s)
./run-tests.sh --logs              # Dump all test logs from the volume
./run-tests.sh --tail-logs         # Live-tail test logs
```

---

# Examples

## Example 1: "Verify the payroll fix" — precise target

**Trigger:** AI just patched `calculate_gross_pay`. User asks to verify.

```bash
# Skip lint/unit — run only the specific test (fastest path)
./run-tests.sh --api --test-only --test=test_payroll_calc
```

**Expected output pattern:**
```
⚡ Test-only mode: skipping lint and unit phases
🔹 Running Integration & E2E Tests [apiserver|agent:test_payroll_calc]...
✅ 1 test passed in 12.3s
```

If it fails → run `./run-tests.sh --logs` to read the error before touching code.

---

## Example 2: "Smoke check before committing"

**Trigger:** User says "quick check before I push".

```bash
./run-tests.sh --all --smoke
```

**Expected:** Lint + unit phases pass within ~2 min. Integration skipped intentionally.

---

## Example 3: "Tests are hanging, no output"

**Trigger:** A command ran but the terminal shows nothing for > 30s.

```
# WRONG: do not re-run or redirect output
# RIGHT: read from the persistent log volume
./run-tests.sh --logs
```

The PTY safety gate is built in — output redirection is never needed and would hide live feedback.

---

## Example 4: "Get full coverage report"

```bash
./run-tests.sh --api --coverage --component=apiserver
# Read results from: .test_logs/coverage-apiserver.lcov
# Or open HTML report: .test_logs/coverage-apiserver/index.html
```

Current target: **≥80%** measured with `cargo-llvm-cov`. Focus on `--lib` density first — integration test boilerplate can dilute the number.

---

## Example 5: "CI is failing, environment is broken"

```bash
# Step 1: Try rebuilding the test image first (usually enough)
./run-tests.sh --docker=test build

# Step 2: If still broken, full reset (warn user — takes 5–10 min)
./run-tests.sh --clean
./run-tests.sh --all --warm-cache  # Warm cache before full run
```

---

# Constraints

- **Orchestrator only.** Never use `cargo test`, `npm test`, or `scripts/*.sh` directly — DB isolation breaks and tests pollute each other's state.
- **No output redirection.** `./run-tests.sh > file.log` is not a PTY workaround — the script handles PTY safety internally. Redirecting hides streaming output.
- **Confirm before `--clean`.** It deletes Docker volumes. Rebuilding takes 5–10 minutes.
- **Read logs before re-running.** A second run on a broken environment rarely helps. `--logs` reveals the actual error in seconds.
- **Coverage target:** ≥80% for `apiserver`. Use `cargo-llvm-cov` with `--lib` to measure core logic density without integration boilerplate dilution.
- **Rust ≥ 1.85 required.** As of March 2026, `edition2024` features in `apiserver` require `rust:1.85+` in containerized builds.
- **Self-discovery.** If any flag behavior is unclear, run `./run-tests.sh --help` — the orchestrator's built-in help is the authoritative reference, not this skill.

---

# Error Recovery

| Situation | Confidence | Action |
|---|---|---|
| User asks for specific test name | 🟢 High | Run `--test-only --test=NAME` directly |
| User says "run tests" with no context | 🟡 Medium | Ask: "API, web, or everything?" — default to `--api --smoke` |
| Test fails | 🟢 High | Immediately run `--logs`, read error, then fix |
| Command hangs | 🟢 High | Run `--logs` (never re-run blind) |
| "Coverage broken" | 🟡 Medium | Run `--docker=test build` first, then `--coverage` |
| User asks to run `cargo test` | 🟢 High | Refuse, explain DB isolation risk, offer `./run-tests.sh --api` |
| Unknown flag needed | 🟢 High | Run `./run-tests.sh --help` to self-discover |
| Environment badly corrupted | 🔴 Low | Confirm with user → `--clean` → `--warm-cache` |

<!-- Generated by Skill Creator Ultra v1.0 -->
