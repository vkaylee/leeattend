# Implementation Plan: Implement Unified Payslip Line Items

## Approach
This plan addresses the need for transparency in PaySlips by explicitly capturing and displaying the "Standard Amount" (Định mức) versus the "Actual Amount" (Thực nhận) for variable salary components (Incentives/Allowances). We will adopt **Approach B (Unified Component Architecture)** from the brainstorming session, which involves enhancing the data model to store these values permanently.

**Why this solution?**
- **Transparency**: Users can see exactly why they received less than the maximum potential (e.g., proration due to unpaid leave).
- **Auditability**: The "Standard" value serves as a reference point for all calculations.
- **Scalability**: Sets the foundation for "Dynamic Payroll" where every line item (including future deductions) follows a consistent `standard` -> `actual` pattern.

## Steps

1.  **Database Migration (15 min)**
    - Create a new migration to add `standard_amount` column to the `payslip_incentives` table.
    - Default existing rows to `amount` (assuming standard = actual for past records) or 0.

2.  **Model Refactoring (15 min)**
    - Modify `IncentiveEvaluationResult` struct in `apiserver/src/models.rs` to include `standard_amount: Decimal`.
    - Update `PayslipIncentive` struct (if exists separate from evaluation result) to match.

3.  **Repository Update (15 min)**
    - Update `payroll_repository::create_payslip_incentive` to accept and insert the `standard_amount`.

4.  **Evaluator Logic Enhancement (30 min)**
    - Refactor `PayrollEvaluator::evaluate_policy` in `apiserver/src/services/payroll_evaluator.rs`.
    - **Logic**:
        - For **Percentage** policies: `standard` = `base_amount`.
        - For **Fixed** policies: `standard` = `tier.payout_amount` (of the selected tier).
        - *Note*: If a user misses a tier completely, `standard` might be the "Base" or "Max Tier". For now, we will use the *Selected Tier's* value as standard (if matched) or the Policy Base.

5.  **Service Integration (15 min)**
    - Ensure `PayrollService::calculate_monthly_payslip` correctly propagates the new field.

6.  **Testing (30 min)**
    - Update `payroll_evaluator_tests.rs` to assert on `standard_amount`.
    - Add integration test in `payroll_flow_tests.rs` to verify the value is persisted and retrieved.

## Timeline
| Phase | Duration |
|-------|----------|
| Database Migration | 15 min |
| Model Refactoring | 15 min |
| Repository Update | 15 min |
| Evaluator Logic | 30 min |
| Testing | 30 min |
| **Total** | **1 hour 45 min** |

## Rollback Plan
1.  Revert the migration: `sqlx migrate revert`.
2.  Restore the previous code using git checkout.

## Security Checklist
- [x] Input validation (Amounts must be valid decimals)
- [x] Auth checks (No changes to existing RBAC)
- [x] Rate limiting (N/A)
- [x] Error handling (Database insertion errors logged)
