# ADR-006: Accumulators & Multi-Period Variable Context

| Property       | Value                                                       |
|----------------|-------------------------------------------------------------|
| **Status**     | Proposed                                                    |
| **Date**       | 2026-08-03                                                  |
| **Author**     | AI + Lee                                                    |
| **Depends on** | [ADR-003](ADR-003-payroll-formula-engine.md)               |
| **Domain**     | Payroll Calculation — Variable Context                      |

---

## 1. Context & Problem Statement

### 1.1 The Single-Period Blind Spot

ADR-003 defines a `VariableContext` that resolves variables for **one payroll period at a time**. This works for simple calculations:

```
VariableContext (February 2026):
  gross_salary    = 15,000,000
  std_units       = 22
  ot_units        = 10
  unpaid_days     = 1

  → BHXH = gross * 8% = 1,200,000 ✅
  → Net = ... ✅
```

But many real-world calculations require **knowledge of previous periods**:

| Calculation | Needs | Variable |
|------------|-------|----------|
| BHXH cap | Has total BHXH base this year hit 20× regional minimum wage? | `ytd_bhxh_base` |
| PIT progressive brackets | What tax bracket are we in based on year-to-date taxable income? | `ytd_taxable_income` |
| PIT bracket exhaustion | Has this bracket's limit already been consumed in prior months? | `ytd_tax_by_bracket` |
| Annual leave entitlement | How many paid leave days have been used this year? | `ytd_paid_leave_used` |
| Annual bonus accrual | What's the year-to-date bonus reserve? | `ytd_bonus_accrual` |

### 1.2 Vietnamese-Specific Requirements

Vietnam's payroll system is particularly accumulator-heavy:

| Accumulator | Legal Basis | Cap/Threshold |
|------------|-------------|---------------|
| BHXH base (social insurance) | Decision 595/QĐ-BHXH | 20 × regional minimum wage × number of months worked |
| BHYT base (health insurance) | Same as above | Same cap |
| BHTN base (unemployment) | Law on Employment | 20 × regional minimum wage (for the region) |
| PIT taxable income | Law on Personal Income Tax | Progressive brackets: 5M, 10M, 18M, 32M, 52M, 80M (monthly) |
| PIT dependents deduction | Circular 111/2013/TT-BTC | 4.4M/dependent/month, accumulated across months |
| Trade union fee | Law on Trade Union | 1% of gross salary, no cap (but deducted from net, not taxable base) |

### 1.3 Current State

The system today has **no accumulator infrastructure**. The `VariableContext` (in ADR-003) is designed for a single period. The current `payslip_calc.rs` hardcodes base values like `bhxh_base` as simple totals from the current period, without considering year-to-date caps. This works for simple cases but fails when:
- An employee's cumulative BHXH base exceeds the cap mid-year
- PIT bracket progression requires knowing cumulative taxable income

---

## 2. Architecture Decision

### 2.1 Core Concept: Accumulator

An **accumulator** is a named variable whose value **accumulates across periods** within a defined scope (typically a fiscal year).

```
┌────────────────────────────────────────────────────────────┐
│                  ACCUMULATOR MODEL                          │
│                                                             │
│  Accumulator: ytd_bhxh_base                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Scope:    Fiscal Year 2026 (Jan 1 – Dec 31)          │   │
│  │ Owner:    Employment E001                             │   │
│  │                                                       │   │
│  │ Jan:  bhxh_base = 15,000,000 → ytd = 15,000,000      │   │
│  │ Feb:  bhxh_base = 15,000,000 → ytd = 30,000,000      │   │
│  │ Mar:  bhxh_base = 15,000,000 → ytd = 45,000,000      │   │
│  │ ...                                                   │   │
│  │ Nov:  bhxh_base = 15,000,000 → ytd = 165,000,000     │   │
│  │       └── BUT cap = 20 × 4,960,000 = 99,200,000 ✋    │   │
│  │       └── Actual BHXH = cap - prior_bhxh_ytd          │   │
│  │       └── Nov BHXH = 99,200,000 - 150,000,000 = 0     │   │
│  │           (cap already reached in Oct)                 │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

### 2.2 Accumulator Storage

Accumulators are stored as **pre-computed snapshots** at the end of each period, NOT re-computed from all prior payslips every time.

```sql
CREATE TABLE payroll_accumulators (
    id UUID PRIMARY KEY,
    employment_id UUID NOT NULL,
    scope_type TEXT NOT NULL        -- 'fiscal_year', 'calendar_year', 'lifetime'
        CHECK (scope_type IN ('fiscal_year','calendar_year','lifetime')),
    scope_key TEXT NOT NULL,        -- e.g., '2026' for fiscal year 2026
    accumulator_code TEXT NOT NULL, -- e.g., 'ytd_bhxh_base', 'ytd_taxable_income'
    
    -- The running total at the END of this period
    running_total DECIMAL NOT NULL DEFAULT 0,
    
    -- The increment added in this specific period
    period_increment DECIMAL NOT NULL DEFAULT 0,
    
    -- Which payslip produced this increment (for audit)
    period_year INTEGER NOT NULL,
    period_month INTEGER NOT NULL,
    payslip_id UUID REFERENCES payslips(id),
    
    -- For PIT: per-bracket breakdown
    detail JSONB DEFAULT '{}',
    -- Example: {"bracket_1": {"limit": 5000000, "consumed": 3000000, "remaining": 2000000},
    --           "bracket_2": {"limit": 5000000, "consumed": 0, "remaining": 5000000}, ...}
    
    created_at TIMESTAMPTZ DEFAULT now(),
    
    -- One row per (employment, scope_type, scope_key, accumulator_code, period)
    UNIQUE (employment_id, scope_type, scope_key, accumulator_code, period_year, period_month)
);

