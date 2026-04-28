# Smart Smoke Mode — Design Document

## Problem

Current `--smoke` mode only runs lint + unit tests, skipping all 512 integration/E2E tests. This means critical regressions in API integration (auth, attendance, payroll) are not caught at pre-commit time. There is dead code in `run-tests.sh:80-82` that was designed to run a smoke subset of integration tests but is unreachable because `PHASE_INTEGRATION=false` in smoke mode.

## Solution

Enable a curated subset of fast integration/E2E tests in smoke mode using nextest filter expressions and naming conventions. No new Cargo features, no separate test binaries, no Cargo.toml changes.

### Mechanism

**Rust (nextest):** `cargo nextest run --test '*' --filter-expr 'test(/smoke/)'`

Matches any test function name containing "smoke" across all packages and binaries (apiserver monolithic binary + agent-main 26 auto-discovered binaries).

**Web (Playwright):** `npx playwright test --grep "smoke"`

Matches any test title containing "smoke".

### Naming Convention

| Layer | Pattern | Example | Filter |
|-------|---------|---------|--------|
| Rust integration | `_smoke` suffix in function name | `test_login_success_smoke` | `test(/smoke/)` |
| Playwright E2E | `smoke:` prefix in test title | `smoke: login with valid credentials` | `--grep "smoke"` |

### Smoke Test Selection Criteria

- **< 5 seconds per test** (total smoke integration < 3 minutes)
- **Critical business paths**: auth, employee CRUD, attendance clock-in/out, leave, payroll, shift
- **Not flaky**: no network dependency, timing, or race condition
- **Minimal setup**: no complex seed data
- **1-2 tests per domain**: representative coverage across auth, employee, attendance, leave, shift, payroll, report, workspace

### Smoke Mode Flow (New)

```
--smoke flag
  → PHASE_LINT=true
  → PHASE_UNIT=true
  → PHASE_INTEGRATION=true   (was false!)
  → INT_PATTERN="smoke"       (filter expression)

Integration phase:
  → apiserver: cargo nextest run --test integration --filter-expr 'test(/smoke/)'
  → agent-main: cargo nextest run --test '*' --filter-expr 'test(/smoke/)'
  → web: npx playwright test --grep "smoke"
```

### Files to Modify

| File | Change |
|------|--------|
| `run-tests.sh` | Enable PHASE_INTEGRATION in smoke mode, fix dead code at L80-82 |
| `scripts/test-integration.sh` | Accept and pass filter expression argument to nextest/playwright |
| `apiserver/tests/integration/*.rs` | Rename ~15-20 fast test functions with `_smoke` suffix |
| `agent-main/tests/*.rs` | Rename ~5-10 fast test functions with `_smoke` suffix |
| `webdashboard/tests/*.spec.ts` | Add `smoke:` prefix to ~3-5 E2E test titles |

### Safety Check

Smoke mode warns if zero tests match the filter (catches naming drift):
```
⚠️  No smoke-tagged integration tests found! Did you forget to add _smoke suffix?
```

### Estimated Impact

| Mode | Before | After |
|------|--------|-------|
| Smoke | ~8-15 min (lint+unit) | ~10-12 min (lint+unit+smoke integration) |
| Missing regressions | Integration entirely skipped | Critical path covered in < 3 min |

### What This Does NOT Change

- Full integration suite behavior (unchanged)
- CI pipeline stages (unchanged)
- No Cargo.toml or nextest.toml modifications needed
- No dual compilation or feature flags
