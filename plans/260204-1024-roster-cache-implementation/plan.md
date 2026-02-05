# Roster Cache Implementation

**Status:** In Progress | **Branch:** main | **Date:** 2026-02-04

## Objective
In-memory LRU + Redis-versioned + singleflight cache for `get_workspace_schedules()`.
Eliminates 8 of 10 repeated DB queries per call during attendance report generation.

## Architecture
- **Store:** `moka` LRU (max 100), key=`Uuid`, value=`(Arc<RosterData>, i64 version)`
- **Version SOT:** Redis `roster:version:{workspace_id}` → `i64` unix timestamp
- **Singleflight:** `tokio::sync::watch` channel. Leader loads DB, followers wait on channel.
- **Fallback:** Redis failure → version defaults to `Utc::now()` → forces fresh DB load

## What's Cached (`RosterData`)
Static workspace schedule config fetched WITHOUT date bounds (date filtering stays in the resolution loop):
- shifts, fixed_assignments, pattern_assignments
- dept_patterns, dept_fixed_shifts, active_rules, rule_patterns, dept_history

**NOT cached** (still per-request): employee list, shift_rosters in date range.

## Invalidation Triggers
Bump Redis version + evict moka after successful DB write to:
- shifts (create/update), rosters (upsert/bulk/delete)
- working_patterns (create/assign/bulk/delete)
- assignment_rules (create/update/delete)

## Phases
| # | Phase | Key Files |
|---|-------|-----------|
| 1 | Dependencies | `Cargo.toml` — add `moka` |
| 2 | RosterCache | `cache/roster_cache.rs` (new) — types, build fn, cache struct |
| 3 | Wire AppState | `cache/mod.rs`, `lib.rs`, `main.rs` |
| 4 | Refactor Repo | `repositories/roster_repository.rs` — extract `build_roster_data`, update caller |
| 5 | Invalidation | `handlers/shifts.rs`, `working_patterns.rs`, `assignment_rules.rs` |
| 6 | Tests | Unit tests in `roster_cache.rs`; regression via `./run-tests.sh` |

## Constraints
- `warnings = "deny"` — zero compiler warnings
- Redis optional in AppState → `roster_cache: Option<Arc<RosterCache>>`
- WSL: use `./run-tests.sh` only, never `cargo test` directly
