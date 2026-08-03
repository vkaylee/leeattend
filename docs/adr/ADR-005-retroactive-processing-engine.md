# ADR-005: Retroactive Processing Engine

| Property       | Value                                                       |
|----------------|-------------------------------------------------------------|
| **Status**     | Proposed                                                    |
| **Date**       | 2026-08-03                                                  |
| **Author**     | AI + Lee                                                    |
| **Depends on** | [ADR-003](ADR-003-payroll-formula-engine.md), [ADR-004](ADR-004-payroll-lifecycle-state-machine.md) |
| **Domain**     | Payroll Retroactive Processing                              |

---

## 1. Context & Problem Statement

### 1.1 What is Retroactive Processing?

When payroll data or formulas change AFTER a period has been calculated, the payslips for that period may be incorrect. Retroactive processing is the mechanism to correct them — NOT by editing the old payslip, but by recalculating with the corrected data/formula and reconciling the difference.

### 1.2 Scenarios (Ranked by Frequency)

| # | Scenario | Frequency | Impact Radius |
|---|----------|-----------|---------------|
| 1 | **Missing OT/attendance** — OT on the 25th wasn't recorded, discovered on the 5th of next month | Weekly | Single employee, single period |
| 2 | **Late-approved leave** — Unpaid leave on the 10th, approved on the 20th, after payslip calculated | Weekly | Single employee, single period |
| 3 | **Backdated salary change** — Promotion decision signed on March 10, effective February 15 | Monthly | Single employee, may cascade |
| 4 | **Formula error** — Admin configured BHXH rate as 8.5% instead of 8%, discovered after 3 months | Rare | All employees, all affected periods |
| 5 | **Regulatory change with retro effect** — Government decree on March 15, effective January 1 | Annual | Entire workspace, all periods |

### 1.3 Why This Is Hard

The difficulty is not recalculating one period — it's the **cascade effect**. Changing a value in period T can change accumulator totals (year-to-date taxable income, year-to-date social insurance base), which change the calculation for periods T+1, T+2, ..., up to the current period.

### 1.4 Industry Patterns

| System | Retro Mechanism |
|--------|----------------|
| Oracle HCM | Retroactive Events → Retro Pay Result (delta payslip) |
| ADP Workforce Now | Void & Recalculate OR Manual Adjustment |
| Workday | Retroactive Payroll Processing (replacement + delta) |
| SAP SuccessFactors | Off-cycle payroll with retroactive accounting |

All four support **multiple retro patterns** chosen based on period state:

| Pattern | When | What Happens |
|---------|------|-------------|
| **Void & Recalculate** | Pre-commit, simple change | Delete (soft) old payslip, recalculate, create new |
| **Delta Adjustment** | Post-commit, small change | Keep old payslip, calculate difference, add adjustment to current period |
| **Replacement Chain** | Post-commit, major change | Old payslip → superseded; new payslip created; delta settled separately |

---

## 2. Architecture Decision

### 2.1 Core Principle: Retro is an Orchestration Layer, Not a Calculation Layer

The **calculation engine** (ADR-003) already supports retro by design:
- Template versioning guarantees the correct formula is used for each period
- Component `evaluate()` is a pure function — safe to rerun with different inputs
- VariableProvider resolves data as-of the period date

What we need is an **orchestration layer** that:
1. Detects WHAT needs recalculating
2. Predicts HOW FAR the cascade extends
3. Presents IMPACT to the admin for confirmation
4. EXECUTES the recalculation across affected periods
5. RECONCILES differences (insurance, tax, net pay)

```
┌─────────────────────────────────────────────────────────────┐
│                  RETROACTIVE ENGINE                          │
│                                                              │
│  ┌──────────┐   ┌─────────────┐   ┌──────────┐             │
│  │ DETECT   │──→│   ANALYZE   │──→│  CONFIRM  │             │
│  │ what     │   │  how far?   │   │  preview  │             │
│  │ changed  │   │  how much?  │   │  ask user │             │
│  └──────────┘   └─────────────┘   └────┬─────┘             │
│                                        │                     │
│                          ┌─────────────▼───────────┐        │
│                          │       EXECUTE            │        │
│                          │  For each affected period:│       │
│                          │  1. Choose pattern        │        │
│                          │  2. Recalculate           │        │
│                          │  3. Update state          │        │
│                          │  4. Check cascade         │        │
│                          └─────────────┬───────────┘        │
│                                        │                     │
│                          ┌─────────────▼───────────┐        │
│                          │      RECONCILE           │        │
│                          │  • Insurance adjustments │        │
│                          │  • Tax adjustments       │        │
│                          │  • Net pay delta         │        │
│                          └──────────────────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Detection: What Changed?

The engine needs to know WHAT changed to determine WHICH periods to recalculate. This is fundamentally about comparing input data at two points in time.

```rust
enum RetroTrigger {
    /// Admin explicitly selects periods to recalculate
    Manual {
        requested_by: Uuid,
        employment_ids: Vec<Uuid>,
        periods: Vec<(i32, u32)>,   // [(year, month), ...]
        reason: String,
    },