CREATE INDEX idx_payroll_accumulators_lookup
    ON payroll_accumulators (employment_id, scope_type, scope_key, accumulator_code, period_year, period_month);
```

### 2.3 Why Store Snapshots Instead of Computing From Payslips?

| Approach | Pros | Cons |
|----------|------|------|
| **Compute from payslips** | No storage overhead, always consistent | O(N) per period: scan all prior payslips. Month 12 scans 11 months × 5000 employees = expensive |
| **Store snapshots** | O(1) lookup: read last period's running_total | Must maintain consistency with payslips |

**Decision: Store snapshots, with reconciliation.**

The snapshot approach is what Oracle and ADP use (they call them "balances" and "accumulators" respectively). The trade-off of maintaining consistency is managed by:
1. Snapshot is written atomically with the payslip in the same transaction
2. Retro recalculation rewrites the snapshot chain (see §2.6)

### 2.4 Accumulator Definitions

```sql
CREATE TABLE payroll_accumulator_definitions (
    id UUID PRIMARY KEY,
    workspace_id UUID,
    template_version_id UUID REFERENCES payroll_template_versions(id),
    accumulator_code TEXT NOT NULL,
    accumulator_name TEXT NOT NULL,       -- i18n key
    scope_type TEXT NOT NULL DEFAULT 'fiscal_year',
    
    -- Which component output feeds this accumulator
    source_variable TEXT NOT NULL,        -- e.g., 'bhxh_base', 'taxable_income'
    
    -- Optional: cap expression
    cap_expression TEXT,                  -- e.g., '20 * regional_min_wage * months_worked'
    cap_variables JSONB,                  -- variables needed: ['regional_min_wage', 'months_worked']
    
    -- Whether this accumulator affects downstream calculations
    is_constraining BOOLEAN DEFAULT false, -- true = cap/bracket limits apply
    
    created_at TIMESTAMPTZ DEFAULT now()
);
```

**Built-in accumulators for Vietnam template:**

| Code | Source Variable | Scope | Cap? | Purpose |
|------|----------------|-------|------|---------|
| `ytd_bhxh_base` | `bhxh_base` | fiscal_year | Yes: 20×MW×months | Determine if BHXH cap reached |
| `ytd_bhxh_paid` | `bhxh_deduction` | fiscal_year | No | Track total BHXH paid (employer+employee) |
| `ytd_bhyt_base` | `bhyt_base` | fiscal_year | Yes: 20×MW×months | Determine if BHYT cap reached |
| `ytd_bhtn_base` | `bhtn_base` | fiscal_year | Yes: 20×regional_MW | Determine if BHTN cap reached |
| `ytd_taxable_income` | `taxable_income` | fiscal_year | No | PIT bracket determination |
| `ytd_pit_paid` | `pit_deduction` | fiscal_year | No | Track total PIT paid |
| `ytd_gross_income` | `gross_salary` | fiscal_year | No | Reporting, 13th month calc |
| `ytd_dependent_deduction` | `dependent_deduction` | fiscal_year | No | PIT dependents accumulation |
| `ytd_paid_leave_used` | `paid_leave_days` | calendar_year | No | Leave entitlement tracking |
| `ytd_net_pay` | `net_salary` | fiscal_year | No | Reporting |

### 2.5 Accumulator Resolution in VariableContext

ADR-003's `VariableContext` is extended to support accumulator variables:

```rust
struct VariableContext {
    // Single-period variables (from ADR-003)
    period_values: HashMap<VariableRef, Decimal>,

    // Accumulator values — resolved from prior period snapshots
    accumulators: HashMap<AccumulatorRef, AccumulatorState>,
}

struct AccumulatorState {
    /// Running total at the START of this period (end of last period)
    opening_balance: Decimal,

    /// Cap value for this period, if applicable (evaluated from cap_expression)
    cap_this_period: Option<Decimal>,

    /// Detailed state (e.g., per-bracket consumption for PIT)
    detail: serde_json::Value,
}

