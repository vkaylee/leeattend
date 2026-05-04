# Rule Engine & Immutable Policies Plan

## 🛡️ Rule Compliance Check
- **GEMINI.md Rule 4.4 & 4.11 (Enterprise-Grade Standard):** Ensuring historical payroll recalculation is accurate by enforcing Append-Only / Immutable Policy patterns via `effective_from` / `effective_to` assignments.
- **GEMINI.md Rule 5.2 (Socratic Gate):** User explicitly chose Option B (Dynamic Rules Array). We are proceeding based on verified requirements.
- **GEMINI.md Layer 2 (COMPLEX CODE):** Creating this plan file prior to implementing the complex architectural shift.

## 🎯 Objective
1. **Option B Implementation:** Refactor the API layer (`payroll_evaluator.rs`) to parse a dynamic array of rule operators (e.g., `[{"metric": "kpi_score", "operator": ">=", "value": 90}]`) instead of flat key-values.
2. **Historical Integrity:** Define the UI/API workflow for "updating" policies so that existing historical payroll recalculations remain unaffected.

## 🛠️ Step 1: Update Rust Evaluator (Option B)
Currently, `rule_config` is accessed via flat keys:
```rust
config.get("max_late_early_count")
```
We will change this. `rule_config` will be an array of objects:
```json
[
  { "metric": "late_early_count", "operator": "<=", "value": 2 },
  { "metric": "kpi_score", "operator": ">=", "value": 90 }
]
```
- Create a `RuleCondition` struct matching this schema.
- Update `matches_tier` in `payroll_evaluator.rs` to iterate over these conditions and evaluate them dynamically using a `match` on the operator (`>`, `<`, `>=`, `<=`, `==`).

## 🛠️ Step 2: Enforce Immutable Policies (API Layer)
To answer the user's concern about recalculating old payroll:
- We have `effective_from` and `effective_to` in `payroll_incentive_assignments`.
- **Workflow:** When an admin edits a policy in the UI, if the policy has already been used in past payrolls, the backend should ideally prevent modifying it, OR the UI should implement a "Clone & Replace" action behind the scenes.
- For now, we will add API validation: Prevent updating `payout_amount` or `rule_config` if a policy is currently assigned and active in a locked payroll period (or simply encourage versioning in the frontend).

## 🛠️ Step 3: Frontend Form Builder (Option B)
- Update the `BonusPolicyManager` React component.
- The Form Builder will change from static inputs to a dynamic list:
  - Dropdown: `[Select Metric]` (Late count, Unpaid Leave, KPI, etc.)
  - Dropdown: `[Operator]` (>, <, >=, <=)
  - Input: `[Value]`
- Support adding/removing multiple rules per tier.
