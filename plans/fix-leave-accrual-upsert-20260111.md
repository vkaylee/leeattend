# Implementation Plan: Fix Leave Accrual Silent Failure (2026-01-11)

## Approach
- **Problem:** `process_workspace_accruals` uses `UPDATE` to add accrued leave. If the balance record for the current year is missing (e.g., new year), the accrual is silently lost.
- **Solution:** Replace `UPDATE` with `INSERT ... ON CONFLICT DO UPDATE` (Upsert) to ensure the balance record is created if missing.

## Steps

### 1. Schema Verification (5 min)
- Check `leave_balances` table definition to confirm unique constraints on `(employee_id, year, leave_type)`.

### 2. Implementation (15 min)
- **File:** `apiserver/src/services/leave_accrual_service.rs`
- **Change:**
  ```rust
  // From:
  // UPDATE leave_balances SET accrued_amount = accrued_amount + $3 ...
  
  // To:
  // INSERT INTO leave_balances (employee_id, year, leave_type, entitlement, accrued_amount, carried_over)
  // VALUES ($1, $2, 'vacation', 0, $3, 0)
  // ON CONFLICT (employee_id, year, leave_type) 
  // DO UPDATE SET accrued_amount = leave_balances.accrued_amount + EXCLUDED.accrued_amount, updated_at = NOW()
  ```

### 3. Testing (15 min)
- **File:** `apiserver/tests/leave_accrual_tests.rs`
- **New Test:** `test_accrual_creates_balance_if_missing`
  - Setup: Employee with no `leave_balances` for 2026.
  - Action: Trigger accrual.
  - Assert: Balance record created with correct accrued amount.

## Timeline
| Phase | Duration |
|-------|----------|
| Schema Check | 5 min |
| Code Change | 15 min |
| Test Creation | 15 min |
| Verify | 5 min |
| **Total** | **40 min** |

## Rollback Plan
- Revert code changes to `apiserver/src/services/leave_accrual_service.rs`.
