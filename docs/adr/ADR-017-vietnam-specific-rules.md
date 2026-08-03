# ADR-017: Vietnam-Specific Rules — Probation, PIT Dependents & Minimum Wage

| Property       | Value                                                       |
|----------------|-------------------------------------------------------------|
| **Status**     | Proposed                                                    |
| **Date**       | 2026-08-03                                                  |
| **Author**     | AI + Lee                                                    |
| **Depends on** | [ADR-003](ADR-003-payroll-formula-engine.md), [ADR-006](ADR-006-accumulators-multi-period-context.md), [ADR-007](ADR-007-permissions-overrides-anomalies.md) |
| **Domain**     | Payroll — Vietnam Labor Law Compliance                      |

---

## 1. Context

Three Vietnam-specific rules impact payroll calculation and compliance:

| Rule | Legal Basis | Impact |
|------|------------|--------|
| **Probation salary** | Labor Code 2019, Article 26 | Must be ≥ 85% of full salary |
| **PIT dependents deduction** | Circular 111/2013/TT-BTC | 4.4M VND/dependent/month, accumulated |
| **Regional minimum wage** | Decree 74/2024/NĐ-CP | 4 zones, must not pay below |

These are not architectural decisions per se — they are **compliance rules** implemented as components within the existing architecture. But they are critical enough to document explicitly.

---

## 2. Probation Salary

### 2.1 Rule

- Probation salary ≥ 85% of the full-time salary for the position
- Probation period: max 60 days (standard jobs), max 180 days (specialized/managerial)
- During probation: employer and employee CAN mutually agree to terminate without notice or severance
- Social insurance: **not required** during probation (but some companies voluntarily enroll)

### 2.2 Implementation

Probation is an **employment status**, not a separate payroll component:

```sql
-- Existing employment table already has status
-- employment status = 'probation' triggers special handling

-- Probation salary is stored as a salary component
-- The system enforces the 85% minimum via a validation rule
```

```rust
/// Validation hook (ADR-007) — checks probation salary compliance
struct ProbationSalaryValidationHook;

impl PayrollEventHook for ProbationSalaryValidationHook {
    fn hook_name(&self) -> &str { "probation_salary_validation" }

    fn on_event(&self, event: &PayrollEvent) -> Result<()> {
        if let PayrollEvent::CalculationCompleted { payslip_id, .. } = event {
            let payslip = self.repo.get_payslip(*payslip_id)?;
            let employment = self.repo.get_employment(payslip.employment_id())?;

            if employment.status() == "probation" {
                // Get the "reference salary" (full salary for this position)
                let reference = self.get_reference_salary(&employment)?;

                // Probation salary must be >= 85% of reference
                let min_probation = reference * Decimal::from_str("0.85")?;
                if payslip.base_pay_amount() < min_probation {
                    return Err(anyhow!(
                        "Probation salary ({}) is below 85% of reference salary ({}) — minimum required: {}",
                        payslip.base_pay_amount(), reference, min_probation
                    ));
                }
            }
        }
        Ok(())
    }
}
```

The validation is a `PayrollEventHook` — it fires on calculation complete and **blocks approval** if violated. This is a **warning** for existing employees, **error** for new hires.

---

## 3. PIT Dependents Deduction

### 3.1 Rule

- Each registered dependent reduces monthly taxable income by 4,400,000 VND
- Dependents include: children (<18 or <24 if in school), spouse (no income), elderly parents (no pension), other legally recognized dependents
- The deduction is **cumulative across months**: 4.4M × months worked in tax year
- Registration requires documentation (birth certificate, marriage certificate, etc.)

### 3.2 Data Model

```sql
CREATE TABLE pit_dependents (
    id UUID PRIMARY KEY,
    employee_id UUID NOT NULL,            -- who claims this dependent
    dependent_name TEXT NOT NULL,
    relationship TEXT NOT NULL
        CHECK (relationship IN ('child','spouse','parent','sibling','other')),
    tax_id TEXT,                           -- MST người phụ thuộc (optional)
    deduction_amount DECIMAL NOT NULL DEFAULT 4400000,
    is_active BOOLEAN DEFAULT true,
    effective_from DATE NOT NULL,
    effective_to DATE,                     -- NULL = still valid
    supporting_documents JSONB DEFAULT '[]', -- [{type: "birth_certificate", file_id: "..."}]
    registered_with_tax_authority BOOLEAN DEFAULT false,
    tax_registration_date DATE,
    created_at TIMESTAMPTZ DEFAULT now()
);
```

### 3.3 Accumulator Integration

The dependent deduction is an **accumulator** (ADR-006):

```rust
// accumulator: ytd_dependent_deduction
// Each period: running_total += dependent_count × 4,400,000
// Used by PIT component: taxable_income -= ytd_dependent_deduction
```

The PIT withholding component uses this accumulator:

```rust
impl PayrollComponent for PitWithholding {
    fn evaluate(&self, config: &ComponentConfig, ctx: &VariableContext) -> Result<Vec<(VariableRef, Decimal)>> {
        let gross = ctx.get_decimal("gross_salary")?;
        let bik = ctx.get_decimal("bik_total_taxable").unwrap_or(Decimal::ZERO);
        let self_deduction = Decimal::from(11_000_000); // 11M/month for self

        // Dependents: accumulated deduction
        let ytd_dependent_deduction = ctx.get_accumulator("ytd_dependent_deduction");
        let period_dependent_deduction = self.calculate_period_dependent_deduction(
            ctx.employment_id,
            ctx.period_end,
        )?;

        // Taxable income
        let taxable = gross + bik
            - self_deduction
            - period_dependent_deduction;

        // Progressive bracket calculation...
        // ...
    }
}
```

