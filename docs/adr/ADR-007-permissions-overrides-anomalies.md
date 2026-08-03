# ADR-007: Permissions, Manual Overrides & Anomaly Detection

| Property       | Value                                                       |
|----------------|-------------------------------------------------------------|
| **Status**     | Proposed                                                    |
| **Date**       | 2026-08-03                                                  |
| **Author**     | AI + Lee                                                    |
| **Depends on** | [ADR-003](ADR-003-payroll-formula-engine.md), [ADR-004](ADR-004-payroll-lifecycle-state-machine.md) |
| **Domain**     | Payroll Governance & Control                                |

---

## 1. Context & Problem Statement

### 1.1 Three Interconnected Concerns

After the engine calculates, after the period is approved, after retro is handled — three governance concerns remain that every enterprise payroll system must address:

| Concern | Core Question | Why It Matters |
|---------|--------------|----------------|
| **Permissions** | Who can see and do what? | Salary is the most sensitive data in any company |
| **Manual Override** | Can humans adjust what the engine computed? | Reality always has exceptions the formula can't capture |
| **Anomaly Detection** | How do we catch errors before they become payments? | A single missed zero in a salary can cost millions |

These three are architecturally linked: permissions control who can override; overrides must generate audit events; anomalies trigger on both calculated and overridden values.

### 1.2 Current State

- **Permissions** exist (`payroll:read`, `payroll:approve`, `payroll:disburse`, `payroll:sensitive:read`, `payroll:calculate`, `payroll:settings:manage`, `payroll:components:manage`, `payroll:incentives:manage`, `ess:payslips`) but are mapped to roles by UUID in the migration seed — not yet enforced at the API level for payroll operations
- **Manual override** does not exist — `assemble_payslip()` produces immutable output
- **Anomaly detection** does not exist — no automated validation of calculation results

---

## 2. Part A: Permissions

### 2.1 Dual-Layer Permission Model

Payroll needs TWO layers of permissions, not one:

