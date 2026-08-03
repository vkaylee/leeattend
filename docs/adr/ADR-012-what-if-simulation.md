# ADR-012: What-If Simulation & Scenario Planning

| Property       | Value                                                       |
|----------------|-------------------------------------------------------------|
| **Status**     | Proposed                                                    |
| **Date**       | 2026-08-03                                                  |
| **Author**     | AI + Lee                                                    |
| **Depends on** | [ADR-003](ADR-003-payroll-formula-engine.md)               |
| **Domain**     | Payroll — Planning & Forecasting                            |

---

## 1. Context

### 1.1 The "What If" Problem

Every HR manager and CFO asks the same questions before committing payroll:

| Question | What Needs to Change | Why Simulate? |
|----------|---------------------|----------------|
| "What if we increase Engineering salaries 10%?" | Salary component amounts | Budget planning |
| "What if the new insurance rate applies?" | Template version (BHXH rate) | Regulatory impact |
| "What if we pay Tết bonus at 2 months?" | Bonus component amount + PIT brackets | Bonus planning |
| "What if we hire 20 more engineers?" | New hypothetical employees | Headcount planning |
| "What if OT policy changes to 2x after 8pm?" | Overtime rules | Policy change impact |

These are **not hypothetical** — these decisions involve millions of VND and legal compliance. Decision-makers need preview before execution.

### 1.2 Why This Is Hard

A simulation is NOT just "calculate with different inputs":
- It must NOT affect real accumulators, real payslips, or real data
- It must be **comparable**: "show me the difference between current policy and new policy side by side"
- It may involve **hypothetical data** (fake employees, future salary scales)
- It must be **fast**: you can't wait 10 minutes for a batch calculation just to see a preview

---

## 2. Architecture Decision

### 2.1 Dry-Run Mode: The Engine Has It Built In

The core insight: **ADR-003's component pipeline already supports simulation** because:
- `PayrollComponent::evaluate()` is a pure function — same inputs → same outputs
- `VariableProvider` is a trait — can be replaced with simulation providers
- `PostProcessStep` is optional — can be skipped

What's needed is a **DryRunMode** wrapper that:
1. Uses the same template, same pipeline
2. But with **overridden inputs** (simulated salary, simulated headcount)
3. **No persistence** — no payslip created, no accumulator written
4. **Returns comparison data** — old vs new, line by line

```rust
struct SimulationEngine {
    calculator: Arc<PayrollEngine>,            // ADR-003 engine
    baseline_providers: Arc<VariableRegistry>,  // Real data providers
}

impl SimulationEngine {
    pub async fn run_simulation(
        &self,
        scenario: SimulationScenario,
    ) -> Result<SimulationResult> {
        let mut results = Vec::new();

        for employment in &scenario.employments {
            // 1. Build VariableContext with simulated overrides
            let mut ctx = self.build_baseline_context(employment).await?;
            ctx.apply_overrides(&scenario.overrides);

            // 2. Run pipeline in DRY-RUN mode
            let result = self.calculator.evaluate_pipeline_dry(
                &scenario.template_version,
                &ctx,
                employment.id,
                scenario.period,
            )?;

            results.push(result);
        }

        // 3. Aggregate results
        Ok(SimulationResult {
            scenario_id: scenario.id,
            per_employee: results,
            aggregate: self.aggregate(&results),
            baseline_comparison: scenario.baseline.as_ref()
                .map(|b| self.compare(&results, b)),
        })
    }
}
```

### 2.2 Scenario Model

```sql
CREATE TABLE payroll_simulation_scenarios (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    name TEXT NOT NULL,                    -- "Tăng lương P.Kỹ thuật 10%"
    description TEXT,
    scenario_type TEXT NOT NULL
        CHECK (scenario_type IN ('salary_change','headcount_change',
                'policy_change','bonus_planning','custom')),

    -- What period to simulate (can be future)
    simulate_year INTEGER NOT NULL,
    simulate_month INTEGER NOT NULL,

    -- What template version to use
    template_version_id UUID REFERENCES payroll_template_versions(id),

    -- If comparing against a baseline
    baseline_scenario_id UUID REFERENCES payroll_simulation_scenarios(id),

    -- What to override (JSON)
    overrides JSONB NOT NULL,
    /*
    Example:
    {
      "salary_multiplier": {"Engineering": 1.10, "Sales": 1.05},
      "component_overrides": {
        "bhxh_rate": {"employee": 0.09, "employer": 0.185}
      },
      "bonus_config": {
        "type": "fixed_months",
        "months": 2.0,
        "applicable_departments": ["*"]
      },
      "hypothetical_employees": [
        {"department": "Engineering", "salary": 20000000, "position": "Senior Dev", "count": 20}
      ]
    }
    */

    -- Results (populated after simulation runs)
    result_summary JSONB,
    -- {"total_payroll_cost": 2_500_000_000, "delta_from_baseline": +350_000_000,
    --  "avg_net_pay": 15_200_000, "affected_count": 45, "tax_bracket_shifts": 12}

    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    executed_at TIMESTAMPTZ
);
```

### 2.3 Override Types

The simulation supports multiple override dimensions:

| Override Type | Example | Mechanism |
|--------------|---------|-----------|
| **Component input** | "Base salary of Engineering × 1.10" | Override in VariableContext before pipeline runs |
| **Component config** | "BHXH rate = 9% instead of 8%" | Use different template version |
| **Hypothetical employees** | "Add 20 employees with salary 20M" | Inject synthetic employment records into VariableProvider |
| **Headcount multiplier** | "Multiply all department salaries by 1.15" | Post-process component outputs |
| **Remove component** | "What if we drop the transport allowance?" | Use template with component disabled |