### 3.4 Multi-Source Dependent Registration

Vietnam law allows dependents to be claimed by only ONE taxpayer. If both parents work, only one can claim the child:

```rust
/// Cross-employee validation: same dependent can't be claimed by two employees
struct DependentUniquenessCheck;

impl PostProcessStep for DependentUniquenessCheck {
    fn execute(&self, period: &PayrollPeriod, all_payslips: &[Payslip]) -> Result<PostProcessOutput> {
        // Check: no two active employees claim the same tax_id
        // Raise anomaly (ADR-007) if duplicate found
        // ...
    }
}
```

---

## 4. Regional Minimum Wage Compliance

### 4.1 Rule

Vietnam has 4 regional minimum wage zones (Decree 74/2024/NĐ-CP, effective July 1, 2024):

| Zone | Monthly Minimum (VND) | Areas |
|------|:--------------------:|-------|
| Zone I | 4,960,000 | HCMC, Hanoi urban districts |
| Zone II | 4,410,000 | Hanoi rural, Da Nang, Hai Phong urban |
| Zone III | 3,860,000 | Provincial cities |
| Zone IV | 3,450,000 | Rural areas |

- Salary must be **at least** the regional minimum for the workplace location
- For **trained workers** (vocational certificate+), salary must be **at least 7% above** the minimum
- The minimum is for **gross salary**, not net

### 4.2 Data Model

```sql
CREATE TABLE regional_minimum_wages (
    id UUID PRIMARY KEY,
    zone TEXT NOT NULL CHECK (zone IN ('I','II','III','IV')),
    effective_from DATE NOT NULL,
    effective_to DATE,
    monthly_minimum DECIMAL NOT NULL,
    trained_worker_bonus_percent DECIMAL DEFAULT 7.0,
    legal_basis TEXT NOT NULL,              -- "Nghị định 74/2024/NĐ-CP"
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Associate employment with a zone (via workplace location)
ALTER TABLE employments
    ADD COLUMN minimum_wage_zone TEXT CHECK (zone IN ('I','II','III','IV'));
```

### 4.3 Compliance Validation

```rust
struct MinimumWageValidationHook;

impl PayrollEventHook for MinimumWageValidationHook {
    fn on_event(&self, event: &PayrollEvent) -> Result<()> {
        if let PayrollEvent::CalculationCompleted { payslip_id, .. } = event {
            let payslip = self.repo.get_payslip(*payslip_id)?;
            let employment = self.repo.get_employment(payslip.employment_id())?;
            let zone = employment.minimum_wage_zone()?;
            let is_trained = employment.is_trained_worker();

            let min_wage = self.repo.get_minimum_wage(&zone, payslip.period_end())?;
            let required_min = if is_trained {
                min_wage * Decimal::from_str("1.07")?
            } else {
                min_wage
            };

            if payslip.base_pay_amount() < required_min {
                return Err(anyhow!(
                    "Gross salary ({}) is below Zone {} minimum wage ({})",
                    payslip.base_pay_amount(), zone, required_min
                ));
            }
        }
        Ok(())
    }
}
```

This is a **blocking validation** — period cannot be approved if any employee is below minimum wage.

### 4.4 Minimum Wage Updates

When the government updates minimum wages (typically annually on July 1):

1. Admin updates `regional_minimum_wages` table with new decree
2. System automatically applies the new minimum from `effective_from`
3. Template version (ADR-003) is not affected — min wage is **input data**, not formula
4. If the update is backdated (rare but possible), it triggers retro (ADR-005)

### 4.5 Edge Case: Multi-Location Employees

An employee working at multiple sites (e.g., salesperson covering multiple provinces):

```rust
fn resolve_minimum_wage_zone(employment: &Employment) -> Zone {
    // Priority:
    // 1. Explicit zone override on employment record
    // 2. Primary work location (office/branch address)
    // 3. Company headquarters zone (fallback)
}
```

---

## 5. Integration Summary

| Rule | Mechanism | ADR Used |
|------|-----------|----------|
| Probation ≥85% salary | `PayrollEventHook` (validation) | ADR-007 |
| PIT dependents | `VariableProvider` + `Accumulator` | ADR-003 + ADR-006 |
| Minimum wage | `PayrollEventHook` (blocking validation) | ADR-007 |
| Zone/rate updates | Input data versioning (not template) | ADR-003 (config vs input distinction) |

All three rules are implemented using existing architectural primitives — no new traits or patterns needed.

---

## 6. References

- Labor Code 2019, Article 26 — Probation
- Labor Code 2019, Article 90-91 — Minimum wage
- Circular 111/2013/TT-BTC — PIT dependents
- Decree 74/2024/NĐ-CP — Regional minimum wage (current)
- [ADR-003](ADR-003-payroll-formula-engine.md) — Component pipeline, hooks
- [ADR-006](ADR-006-accumulators-multi-period-context.md) — Accumulator model
- [ADR-007](ADR-007-permissions-overrides-anomalies.md) — Validation hooks
