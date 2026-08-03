# ADR-008: Employer-Side Contributions

| Property       | Value                                                       |
|----------------|-------------------------------------------------------------|
| **Status**     | Proposed                                                    |
| **Date**       | 2026-08-03                                                  |
| **Author**     | AI + Lee                                                    |
| **Depends on** | [ADR-003](ADR-003-payroll-formula-engine.md), [ADR-006](ADR-006-accumulators-multi-period-context.md) |
| **Domain**     | Payroll — Employer Contributions                            |

---

## 1. Context

### 1.1 The Missing Half

ADR-003 through ADR-007 cover the **employee-side** payroll calculation: gross → deductions → net. But every employer in Vietnam also incurs mandatory contributions that are NOT deducted from employee pay:

| Contribution | Employer Rate | Employee Rate | Base | Cap |
|-------------|:------------:|:------------:|------|-----|
| BHXH (social insurance) | 17.5% | 8% | BHXH base salary | 20× regional minimum wage |
| BHYT (health insurance) | 3% | 1.5% | Same base | Same cap |
| BHTN (unemployment) | 1% | 1% | Same base | 20× regional MW (for region) |
| Trade Union Fee | 2% | — | **Total payroll fund** (all employees) | No cap |

### 1.2 Why This Matters

```
Employee sees:                     Employer actually pays:
┌──────────────────┐              ┌───────────────────────────┐
│ Gross: 15,000,000│              │ Gross:        15,000,000  │
│ - BHXH: -1,200K │              │ + BHXH (DN):  +2,625,000  │
│ - BHYT:   -225K │              │ + BHYT (DN):    +450,000  │
│ - BHTN:   -150K │              │ + BHTN (DN):    +150,000  │
│ - PIT:    -500K │              │ + CĐ Phí:       +300,000  │
│ ──────────────── │              │ ────────────────────────── │
│ Net:   12,925,000│              │ Total cost:   19,525,000  │
└──────────────────┘              └───────────────────────────┘
                                   (+51% from what employee sees)
```

Without employer contributions, the system cannot answer "How much does payroll actually cost the company?" — which is the #1 question every CFO asks.

---

## 2. Architecture Decision

### 2.1 Separate Pipeline, Same Accumulators

Employer contributions run as a **separate pipeline** AFTER the employee pipeline completes, because:
- They don't affect net pay — they're not inputs to the employee pipeline
- They share the same accumulator snapshots (same BHXH base, same cap)
- They must retro when employee side retroes

```
┌────────────────────────────────────────────────────────────┐
│                                                             │
│  PHASE 1: EMPLOYEE PIPELINE (ADR-003)                       │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐               │
│  │ GROSS    │ → │ DEDUCT   │ → │ NET      │               │
│  │          │   │ (BHXH E) │   │          │               │
│  └──────────┘   └──────────┘   └──────────┘               │
│       │                                               │
│       │ shares accumulators                                  │
│       ▼                                                      │
│  PHASE 2: EMPLOYER PIPELINE (NEW)                            │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   │
│  │ BHXH Employer│   │ BHYT Employer│   │ BHTN Employer│   │
│  │ 17.5% × base │   │ 3% × base    │   │ 1% × base    │   │
│  └──────────────┘   └──────────────┘   └──────────────┘   │
│       │                                                        │
│       ▼                                                        │
│  ┌──────────────────────────────────────────────────────┐    │
│  │ TRADE UNION FEE (workspace-level, not per-employee)   │    │
│  │ 2% × SUM(all employees' base salary for the period)   │    │
│  └──────────────────────────────────────────────────────┘    │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

### 2.2 Reusing ADR-003 Component Trait

Employer contributions ARE `PayrollComponent`s — same trait, different context:

```rust
impl PayrollComponent for EmployerSocialInsurance {
    fn component_type(&self) -> ComponentType { ComponentType::EmployerSocialInsurance }
    fn inputs(&self) -> Vec<VariableRef> {
        vec!["bhxh_base".into(), "ytd_bhxh_base".into()]
    }
    fn outputs(&self) -> Vec<VariableRef> {
        vec!["employer_bhxh_contribution".into()]
    }
    fn evaluate(
        &self,
        config: &ComponentConfig,
        ctx: &VariableContext,
    ) -> Result<Vec<(VariableRef, Decimal)>> {
        let rate = Decimal::from_str("0.175")?;
        let base = ctx.get_decimal("bhxh_base")?;
        let uncapped = base * rate;

        // Same cap check as employee side
        let remaining = ctx.accumulator_remaining_capacity("ytd_bhxh_base");
        let actual = match remaining {
            Some(cap) => std::cmp::min(uncapped, cap * Decimal::from_str("0.175")? / Decimal::from_str("0.08")?),
            None => uncapped,
        };

        Ok(vec![("employer_bhxh_contribution".into(), actual)])
    }
}
```

Key insight: employer component is **almost identical** to employee insurance component — same base variable, same accumulator, same cap. Only the rate differs. This means we could even reuse the same `InsuranceDeduction` component type with a `payer: "employee" | "employer"` config parameter. But for clarity, separate component types are preferred.

### 2.3 Trade Union Fee — The Exception

Trade union fee is **not per-employee**. It's 2% of the **total payroll fund** of the entire workspace. This breaks the assumption that every component is single-employee.

```rust
/// Workspace-level component — evaluates ONCE per period, not per employee.
trait WorkspaceComponent: Send + Sync {
    fn component_type(&self) -> ComponentType;
    fn evaluate_workspace(
        &self,
        config: &ComponentConfig,
        all_results: &[PerEmployeeResult],  // ← needs ALL employee results
        ctx: &WorkspaceContext,
    ) -> Result<Vec<(VariableRef, Decimal)>>;
}
```

This runs AFTER all employee pipelines complete for the period. It sums the base salary across all employees and applies 2%.

### 2.4 Database

No new tables needed for individual employer contributions — they're just component outputs like any other. Trade union fee is a workspace-level record:

```sql
-- Optional: persist workspace-level contribution results
ALTER TABLE payroll_periods
    ADD COLUMN employer_contributions JSONB;
-- {"total_bhxh_employer": 125_000_000, 
--  "total_bhyt_employer": 21_428_571, ...}
```

---

## 3. Integration with ADR-006 (Accumulators)

Employer and employee contributions share accumulators:

| Accumulator | Used By Employee | Used By Employer |
|------------|:----------------:|:----------------:|
| `ytd_bhxh_base` | ✅ BHXH deduction | ✅ BHXH employer |
| `ytd_bhxh_paid` | ✅ Tracking | ❌ (separate accumulator for employer) |

**Question:** Should employer and employee share the same `ytd_bhxh_paid` or track separately?

**Answer:** Track separately — they have different caps and rates. But they share `ytd_bhxh_base` because the base is identical.

---

## 4. Impact on GL Accounting

Employer contributions are the primary driver of GL entries:

```
Employee BHXH:  Nợ 334  / Có 3383   (giữ lại từ lương NLĐ)
Employer BHXH:  Nợ 622  / Có 3383   (chi phí của DN)
Total 3383:     Nợ 622 + 334 / Có 3383  (tổng phải nộp BHXH)
```

Without employer contributions, the GL integration (ADR-009) cannot produce complete journals.

---

## 5. References

- [ADR-003](ADR-003-payroll-formula-engine.md) — PayrollComponent trait
- [ADR-006](ADR-006-accumulators-multi-period-context.md) — Accumulator sharing
- [ADR-009](ADR-009-gl-accounting-integration.md) — GL entries
- Decision 595/QĐ-BHXH — Vietnam social insurance regulations
