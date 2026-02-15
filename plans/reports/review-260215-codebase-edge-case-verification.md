# Edge Case Verification Report — Full Codebase Review

**Date:** 2026-02-15
**Scope:** Entire codebase (apiserver, agent-main, agent-zk-worker, webdashboard)
**Reviewers:** 5 parallel code-reviewer agents

---

## Summary

| Metric | Count |
|--------|-------|
| Total edge cases verified | 30 |
| Handled ✅ | 15 |
| Unhandled ❌ | 9 |
| Partial ⚠️ | 6 |

---

## CRITICAL ❌ (5 issues)

| # | Edge Case | File | Details |
|---|-----------|------|---------|
| 1 | `get_users` exposes ALL users without auth | `apiserver/src/handlers/auth.rs:299` | Returns `SELECT * FROM users` including password hashes. Registered in authenticated router but no MemberContext check. Any authenticated user sees all users across all workspaces. |
| 2 | Job polling lacks workspace ownership validation | `apiserver/src/handlers/report_export.rs`, `import.rs` | Endpoints check auth but don't verify the job belongs to the requesting user's workspace. Any authenticated user can poll/download any job result by ID. |
| 3 | Unbounded reprocess date range — DoS vector | `apiserver/src/handlers/attendance/logs.rs:146-162` | `force_reprocess_attendance` accepts arbitrary `from`/`to` dates. Admin-only but no range limit. A malicious admin could trigger reprocess for years of data, causing DB/CPU exhaustion. |
| 4 | Unbounded batch payroll concurrency — OOM risk | `apiserver/src/services/payroll_service.rs:354` | `calculate_batch_payslips()` uses `join_all` on all employees without semaphore. Large workspace (1000+ employees) spawns unbounded concurrent futures, risking memory exhaustion. |
| 5 | Grace period override discards PolicyResolver result | `apiserver/src/services/payroll_service.rs:~205` | Payroll calculation overrides PolicyResolver's grace period with a hardcoded/config value, making the 4-tier policy hierarchy (Individual > Dept > Workspace > System) ineffective for grace periods. |

## HIGH ❌ (4 issues)

| # | Edge Case | File | Details |
|---|-----------|------|---------|
| 6 | WebSocket report handler no authentication | `apiserver/src/handlers/report_websocket.rs` | WS upgrade path lacks auth middleware. Anyone with the URL can connect and receive report data streams. |
| 7 | Overnight OT intervals lose date context | `apiserver/src/services/payroll_service.rs:238-241` | `.time()` call strips date from NaiveDateTime, so overnight OT spanning midnight (e.g., 23:00→01:00) produces inverted time range. |
| 8 | Deductions always zero — financial incorrectness | `apiserver/src/services/payroll_service.rs:324` | `deductions_amount: Decimal::ZERO` hardcoded. No deduction logic implemented despite field existing in payslip model. |
| 9 | Signature cache unbounded growth — agent OOM | `agent-main/src/memory_store/store.rs` | DashSet storing processed log signatures grows indefinitely. Long-running agent processing thousands of logs/day will eventually exhaust memory. |

## MEDIUM ⚠️ (6 issues)

| # | Edge Case | File | Details |
|---|-----------|------|---------|
| 10 | No email/password validation on registration | `apiserver/src/handlers/auth.rs:register()` | No email format check, no password strength requirements. Accepts empty strings, invalid emails. |
| 11 | DST gap handling assumes 1-hour shift | `apiserver/src/services/pairing_service.rs:315-329` | `local_to_tz()` adds `Duration::hours(1)` for DST gaps. Fails for 30-min DST zones (Lord Howe Island, Chatham Islands). Low risk — current deployments are Vietnam. |
| 12 | 6-hour buffer hardcoded for log fetching | `apiserver/src/services/pairing_service.rs:147` | Previous-day reprocess window fixed at 6 hours. Not configurable per workspace or shift pattern. May miss logs for workplaces with unusual shift starts. |
| 13 | Minute-level dedup loses sub-minute punches | `apiserver/src/services/pairing_service.rs:179` | Dedup truncates to minute precision. Two legitimate punches within same minute (e.g., quick in-out correction) get merged. |
| 14 | Silent fallback on leave breakdown failure | `apiserver/src/services/payroll_service.rs:300-301` | `unwrap_or()` silently returns empty leave breakdown on DB error. Payslip appears correct but leave data is missing. No logging. |
| 15 | No workspace_id filter on working_conditions query | `apiserver/src/domain/policy/resolver.rs:145` | `resolve_working_conditions` may return conditions from other workspaces if employee has cross-workspace records. |

## HANDLED ✅ (15 issues)

| # | Edge Case | Verdict |
|---|-----------|---------|
| 16 | i32 overflow in attendance minutes | Safe — 16h session limit caps at 960 minutes |
| 17 | Division by zero in payroll seg_units_per_day | DB constraint `work_unit_minutes NOT NULL DEFAULT 1` prevents |
| 18 | Break overnight handling | Correctly linearizes via midnight split |
| 19 | Same-day multi-component payroll | Handled via change-point detection and segmentation |
| 20 | MemberContext workspace isolation | Axum extractor validates membership on every request |
| 21 | JWT token generation & verification | Standard implementation with configurable expiry |
| 22 | Batch conditions date range | Correct start/end date filtering |
| 23 | Hash collision risk in signature cache | Acceptable — SHA256 collision probability negligible |
| 24 | Agent capacity check ordering | DashMap atomic operations prevent race |
| 25 | Pagination defaults | Sensible defaults (page=1, limit=20) |
| 26 | Session refresh token rotation | Implemented (though has race condition — see note) |
| 27 | Password hashing with argon2 | Correct implementation |
| 28 | Timezone-aware attendance calculation | Proper NaiveDateTime → timezone conversion |
| 29 | Unknown policy condition types | Silently ignored via `_ => {}` — acceptable for forward compat |
| 30 | Dev super-user endpoint | Behind `#[cfg(debug_assertions)]`, not in release builds |

---

## Notes

- **Session refresh race condition**: Concurrent refresh calls could reuse same token before rotation (non-atomic read+update in `auth_service.rs:87-113`). Low risk in practice but worth noting.
- **Hardcoded Vietnamese error messages** in `auth.rs:change_password()` — i18n concern, not a bug.
- **No rate limiting** on login endpoint — standard security concern but not an edge case per se.

---

## Recommendation Priority

1. **Immediate** (security): Fix #1 (get_users auth), #2 (job ownership), #6 (WS auth)
2. **High** (correctness): Fix #4 (batch concurrency), #5 (grace period override), #7 (overnight OT), #8 (deductions)
3. **Medium** (robustness): Fix #3 (reprocess limit), #9 (cache eviction), #10 (input validation)
4. **Low** (hardening): Address #11-#15 based on deployment context
