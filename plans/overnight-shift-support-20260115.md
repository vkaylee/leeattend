# Implementation Plan: Overnight Shift Support

Enable the system to correctly handle and pair attendance for shifts that span across two calendar days (e.g., 22:00 to 06:00).

## Problem
- **Pairing Logic**: Currently uses `NaiveTime` which doesn't handle day wrapping (e.g., 06:00 < 22:00, but in an overnight shift, 06:00 is "after" 22:00).
- **Data Fetching**: `PairingService` currently fetches logs for a single calendar day (00:00-23:59), missing the checkout punch if it happens the next morning.
- **Reporting**: Needs to attribute the work hours to the logical "Shift Date" (Day 1) even if the work ends on Day 2.

## Approach
- **Relative Time (Linearization)**: Refactor `pairing_algo` to work with "Minutes from Reference" instead of raw `NaiveTime`. The reference point will be dynamically determined based on the shift start (e.g., Shift Start - 6 hours).
- **Extended Fetch Window**: Update `PairingService` to fetch logs for `[ShiftDate 00:00]` to `[ShiftDate + 1 12:00]` (or configurable buffer) when an overnight shift is detected.
- **Shift Config**: Detect overnight shifts by checking `start_time > end_time`.

## Steps

### 1. Refactor Pairing Algorithm (`pairing_algo.rs`) (40 min)
- **Change Input**: Accept `minutes_from_midnight` (i32) or linearize inside.
- **Linearization Logic**:
    - If `shift.start > shift.end` (Overnight):
        - Define `offset = 0`.
        - For any time `t`: if `t < shift.start - buffer`, treat as `t + 24h`.
    - Actually, easier: Convert everything to "Minutes from Shift Start minus Buffer".
- **Update Structures**: `PairedSession` should include `end_is_next_day: bool`.

### 2. Update Pairing Service (`pairing_service.rs`) (30 min)
- **Shift Detection**: Check if `effective_shift.start_time > effective_shift.end_time`.
- **Query Window**: 
    - Standard: `00:00` to `23:59`.
    - Overnight: `start_time - buffer` to `end_time (next day) + buffer`.
    - Safe default: Query `Date 00:00` to `Date+1 12:00` (36 hours) for all, and let the pairing algo filter by "cost".
- **Result Construction**: When saving `TimeEntry`, if `end_is_next_day`, add 1 day to the `punch_out_time` date component.

### 3. Update Integration Tests (20 min)
- Create `tests/overnight_shift_tests.rs`.
- Scenario: Shift 22:00 - 06:00.
- Punch In: Day 1 21:55.
- Punch Out: Day 2 06:05.
- Expect: One paired session, duration ~8h 10m.

## Timeline
| Phase | Duration |
|-------|----------|
| Algo Refactor | 40 min |
| Service Update | 30 min |
| Testing | 20 min |
| **Total** | **1.5 hours** |

## Rollback Plan
- Revert `pairing_algo.rs` and `pairing_service.rs`.

## Security Checklist
- [x] No data loss (we only read raw logs and write time entries).
- [x] Date boundary safety (handle end-of-month/year correctly).
