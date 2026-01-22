# Implementation Plan: Variable-based Payroll Incentive Engine

Update the payroll system to support flexible, tiered incentive policies (like Diligence Bonus) based on attendance metrics.

## ## Approach
- **Data-Driven Rules**: Move logic from code to database tables.
- **Tiered Evaluation**: Match employee attendance summaries against priority-ordered thresholds.
- **Scalable Variables**: Support late counts, sick leave, unpaid leave, and total leave as trigger variables.

## ## Steps

1. **Database Migration** (10 min)
   - Create `payroll_incentive_policies` and `payroll_incentive_tiers` tables.

2. **Backend Domain Models** (20 min)
   - Define `PayrollIncentivePolicy` and `IncentiveTier` structs.
   - Implement the `evaluate_policy` logic.

3. **Repository Layer** (15 min)
   - Add methods to `PayrollRepository` to fetch active policies for a workspace.

4. **Payroll Service Integration** (20 min)
   - Update `calculate_payroll` to include incentive evaluation.
   - Inject incentive results into `PayslipResponse`.

5. **Testing & Verification** (25 min)
   - Create integration tests for different violation scenarios.

## ## Timeline
| Phase | Duration |
|-------|----------|
| Database | 10 min |
| Models & Logic | 20 min |
| Service Integration | 35 min |
| Testing | 25 min |
| **Total** | **1.5 hours** |

## ## Rollback Plan
- Revert migration and code changes in `apiserver`.
