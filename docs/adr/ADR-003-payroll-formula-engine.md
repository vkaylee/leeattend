# ADR-003: Payroll Formula Engine — Component Pipeline Architecture

| Property       | Value                                          |
|----------------|------------------------------------------------|
| **Status**     | Accepted — vertical slice implementation       |
| **Date**       | 2026-11-14                                     |
| **Author**     | AI + Lee                                       |
| **Supersedes** | N/A (greenfield layer over existing payroll)   |
| **Domain**     | Payroll Calculation Engine                     |

---

## Implementation Reality (2026-11-14)

This ADR's original Rust/workspace/template-DAG design is retained as historical direction. Current LeeAttend runtime is Go `net/http` with PostgreSQL and one implicit organization. Shipped formula architecture is an additive vertical slice:

```text
production metric → normalized PayrollInput → submitted/approved/locked batch
→ published safe-DSL formula version → KPI_BONUS component
→ payroll calculation run → immutable input/output/trace snapshot
```

Current source of truth:

- `apiserver/internal/domain/payroll/formula`: bounded expression evaluator; no arbitrary code, SQL, HTTP, or JavaScript.
- `apiserver/internal/repository/payroll.go`: input batches, formula versions, lifecycle transitions.
- `apiserver/internal/repository/payroll_kpi.go`: locked-input KPI calculation and snapshots.
- `apiserver/migrations/20261113030000_payroll_input_registry`: metric/input registry.
- `apiserver/migrations/20261113040000_payroll_formula_versions`: formula versions/calculation runs.
- `apiserver/migrations/20261114020000_payroll_input_row_governance`: row statuses and correction history.

Formula lifecycle: `draft → published → retired`. Published versions are not edited; new drafts use `POST /api/v1/payroll/formulas/{rule_id}/versions`. Calculation traces contain safe metadata only. Audit events use `audit_logs` and omit salary, metric values, expressions, reasons, source references, and PII.

Deferred from this ADR: multi-tenant/workspace selectors, template DAG replacement, arbitrary custom code, multi-country providers, and post-processing plugins. Revisit only after KPI vertical slice compatibility, migration, and role acceptance gates pass.

---

## 1. Context & Problem Statement

### 1.1 Current State

The system today has a **config-driven payroll engine** that works well for Vietnam-standard payroll:

- **`PayrollRuleDefinition`** — statutory rules with 3 calculation methods: `Percentage`, `Fixed`, `Tiered`
- **`PayrollIncentivePolicy`** — bonus policies with JSON condition matching
- **`PayrollRuleEvaluator`** — evaluates percentage/fixed/tiered rules against a `base_values` HashMap
- **`FinancialCalculator`** — hardcoded arithmetic: daily rate, hourly rate, OT cap, incentives, deductions, payslip assembly
- **`PayrollService::calculate_payslip_with_context()`** — 8-phase hardcoded pipeline (fetch → segments → attendance → OT cap → incentives → deductions → assembly → persist)

**What works well:**
- Effective dating on salary components (`employee_salary_components.effective_date`)
- Effective dating on rule assignments (`payroll_rule_assignments.effective_from/to`)
- Segment-based month splitting (salary/dept changes mid-month)
- Batch calculation with 50-employee chunk parallelism

**What does NOT work for a SaaS Formula Engine:**

| Gap | Impact |
|-----|--------|
| Pipeline order is hardcoded (8 phases in `payslip_calc.rs`) | Cannot reorder components per tenant |
| Variable flow is implicit (magic strings: `"gross"`, `"base_pay"`, `"bhxh_base"`) | No discoverability, no validation |
| Formula config (`rule_config` JSONB) has no version history | Recalculate old periods uses current formula → **WRONG** |
| No custom expression support | Unusual tenant requirements need code changes |
| No data-source extension point | Adding ESOP, commission, BIK requires modifying fetch.rs |
| No post-processing layer | Cost allocation, GL mapping, bank files have no plug-in points |
| No event hooks | Audit, validation, notification logic bleeds into core calculator |

### 1.2 Driving Forces

1. **Multi-tenant SaaS**: Each workspace may customize payroll structure differently
2. **Point-in-Time Correctness**: Recalculating January 2026 in March 2026 MUST use the formulas effective in January, not March
3. **Multi-country Future**: Vietnam today, potentially Singapore/Indonesia tomorrow — different tax systems, different insurance models
4. **Regulatory Volatility**: Tax brackets, insurance rates, minimum wage change annually by decree
5. **Enterprise Extensibility**: The engine must accommodate ESOP, commission, benefit-in-kind, garnishments, cost allocation without core refactors

---

