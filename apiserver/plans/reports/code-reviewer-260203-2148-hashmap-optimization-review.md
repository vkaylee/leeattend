# Code Review — HashMap Optimisation (`roster_repository`) + New Tests

**Date:** 2026-02-03
**Scope:** `roster_repository.rs` lines 80-380 + 550-748 (`get_workspace_schedules` batch path &
`get_effective_shift` single-employee path); `workspace_schedule_hashmap_tests.rs` (entire file, ~328 lines)
**Focus:** Correctness of the O(1) HashMap grouping, composite-key semantics, priority-chain logic,
and test coverage of all six priority tiers.

---

## Overall Assessment

The optimisation is structurally sound. Replacing the previous nested `.iter().find()` loops with
five per-employee HashMaps and one composite-key map eliminates the O(N*M*K) hot path and is the
right approach for the batch endpoint.  The tests are well-organised: each of the three test
functions isolates a distinct subset of the priority chain, the date helpers are verified in
comments, and the day-off (`shift_id = NULL`) semantics are explicitly exercised for both the
pattern path (P2) and the rule-pattern path (P3).

Three issues require attention before this lands.  Two are bugs; one is a comment that actively
misleads future readers.

---

## Critical Issues

### C1 — `dept_fixed_shifts` query (step 7, line 142) has no workspace filter

**Location:** `roster_repository.rs` lines 142-151

```rust
// 7. Fetch Department Default Shifts
let dept_fixed_shifts = sqlx::query!(
    r#"
    SELECT edh.employee_id, dds.shift_id, edh.effective_date
    FROM employee_department_history edh
    JOIN department_default_shifts dds ON edh.department_id = dds.department_id
    WHERE edh.effective_date <= $1          // <-- only date filter
    ORDER BY edh.effective_date DESC
    "#,
    end_date
)
```

Every other batch query in the function scopes to `workspace_id` (steps 3-6, 8-10).  This one
does not.  In a multi-tenant deployment every `employee_department_history` row in the entire
database is pulled, and rows from other workspaces can leak into `dept_fixed_shifts_map`.
An employee whose UUID happens to collide (impossible with v4, but structurally wrong) or, more
realistically, an employee who exists in multiple workspaces via shared infrastructure will get
a cross-workspace shift assigned.

**Fix:** Add a join back through `departments` or a sub-select, matching the pattern used in the
`dept_patterns` query at step 6:

```sql
WHERE edh.department_id IN (
        SELECT id FROM departments WHERE workspace_id = $1
      )
  AND edh.effective_date <= $2
```

Bind `workspace_id` as `$1`, shift `end_date` to `$2`.

---

## High Priority

### H1 — P3 rule path in `get_workspace_schedules`: missing "key absent" fall-through

**Location:** `roster_repository.rs` lines 255-260

```rust
if let Some(rule) = matched_rule {
    effective_shift_id = rule_patterns_map
        .get(&(rule.pattern_id, dow))
        .copied()
        .flatten();          // <-- always assigns, even when key is absent
    source = "assignment_rule".to_string();
}
```

The stated `Option<Option<Uuid>>` contract is:
- **key present, value `Some(id)`** -> working day with that shift
- **key present, value `None`**     -> day off (intentional)
- **key absent**                    -> pattern does not cover this day; fall through to P4

`.get()` returns `None` when the key is absent.  `.copied()` propagates that `None`.  `.flatten()`
on `None` yields `None`.  The code then unconditionally sets `source = "assignment_rule"` and
stops the priority chain.

Net effect: if a rule's pattern simply does not have a row for a given `day_of_week`, the employee
is treated as having a day off *attributed to the rule*, when the correct behaviour is to continue
evaluating P4 (department pattern), P5 (fixed), P6 (department fixed).

**Compare with `get_effective_shift`** (line 655), which handles this correctly:

```rust
if let Some(&shift_opt) = rule_patterns_map.get(&(rule.pattern_id, day_of_week)) {
    return Ok((shift_opt, traces));   // only when key IS present
}
// falls through to P4 when key is absent
```

**Fix:** Mirror the single-employee path:

```rust
if let Some(rule) = matched_rule {
    if let Some(&shift_opt) = rule_patterns_map.get(&(rule.pattern_id, dow)) {
        effective_shift_id = shift_opt;
        source = "assignment_rule".to_string();
        // push trace …
    }
    // else: key absent → fall through to P4 (no source set yet)
}
```

> **Why the tests do not catch this:** Test 3 creates a pattern with rows for both dow=1 (Monday)
> and dow=0 (Sunday).  Every day queried has a matching key, so the absent-key branch is never
> exercised.  A test that queries a Wednesday (dow=3), where the pattern has no row, would expose
> the bug.

---