    /// Data change detected after period was calculated
    DataChange {
        employment_id: Uuid,
        change_type: DataChangeType,
        affected_date: NaiveDate,
        detected_at: DateTime<Utc>,
    },

    /// Formula/template version change with retroactive effective date
    FormulaChange {
        template_version_id: Uuid,
        effective_from: NaiveDate,  // may be backdated
    },
}

enum DataChangeType {
    AttendanceChanged,         // new/missing punch records
    LeaveRequestApproved,      // leave approved after period calculated
    SalaryComponentChanged,   // salary change with backdated effective date
    OvertimeRequestApproved,  // OT approved late
}
```

**Auto-detection** uses a **data hash** per employment per period:

```sql
-- Stored after initial calculation, compared on data changes
ALTER TABLE payroll_period_employment_state
    ADD COLUMN input_data_hash TEXT;  -- SHA-256 of all input values

-- Detection: when attendance/leave/salary data for employment E in period P
-- is modified, compare new hash with stored hash. If different → flag for retro.
```

### 2.3 Cascade Analysis

The critical insight: **a retro cascade STOPS when accumulator values at the end of a period don't change.**

```
Recalculate period T:
  → Component outputs for T change
  → Accumulators at end of T (ytd_taxable_income, ytd_bhxh_base, ...) 
      are recalculated:
  
  ┌───────────────────────────────────────────────────────────┐
  │  Acc(T) compares: old_ytd_taxable vs new_ytd_taxable       │
  │                                                             │
  │  IF difference = 0  → NO CASCADE → DONE ✅                  │
  │  IF difference ≠ 0  → Flag period T+1 for recalculation     │
  │                       → Continue to T+1                     │
  │                       → Repeat until:                       │
  │                         a) Acc stops changing, OR            │
  │                         b) We reach the current period       │
  └───────────────────────────────────────────────────────────┘
```

This is efficient because:
- Most retro events (missing 1 day of OT, correcting 1 leave request) produce tiny changes
- Accumulators for insurance and tax operate on large aggregates
- A single missing OT day in a 20M/month salary changes BHXH base by ~0.5% — nowhere near enough to change tax brackets or insurance caps
- **Result: cascade stops after 1 period for 95% of retro events**

### 2.4 Pattern Selection Matrix

The pattern to use depends on the period's current lifecycle state:

| Period State | Pattern | Payslip Action | Delta Settlement |
|-------------|---------|----------------|-----------------|
| `calculated` / `verified` | Void & Recalculate | Old → voided; New → draft | N/A (nothing committed yet) |
| `approved` (pre-commit) | Void & Recalculate | Old → voided; New → draft | N/A |
| `committed` | Delta Adjustment | Old unchanged | Create adjustment in current period |
| `committed` (major change) | Replacement | Old → superseded; New → draft → commits | Delta in current period |
| `closed` | ⛔ Only via superadmin + Delta | Manual with audit | Manual |

```rust
fn select_pattern(
    period_status: &str,
    total_delta_percentage: Decimal,
    crosses_tax_bracket: bool,
) -> RetroPattern {
    match period_status {
        "calculated" | "verified" => RetroPattern::VoidAndRecalculate,
        "approved" => RetroPattern::VoidAndRecalculate,
        "committed" => {
            if crosses_tax_bracket || total_delta_percentage > Decimal::from(10) {
                RetroPattern::Replacement
            } else {
                RetroPattern::DeltaAdjustment
            }
        }
        "closed" => RetroPattern::ManualAmendment,
        _ => RetroPattern::VoidAndRecalculate,
    }
}
```

### 2.5 Impact Preview

Before executing retro, admin must SEE what will change:

```
┌─────────────────────────────────────────────────────────────────────┐
│                     RETRO IMPACT PREVIEW                             │
│                                                                      │
│  Trigger: Salary change for Nguyễn Văn A (backdated to 15/02/2026)  │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ Period     │ Net Change  │ BHXH Change │ PIT Change │ Action  │   │
│  ├──────────────────────────────────────────────────────────────┤   │
│  │ Feb 2026   │ +2,500,000  │ +200,000    │ +250,000   │ Replace │   │
│  │ Mar 2026   │ -150,000    │ 0           │ -150,000   │ Delta   │   │
│  │ Apr 2026   │ -150,000    │ 0           │ -150,000   │ Delta   │   │
│  │ May 2026   │ 0 (no Δ)    │ 0           │ 0          │ Skip    │   │
│  │ Jun-Aug    │ 0 (no Δ)    │ 0           │ 0          │ Skip    │   │
│  ├──────────────────────────────────────────────────────────────┤   │
│  │ TOTAL      │ +2,200,000  │ +200,000    │ -50,000    │ 3 periods │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  ⚠️  BHXH adjustment of 200,000 VND needs manual filing             │
│  ⚠️  Net pay delta will be added to current period (Aug 2026)        │
│                                                                      │
│  [CANCEL]                              [CONFIRM & EXECUTE]           │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.6 Delta Settlement — Where Does the Money Go?