## 2. Architecture Decision

### 2.1 Core Pattern: Component Pipeline + Template Versioning

We adopt a **Component Pipeline** architecture with **Immutable Template Snapshots**, mirroring the proven patterns used by Oracle HCM ("Elements + Fast Formulas + Payroll Flow"), ADP ("Earning/Deduction Codes + Packages"), and Workday ("PCCs + Calculation Steps").

```
┌────────────────────────────────────────────────────────────────────┐
│                    FORMULA PAYROLL ENGINE                            │
│                                                                     │
│  ┌──────────────────────────────────────────────────────┐          │
│  │              TEMPLATE VERSIONING LAYER                 │          │
│  │  payroll_templates → payroll_template_versions         │          │
│  │  (immutable snapshots with effective_from)             │          │
│  └──────────────────────┬───────────────────────────────┘          │
│                         │                                           │
│  ┌──────────────────────▼───────────────────────────────┐          │
│  │              COMPONENT PIPELINE (DAG)                  │          │
│  │                                                       │          │
│  │  ┌─────────┐    ┌──────────┐    ┌──────────────┐     │          │
│  │  │ GROSS   │───→│ OVERTIME │───→│ DEDUCTIONS   │     │          │
│  │  │ SALARY  │    │ PAY      │    │ (BHXH, TAX)  │     │          │
│  │  └─────────┘    └──────────┘    └──────┬───────┘     │          │
│  │       │                               │               │          │
│  │       └───────────┬───────────────────┘               │          │
│  │                   ▼                                   │          │
│  │            ┌──────────┐                               │          │
│  │            │   NET    │                               │          │
│  │            │  SALARY  │                               │          │
│  │            └──────────┘                               │          │
│  └───────────────────────────────────────────────────────┘          │
│                         │                                           │
│  ┌──────────────────────▼───────────────────────────────┐          │
│  │              VARIABLE CONTEXT (ACCUMULATORS)           │          │
│  │  Resolves variables from multiple providers at runtime │          │
│  └──────────────────────┬───────────────────────────────┘          │
│                         │                                           │
│  ┌──────────────────────▼───────────────────────────────┐          │
│  │              EXTENSION POINTS                          │          │
│  │  VariableProvider │ PostProcessPipeline │ EventHook   │          │
│  └───────────────────────────────────────────────────────┘          │
└────────────────────────────────────────────────────────────────────┘
```

### 2.2 Key Design Decisions

#### Decision 1: Components are CODE-defined, not USER-defined

**Rationale:** 90% of payroll calculations are standard across companies sharing the same jurisdiction. BHXH is always `gross × rate × cap`. PIT is always progressive brackets. These should be implemented once, tested thoroughly, and reused.

**Implementation:**

```rust
/// Core trait — every payroll component implements this.
trait PayrollComponent: Send + Sync {
    /// Unique identifier for this component type (e.g., "insurance_deduction")
    fn component_type(&self) -> ComponentType;

    /// Variables this component requires as input
    fn inputs(&self) -> Vec<VariableRef>;

    /// Variables this component produces as output
    fn outputs(&self) -> Vec<VariableRef>;

    /// Execute the calculation. Pure function — no DB, no side effects.
    fn evaluate(
        &self,
        config: &ComponentConfig,
        ctx: &VariableContext,
    ) -> Result<Vec<(VariableRef, Decimal)>, PayrollFormulaError>;
}

/// Built-in component types — extend this enum to add new types.
enum ComponentType {
    SumBaseComponents,      // Sum salary components into gross
    InsuranceDeduction,     // BHXH, BHYT, BHTN, CPF, EPF, etc.
    ProgressiveTax,         // PIT, slab-based taxes
    OvertimeCalculation,    // OT from attendance data
    OvertimeCap,            // Monthly OT hour cap
    IncentiveMatching,      // Bonus policy tier matching
    UnpaidLeaveDeduction,   // Deduction for unpaid absences
    NetAggregator,          // Final net pay assembly
    CustomExpression,       // Escape hatch: sandboxed expression parser
}
```

**CustomExpression** is the 10% escape hatch — not the default. It is a `PayrollComponent` like any other, but its config contains a sandboxed expression string. This prevents "every tenant writes their own BHXH formula" while still allowing "Company X has a unique loyalty bonus formula."

#### Decision 2: Immutable Template Snapshots for Point-in-Time

**Rationale:** When a formula changes (e.g., BHXH rate 8% → 8.5%), the change must NOT retroactively affect already-calculated periods. Recalculation of old periods must use the formula version effective at that time.

**Implementation:**