### H2 — `get_effective_shift` priority-label comments are misnumbered (P4 and P5)

**Location:** `roster_repository.rs` lines 693 and 721

```rust
// Priority 4: Employee Default Shift (Legacy Fixed Assignment)   // <-- should be 5
…
// Priority 5: Department Default Shift                            // <-- should be 6
```

The six-tier model used everywhere else is:
P1 Roster / P2 Individual Pattern / P3 Dynamic Rule /
**P4 Department Pattern / P5 Fixed / P6 Department Fixed**

The comments at lines 693 and 721 label what is clearly P5 (individual fixed) as "Priority 4"
and P6 (department fixed) as "Priority 5".  This is not a runtime bug — it is a maintenance trap:
anyone reading `get_effective_shift` in isolation will misunderstand the authority ordering and
may introduce a regression when modifying fallback logic.

**Fix:** Rename to match the canonical numbering and drop the parenthetical "Legacy" qualifier,
which is inconsistent with the batch path that calls the same tier "Fixed Assignment".

---

## Medium Priority

### M1 — Test gap: absent composite key (P3 fall-through) not exercised

As noted in H1, no test queries a day whose `day_of_week` is absent from the rule pattern.
Once H1 is fixed, a test asserting that the employee falls through to P4/P5/P6 on such a day
should be added to `test_dynamic_rule_via_workspace_schedules` (e.g. query Wednesday).

### M2 — Test gap: `employee_ids` filter parameter not exercised

`get_workspace_schedules` accepts `employee_ids: Option<&Vec<Uuid>>`.  All three tests pass
`None`.  At least one assertion with `Some(&vec![emp1])` would confirm the SQL `ANY($2)` path
works end-to-end through the HashMap grouping.

### M3 — `dept_fixed_shifts_map` vectors are never trimmed to `>= start_date`

`dept_patterns_map` vectors are implicitly filtered by `day_of_week` in the inner `.find()` (line
264-266).  `dept_fixed_shifts_map` vectors are filtered only by `effective_date <= curr_date` (line
301).  The SQL itself fetches all history rows up to `end_date`.  This is not a correctness bug
because the `.find()` still picks the most-recent valid row (the SQL `ORDER BY … DESC` preserves
that order in the Vec), but it means the map can hold rows from well before `start_date` that
will never be selected.  Low-impact; noted for awareness.

---

## Low Priority

### L1 — `#[allow(clippy::type_complexity)]` on `shifts_map` (line 35)

The 9-tuple stored as the value is hard to read.  A lightweight named struct (even a private one
in the same file) would remove the lint suppression and make the `.get()` destructuring at line
340 self-documenting.  Noted as out-of-scope for this PR if the tuple predates it; flag for
follow-up.

### L2 — Minor: `emp_tags.clone().unwrap_or_default()` inside the date loop (line 245)

`emp_tags` is `Option<Vec<String>>`.  The `.clone()` fires on every iteration of the date loop
for every employee.  `.as_deref().unwrap_or_default()` would yield `&[String]` without cloning;
or hoist the resolved value above the `while` loop.  Negligible at current data volumes;
matters if the date range grows to months.

---

## Positive Observations

- The batch pre-fetch strategy (10 queries, zero per-employee, zero per-day) is correct and
  well-structured.  Each step is numbered and grouped logically.
- `succ_opt()` usage avoids a panic on `NaiveDate::MAX`.
- The test helpers (`monday()`, `sunday()`, `jan1()`) carry verified-date comments — good hygiene.
- Day-off semantics (`shift_id = NULL` in DB → `None` in Rust) are tested explicitly for two
  distinct code paths (P2 pattern, P3 rule pattern).
- `get_effective_shift` composite-key lookup (lines 630-657) is a clean, correct implementation
  of the pre-fetch pattern.

---

## Recommended Actions (priority order)

1. **Fix C1** — add `workspace_id` filter to step-7 query.  Cross-workspace data leak.
2. **Fix H1** — guard the P3 assignment behind a key-present check, matching `get_effective_shift`.
3. **Fix H2** — correct priority numbers in `get_effective_shift` comments.
4. **Add test** for absent composite key (M1) — a Wednesday query in test 3.
5. **Add test** for the `employee_ids` filter (M2).

---

## Unresolved Questions

- Q1: Is `department_default_shifts` guaranteed to be single-row per department, or can there be
  multiple rows (e.g. historical)?  If multiple, the step-7 query and its `.find()` at line 300
  need an `ORDER BY` on a timestamp column, not just `effective_date` from the history join.
- Q2: The "Legacy Fixed Assignment" label at line 693 — is `employee_shifts` actually deprecated
  in favour of patterns?  If so, a migration plan should be tracked; if not, drop "Legacy" from
  the comment to avoid confusion.
