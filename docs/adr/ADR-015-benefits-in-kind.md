# ADR-015: Benefits-in-Kind (BIK)

| Property       | Value                                                       |
|----------------|-------------------------------------------------------------|
| **Status**     | Proposed                                                    |
| **Date**       | 2026-08-03                                                  |
| **Author**     | AI + Lee                                                    |
| **Depends on** | [ADR-003](ADR-003-payroll-formula-engine.md), [ADR-006](ADR-006-accumulators-multi-period-context.md) |
| **Domain**     | Payroll — Non-Cash Compensation                             |

---

## 1. Context

### 1.1 What is BIK?

Benefits-in-Kind are non-cash benefits provided to employees that Vietnamese tax law considers **taxable income**:

| Benefit | Typical Value | Tax Treatment |
|---------|--------------|---------------|
| **Company car (personal use)** | 5-15M/month (depends on car value) | 100% taxable |
| **Housing provided by employer** | Actual rent paid | ≤15% of gross salary → tax exempt; excess → taxable |
| **Electricity/water paid by employer** | Actual bill | 100% taxable at fixed rate |
| **Personal phone/device** | Varies | Tax-free if for work use; taxable if personal |
| **Club/gym membership** | Varies | 100% taxable |
| **Children's school fees** | Actual fees paid | 100% taxable (unless part of expat package with specific rules) |
| **Home leave travel (expats)** | Flight tickets | 1 round trip/year → tax exempt |
| **Meal allowance** | Up to 730K/month | ≤730K → tax exempt; excess → taxable |
| **Uniform/protective clothing** | Actual cost | Tax exempt (in-kind, not cash) |

### 1.2 Why BIK Matters for Payroll

BIK is NOT cash — the employee never sees it in their bank account. But it IS taxable income. This creates a paradox:

```
Anh B receives:
  Cash salary:     15,000,000 VND  (taxable)
  Company car:      8,000,000 VND  (taxable — but not in his pocket)
  Housing:         12,000,000 VND  (taxable — but employer pays landlord directly)
  ─────────────────────────────────
  Taxable income:  35,000,000 VND
  Net cash:        15,000,000 - BHXH - PIT  ← PIT is higher because of BIK!
  
  ⚠️  Net cash could be NEGATIVE after tax if BIK is huge and cash salary is small
```

---

## 2. Architecture Decision

### 2.1 BIK as a VariableProvider

BIK values are **input data**, not calculation results. They are resolved by a dedicated `VariableProvider`:

```rust
struct BikVariableProvider {
    bik_repo: Arc<dyn BikRepository>,
}

impl VariableProvider for BikVariableProvider {
    fn provider_id(&self) -> &str { "bik" }

    fn provides(&self) -> Vec<VariableRef> {
        vec![
            "bik_car".into(),
            "bik_housing".into(),
            "bik_electricity_water".into(),
            "bik_phone".into(),
            "bik_club".into(),
            "bik_school_fees".into(),
            "bik_meal".into(),
            "bik_home_leave".into(),
            "bik_total_taxable".into(),    // pre-computed total
            "bik_total_exempt".into(),      // pre-computed exempt portion
        ]
    }

    fn resolve(
        &self,
        variables: &[VariableRef],
        employment_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<HashMap<VariableRef, Decimal>> {
        let biks = self.bik_repo.get_for_period(employment_id, period_start, period_end)?;
        let mut values = HashMap::new();

        let mut total_taxable = Decimal::ZERO;
        let mut total_exempt = Decimal::ZERO;

        for bik in &biks {
            let (taxable, exempt) = bik.tax_split();
            values.insert(VariableRef::from(bik.code()), taxable + exempt);
            total_taxable += taxable;
            total_exempt += exempt;
        }

        values.insert("bik_total_taxable".into(), total_taxable);
        values.insert("bik_total_exempt".into(), total_exempt);

        Ok(values)
    }
}
```

### 2.2 BIK Data Model

```sql
CREATE TABLE employee_biks (
    id UUID PRIMARY KEY,
    employment_id UUID NOT NULL,
    bik_type TEXT NOT NULL
        CHECK (bik_type IN ('car','housing','electricity_water','phone',
                            'club','school_fees','home_leave','meal','uniform','other')),
    bik_label TEXT NOT NULL,              -- "Xe Toyota Camry 2024"
    
    -- Value
    gross_value DECIMAL NOT NULL,         -- Total value of the benefit
    currency TEXT DEFAULT 'VND',
    
    -- Tax treatment
    taxability TEXT NOT NULL DEFAULT 'fully_taxable'
        CHECK (taxability IN ('fully_taxable','fully_exempt','partially_exempt')),
    exemption_rule TEXT,                  -- JSON: {"type": "percentage_of_salary", "value": 15}
    exempt_amount DECIMAL DEFAULT 0,      -- Pre-computed exempt portion
    
    -- Period
    effective_from DATE NOT NULL,
    effective_to DATE,
    
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Pre-built exemption rules
CREATE TABLE bik_exemption_rules (
    id UUID PRIMARY KEY,
    rule_code TEXT NOT NULL UNIQUE,        -- 'housing_15pct_gross', 'meal_730k'
    bik_type TEXT NOT NULL,
    exemption_formula TEXT NOT NULL,       -- 'MIN(value, gross_salary * 0.15)'
    max_exempt_amount DECIMAL,            -- 730000 for meal
    legal_basis TEXT,                      -- "Circular 111/2013/TT-BTC, Article 11"
    is_active BOOLEAN DEFAULT true
);
```