```sql
-- Stable reference to a template
CREATE TABLE payroll_templates (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    name TEXT NOT NULL,
    country_code TEXT NOT NULL DEFAULT 'VN',
    is_system BOOLEAN DEFAULT false  -- vendor-managed templates
);

-- Immutable version snapshot — NEVER updated after activation
CREATE TABLE payroll_template_versions (
    id UUID PRIMARY KEY,
    template_id UUID NOT NULL REFERENCES payroll_templates(id),
    version_number INTEGER NOT NULL,
    effective_from DATE NOT NULL,        -- ← KEY: applies from this date
    created_at TIMESTAMPTZ NOT NULL,
    created_by UUID,
    change_note TEXT,                     -- reason for change (e.g., "Decree 123/2026")
    is_draft BOOLEAN DEFAULT false,      -- draft = editable, !draft = IMMUTABLE
    UNIQUE (template_id, version_number)
);

-- Frozen component definitions within a version (IMMUTABLE when !draft)
CREATE TABLE payroll_template_version_components (
    id UUID PRIMARY KEY,
    template_version_id UUID NOT NULL REFERENCES payroll_template_versions(id),
    component_type TEXT NOT NULL,
    position INTEGER NOT NULL,            -- execution order (topological sort result)
    component_config JSONB NOT NULL,      -- frozen formula parameters
    input_refs JSONB NOT NULL,            -- ["gross_salary", "bhxh_base"]
    output_refs JSONB NOT NULL            -- ["bhxh_deduction"]
);

-- Which version applies to which workspace period
CREATE TABLE payroll_template_assignments (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    template_id UUID NOT NULL,
    effective_from DATE NOT NULL,
    effective_to DATE                     -- NULL = current
);

-- Audit trail: each payslip records which version was used
ALTER TABLE payslips ADD COLUMN template_version_id UUID
    REFERENCES payroll_template_versions(id);
```

**Resolution logic:**

```rust
fn resolve_template_version(
    workspace_id: Uuid,
    as_of_date: NaiveDate,   // period_end date
) -> Result<TemplateVersion> {
    // 1. Find active template assignment at as_of_date
    // 2. Find latest activated version with effective_from <= as_of_date
    // 3. Return frozen version — guarantees same result every time
}
```

**Constraint enforced at application layer:**
> Template version components are IMMUTABLE after `is_draft = false`. Any configuration change MUST create a new version. This is the single most important invariant of the system.

#### Decision 3: VariableProvider Trait for Extensible Data Sources

**Rationale:** The current system has a monolithic `fetch_context()` that gathers all data upfront. When a new data source appears (ESOP grants, commission records, benefit-in-kind), we must modify core code. A provider trait allows new sources to register without touching the engine.

**Implementation:**

```rust
/// Each data source implements this trait and registers with the engine.
trait VariableProvider: Send + Sync {
    /// Unique provider identifier (e.g., "salary", "esop", "commission")
    fn provider_id(&self) -> &str;

    /// Declare which variables this provider can resolve
    fn provides(&self) -> Vec<VariableRef>;

    /// Resolve one or more variables for a given employment and time window
    fn resolve(
        &self,
        variables: &[VariableRef],
        employment_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<HashMap<VariableRef, Decimal>, PayrollFormulaError>;
}

/// Registry aggregates all providers and routes variable resolution.
struct VariableRegistry {
    providers: HashMap<String, Arc<dyn VariableProvider>>,
    // Maps variable name → provider_id for fast routing
    routing: HashMap<VariableRef, String>,
}
```

**Built-in providers:**
| Provider | Variables | Source |
|----------|-----------|--------|
| `salary` | `gross_salary`, `base_pay`, salary component totals | `employee_salary_components` |
| `attendance` | `std_units`, `ot_units`, `late_count`, `absent_days` | `attendance_daily_summaries` |
| `leave` | `paid_leave_days`, `unpaid_leave_days`, `sick_days` | `leave_requests` |
| `holiday` | `paid_holidays`, `holiday_work_days`, `holiday_premium` | `holidays` |
| `system` | `std_working_days`, `hours_per_day`, `regional_min_wage` | `workspace_config_entries` |
| `esop` (future) | `esop_taxable_income`, `esop_shares_exercised` | `esop_grants` |

#### Decision 4: Post-Processing Pipeline for Non-Calculation Operations

**Rationale:** Everything AFTER net pay is computed — cost allocation to GL accounts, bank file generation, tax filing data, employer-side contributions — should NOT live inside the calculation engine. They are different concerns with different extension patterns.

**Implementation:**

