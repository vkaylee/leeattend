# ADR-016: Leave Encashment

| Property       | Value                                                       |
|----------------|-------------------------------------------------------------|
| **Status**     | Proposed                                                    |
| **Date**       | 2026-08-03                                                  |
| **Author**     | AI + Lee                                                    |
| **Depends on** | [ADR-003](ADR-003-payroll-formula-engine.md), [ADR-010](ADR-010-off-cycle-payroll-run-types.md) |
| **Domain**     | Payroll — Leave Monetization                                |

---

> **2026-11-13 update:** Salary base groups and their membership table are retired by `20261113020000_retire_salary_base_groups`. The group-based schema proposal below is historical, not an implementation contract. Future encashment must select salary components through payroll formulas/rules; that replacement is not implemented by the retirement.

## 1. Context

### 1.1 Legal Basis

Vietnamese Labor Code 2019 (Article 113) requires:
- Employees are entitled to paid annual leave (12-16 days/year depending on job type)
- Upon **termination or job loss**, unused annual leave must be compensated in cash
- Some companies also allow **voluntary encashment** at year-end for unused days

### 1.2 Calculation

```
Leave encashment amount = (total_salary / standard_working_days) × unused_leave_days

Where:
  total_salary = sum of all salary components used as encashment base
  standard_working_days = from payroll config (typically 26)
  unused_leave_days = entitlement - used_days - pending_approval_days
```

### 1.3 Tax Treatment

