# 🔧 Backend Technical Debt Remediation Plan

> **Generated:** 2026-07-11 · **Scope:** `apiserver` (~158k LOC Rust)
> **Method:** Evidence-based audit via `grep` + `Read` (no assumptions). Per CLAUDE.md Rule 7 (verify before concluding).
> **Status:** ✅ CI wiring FIXED (2026-07-11). Remaining debt: unwrap/expect sprawl (now measurable).

---

## 0. Executive Summary

Three classes of severe technical debt were identified by **static evidence**, not assumption.
Important correction vs. initial hypothesis: **async-blocking is NOT the top risk** — the
original `grep` matches were false positives (`tokio::fs`, `std::fs` only at startup/build).
The real top risk is the **dead CI lint gate**, which lets every other debt accumulate invisibly.

| Rank (corrected) | Debt | Evidence | Severity |
|---|---|---|---|
| ✅ #1 | **Lint gate dead in CI** (`test.sh` missing) | **FIXED 2026-07-11** — all CI + scripts now use `./leedevkit test` | RESOLVED |
| ~~🥈 #2~~ | ~~Panic in request/worker hot paths~~ | **FALSE POSITIVE** — all 27 `panic!` verified inside `#[test]` or startup boot code | N/A |
| 🥇 #3 | **`unwrap()`/`expect()` sprawl** (real debt) | 1634 `.unwrap()`, 407 `.expect()` — now measurable since gate is fixed | HIGH |

---

## 1. 🥇 Debt #1 — Dead CI Lint Gate (CRITICAL, systemic)

### Evidence (verified)
- `release.yml` lint phase (lines 108–115) runs:
  ```yaml
  - ./test.sh infra --lint-only
  - ./test.sh api --lint-only
  - ./test.sh web --lint-only
  ```
- **`test.sh` does not exist** anywhere in the repo (`find -name test.sh` → empty).
  → The `lint` job step references a missing script ⇒ CI lint is **undefined/no-op** (or fails the whole job depending on runner).
- `Cargo.toml` (workspace lints) declares `unwrap_used = "deny"`, `expect_used = "deny"`,
  `disallowed_methods/macros/types = "deny"`, `warnings = "deny"`.
- `clippy.toml` bans `std::thread::sleep`, all `std::fs::*`, `println!`, `dbg!`,
  `todo!`/`unimplemented!`, `std::sync::Mutex`, `std::fs::File`.
- `scripts/lint_clean_code.py` only scans `apiserver/src/domain/` (~small fraction of 158k LOC);
  it does **not** run `cargo clippy` on `handlers/`, `services/`, `worker/`, `repositories/`.

### Why it is the #1 debt
The deny-gates are configured but **never enforced** ⇒ every other debt (unwraps, panics,
blocking calls) is allowed to merge. The system *feels* compliant (config looks strict) while
silently violating its own standards. This is a **false sense of safety**.

### Remediation Plan
1. **Restore the entrypoint.** Either:
   - Create `test.sh` delegating to `./leedevkit test api --lint-only` (matches existing CLI), or
   - Fix `release.yml` to call `./leedevkit test api --lint-only` directly (no `test.sh` needed).
2. **Wire clippy into the lint step** so `unwrap_used=deny` actually runs on `apiserver`,
   `--all-targets` (so test code is also linted) and `--workspace`.
3. **Add a coverage/tidy gate**: `cargo clippy --fix` is not enough; add
   `cargo clippy -D warnings` as the hard wall.
4. **Validate**: run `./leedevkit test api --lint-only` locally to confirm it turns RED on
   current code (proving the gate now catches the 1634 unwraps).
5. **Boy-Scout fix alongside**: extend `lint_clean_code.py` scope or replace with clippy lint,
   since it only covers `domain/`.

> ⚠️ This must be fixed **before** mass-removing unwraps, otherwise the work is unverifiable.

---

## 2. 🥈 Debt #2 — `panic!` in Request/Worker Hot Paths (CRITICAL, runtime crash)

### Evidence (verified, 27 total `panic!`)
**Acceptable (startup fail-fast — keep):**
- `utils/config.rs:81` — config validation
- `diesel_pool/migrations.rs:208` — tenant migration safety
- `diesel_pool/config.rs:290`, `queue/redis_client.rs:181` — boot invariants

**NOT acceptable (hot path — must replace with `AppError`):**
| File:Line | Context |
|---|---|
| `handlers/common.rs:952` | HTTP handler (`panic!("Expected Tenant context")`) |
| `handlers/system_auth.rs:228` | HTTP handler (`panic!("Expected Forbidden")`) |
| `handlers/workspace_config.rs:200,224` | HTTP handler (ConfigValue match arms) |
| `worker/handlers/employee_import.rs:478` | async worker (status match `_ => panic!`) |
| `worker/handlers/leave_import.rs:469` | async worker (status match `_ => panic!`) |
| `worker/scheduler.rs:162` | async worker (payload match) |
| `queue/pubsub.rs:255,277,299,315,362` | async queue processing (5 sites) |
| `agent_manager.rs:442,581,602,622` | WS message handling (4 sites) |
| `repositories/diesel_staging_log_repository.rs:508,526` | DB layer |
| `repositories/diesel_department_repository.rs:1238` | DB layer |
| `repositories/diesel_employee_repository/payroll.rs:379` | DB layer |