```
┌─────────────────────────────────────────────────────────────────┐
│                    DELTA SETTLEMENT                               │
│                                                                  │
│  SCENARIO: Employee was UNDERPAID 2,500,000 in February          │
│  Period is COMMITTED — can't change the bank transfer            │
│                                                                  │
│  SOLUTION:                                                        │
│                                                                  │
│  Current Payroll Period (August 2026):                            │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Regular August Salary:         Net = 15,000,000            │  │
│  │  Retro Adjustment (Feb 2026):   Net = +2,500,000            │  │
│  │  Retro Adjustment (Mar 2026):   Net = -150,000              │  │
│  │                                                   │  │
│  │  August Take-Home:              Net = 17,350,000            │  │
│  │  (Appears as a separate line item on payslip)               │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                  │
│  SCENARIO: Employee was OVERPAID → need to RECOVER               │
│                                                                  │
│  Policy configuration:                                            │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  recovery_strategy:                                         │  │
│  │    "one_time"    → Deduct full amount from next payslip     │  │
│  │    "installment" → Spread over N months                     │  │
│  │    "manual"      → HR handles manually outside system       │  │
│  │                                                              │  │
│  │  recovery_limit_percent: 30  → Cannot exceed 30% of net pay │  │
│  │  (Vietnamese labor law: maximum deduction is 30% of salary) │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.7 Insurance & Tax Reconciliation

The system tracks what was REPORTED vs what SHOULD HAVE BEEN reported:

```sql
CREATE TABLE payroll_insurance_adjustments (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    employment_id UUID NOT NULL,
    retro_event_id UUID NOT NULL,
    insurance_type TEXT NOT NULL,       -- 'bhxh','bhyt','bhtn'
    affected_period (year, month),
    reported_amount DECIMAL NOT NULL,    -- what was filed
    corrected_amount DECIMAL NOT NULL,  -- what should have been
    delta DECIMAL NOT NULL,             -- difference
    filing_status TEXT DEFAULT 'pending' -- pending | filed_amended | resolved
        CHECK (filing_status IN ('pending','filed_amended','resolved')),
    filed_at TIMESTAMPTZ,
    c10_number TEXT,                    -- Vietnamese social insurance form identifier
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE payroll_tax_adjustments (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    employment_id UUID NOT NULL,
    retro_event_id UUID NOT NULL,
    affected_period (year, month),
    reported_taxable DECIMAL NOT NULL,
    corrected_taxable DECIMAL NOT NULL,
    reported_pit DECIMAL NOT NULL,
    corrected_pit DECIMAL NOT NULL,
    delta_pit DECIMAL NOT NULL,
    filing_status TEXT DEFAULT 'pending',
    tax_declaration_ref TEXT,           -- Vietnamese tax form reference
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);
```

**Important:** The system tracks adjustments but does NOT automatically file them with government agencies. It generates the data for HR to manually file, because:
1. Vietnamese social insurance filing (C10 form) is a manual/semi-automated external process
2. Tax declaration amendments require specific procedures that vary by province
3. Auto-filing without human review creates legal risk

### 2.8 Execution Flow

```rust
impl RetroactiveEngine {
    pub async fn execute_retro(
        &self,
        event: RetroEvent,
        confirmed_by: Uuid, // must be admin who confirmed the preview
    ) -> Result<RetroResult, RetroError> {
        // 1. Validate (period not closed, user has permission, etc.)
        self.validate(&event)?;

        // 2. Build the cascade plan
        let plan = self.build_cascade_plan(&event).await?;

        // 3. Execute period by period, checking cascade after each
        let mut results = Vec::new();
        for period in &plan.periods {
            let period_result = self.recalculate_period(
                period,
                &plan.pattern_for(period),
            ).await?;
            results.push(period_result.clone());

            // Check if accumulators changed → cascade to next period
            if !period_result.accumulators_changed {
                // Cascade STOPS here — skip remaining periods in plan
                break;
            }
        }

        // 4. Generate reconciliation records
        let reconciliation = self.reconcile(&results).await?;

        // 5. Build adjustment entries for current period
        let adjustments = self.build_current_period_adjustments(&results)?;

        Ok(RetroResult {
            event_id: event.id,
            periods_affected: results,
            reconciliation,
            current_period_adjustments: adjustments,
            executed_by: confirmed_by,
            executed_at: Utc::now(),
        })
    }
}
```

### 2.9 Database: Retro Events

```sql
CREATE TABLE retro_events (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    trigger_type TEXT NOT NULL
        CHECK (trigger_type IN ('manual','data_change','formula_change')),
    trigger_detail JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'detected'
        CHECK (status IN ('detected','analyzing','pending_confirmation',
                          'executing','completed','failed','cancelled')),
    cascade_plan JSONB,         -- computed by analyzer
    execution_result JSONB,     -- populated after execution
    requested_by UUID,
    confirmed_by UUID,
    confirmed_at TIMESTAMPTZ,
    executed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE retro_affected_periods (
    id UUID PRIMARY KEY,
    retro_event_id UUID NOT NULL REFERENCES retro_events(id),
    employment_id UUID NOT NULL,
    period_year INTEGER NOT NULL,
    period_month INTEGER NOT NULL,
    old_payslip_id UUID REFERENCES payslips(id),
    new_payslip_id UUID REFERENCES payslips(id),
    pattern_used TEXT NOT NULL,  -- 'void_recalculate', 'delta', 'replacement'
    net_delta DECIMAL,
    accumulators_changed BOOLEAN DEFAULT false,
    cascade_order INTEGER NOT NULL,  -- 0 = first affected, 1 = next, etc.
    error_message TEXT,
    completed_at TIMESTAMPTZ
);
```

---

## 3. Integration with ADR-003 and ADR-004

| Component | Interaction |
|-----------|------------|
| **Component Pipeline** (ADR-003) | `recalculate_period()` calls the same pipeline, just with corrected data/formula version |
| **Template Versioning** (ADR-003) | Formula-change retro resolves the corrected template version via `effective_from` comparison |
| **Lifecycle (ADR-004)** | Pattern selection depends on period state; retro respects state locks |
| **Event Hooks** (ADR-003) | Retro engine fires the same hooks: `CalculationStarted`, `ComponentEvaluated`, etc. |
| **Post-Processing** (ADR-003) | Retro recalculations re-run post-processing for the affected period |

---

## 4. Risks & Trade-offs

| Risk | Mitigation |
|------|-----------|
| **Cascade runs to infinity** | Hard limit: max cascade = from affected period to current period (typically ≤ 12) |
| **Preview ≠ actual result** (race condition) | Lock input data for the retro event duration; detect concurrent changes |
| **User confirms wrong preview** | All retro events are logged; can be reversed by another retro event |
| **Performance: cascade of 100K payslips** | Execute per-period with parallel per-employee batches; show progress |
| **Insurance/tax delta never resolved** | Dashboard alerts: "12 pending insurance adjustments"; periodic reminder emails |

---

## 5. Key Design Insight

> **The retro engine does NOT recalculate differently from the normal engine.** It uses the EXACT same `evaluate()` pipeline. The only difference is: (a) which data's fed in, (b) which template version is resolved, (c) what happens to the old payslip, and (d) where the delta goes. This separation of calculation from orchestration is why ADR-003's architecture can support retro without modification.

---

## 6. References

- [ADR-003: Payroll Formula Engine](ADR-003-payroll-formula-engine.md) — Calculation pipeline
- [ADR-004: Payroll Lifecycle State Machine](ADR-ADR-004-payroll-lifecycle-state-machine.md) — State transitions
- Industry: Oracle HCM Retroactive Events, Workday Retroactive Payroll Processing
