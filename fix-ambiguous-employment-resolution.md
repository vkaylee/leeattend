# Fix: Ambiguous Employment Resolution (Việc 1)

## Problem

Employment resolution for attendance logs silently guesses when evidence is insufficient.
Three sites share the same defect class:

| # | Site | Current behaviour | Defect |
|---|------|-------------------|--------|
| 1 | `diesel_employment_repository.rs:307` `get_employment_effective_on_date` | `.order(start_date.desc()).first()` | 2 employments covering the punch date → silently picks the newer one. No signal. |
| 2 | `diesel_staging_log_repository.rs:228` `claim_staged_logs` | `status.eq("active")` + `.first()`, **no ORDER BY** | Ignores punch date entirely. Historical logs of a re-hired employee attach to the *current* employment. Non-deterministic on multiple active rows. |
| 3 | `diesel_staging_log_repository.rs:322` `resolve_staging_log` | same as #2 | same as #2. |

Sites 2 and 3 are a correctness bug beyond ambiguity: the date-based resolution that
`process_log` performs correctly is *not* applied on the claim/resolve path, so
`bài toán 2` (re-hire) is only half-solved.

DB does not prevent overlap: `employments` has only plain btree indexes
(`idx_employments_query`, `idx_employments_date_range`) — **no EXCLUDE constraint**.
So overlapping rows are insertable and the guess is reachable.

## Rule IDs applied

1. **Core Principle 3 / coding-standards §3 — Structured Errors.** Ambiguity becomes an
   explicit `AppError` + `I18nKey`, never a silent pick.
2. **Core Principle 4 / coding-standards §5 — Boy Scout Rule.** Sites 2 and 3 are the same
   bug in code I must touch; fixed in the same change rather than deferred.
3. **database-rules §1 — Tenant Isolation.** Every new/edited query keeps its explicit
   `workspace_id` filter.
4. **database-rules §2 — Zero N+1.** Ambiguity detection reuses one `.load()` instead of
   adding a second round-trip; `claim_staged_logs` batch-fetches employments for all staged
   dates in one query, no query inside the loop.
5. **database-rules §3 — Transactional Boundaries.** `claim_staged_logs` / `resolve_staging_log`
   keep their existing `conn.transaction(...)` wrapper.
6. **coding-standards §2 — Strict Encapsulation.** New model/DTO fields stay private with getters.
7. **testing-standards §2 — AAA + ≥80% coverage** for the new branch logic.

## Design

Resolution outcome becomes explicit instead of `Option<Employment>`:

```
0 employments covering punch date → Unresolved  (existing: error log, no retry)
1 employment                      → Resolved(id)
>1 employments                    → Ambiguous(candidate ids)  ← NEW
```

- `get_employment_effective_on_date` is **renamed** to `resolve_employment_on_date` and now
  returns `EmploymentResolution` instead of `Option<Employment>` (no `.first()`). Renaming
  rather than adding a parallel method is safe — it had only two callers (`process_log` and
  one test stub) — and it makes the old silently-guessing signature unreachable.
- `process_log`: on `Ambiguous`, park the punch in `attendance_staging_logs` with
  `status = 'ambiguous'` instead of inserting a guessed `raw_attendance_log`.
  No migration needed — `status` is plain `text` with no value CHECK constraint.
- Staging list/count include `ambiguous` so the rows surface to HR.
- `claim_staged_logs` auto-claims only `pending` (ambiguous needs a human choice) and
  resolves **per punch date**, batch-fetched.
- `resolve_staging_log` accepts an optional explicit `employment_id` (validated:
  belongs to employee + workspace, covers punch date); without it, resolves by punch
  date and returns `AmbiguousEmployment` rather than guessing.

## Out of scope (deliberate)

- **Bước 3b** (workcode / device location / roster tie-breakers). Not built — overlap count
  in real data is unmeasured, and building an inference tier for a possibly-empty set is the
  over-engineering trap. Revisit only if measurement shows real overlaps.
- **EXCLUDE constraint on `employments`.** Correct long-term guard, but adding it blindly can
  fail on existing overlapping rows in live tenants; needs the measurement query first.
