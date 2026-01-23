# Implementation Plan: Refactor Payroll Calculation to Strict Boolean Logic

**Task:** Refactor payroll calculation to rely strictly on `is_base_for_ot` flag and remove magic strings ("BASE", "Lương cơ bản").

## Approach
The goal is to eliminate technical debt and "magic string" logic in the payroll service. By enforcing a single source of truth (the database schema's boolean flag), we make the system robust, language-agnostic, and easier to maintain.

However, simply changing the code is risky for existing data. The plan includes a **Data Migration** step to ensure all legacy "Base" salary components have their `is_base_for_ot` flag correctly set to `TRUE` before we enforce the strict check.

## Steps

1. **Data Migration (Safety First)** (10 min)
   - Create a new migration file: `apiserver/migrations/20260123000000_ensure_base_salary_flags.sql`.
   - Update all existing salary component definitions where `code = 'BASE'` or `name = 'Lương cơ bản'` to have `is_base_for_ot = TRUE`.
   - This ensures legacy data conforms to the new strict rule.

2. **Core Implementation (Refactor)** (10 min)
   - File to modify: `apiserver/src/services/payroll_service.rs`
   - Action: Remove the legacy string checks.
   - Code change:
     ```rust
     // Before
     if scd.code == "BASE" || scd.name == "Lương cơ bản" || scd.is_base_for_ot {
     
     // After
     if scd.is_base_for_ot {
     ```

3. **Testing (Verification)** (15 min)
   - Update/Verify test: `apiserver/tests/payroll_custom_base_test.rs` (formerly reproduction test). It should still pass.
   - Update/Verify test: `apiserver/tests/payroll_tests.rs`. Ensure standard base salary tests still pass (this confirms the migration worked for standard cases).
   - Create a new regression test case in `payroll_custom_base_test.rs` explicitly testing a component named "Lương cơ bản" but with `is_base_for_ot = false` to ensure it is **NOT** included (proving the magic string is gone).

## Timeline
| Phase | Duration |
|-------|----------|
| Data Migration | 10 min |
| Implementation | 10 min |
| Testing | 15 min |
| **Total** | **35 min** |

## Rollback Plan
1. Revert the code change in `payroll_service.rs` to include the string checks again.
2. The data migration is additive (fixing flags) so it generally doesn't need rollback, but can be ignored if code is reverted.

## Security Checklist
- [x] Logic simplification reduces potential for injection or spoofing via naming.
- [ ] No new auth checks needed (internal logic).
