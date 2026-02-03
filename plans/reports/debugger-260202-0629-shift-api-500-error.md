# Debug Report: Shift Version Update 500 Error

**Date:** 2026-02-02
**Issue:** Test `shift_api_extended_tests::test_shift_api_lifecycle` fails at line 196 with HTTP 500 on shift version update

## Root Cause

**Unique constraint violation** in `employee_shifts` table during shift versioning.

### Code Path

1. **Test Request** (line 171-181):
   - Endpoint: `PUT /api/workspaces/{ws_id}/shifts/{shift_id}`
   - Payload: `is_correction: false`, `effective_date: "2026-02-01"`

2. **Handler** (`apiserver/src/handlers/shifts.rs:69-83`):
   - Routes to `ShiftService::update_shift()`

3. **Service** (`apiserver/src/services/shift_service.rs:56-143`):
   - Detects `is_correction = false` → triggers versioning logic (lines 91-142)
   - Starts transaction
   - Calls `archive_shift()` to mark old shift inactive
   - Creates new shift version via `create_shift()`
   - **Calls `update_assignments_to_new_version_tx()`** ← FAILURE HERE

4. **Repository** (`apiserver/src/repositories/shift_repository.rs:202-236`):
   ```rust
   // Line 211-221: Creates new assignments FIRST
   sqlx::query!(
       "INSERT INTO employee_shifts (employee_id, shift_id, start_date, end_date)
        SELECT employee_id, $2, $3, end_date
        FROM employee_shifts
        WHERE shift_id = $1 AND (end_date IS NULL OR end_date >= $3)",
       old_shift_id,
       new_shift_id,
       effective_date  // "2026-02-01"
   )

   // Line 224-233: Closes old assignments AFTER
   sqlx::query!(
       "UPDATE employee_shifts
        SET end_date = $2
        WHERE shift_id = $1 AND (end_date IS NULL OR end_date >= $3)",
       old_shift_id,
       _yesterday,  // "2026-01-31"
       effective_date
   )
   ```

### Constraint Violation

**Constraint:** `uq_employee_shift_start UNIQUE (employee_id, start_date)`
**Location:** `apiserver/migrations/20260109120000_unique_shift_assignment.sql:10`

**Scenario:**
1. Employee has existing assignment: `(emp_1, shift_old, start_date: "2026-01-01", end_date: NULL)`
2. Test assigns shift: `(emp_1, shift_id, start_date: TODAY, end_date: NULL)`
3. Version update tries INSERT: `(emp_1, shift_new, start_date: "2026-02-01", end_date: NULL)`
4. **Conflict**: If TODAY's assignment hasn't been closed and has `start_date = "2026-02-01"`, INSERT violates unique constraint

**Actual Error:** INSERT executes before UPDATE, so existing assignments with matching `employee_id + start_date` cause duplicate key error.

## Recommended Fix

**Reorder operations in `update_assignments_to_new_version_tx`:**

```rust
pub async fn update_assignments_to_new_version_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    old_shift_id: Uuid,
    new_shift_id: Uuid,
    effective_date: NaiveDate,
) -> Result<(), sqlx::Error> {
    let yesterday = effective_date.pred_opt().unwrap_or(effective_date);

    // 1. Close old assignments FIRST (prevent constraint conflicts)
    sqlx::query!(
        "UPDATE employee_shifts
         SET end_date = $2, updated_at = CURRENT_TIMESTAMP
         WHERE shift_id = $1 AND (end_date IS NULL OR end_date >= $3)",
        old_shift_id,
        yesterday,
        effective_date
    )
    .execute(&mut **tx)
    .await?;

    // 2. Create new assignments AFTER (no conflicts now)
    sqlx::query!(
        "INSERT INTO employee_shifts (employee_id, shift_id, start_date, end_date)
         SELECT employee_id, $2, $3, end_date
         FROM employee_shifts
         WHERE shift_id = $1 AND end_date = $2",
        old_shift_id,
        new_shift_id,
        effective_date,
        yesterday
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}
```

**Key Changes:**
1. **Swap order**: UPDATE before INSERT
2. **Fix SELECT filter**: Query assignments WHERE `end_date = yesterday` (just closed) to get correct employee list
3. Eliminates duplicate key violations

## Code Locations

- **Handler:** `/home/elt1541/leeattend/apiserver/src/handlers/shifts.rs:69-83`
- **Service:** `/home/elt1541/leeattend/apiserver/src/services/shift_service.rs:91-142`
- **Repository:** `/home/elt1541/leeattend/apiserver/src/repositories/shift_repository.rs:202-236`
- **Constraint:** `/home/elt1541/leeattend/apiserver/migrations/20260109120000_unique_shift_assignment.sql:10`

## Test Impact

**Affected Test:** `/home/elt1541/leeattend/apiserver/tests/shift_api_extended_tests.rs:196`

Test expects HTTP 200 but receives 500 due to unhandled database constraint violation in transaction.

## Priority

**HIGH** - Blocks shift versioning feature, critical for production shift schedule updates.