### 2.3 Tax Split Logic

```rust
impl EmployeeBik {
    fn tax_split(&self) -> (Decimal, Decimal) {  // (taxable, exempt)
        match self.taxability {
            "fully_taxable" => (self.gross_value, Decimal::ZERO),
            "fully_exempt" => (Decimal::ZERO, self.gross_value),
            "partially_exempt" => {
                match self.bik_type {
                    "housing" => {
                        // Exempt: min(actual_rent, 15% of gross salary)
                        let cap = self.employee_gross * Decimal::from_str("0.15")?;
                        let exempt = std::cmp::min(self.gross_value, cap);
                        (self.gross_value - exempt, exempt)
                    }
                    "meal" => {
                        // Exempt: min(actual, 730,000 VND/month)
                        let cap = Decimal::from(730_000);
                        let exempt = std::cmp::min(self.gross_value, cap);
                        (self.gross_value - exempt, exempt)
                    }
                    "home_leave" => {
                        // Exempt: 1 round trip/year (pro-rata per period)
                        // Implementation depends on company policy
                        // ...
                    }
                    _ => (self.gross_value, Decimal::ZERO) // default: fully taxable
                }
            }
        }
    }
}
```

### 2.4 Impact on the Calculation Pipeline

BIK values feed into the pipeline like any other VariableProvider output:

```
VariableContext:
  gross_salary      = 15,000,000 (from SalaryProvider)
  bik_car           =  8,000,000 (from BikProvider)
  bik_housing       = 12,000,000 (from BikProvider)
  bik_total_taxable = 17,750,000 (from BikProvider, after exemptions)
  ─────────────────────────────────
  
  → DeductionComponents read these variables:
    taxable_income = gross_salary + bik_total_taxable
                   = 15,000,000 + 17,750,000
                   = 32,750,000

  → BHXH base: ONLY cash salary (BIK is not insured)
    bhxh_base = gross_salary = 15,000,000

  → PIT base: INCLUDES BIK (it's taxable income)
    pit_base = taxable_income - dependents_deduction - own_deduction
```

Key distinction: **BIK is taxable but NOT insured.** The `bhxh_base` accumulator (ADR-006) only includes cash salary components, not BIK variables.

### 2.5 BIK on Payslip

BIK appears on the payslip (ADR-011) but NOT in the net pay calculation:

```
┌─────────────────────────────────────────────────────┐
│  PHIẾU LƯƠNG - Tháng 02/2026                         │
│                                                      │
│  THU NHẬP                                           │
│  Lương cơ bản                    15,000,000          │
│  Phụ cấp đi lại                   3,000,000          │
│  ─────────────────────────────────────────           │
│  Tổng thu nhập tiền mặt          18,000,000          │
│                                                      │
│  PHÚC LỢI (thu nhập chịu thuế)                       │
│  Xe công ty                       8,000,000          │
│  Nhà ở (miễn thuế: -2,250,000)    9,750,000          │
│  ─────────────────────────────────────────           │
│  Tổng thu nhập chịu thuế          35,750,000          │
│                                                      │
│  KHẤU TRỪ                                           │
│  BHXH (8% × 18,000,000)           -1,440,000          │
│  Thuế TNCN                        -3,200,000          │
│  ─────────────────────────────────────────           │
│  THỰC NHẬN                       13,360,000          │
└─────────────────────────────────────────────────────┘
```

Notice: BIK increases PIT but doesn't add to net pay. The employee receives 13.36M in cash — the rest is the implicit value of benefits.

---

## 3. Integration

| ADR | How BIK Interacts |
|-----|-------------------|
| ADR-003 | BIK is a VariableProvider; no pipeline changes |
| ADR-006 | BIK feeds `ytd_taxable_income` accumulator but NOT `ytd_bhxh_base` |
| ADR-007 | BIK values are visible only to `payroll:sensitive:read` permission |
| ADR-011 | BIK displayed on payslip as informational, not affecting net pay |
| ADR-009 | BIK may generate GL entries: Nợ 642 / Có 334 (if BIK is accounted as salary cost) |

---

## 4. Risks & Trade-offs

| Risk | Mitigation |
|------|-----------|
| **Negative net pay due to BIK** | Validation rule: net_cash >= 0 after all deductions. If negative, warn admin. |
| **BIK value disputes** | BIK valuation rules must be documented per Circular 111. System shows legal basis on payslip. |
| **Employer also incurs cost for BIK** | BIK cost feeds into employer cost reports; separate from payroll but linked. |
| **Mid-year BIK changes** | Effective dating like salary components |

---

## 5. References

- Circular 111/2013/TT-BTC — Vietnam PIT law: BIK classification and valuation
- Circular 92/2015/TT-BTC — Amendments to PIT on BIK
- [ADR-003](ADR-003-payroll-formula-engine.md) — VariableProvider
- [ADR-011](ADR-011-payslip-generation-reporting.md) — Payslip layout