```
┌──────────────────────────────────────────────────────────────┐
│                    PAYROLL PERMISSION MODEL                   │
│                                                               │
│  LAYER 1: OPERATIONAL PERMISSIONS                             │
│  (What actions can this user perform?)                        │
│  ┌──────────────────────────────────────────────────────┐    │
│  │ payroll:calculate    → Trigger batch calculation      │    │
│  │ payroll:verify       → Mark payslips as verified      │    │
│  │ payroll:approve      → Approve period for payment     │    │
│  │ payroll:commit       → Finalize and lock period       │    │
│  │ payroll:close        → Permanently close period       │    │
│  │ payroll:void         → Cancel individual payslip      │    │
│  │ payroll:override     → Manually adjust payslip values │    │
│  │ payroll:retro        → Trigger retro recalculation    │    │
│  │ payroll:reopen       → Reopen closed period           │    │
│  │ payroll:settings:write → Modify template versions    │    │
│  └──────────────────────────────────────────────────────┘    │
│                                                               │
│  LAYER 2: DATA VISIBILITY PERMISSIONS                         │
│  (What data can this user SEE?)                               │
│  ┌──────────────────────────────────────────────────────┐    │
│  │ payroll:read:own       → Employee sees own payslip    │    │
│  │ payroll:read:team      → Manager sees team's payslips │    │
│  │ payroll:read:all       → HR sees all payslips         │    │
│  │ payroll:sensitive:read → View unmasked salary + PII   │    │
│  │ payroll:sensitive:mask → View masked salary (e.g.,    │    │
│  │                           "15.000.***")                │    │
│  └──────────────────────────────────────────────────────┘    │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

### 2.2 Data Visibility Matrix

| Role | Own Payslip | Team Payslips | All Payslips | Unmasked PII | Unmasked Salary |
|------|:-----------:|:-------------:|:------------:|:------------:|:---------------:|
| Employee | ✅ | — | — | ✅ (own) | ✅ (own) |
| Team Lead | ✅ | ✅ (masked) | — | — | — |
| Department Head | ✅ | ✅ (masked) | — | — | — |
| HR Officer | ✅ | ✅ | ✅ | ✅ | ✅ |
| HR Manager | ✅ | ✅ | ✅ | ✅ | ✅ |
| Accountant | — | — | ✅ (masked) | — | — |
| Auditor | — | — | ✅ (masked) | — | — |
| Owner/Superadmin | ✅ | ✅ | ✅ | ✅ | ✅ |

### 2.3 Segregation Enforcement

From ADR-004, segregation of duties is enforced at the application layer:

```rust
impl PayrollAuthorizationService {
    pub fn validate_transition(
        &self,
        actor: &Actor,
        period: &PayrollPeriod,
        transition: PeriodTransition,
    ) -> Result<(), AuthorizationError> {
        // 1. Permission check
        actor.require_permission(transition.required_permission())?;

        // 2. Segregation check
        match transition {
            PeriodTransition::Approve => {
                if actor.id == period.calculator_id {
                    let mode = self.workspace_segregation_mode(period.workspace_id);
                    match mode {
                        SegregationMode::Strict | SegregationMode::Standard => {
                            return Err(AuthorizationError::SegregationViolation(
                                "Calculator cannot also be Approver"
                            ));
                        }
                        SegregationMode::Relaxed | SegregationMode::SingleUser => {
                            // Warning only, logged but not blocked
                            tracing::warn!(
                                actor_id = %actor.id,
                                period = %period.id,
                                "Calculator also approving period (relaxed mode)"
                            );
                        }
                    }
                }
            }
            PeriodTransition::Commit => {
                if actor.id == period.calculator_id {
                    // ALWAYS blocked — calculator never disburses
                    return Err(AuthorizationError::SegregationViolation(
                        "Calculator cannot also be Disburser"
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }
}
```

### 2.4 PII Masking Strategy

```rust
trait PayslipMasker {
    fn mask_for_viewer(
        payslip: &Payslip,
        viewer: &Actor,
        viewer_relation: ViewerRelation, // 'self', 'team_member', 'org_member', 'none'
    ) -> MaskedPayslip;
}

struct MaskedPayslip {
    // Always visible
    period: String,
    standard_units: i32,
    overtime_units: i32,

    // Conditionally masked
    net_pay_amount: MaskedDecimal,
    base_pay_amount: MaskedDecimal,
    overtime_pay_amount: MaskedDecimal,
    deductions_amount: MaskedDecimal,

    // Conditionally visible detail
    salary_components: Option<Vec<MaskedSalaryComponent>>, // None = hidden entirely
    incentive_details: Option<Vec<MaskedIncentive>>,
    deduction_details: Option<Vec<MaskedDeduction>>,
}

enum MaskedDecimal {
    Full(Decimal),           // "15.000.000"
    Rounded(Decimal),        // "15.000.000" (rounded to thousand)
    Masked(String),          // "15.***.***"
    Hidden,                  // "—"
}
```

---

## 3. Part B: Manual Override

### 3.1 Why Override Exists

No payroll formula captures 100% of reality. Manual overrides exist for:

| Example | Why Formula Can't Handle |
|---------|--------------------------|
| "Director approved a special 2M hardship allowance this month" | One-time, not policy-driven |
| "Deduct 500K for personal phone usage this month" | Not a salary component, not a rule |
| "HR discovered the OT rate was wrong for this one person, fix manually" | Quick fix before retro |
| "This employee's bonus is calculated externally (sales commission sheet)" | External system, different logic |

### 3.2 Override Model

```
┌─────────────────────────────────────────────────────────────┐
│                   MANUAL OVERRIDE MODEL                      │
│                                                              │
│  Override targets (what can be adjusted):                    │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  COMPONENT-LEVEL OVERRIDE                              │   │
│  │  • Adjust the OUTPUT of any named component            │   │
│  │  • Example: "BHXH deduction: -200,000 from calculated" │   │
│  │  • Example: "PIT deduction: override to 1,500,000"     │   │
│  │                                                        │   │
│  │  LINE-ITEM OVERRIDE                                    │   │
│  │  • Add arbitrary credit/debit line items               │   │
│  │  • Example: "+ 2,000,000 (Hardship allowance)"         │   │
│  │  • Example: "- 500,000 (Phone deduction)"              │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  Override types:                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  ADJUST   → Add/subtract from calculated value         │   │
│  │  REPLACE  → Override calculated value entirely         │   │
│  │  LINE     → New line item (not linked to any component)│   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 3.3 Database Schema

```sql
CREATE TABLE payroll_overrides (
    id UUID PRIMARY KEY,
    payslip_id UUID NOT NULL REFERENCES payslips(id),
    actor_id UUID NOT NULL,
    override_type TEXT NOT NULL
        CHECK (override_type IN ('adjust','replace','line')),
    target_type TEXT NOT NULL
        CHECK (target_type IN ('component','line_item')),
    target_ref TEXT NOT NULL,           -- component_type or custom line name
    calculated_amount DECIMAL,          -- NULL for line items
    override_amount DECIMAL NOT NULL,   -- the adjustment or final value
    delta DECIMAL NOT NULL,             -- override_amount - calculated_amount (or override_amount for line)
    reason TEXT NOT NULL,               -- mandatory justification
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_payroll_overrides_payslip ON payroll_overrides(payslip_id);

-- View: payslip with overrides applied
-- net_pay_amount_adjusted = net_pay_amount + SUM(overrides.delta)
```

### 3.4 Override Interaction with Retro

A critical design decision: **what happens to manual overrides when retro recalculates?**

| Scenario | Behavior |
|----------|----------|
| Retro replaces payslip (Void & Recalculate) | Manual overrides on the voided payslip are preserved in audit. New payslip has NO overrides — admin must re-apply if still needed. |
| Retro creates delta | Manual overrides on the original payslip are preserved. Delta payslip has no overrides. |
| Retro triggered AFTER admin manually fixed something | The retro preview SHOWS that the old payslip had manual overrides. Admin is prompted: "Keep or discard previous overrides?" |

The rule: **retro performs PURE calculation. Overrides are applied AFTER calculation and are never carried forward automatically.** This prevents override accumulation (applying the same override twice because it was on both old and new payslip).

### 3.5 Audit Trail

Every override is logged:

```rust
// Hooked into ADR-003's PayrollEventHook system
struct OverrideAuditHook;

impl PayrollEventHook for OverrideAuditHook {
    fn on_event(&self, event: &PayrollEvent) -> Result<()> {
        match event {
            PayrollEvent::ManualOverrideApplied {
                payslip_id,
                actor_id,
                override_type,
                target_ref,
                calculated_amount,
                override_amount,
                delta,
                reason,
            } => {
                // Write to immutable audit log
                // Alert if override exceeds threshold (e.g., > 20% of net)
            }
            _ => {}
        }
        Ok(())
    }
}
```

---

## 4. Part C: Anomaly Detection

### 4.1 Design Philosophy

Anomaly detection is NOT a separate engine. It's a set of **validation rules evaluated as PostProcessSteps** (ADR-003) — because validation happens AFTER calculation, BEFORE commit.

### 4.2 Anomaly Rule Types

```
┌────────────────────────────────────────────────────────────┐
│                  ANOMALY DETECTION RULES                    │
│                                                             │
│  TYPE 1: THRESHOLD VIOLATION                                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Rule: "Net pay for any employee > 200% of their      │   │
│  │        12-month average"                              │   │
│  │ Severity: WARNING                                     │   │
│  │ Action: Flag for review, do not block                 │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  TYPE 2: IMPOSSIBILITY CHECK                                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Rule: "Work days > calendar days in month"            │   │
│  │ Severity: ERROR                                        │   │
│  │ Action: BLOCK approval until resolved                 │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  TYPE 3: PERIOD-OVER-PERIOD COMPARISON                      │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Rule: "Net pay changed > 30% vs previous month        │   │
│  │        without corresponding salary change"           │   │
│  │ Severity: WARNING                                     │   │
│  │ Action: Flag for review                               │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  TYPE 4: AGGREGATE ANOMALY                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Rule: "Total payroll expense > 120% of budget"         │   │
│  │ Severity: WARNING                                     │   │
│  │ Action: Flag, require additional approver             │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  TYPE 5: DISTRIBUTION ANOMALY                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Rule: "Any deduction component exceeds               │   │
│  │        3 standard deviations from department mean"    │   │
│  │ Severity: WARNING                                     │   │
│  │ Action: Flag for review                               │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

### 4.3 Built-in Anomaly Rules

```sql
CREATE TABLE payroll_anomaly_rules (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    rule_name TEXT NOT NULL,
    rule_type TEXT NOT NULL
        CHECK (rule_type IN ('threshold','impossibility','period_comparison','aggregate','distribution')),
    severity TEXT NOT NULL DEFAULT 'warning'
        CHECK (severity IN ('warning','error','blocking')),
    rule_config JSONB NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Built-in rules seeded for all workspaces:
-- 1. net_pay_in_range: net_pay between 0 and configurable max
-- 2. work_days_plausible: work_days ≤ calendar_days
-- 3. component_consistency: SUM(component amounts) = payslip totals
-- 4. prior_period_delta: net_pay unchanged vs prior month > threshold
-- 5. duplicate_payslip: no two payslips for same employment in same period
```

### 4.4 Anomaly Detection as a PostProcessStep

```rust
struct AnomalyDetectionStep {
    rules: Vec<PayrollAnomalyRule>,
}

impl PostProcessStep for AnomalyDetectionStep {
    fn step_name(&self) -> &str { "anomaly_detection" }

    fn execute(
        &self,
        payslip: &Payslip,
        component_results: &[(VariableRef, Decimal)],
        ctx: &PayrollContext,
    ) -> Result<PostProcessOutput> {
        let mut anomalies = Vec::new();

        for rule in &self.rules {
            if let Some(anomaly) = rule.evaluate(payslip, component_results, ctx)? {
                anomalies.push(anomaly);
            }
        }

        if anomalies.iter().any(|a| a.severity == "blocking") {
            return Err(PayrollFormulaError::AnomalyBlocking(
                format!("{} blocking anomalies detected", 
                    anomalies.iter().filter(|a| a.severity == "blocking").count())
            ));
        }

        Ok(PostProcessOutput::AnomalyReport(AnomalyReport {
            payslip_id: payslip.id(),
            anomalies,
            generated_at: Utc::now(),
        }))
    }
}
```

### 4.5 Anomaly Resolution Workflow

```
┌─────────────────────────────────────────────────────────────┐
│                ANOMALY RESOLUTION WORKFLOW                   │
│                                                              │
│  Post-Processing generates AnomalyReport                     │
│                       │                                      │
│                       ▼                                      │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ WARNING anomalies → auto-flagged, period can proceed  │   │
│  │ ERROR anomalies   → period CANNOT be approved          │   │
│  │ BLOCKING anomalies → period CANNOT be committed        │   │
│  └──────────────────────────────────────────────────────┘   │
│                       │                                      │
│                       ▼                                      │
│  Admin reviews each anomaly:                                 │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ [ACKNOWLEDGE] "I see this, it's expected"              │   │
│  │ [DISMISS]     "This is a false positive"               │   │
│  │ [FIX]         "Go fix the underlying data/override"    │   │
│  │ [OVERRIDE]    "Manually adjust to resolve"             │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  Resolution stored in:                                       │
│  payroll_anomaly_resolutions (anomaly_id, resolution,        │
│    actor_id, resolved_at, comment)                           │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 5. Integration Summary

```
┌─────────────────────────────────────────────────────────────┐
│                   GOVERNANCE FLOW                            │
│                                                              │
│  1. CALCULATE                                               │
│     Engine runs (ADR-003) → produces payslips               │
│                       │                                      │
│  2. DETECT ANOMALIES                                        │
│     PostProcessStep runs → AnomalyReport                    │
│     Blocking errors → back to step 1 (fix data)             │
│                       │                                      │
│  3. MANUAL OVERRIDE (optional)                              │
│     Admin adjusts values → overrides logged                 │
│     Requires payroll:override permission                    │
│                       │                                      │
│  4. VERIFY                                                  │
│     HR reviews totals + anomalies + overrides               │
│     Requires payroll:verify permission                      │
│                       │                                      │
│  5. APPROVE                                                 │
│     Segregation enforced (approver ≠ calculator)            │
│     Requires payroll:approve permission                     │
│                       │                                      │
│  6. COMMIT                                                  │
│     Data frozen (ADR-004), accumulators written (ADR-006)   │
│     Requires payroll:commit permission                      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 6. References

- [ADR-003: Payroll Formula Engine](ADR-003-payroll-formula-engine.md) — PostProcessStep, EventHook
- [ADR-004: Payroll Lifecycle State Machine](ADR-004-payroll-lifecycle-state-machine.md) — Permission gates per transition
- [ADR-005: Retroactive Processing](ADR-005-retroactive-processing-engine.md) — Override + retro interaction
- [ADR-006: Accumulators](ADR-006-accumulators-multi-period-context.md) — Period-over-period anomaly rules