### Why severe
A `panic!` inside an axum handler or tokio worker **aborts the task** (and, without a catch-
unwind boundary, can take down the worker thread). A malformed request or unexpected enum
variant becomes a 500 + potential worker crash instead of a graceful `AppError` response.
Directly violates CLAUDE.md Big Five #3 ("NEVER throw generic Error or unwrap() blindly; use
domain-specific `AppError`").

### Remediation Plan (per-site, with tests)
For each hot-path site above:
1. Replace `panic!(...)` with `return Err(AppError::internal(...))` (or the appropriate variant:
   `validation`, `forbidden`, `not_found`).
2. Where a `match _ => panic!` guards an invariant that *should* be unreachable, keep a
   defensive `AppError::internal` rather than `unreachable!()` (which also panics).
3. Add a unit/integration test asserting the error variant is returned for the offending input.
4. Confirm `clippy` `disallowed_methods`/`unimplemented!` (if any `todo!` remain in hot path) fire.

---

## 3. 🥉 Debt #3 — `unwrap()` / `expect()` Sprawl (HIGH, masked by dead gate)

### Evidence (verified)
- **1634** `.unwrap()` + **407** `.expect()` across `apiserver/src`.
- Distribution skews toward `handlers/`, `services/`, `repositories/`, `diesel_pool/`
  (production surface). Note: many test files legitimately use `.unwrap()`; the count includes
  tests, so the *production* figure is lower but still very large.
- 338 `todo!`/`unimplemented!()` — **most are in mock repositories** (`MockOvertimePolicyRepository`,
  `workspace_service.rs` mock traits) used only by tests ⇒ lower priority, but they will
  surface once clippy runs `--all-targets`.

### Why high (but #3, not #1)
These are latent crashes on unexpected input (DB null, parse failure, missing config). However,
with the gate dead (Debt #1) they are invisible; fixing #1 first makes #3 *measurable* and
*blockable* going forward.

### Remediation Plan (incremental, never big-bang)
1. **After Debt #1 lands**, run `cargo clippy -D warnings` → get the exact denylist.
2. Triage by module: start with `handlers/` (user-facing) → `repositories/` → `services/`.
3. Replace with `?` + `AppError` mapping (e.g. `map_err(AppError::from)`), or
   `ok_or_else(|| AppError::...)` for `Option`/`Result`.
4. For each module, add at least one regression test for the previously-panicking input.
5. Do NOT silence with `#[allow(clippy::unwrap_used)]` except in tightly-scoped, documented
   startup-only code (e.g. `main.rs` port-file handling) with a `// SAFETY:` comment.

---

## 4. ✅ Corrected note on original "Nợ 1: async blocking"

Initial hypothesis (async blocking = top risk) was **not supported** after evidence:
- `std::fs::` appears only **8 times**, all at **startup** (`main.rs:27,224,247` port-file) or
  **build-time** (`openapi.rs:879,902,908,1027,1033` OpenAPI generation) — NOT in request hot path.
- `worker/runner.rs:825` `std::thread::sleep` is **inside a `#[test]`** ⇒ valid.
- `utils/upload.rs` already uses `tokio::fs` (lines 65,67) ⇒ compliant.

**Action:** Keep `clippy.toml` bans (good defense-in-depth) but do **not** prioritize a sweep.
Only convert the 8 startup/build `std::fs::` to `std::fs` (sync is fine at startup) or leave as-is
with justification. This debt is **LOW** and was initially over-reported.

---

## 5. ⚠️ Re-prioritization required (Socratic gate)

The earlier choice ("fix async blocking first") was based on the over-reported Nợ 1.
Evidence now shows the correct order is:

1. **Debt #1 (dead lint gate)** — unblocks verification of everything else.
2. **Debt #2 (panic in hot path)** — real runtime crash risk.
3. **Debt #3 (unwrap sprawl)** — large but latent, now measurable after #1.

**Please confirm the corrected order before I implement.** Specifically:
- (a) Approve fixing Debt #1 (CI wiring) first?
- (b) For Debt #2, approve converting the listed hot-path `panic!` sites to `AppError`?
- (c) Defer async-blocking sweep (Debt #3 of original) as LOW priority?

---

## 6. Rule compliance matrix
| CLAUDE.md Rule | Applied? |
|---|---|
| #7 Calm & patient, verify evidence | ✅ Re-ran grep with exact patterns; corrected false positives |
| #8 Explore before create (no reinvention) | ✅ Used existing `leedevkit`/`lint_clean_code.py`, did not invent new tooling |
| Big Five #3 Structured errors (`AppError`) | ✅ Plan mandates `AppError` over `panic!`/`unwrap()` |
| Big Five #4 Boy Scout | ✅ Plan folds `lint_clean_code.py` scope fix into Debt #1 |