impl VariableContext {
    /// Get the running total of an accumulator before this period
    fn get_accumulator(&self, code: &str) -> Decimal {
        self.accumulators
            .get(code)
            .map(|a| a.opening_balance)
            .unwrap_or(Decimal::ZERO)
    }

    /// Check if an accumulator has remaining capacity
    fn accumulator_remaining_capacity(&self, code: &str) -> Option<Decimal> {
        self.accumulators.get(code).and_then(|a| {
            a.cap_this_period.map(|cap| {
                let remaining = cap - a.opening_balance;
                if remaining < Decimal::ZERO { Decimal::ZERO } else { remaining }
            })
        })
    }
}
```

### 2.6 Accumulator Lifecycle During Retro

When a period is recalculated via retro (ADR-005), the accumulator chain for that employment must be **rewritten from the recalculated period forward**:

```
Original chain:
  Jan: ytd_bhxh = 15M → Feb: ytd_bhxh = 30M → Mar: ytd_bhxh = 45M

Retro recalculates February (salary increased from 15M to 18M):
  Jan: ytd_bhxh = 15M (UNCHANGED ✅)
  Feb: ytd_bhxh = 33M (REWRITTEN: 15M + 18M)
  Mar: ytd_bhxh = 48M (REWRITTEN: 33M + 15M → cascade!)
```

This is handled by the cascade mechanism in ADR-005: the retro engine detects that `ytd_bhxh` changed at the end of February → flags March for recalculation → rewrites March's accumulator snapshot.

### 2.7 Accumulator-Scoped Cap in Components

Components can query accumulator state to apply caps:

```rust
impl PayrollComponent for InsuranceDeduction {
    fn evaluate(&self, config: &ComponentConfig, ctx: &VariableContext) -> Result<Vec<(VariableRef, Decimal)>> {
        let rate = config.get_decimal("rate")?;
        let base_variable = config.get_string("base_variable")?;
        let accumulator_code = config.get_string("accumulator")?;

        let gross = ctx.get_decimal(&base_variable)?;
        let uncapped = gross * rate;

        // Check accumulator cap
        let remaining = ctx.accumulator_remaining_capacity(&accumulator_code);

        let actual = match remaining {
            Some(cap) => std::cmp::min(uncapped, cap),
            None => uncapped, // No cap configured
        };

        // Publish both uncapped (for accumulator tracking) and actual (for payslip)
        Ok(vec![
            (format!("{}_uncapped", self.output_name()), uncapped),
            (self.output_name(), actual),
        ])
    }
}
```

---

## 3. Interaction with ADR-003, ADR-004, ADR-005

| Interaction | Mechanism |
|------------|-----------|
| **ADR-003 Component Pipeline** | Components read `ctx.get_accumulator()` for cap decisions; publish outputs that feed accumulators |
| **ADR-004 Lifecycle** | Accumulator snapshots are written in the same transaction as payslip commit |
| **ADR-005 Retro Engine** | Retro cascade is gated by "did accumulators change?"; rewriting snapshots is part of retro execution |

---

## 4. Performance Considerations

| Concern | Approach |
|---------|----------|
| **Month 12 reading 11 prior periods** | Single row lookup: `SELECT running_total FROM payroll_accumulators WHERE ... ORDER BY period_year DESC, period_month DESC LIMIT 1` |
| **Batch calculation for 5000 employees** | Accumulators are loaded once per employment in the fetch phase (1 query with `WHERE employment_id IN (...)` + `ORDER BY ... LIMIT 1` per accumulator code) |
| **Retro cascade rewrites** | Each period rewrite is 1 UPDATE or INSERT; 5000 employees × 6 months cascade = 30K writes (acceptable for an infrequent operation) |
| **Snapshot consistency** | Written in same transaction as payslip; no eventual consistency issues |

---

## 5. Risks & Trade-offs

| Risk | Mitigation |
|------|-----------|
| **Snapshot drifts from payslip reality** | Reconciliation query: `SUM(payslip component) vs snapshot.running_total`; alert on mismatch |
| **Fiscal year boundary logic** | Scope is explicit (`scope_type`, `scope_key`); no heuristics |
| **New accumulator added mid-year** | Backfill from existing payslip data via migration; new periods auto-populate |
| **Divergence between VN fiscal year (Jan-Dec) and client FY (Apr-Mar)** | `scope_type` and `scope_key` are configurable per template |

---

## 6. References

- [ADR-003: Payroll Formula Engine](ADR-003-payroll-formula-engine.md) — VariableContext design
- [ADR-005: Retroactive Processing](ADR-005-retroactive-processing-engine.md) — Cascade mechanism
- Oracle HCM: "Payroll Balances" — accumulator snapshot pattern
- Decision 595/QĐ-BHXH — Vietnam social insurance contribution regulations
- Law on Personal Income Tax No. 04/2007/QH12 — progressive tax brackets
