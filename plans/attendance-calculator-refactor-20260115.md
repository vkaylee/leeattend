# Implementation Plan: AttendanceCalculator Bucket Refactor

### ## Approach
- **Why this solution**: Current calculation is a simple subtraction. The new approach uses **Interval Intersection (Clamping)** to separate Standard time from Overtime (Early/Late).
- **Buckets**:
    - `Standard`: Intersection of `[PunchIn, PunchOut]` and `[ShiftStart, ShiftEnd]`.
    - `Early OT`: Time before `ShiftStart`.
    - `Late OT`: Time after `ShiftEnd`.
- **Rounding**: Apply `Floor` rounding on the *total duration* of each bucket after break deduction to handle the "late 10m = lose 30m" requirement.
- **Policy**: Add `allow_early_ot` to `Shift` model to filter Early OT bucket.

### ## Steps

1. **Database Migration** (5 min)
   - Create `apiserver/migrations/20260115000001_add_allow_early_ot_to_shifts.sql`
   ```sql
   ALTER TABLE shifts ADD COLUMN allow_early_ot BOOLEAN NOT NULL DEFAULT FALSE;
   ```

2. **Model Updates** (10 min)
   - Modify `apiserver/src/models.rs`:
     - Update `Shift` struct.
     - Update `CreateShiftRequest` and `UpdateShiftRequest`.

3. **Core Implementation** (30 min)
   - Refactor `apiserver/src/services/attendance_calculator.rs`:
     - Introduce `WorkBuckets` struct.
     - Implement `calculate_buckets` method using interval logic.
     - Implement `apply_rounding` using `Floor` by default for Standard time.

4. **Integration** (15 min)
   - Update `apiserver/src/repositories/shift_repository.rs` (if needed).
   - Update `ReportService` or `PayrollService` if they call the calculator.

5. **Testing** (20 min)
   - Update `AttendanceCalculator` unit tests.
   - Add cases for:
     - Early In (07:25 for 07:30) with `allow_early_ot=false` -> 0 OT.
     - Late In (07:40 for 07:30) with `work_unit=30` -> Loses 30 mins.
     - Normal case (On time).

### ## Timeline
| Phase | Duration |
|-------|----------|
| Migration & Models | 15 min |
| Calculator Refactor | 30 min |
| Integration | 15 min |
| Testing | 20 min |
| **Total** | **1 hour 20 min** |

### ## Rollback Plan
- Revert code changes.
- Manual SQL: `ALTER TABLE shifts DROP COLUMN allow_early_ot;`

### ## Security Checklist
- [ ] Ensure `allow_early_ot` is properly validated in API requests.