### 2.4 Comparison Engine

The real value of simulation is comparison:

```rust
struct SimulationComparison {
    scenario_a: SimulationResult,
    scenario_b: SimulationResult,       // Typically baseline (current state)
    diffs: Vec<SimulationDiff>,
}

struct SimulationDiff {
    dimension: DiffDimension,           // 'total_cost', 'avg_net', 'tax_liability', ...
    scenario_a_value: Decimal,
    scenario_b_value: Decimal,
    delta: Decimal,
    delta_percentage: Decimal,
    severity: DiffSeverity,             // 'neutral', 'positive', 'negative', 'critical'
}

impl SimulationComparison {
    fn highlight_critical_changes(&self) -> Vec<SimulationDiff> {
        self.diffs.iter()
            .filter(|d| d.severity == DiffSeverity::Critical)
            .collect()
        // Example: "Tax liability increases 50% for 12 employees"
        // Example: "3 employees drop below regional minimum wage"
    }
}
```

### 2.5 Simulation Dashboard Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    SIMULATION WORKFLOW                           │
│                                                                  │
│  STEP 1: CREATE SCENARIO                                         │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Name: "Tết bonus 2 tháng cho toàn công ty"                │   │
│  │ Type: bonus_planning                                      │   │
│  │ Period: Tháng 1/2027 (future)                             │   │
│  │ Override: {bonus_months: 2.0}                             │   │
│  └──────────────────────────────────────────────────────────┘   │
│                           │                                      │
│                           ▼                                      │
│  STEP 2: RUN SIMULATION                                          │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ [EXECUTING] Pipeline running for 250 employees...          │   │
│  │ Estimated time: ~8 seconds                                 │   │
│  └──────────────────────────────────────────────────────────┘   │
│                           │                                      │
│                           ▼                                      │
│  STEP 3: REVIEW RESULTS                                          │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Total cost:        3,200,000,000 VND  (+800M vs baseline) │   │
│  │ Avg employee net:     12,800,000 VND  (+5.2M vs baseline) │   │
│  │ PIT liability:        320,000,000 VND  (+120M vs baseline) │   │
│  │ BHXH impact:                    0 VND  (bonus not insured)  │   │
│  │                                                             │   │
│  │ ⚠️  15 employees shift to higher PIT bracket               │   │
│  │ ⚠️  Total cost exceeds Q1/2027 budget by 15%               │   │
│  └──────────────────────────────────────────────────────────┘   │
│                           │                                      │
│                           ▼                                      │
│  STEP 4: COMPARE SCENARIOS                                       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │               │ Baseline   │ 1-Month    │ 2-Month (this) │   │
│  │  Total Cost   │ 2.4B       │ 2.8B       │ 3.2B           │   │
│  │  Avg Net      │ 7.6M       │ 9.2M       │ 12.8M          │   │
│  │  PIT Impact   │ 200M       │ 260M       │ 320M           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.6 Performance Considerations

| Concern | Approach |
|---------|----------|
| **Slow batch calculation for large companies** | Simulation runs per department, not per employee. Use statistical sampling (20% random sample) for quick estimates, full run for final. |
| **Expression compilation overhead** | Template version's compiled expressions are cached and reused |
| **Concurrent simulations** | Each simulation is isolated by scenario ID; multiple can run in parallel |
| **Future period data** | Future periods have no attendance data. Simulation uses employee's **average** attendance from past 3 months or a configurable default. |

---

## 3. Integration with ADR-003

```
Simulation reuses ADR-003 pipeline wholesale:

┌──────────────────────────────────────────────────┐
│              SIMULATION EXECUTION                  │
│                                                    │
│  VariableProvider (override)                        │
│      │                                              │
│      ▼                                              │
│  ┌────────────────────────────────────────────┐    │
│  │  ADR-003 COMPONENT PIPELINE (same code!)    │    │
│  │  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐   │    │
│  │  │Gross │→ │OT    │→ │BHXH  │→ │Net   │   │    │
│  │  └──────┘  └──────┘  └──────┘  └──────┘   │    │
│  └────────────────────────────────────────────┘    │
│      │                                              │
│      ▼  (DRY-RUN: no persistence)                   │
│  ┌────────────────────────────────────────────┐    │
│  │  SimulationResult (in-memory only)          │    │
│  │  • Per-employee outputs                     │    │
│  │  • Aggregate totals                         │    │
│  │  • Comparison to baseline                   │    │
│  └────────────────────────────────────────────┘    │
│                                                    │
└──────────────────────────────────────────────────┘
```

**No new traits needed.** Simulation just swaps VariableProvider implementations and skips PostProcessSteps that involve persistence.

---

## 4. Risks & Trade-offs

| Risk | Mitigation |
|------|-----------|
| **Simulation results misinterpreted as actual** | Every simulation is clearly labeled "SIMULATION — NOT FINAL" in UI. Scenario ID is stored, not linked to any real payslip. |
| **Future period data gaps** | Simulation uses statistical defaults for missing data; clearly shows which values are "estimated" vs "actual" |
| **Simulation ≠ actual due to real data changes** | Warning: "Simulation was run on 2026-08-03. Actual attendance data for January 2027 may differ." |
| **Users create hundreds of scenarios** | Auto-cleanup: scenarios older than 90 days are archived |

---

## 5. References

- [ADR-003](ADR-003-payroll-formula-engine.md) — Core pipeline reuse
- [ADR-006](ADR-006-accumulators-multi-period-context.md) — Accumulator projection for future periods