- Leave encashment on termination: **taxable income** (part of final settlement)
- Leave encashment while still employed: **taxable income** (treated as bonus)
- **Not subject to social insurance** (it's compensation, not salary for work performed)

---

## 2. Architecture Decision

### 2.1 Leave Encashment as a PayrollComponent

Leave encashment is a component that can be included in **termination** and **annual_settlement** run templates (ADR-010):

```rust
struct LeaveEncashmentComponent;

impl PayrollComponent for LeaveEncashmentComponent {
    fn component_type(&self) -> ComponentType { ComponentType::LeaveEncashment }

    fn inputs(&self) -> Vec<VariableRef> {
        vec![
            "encashment_base_salary".into(),   // salary base for encashment
            "standard_working_days".into(),
            "unused_leave_days".into(),
            "leave_encashment_rate".into(),     // default 1.0 (100%)
        ]
    }

    fn outputs(&self) -> Vec<VariableRef> {
        vec![
            "leave_encashment_amount".into(),
            "leave_encashment_leave_days_paid".into(),
        ]
    }

    fn evaluate(
        &self,
        config: &ComponentConfig,
        ctx: &VariableContext,
    ) -> Result<Vec<(VariableRef, Decimal)>> {
        let base = ctx.get_decimal("encashment_base_salary")?;
        let std_days = ctx.get_decimal("standard_working_days")?;
        let unused_days = ctx.get_decimal("unused_leave_days")?;
        let rate = config.get_decimal("rate").unwrap_or(Decimal::ONE);

        let daily_rate = if std_days.is_zero() {
            Decimal::ZERO
        } else {
            base / std_days
        };

        let amount = daily_rate * unused_days * rate;

        Ok(vec![
            ("leave_encashment_amount".into(), amount),
            ("leave_encashment_days_paid".into(), unused_days),
        ])
    }
}
```

### 2.2 Unused Leave Balance — VariableProvider

```rust
struct LeaveBalanceProvider {
    leave_repo: Arc<dyn LeaveRepository>,
}

impl VariableProvider for LeaveBalanceProvider {
    fn provider_id(&self) -> &str { "leave_balance" }

    fn provides(&self) -> Vec<VariableRef> {
        vec![
            "unused_leave_days".into(),
            "annual_leave_entitlement".into(),
            "leave_days_used_ytd".into(),
        ]
    }

    fn resolve(
        &self,
        variables: &[VariableRef],
        employment_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<HashMap<VariableRef, Decimal>> {
        let balance = self.leave_repo.get_balance(
            employment_id,
            period_end,  // as-of date for balance calculation
        )?;

        let unused = balance.entitlement
            - balance.used_days
            - balance.pending_days
            + balance.carried_forward;

        Ok(hashmap! {
            "unused_leave_days" => Decimal::from(max(0, unused)),
            "annual_leave_entitlement" => Decimal::from(balance.entitlement),
            "leave_days_used_ytd" => Decimal::from(balance.used_days),
        })
    }
}
```

### 2.3 Encashment Base Configuration

Which salary components are included in the encashment base?

```sql
-- Reuse salary_base_groups from existing schema
INSERT INTO salary_base_groups (name, slug, is_system, workspace_id)
VALUES ('Encashment Base', 'encashment_base', true, :workspace_id);

-- Add components to the group
-- Example: only basic salary, not allowances
INSERT INTO salary_component_base_members (component_definition_id, base_group_id)
SELECT id, :encashment_group_id
FROM salary_component_definitions
WHERE component_type = 'salary' AND include_in_encashment = true;
```

The `SalaryBaseGroup` model (existing in codebase) already supports this. No new tables needed.

### 2.4 Termination Run: Leave Encashment in Context

When a termination run (ADR-010) executes, the template includes:

```
Termination Template Components:
1. ProratedSalary        — salary for days worked this month
2. LeaveEncashment       — unused annual leave payout
3. SeveranceAllowance    — ½ month salary per year of service
4. JobLossAllowance      — if applicable
5. InsuranceDeduction    — final month's BHXH/BHYT/BHTN
6. FinalPIT              — year-to-date PIT reconciliation
7. NetAggregator         — final net payout
```

### 2.5 Voluntary Year-End Encashment

Some companies let employees convert unused leave to cash at year-end:

```sql
CREATE TABLE leave_encashment_requests (
    id UUID PRIMARY KEY,
    employment_id UUID NOT NULL,
    request_year INTEGER NOT NULL,
    days_requested DECIMAL NOT NULL,
    status TEXT DEFAULT 'pending'
        CHECK (status IN ('pending','approved','rejected','processed')),
    encashed_in_run_id UUID REFERENCES payroll_runs(id),
    amount DECIMAL,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now()
);
```

A year-end encashment creates a **separate bonus run** (ADR-010, `run_type = 'bonus'`) with the `LeaveEncashment` component.

---

## 3. Tax & Insurance Treatment

| Treatment | Encashment on Termination | Voluntary Encashment (Still Employed) |
|-----------|:-------------------------:|:-------------------------------------:|
| **PIT taxable?** | ✅ Yes | ✅ Yes (treated as bonus) |
| **BHXH base?** | ❌ No | ❌ No |
| **Accumulator: ytd_taxable_income** | ✅ Yes | ✅ Yes |
| **Accumulator: ytd_bhxh_base** | ❌ No | ❌ No |
| **Payslip section** | "Final Settlement" | "Bonus / Other" |

---

## 4. Edge Cases

| Case | Handling |
|------|----------|
| **Negative leave balance** (employee took more than entitled) | Deduct from final settlement as negative encashment |
| **Leave carried forward from previous year** | Carried-forward days are encashable unless company policy says otherwise |
| **Probation employee** | Pro-rata entitlement (1 day/month of probation, if company policy grants it) |
| **Part-time employee** | Entitlement proportional to working hours ratio |
| **Encashment cap** | Company policy: max X days can be encashed; remainder forfeited |
| **Multiple employment periods** | Encashment calculated per employment period, per employer |

---

## 5. References

- Labor Code 2019, Article 113 — Annual leave entitlement
- Labor Code 2019, Article 48 — Termination settlement
- [ADR-003](ADR-003-payroll-formula-engine.md) — PayrollComponent, VariableProvider
- [ADR-010](ADR-010-off-cycle-payroll-run-types.md) — Termination and bonus run types
- [ADR-006](ADR-006-accumulators-multi-period-context.md) — Accumulator scope (taxable vs insured)
