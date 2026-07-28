# Migration Expected Diffs — ADR-005 Group A Registry

> **Purpose:** Document every **Group A (expected fix)** diff between the cascade engine
> and legacy resolver. Only diffs listed here are considered "good" diffs.
> All others are Group B/C (unexpected) and **block cutover** (ADR-005 §2).

## How to use

When the diff report (`scripts/adr005_diff_report.py`) flags a diff:

1. Verify the diff is a *legacy bug fix* (i.e., cascade engine is correct, legacy was wrong).
2. Add a row to the table below with:
   - The field name (exact match with `cascade_diff_report_v` column)
   - Typical legacy value range and cascade value range
   - The typical delta
   - A justification with a reference commit or issue
3. Re-run the report — the diff will move from Group B/C to Group A.

## Registered Group A Diffs

| Field | Legacy Value | Cascade Value | Delta | Justification |
|-------|-------------|---------------|-------|---------------|
| _(no diffs documented yet — populate during shadow run)_ |

---

## Cutover Threshold (ADR-005 §2)

- 0 Group B/C diffs required.
- Group A ≤ 5% of total rows unless HR approves exception.
- If Group A > 5%: documented HR approval required before cutover.

## Related

- ADR-005: Dual-Run Shadow Mode & Rollback Protocol
- `scripts/adr005_diff_report.py`: Daily diff report script
- `cascade_diff_report_v`: SQL view powering the report