```rust
trait PostProcessStep: Send + Sync {
    fn step_name(&self) -> &str;
    fn execute(
        &self,
        payslip: &Payslip,
        component_results: &[(VariableRef, Decimal)],
        ctx: &PayrollContext,
    ) -> Result<PostProcessOutput, PayrollFormulaError>;
}

enum PostProcessOutput {
    GlEntries(Vec<GlEntry>),        // Debit/Credit pairs
    EmployerContributions(Vec<EmployerContribution>),
    BankPaymentFile(BankFileData),
    TaxFilingData(TaxFilingRecord),
    Custom(serde_json::Value),
}
```

**Built-in post-process steps:**

| Step | Input | Output |
|------|-------|--------|
| `EmployerInsurance` | gross, insurance rates | Employer BHXH/BHYT/BHTN contributions |
| `GlAllocation` | all component results + department cost center | Debit/Credit journal entries |
| `BankPaymentFile` | net_pay + employee bank details | Bank-compatible payment file |
| `TaxWithholdingReport` | PIT deductions, dependents | Monthly tax filing data |
| `CostSplit` | component results + project allocation | Multi-entity cost allocation |

Post-processing steps run sequentially (NOT DAG — they're typically order-dependent: GL entries → employer contributions → bank file).

#### Decision 5: Event/Hook System for Observability

**Rationale:** Without hooks, audit logging, custom validations, and notifications leak into core calculator code. A hook system provides explicit extension points without modifying the engine.

**Implementation:**

```rust
enum PayrollEvent {
    CalculationStarted {
        workspace_id: Uuid,
        employment_id: Uuid,
        period: (NaiveDate, NaiveDate),
        template_version_id: Uuid,
    },
    ComponentEvaluated {
        component_type: String,
        position: i32,
        inputs: HashMap<VariableRef, Decimal>,
        outputs: HashMap<VariableRef, Decimal>,
        duration_ms: u64,
    },
    CalculationCompleted {
        payslip_id: Uuid,
        net_pay: Decimal,
        total_duration_ms: u64,
    },
    CalculationFailed {
        error: PayrollFormulaError,
        partial_results: Option<HashMap<VariableRef, Decimal>>,
    },
    PostProcessStepCompleted {
        step_name: String,
        output: PostProcessOutput,
        duration_ms: u64,
    },
}

trait PayrollEventHook: Send + Sync {
    fn hook_name(&self) -> &str;
    fn on_event(&self, event: &PayrollEvent) -> Result<(), PayrollFormulaError>;
}
```

**Built-in hooks:**
| Hook | Purpose |
|------|---------|
| `AuditLogHook` | Write every event to `payroll_calculation_logs` table |
| `ValidationHook` | Custom validation rules (e.g., "net pay cannot be negative") |
| `NotificationHook` | Notify admin on calculation failures |
| `MetricsHook` | Record component timing for performance monitoring |

---

## 3. Complete Execution Flow

```
BATCH CALCULATE (workspace, month/year)
│
│  FOR EACH employment:
│
├─ STEP 1: RESOLVE TEMPLATE
│   resolve_template_version(workspace, period_end)
│   → Returns frozen TemplateVersion with ordered components
│
├─ STEP 2: BUILD VARIABLE CONTEXT
│   For each VariableRef needed by any component:
│     route to registered VariableProvider
│     provider.resolve(variables, employment, period)
│   → VariableContext populated with all input data
│
├─ STEP 3: EXECUTE COMPONENT PIPELINE (topological order)
│   For each component in template_version.components (sorted by position):
│     ├─ FIRE PreEvaluate hook
│     ├─ component.evaluate(config, &ctx)
│     ├─ Write outputs back to VariableContext (available to downstream)
│     └─ FIRE PostEvaluate hook
│   → VariableContext now contains ALL computed variables
│
├─ STEP 4: ASSEMBLE PAYSLIP
│   NetAggregator component produces net_pay
│   Build Payslip record with template_version_id FK
│
├─ STEP 5: POST-PROCESSING PIPELINE (sequential)
│   For each registered PostProcessStep:
│     step.execute(payslip, component_results, ctx)
│   → GL entries, employer contributions, bank file
│
├─ STEP 6: PERSIST
│   Save payslip + segments + incentive results + rule evaluations
│   Save post-processing outputs
│
└─ STEP 7: FIRE CompletionEvent
    All registered hooks receive CalculationCompleted
```

---

## 4. Migration Path from Current System

The migration is incremental — we do NOT rewrite the working engine:

### Phase 1: Foundation (Template Versioning + Component Trait)
- Define `PayrollComponent` trait and `ComponentType` enum
- Create `payroll_templates`, `payroll_template_versions`, `payroll_template_version_components` tables
- Migrate existing hardcoded 8-phase flow into individual component implementations:
  - `SumBaseComponents` ← salary resolution in `payslip_calc.rs` lines 52-63
  - `OvertimeCalculation` ← attendance matching + OT engine
  - `IncentiveMatching` ← `FinancialCalculator::calculate_incentives()`
  - `DeductionAggregator` ← `FinancialCalculator::calculate_deductions()`
  - `NetAggregator` ← `FinancialCalculator::assemble_payslip()`
- Implement `resolve_template_version()` and version snapshotting
- **Outcome:** Existing calculation produces identical results, but runs through the new DAG pipeline. All formulas are versioned.

### Phase 2: VariableProvider Extraction
- Extract `VariableProvider` trait
- Move data-fetch logic from `fetch.rs` into provider implementations
- Add `VariableRegistry` for runtime routing
- **Outcome:** New data sources can be added without modifying core fetch logic.

### Phase 3: Expression Parser for CustomExpression
- Integrate `evalexpr` crate (or similar sandboxed parser)
- Implement `CustomExpression` component type
- Add expression validation at template save time
- Add compile cache for performance
- **Outcome:** Tenants can define custom formulas for edge cases.

### Phase 4: Post-Processing + Event Hooks
- Implement `PostProcessStep` trait with employer insurance, GL allocation steps
- Implement `PayrollEventHook` with audit log, validation hooks
- **Outcome:** Full enterprise payroll lifecycle.

### Phase 5: Template Library + Multi-Country
- Create system templates: "Vietnam Standard", "Vietnam Manufacturing", etc.
- Template inheritance and overrides
- Singapore template (CPF instead of BHXH)
- **Outcome:** New country = new template, not new code.

---

## 5. Extensibility Verification

The architecture is stress-tested against 7 future extension scenarios:

| Scenario | Mechanism Used | Core Engine Change? |
|----------|---------------|---------------------|
| **New country (Singapore)** | New template + new component types (e.g., `CpfContribution`) | No |
| **ESOP / stock options** | New `VariableProvider` (EsopProvider) + new `CustomExpression` component | No |
| **Cost allocation / multi-entity** | New `PostProcessStep` (CostSplit) | No |
| **Retroactive law change** | Template version with `effective_from` backdated + recalculate | No |
| **Custom bonus formula** | `CustomExpression` component type | No |
| **Garnishment / legal priority** | New `PayrollComponent` type: `PriorityDeduction` | No |
| **Scale 100K+ employees** | DAG-aware parallelism (independent components run concurrently) + expression compile cache | No |

---

## 6. Risks & Trade-offs

| Risk | Mitigation |
|------|-----------|
| **Template version proliferation** | Auto-archive versions older than retention period; only keep versions referenced by existing payslips |
| **CustomExpression performance** | Compile expressions once per template version, cache the AST; benchmark against 10K-employee batch |
| **DAG cycle detection** | Validate at template save time (not runtime); show cycle path to admin |
| **Migration complexity** | Phase 1 produces identical results to current system (validated by integration tests) before any new features are added |
| **Frontend complexity** | Phase 1-2 require no UI changes (templates auto-generated from existing config); Phase 3+ adds visual template builder |

---

## 7. References

- **Current payroll engine:** `apiserver/src/services/payroll_service/payslip_calc.rs` (375 lines)
- **Financial calculator:** `apiserver/src/services/financial_calculator.rs` (527 lines)
- **Rule evaluator:** `apiserver/src/services/payroll_rule_evaluator.rs` (247 lines)
- **Incentive evaluator:** `apiserver/src/services/payroll_evaluator.rs` (660 lines)
- **Domain models:** `apiserver/src/models/payroll.rs` (835 lines)
- **Database schema:** `apiserver/migrations/tenant/20260609000001_init_tenant/up.sql` (lines 960-1364)
- **Industry reference:** Oracle HCM Payroll Fast Formulas, ADP Workforce Now Earning/Deduction Codes, Workday Payroll Calculation Components

---

## 8. Decision Outcome

**APPROACH:** Component Pipeline with Immutable Template Snapshots and 3 pluggable extension points (VariableProvider, PostProcessStep, PayrollEventHook).

**RATIONALE:** This architecture:
1. Matches enterprise payroll patterns used by Oracle, ADP, and Workday
2. Guarantees point-in-time correctness through immutable version snapshots
3. Covers 90% of use cases with pre-built, tested component types
4. Provides a sandboxed `CustomExpression` escape hatch for the remaining 10%
5. Enables extensibility through traits without core engine modification
6. Builds incrementally on the existing working payroll engine
