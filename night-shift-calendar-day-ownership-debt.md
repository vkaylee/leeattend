# Tech Debt: Calendar-Day Time-Entry Ownership Model

**Status:** Open (documented, not yet paid down)
**Logged:** 2026-07-16
**Trigger to pay down:** see "When this must be paid down" below.

## Summary

The pairing → persistence → reporting pipeline assumes an invariant:

> Every `time_entries` row belongs to the **calendar day of its `punch_in_time`**
> (in the employee's timezone).

Night shifts (that span midnight) inherently strain this invariant. The model
holds for a *single continuous* overnight session because its `punch_in` lands
on the shift's start day (day D). It breaks the moment a session is **split into
multiple rows where a later segment starts on day D+1**.

## Where the invariant is encoded

- `apiserver/src/services/pairing/context.rs:158` — `build_time_entries` drops any
  session whose `start_time.date_naive() != self.date`.
- `apiserver/src/services/pairing/service_range.rs:192` — `save_range_entries`
  deletes existing rows by a **calendar-day** window on `punch_in_time`.
- `apiserver/src/repositories/diesel_time_entry_repository.rs:484` —
  `replace_time_entries` takes a **per-(employee, calendar-day)** advisory lock.
- `apiserver/src/services/pairing/context_helpers.rs` — consumed-log detection is
  keyed off calendar-day attribution.
- `apiserver/src/services/attendance_utils.rs:287` (`group_entries_by_date`) and
  `apiserver/src/services/report_service/daily.rs:365` — the report groups entries
  by `punch_in_time`'s calendar day.
- `apiserver/src/services/attendance_summary_service.rs:311,480` — the summary
  filters day entries the same way.

## The bug this caused (fixed 2026-07-16)

Shift `SX Đêm KNGC`: 19:30–04:30 (+1 day), break 23:30–00:30 (**crosses midnight**).
Employee 2-punch day: check-in 19:21, check-out 07:31 (next day).

`split_by_breaks` cut the session at the configured break boundary, producing a
`second_half` segment starting **00:30 on day D+1**. `build_time_entries` then
silently discarded it (the `date_naive() != self.date` guard), so the report
showed a check-out of 23:30 instead of the real 07:31. Even if the guard were
removed, the `second_half` row would land under day D+1 and be re-deleted by the
next day's calendar-day reprocess — so the naïve fix does not hold either.

### The fix applied (minimal, low-risk)

`apiserver/src/services/pairing/strategy_window.rs` — a break is only treated as a
valid split point when `break.end_minutes() < 1440` (i.e. the post-break segment
still starts on day D). A **midnight-crossing break no longer splits** the
session; it stays one continuous row (19:21 → 07:31). Hours are unchanged because
`AttendanceCalculator::calculate_buckets` re-derives the break deduction and
break-OT from the shift config independently of how many rows represent the
session (verified: std 480', break_ot 60', late_ot 181' for this case).

**This fix does not add debt** — it respects the existing invariant rather than
violating it. It is correct for the whole class of *2-punch night shifts with a
config-only break crossing midnight*.

## Why this is still debt

The split was only ever a **presentation** of the session in the `time_entries`
table; it is not the source of any computed number. The underlying limitation —
that the persistence/lock/attribution stack cannot represent a work segment whose
`punch_in` is on a different calendar day than the shift it belongs to — remains.

## When this must be paid down

Pay it down when a business requirement forces **true cross-midnight segment rows**,
e.g.:

- **Night 4-punch shifts** where the employee records real punches at the break
  boundary (e.g. out 23:35, in 00:25). Here the break is materialized by real
  events and genuinely *should* split into two rows — one of which starts on D+1.
- **Night split-shifts** (multiple scheduled work windows crossing midnight).
- Any feature that surfaces break intervals as their own `time_entries` rows.

Until then, the minimal fix above fully covers observed production behavior.

## What paying it down looks like

Introduce an explicit **shift-day** dimension instead of inferring ownership from
`punch_in`'s calendar day:

1. Add a `shift_date` (a.k.a. `business_date`) column to `time_entries` + backfill
   migration (see `.agent/rules/database-rules.md` for migration standards).
2. Rework delete-window, advisory-lock key, and consumed-log detection to key off
   `shift_date` rather than `punch_in_time.date_naive()`.
3. Rework `group_entries_by_date` / summary filters to group by `shift_date`.
4. Remove the `date_naive() != self.date` guard in `build_time_entries` and the
   `break.end_minutes() < 1440` split suppression in `strategy_window.rs`.
5. Comprehensive tests: night 4-punch, night split-shift, DST edges, concurrent
   reprocess of adjacent days.

This is a core, concurrency-sensitive refactor and must be a **separate task**
with its own migration review — it must not be bundled into a bug fix.
