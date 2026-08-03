# ADR-004: Attendance Policy Engine — Rule Catalog + Late/Absence Modeling + Versioning

| Property       | Value                                          |
|----------------|------------------------------------------------|
| **Status**     | Proposed                                       |
| **Date**       | 2026-08-03                                     |
| **Author**     | AI + Lee                                       |
| **Supersedes** | N/A (refactor of existing attendance policy resolution) |
| **Domain**     | Attendance Policy Engine                       |

---

## 1. Context & Problem Statement

### 1.1 Current State

The system has a **working policy resolution pipeline** that resolves attendance rules through three layers:

```
attendance_policies → attendance_policy_versions (rules JSONB + valid_from/to)
  → attendance_policy_assignments (target_type, target_id, valid_from/to)
     ↓ resolve
AttendanceRules (workspace-level: work_unit_minutes, rounding_policy, early_arrival_strategy, grace_period_minutes)
  + working_conditions (per-employment, start_date/end_date, minute_allowance, is_paid)
    × condition_type_definitions.affects_rules (JSONB, data-driven catalog)
  + ShiftPolicyOverrides (per-shift: break_auto_deduct, allow_early_ot)
     ↓
PolicyRules (7-field struct: grace_period_minutes, authorized_early_leave_minutes, is_early_leave_paid,
             early_arrival_strategy, is_break_auto_deduct, is_break_overtime, allow_early_ot)
     ↓
AttendanceCalculator → standard / early_OT / late_OT / break_OT buckets
```

**What works well:**

- **Precedence resolver**: 4-tier (employee > department > employer > workspace), pure function, no I/O (`domain/policy/resolver.rs:44-82`)
- **Effective dating**: `attendance_policy_versions(valid_from, valid_to)`, `leave_policy_assignments(effective_from/to)`, `working_conditions(start_date, end_date)`, DST-aware timezone conversion (`policy_resolver_service.rs:66`)
- **Approval workflow**: BPMN 2.0 Late-Binding Task Worklist, candidate roles, snapshot-on-submit, generic via `(entity_type, entity_id)`
- **Leave accrual engine**: frequency, accrual_amount, seniority_bonus, carry_over_limit, max_balance, reset_month, `LeaveAccrualService`
- **Catalog foundation**: `condition_type_definitions` table + `affects_rules` JSONB for data-driven condition effects
- **Shift overrides**: per-shift break_auto_deduct, allow_early_ot, break_is_overtime

**What does NOT work — critical gaps:**

| Gap | Evidence | Impact |
|-----|----------|--------|
| **Catalog is empty** — no production seeder | `condition_type_definitions` starts empty for new tenants; `tenant_provisioner.rs` only seeds `work_locations`; `startup_reconciliation.rs` only seeds overtime config; `dev_initialization.rs` was deleted. Integration test confirms: `catalog_is_authoritative_when_definition_is_missing` (`resolver.rs:657`) | **Every `working_conditions` row silently no-ops.** Maternity leave, breastfeeding accommodation, sick leave — all save and display but ZERO effect on computed attendance. |
| **Late/early not persisted** | `attendance_daily_summaries` has no columns for late_minutes, early_leave_minutes, absence_flag. Only `actual_work_minutes` + OT columns exist. | Retroactive policy changes silently change historical reports. Dashboard recomputes on-the-fly every query. |
| **Three divergent "late" implementations, three grace defaults** | `attendance_utils.rs:189` → grace=**5**; `scoring.rs:50` → grace from policy chain; `workspace_stats_repository.rs:812` → grace=**0** (ignored) | Same employee, same day: status says "Late" by 5-min rule, `late_early_count` may not increment by 15-min rule, dashboard chart shows different minutes by 0-min rule. No single source of truth. |
| **`grace_period_minutes` conflated with OT window** | `shifts.early_punch_in_grace_period_minutes` (semantics: "window before shift where OT is suppressed") is passed as the *base* for late-tolerance at `report_service/daily.rs:390`, `scoring.rs:110`, `attendance_summary_service.rs:1104`. Then `ShiftPolicyOverrides` ignores grace entirely (`resolver.rs:198-211` — field received but not applied, test-locked at line 466). | Cannot set per-shift late tolerance independently from per-shift early-OT window. |
| **Authorized early leave is all-or-nothing cliff** | `attendance_calculator.rs:237-241`: `missing_minutes <= authorized_early_leave_minutes` — if employee leaves 61 min early with 60-min allowance, ZERO credit (not even 60). | No partial-credit or pro-rata mode. Not configurable. |
| **PolicyRules vocabulary is only 7 fields** | `models/workspace.rs:10-18` — the entire set of fields a working condition can affect. | Cannot model: overtime prohibition, night-shift restriction, max daily hours, daily reduction (e.g., -1h for nursing mothers), compulsory break extension. |
| **`affects_rules` has no versioning** | `condition_type_definitions.affects_rules` is edited in-place (UPDATE). Same for `leave_policy_definitions.rule_config`. | Editing "MATERNITY" affects_rules today retroactively changes all recalculations of past periods. Point-in-time correctness is broken for these tables. |
| **No eligibility rule engine** | `AssignmentRuleConditions` (`models/employee.rs:511`) has only 5 static fields: department_id, employee_id_numbers, tags, location_id, employer_id. No operators, no seniority, no gender, no contract-type. | Must manually assign every working condition per employee. Cannot say "all female employees with >12 months tenure get breastfeeding accommodation." |
| **`workspace_compliance_rules` has no UI and no versioning** | `ComplianceRepository` exists with `get_rules`/`upsert_rules`/`is_enabled`, but no HTTP route, no frontend page. | 12h rest / 48h weekly / 40h OT thresholds changed only via direct SQL. |
| **Three policy assignment vocabularies** | Attendance uses `'EMPLOYMENT'/'USER'/'GROUP'` (uppercase, no CHECK). Leave/OT use `'workspace'/'department'/'employee'` (lowercase, CHECK constraint). None supports `'employer'`. | Cross-policy consistency impossible; employer-level override not available for leave/OT assignments. |
| **No Leave of Absence distinct from Time Off** | `leave_balances` has `year` + `entitlement int DEFAULT 12` — accrual-by-year model. | Maternity is event-based entitlement ("per birth"), not year-based accrual. No return-to-work process, no "On Leave" flag. |

### 1.2 Systemic Pattern: "Phase 1 Syndrome"

After surveying the entire codebase, a recurring pattern emerged: **infrastructure is built to a high standard, partially wired up, then work moves on before completion.** This is not a criticism — it is a natural consequence of building a large system incrementally. But naming the pattern helps avoid repeating it.

Evidence across the attendance/leave/policy domain:

| Built (well) | Wire-up gap | Impact |
|---|---|---|
| `condition_type_definitions` table + CRUD UI + `affects_rules` JSONB resolver | No seeder — catalog is empty for all tenants | Every `working_conditions` row silently no-ops |
| `workspace_compliance_rules` table + repository trait + implementation | No HTTP route, no frontend page | 12h rest / 48h weekly / 40h OT thresholds are SQL-only |
| Leave accrual engine with `seniority_bonus`, `carry_over_limit`, `max_balance` | `leave_balances.entitlement` hardcoded DEFAULT 12 in DDL | Accrual engine works but baseline entitlement is not configurable |
| `shift_versions` / `shift_templates` with immutable versioning | Roster resolution still reads `shifts` (no temporal) | Shift changes retroactively affect history |
| `working_conditions` with `start_date`/`end_date` + `condition_type_id` FK | No catalog seeding; no integration test proving end-to-end effect | Catalog-driven policy exists in unit tests only |
| ADR-003 identifies versioning as critical for payroll | Same problem exists for `affects_rules`/`rule_config` — not addressed | Two engines, same problem, only one has a plan |

**The principle this ADR enforces:** every infrastructure component must have at least one integration test proving it works end-to-end in production paths before the component is considered "done." No more silent no-ops behind beautiful abstractions.

### 1.3 The Three Policy Systems: Fragmented by Accident, Not by Design

Attendance, leave, and overtime policies are three separate subsystems with three different vocabularies:

| Aspect | Attendance | Leave | Overtime |
|---|---|---|---|
| Assignment target types | `'EMPLOYMENT'/'USER'/'GROUP'` | `'workspace'/'department'/'employee'` | `'workspace'/'department'/'employee'` |
| CHECK constraint on target_type | **None** (free-text varchar) | Yes | Yes |
| Temporal column type | `timestamptz` | `date` | `date` |
| Temporal column names | `valid_from`/`valid_to` | `effective_from`/`effective_to` | `effective_from`/`effective_to` |
| Policy versioning | Yes (`policy_versions` with `rules` JSONB) | Yes (`policy_definitions.rule_config` — but edited in-place) | No (only assignment-level temporal) |
| Supports `employer` scope | Via `'EMPLOYMENT'` target | **No** | **No** |
| Exclusion constraint on temporal overlap | **No** | Yes (migration `20260623000000`) | **No** |

This fragmentation was not an architectural choice — it happened because each subsystem was built at a different time, by a different phase of development. The question for this ADR is: **should we unify them?**

**Answer: unify the assignment infrastructure; keep the rule content separate.**

- **What should be unified:** assignment vocabulary (one `assignee_type` enum), temporal resolution (one date type, one column naming convention), exclusion constraints, and employer scoping. All three systems should resolve "which policy applies to this employee on this date" through the same code path.
- **What should stay separate:** the policy content itself. Leave needs accrual config, overtime needs multiplier rules, attendance needs grace period and rounding. These are genuinely different domains with different schemas. A single `policy_rules` JSONB for all three would become an untyped mess.

The 4-tier precedence resolver in `WorkspaceConfigResolver` (employee > department > employer > workspace) is already the right model — it just needs to be extracted into a shared utility and reused by all three policy assignment paths.

### 1.4 Driving Forces

1. **Correctness baseline**: A working-condition engine that silently no-ops is a data-integrity risk, not a missing feature. This is the #1 priority to fix.
2. **Single source of truth for attendance status**: Late/absence must be a persisted, policy-resolved, versioned fact, not three runtime computations with three defaults.
3. **Multi-country compliance**: Vietnam labor law has specific maternity, breastfeeding, sick-leave, and disability accommodation rules. The policy engine must express these without code changes per jurisdiction (see §5 for concrete test cases).
4. **Point-in-time correctness**: Any formula, rule, or catalog entry that affects payroll-adjacent attendance computation MUST be immutable after a period is closed. Editing in-place is a regulatory risk.
5. **Enterprise multi-tenant**: Each workspace defines its own condition types, leave policies, and overtime rules. The engine must support this with a consistent, typed vocabulary.
6. **ADR-003 alignment — with discrimination**: ADR-003 established Component Pipeline + Immutable Snapshots + VariableProvider + EventHooks for payroll. The attendance policy engine should adopt Immutable Snapshots (the pattern that applies universally) but should NOT blindly copy Component Pipeline or VariableProvider (which address payroll-specific problems). See §2.2 Decision 9.

---

## 2. Architecture Decision

### 2.1 Core Pattern: Policy Catalog with Versioned Snapshots + Unified Attendance Status

We refactor the attendance policy engine across four dimensions, each building on existing infrastructure:

```
┌──────────────────────────────────────────────────────────────────────┐
│                    ATTENDANCE POLICY ENGINE v2                         │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────┐         │
│  │        CONDITION TYPE CATALOG (SEEDED + VERSIONED)        │         │
│  │  condition_type_definitions → condition_type_versions      │         │
│  │  (immutable affects_rules snapshots with effective_from)   │         │
│  └──────────────────────┬──────────────────────────────────┘         │
│                         │                                             │
│  ┌──────────────────────▼──────────────────────────────────┐         │
│  │           ELIGIBILITY RULE ENGINE (enhanced)              │         │
│  │  AssignmentRuleConditions → ConditionalExpression[]       │         │
│  │  (operators: EQ/NE/GT/GTE/LT/LTE/IN/CONTAINS/MATCH)       │         │
│  │  (attributes: gender, seniority, contract_type, age, …)   │         │
│  └──────────────────────┬──────────────────────────────────┘         │
│                         │                                             │
│  ┌──────────────────────▼──────────────────────────────────┐         │
│  │           UNIFIED POLICY RULES (expanded vocabulary)      │         │
│  │  ConditionRuleEffects + block_overtime, block_night,       │         │
│  │  max_daily_minutes, daily_reduction_minutes,               │         │
│  │  compulsory_break_extension, is_absence_allowed             │         │
│  └──────────────────────┬──────────────────────────────────┘         │
│                         │                                             │
│  ┌──────────────────────▼──────────────────────────────────┐         │
│  │         ATTENDANCE STATUS ENGINE (unified)                │         │
│  │  SINGLE resolve function → persist to daily_summaries     │         │
│  │  Fields: late_minutes, early_leave_minutes, is_absent,    │         │
│  │  grace_used, policy_version_id (FK)                        │         │
│  └─────────────────────────────────────────────────────────┘         │
│                         │                                             │
│  ┌──────────────────────▼──────────────────────────────────┐         │
│  │         LEAVE OF ABSENCE (new domain, if needed)          │         │
│  │  event-based entitlement, return-to-work process,         │         │
│  │  "On Leave" flag, Absence Accrual Effect                  │         │
│  └─────────────────────────────────────────────────────────┘         │
└──────────────────────────────────────────────────────────────────────┘
```

### 2.2 Key Design Decisions

#### Decision 1: Seed the catalog — fix the silent no-op bug FIRST

**Problem:** `condition_type_definitions` is empty for new tenants. `PolicyResolver::apply_condition_effects` checks the catalog as authoritative and returns early when a definition is missing, so every `working_conditions` row is silently ignored.

**Fix:** Seed 5 system types at tenant provision time with production-grade `affects_rules`:

| Code | `affects_rules` | `default_minute_allowance` |
|------|-----------------|---------------------------|
| `EARLY_LEAVE` | `authorized_early_leave_minutes`, `is_early_leave_paid` | 60 |
| `BREASTFEEDING` | `authorized_early_leave_minutes`, `is_early_leave_paid` | 60 |
| `MATERNITY` | `block_overtime`, `block_night_shift`, `daily_reduction_minutes`, `is_absence_allowed` | 0 |
| `SICK` | `is_absence_allowed`, `authorized_early_leave_minutes` | 0 |
| `UNPAID` | `is_absence_allowed`, `is_early_leave_paid` (false) | 0 |

**Seeding location:** `TenantProvisioner::provision_tenant_db()` in `apiserver/src/services/tenant_provisioner.rs`, alongside the existing `work_locations` seeding. This is the canonical post-migration hook.

**Validation:** Integration test that creates a workspace, seeds the catalog, assigns a `MATERNITY` working condition to an employee, and asserts that `block_overtime` and `block_night_shift` are reflected in the resolved `PolicyRules`. This test currently does not exist.

**Rationale:** This is a bug fix, not a feature. It unblocks all working-condition functionality for every tenant created to date. The migration comment already anticipated this seeding — we are simply implementing what the schema design intended.

#### Decision 2: Version the catalog — immutable `affects_rules` snapshots

**Rationale:** `condition_type_definitions.affects_rules` is JSONB edited in-place. When an admin changes MATERNITY from "allow early leave 60 min" to "allow early leave 90 min," every recalculation of past periods uses the new value — producing different (wrong) results. Same problem exists for `leave_policy_definitions.rule_config`.

This is the exact same point-in-time correctness problem that ADR-003 identified for payroll `rule_config` and solved with `payroll_template_versions`. We apply the same pattern here.

**Implementation:**

```sql
-- Immutable version snapshots for condition type definitions
CREATE TABLE condition_type_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    definition_id UUID NOT NULL REFERENCES condition_type_definitions(id),
    version_number INTEGER NOT NULL,
    affects_rules JSONB NOT NULL,        -- frozen at creation time
    effective_from DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID,
    change_note TEXT,
    UNIQUE (definition_id, version_number)
);

-- Same pattern for leave policy definitions
CREATE TABLE leave_policy_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id UUID NOT NULL REFERENCES leave_policy_definitions(id),
    version_number INTEGER NOT NULL,
    rule_config JSONB NOT NULL,           -- frozen at creation time
    effective_from DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID,
    change_note TEXT,
    UNIQUE (policy_id, version_number)
);
```

**Resolution logic:**
```rust
fn resolve_catalog_entry(
    code: &str,
    workspace_id: Uuid,
    as_of_date: NaiveDate,
) -> Result<ConditionTypeDefinition> {
    // 1. Find definition by workspace_id + code
    // 2. Find latest version with effective_from <= as_of_date
    // 3. Return definition with affects_rules from that version
}
```

**Constraint:** Once a version's `effective_from` is in the past (relative to the workspace's current time), the version is immutable. Any edit creates a new version. This is enforced at the application layer, matching the approach in ADR-003.

**Migration path:** On deployment, backfill `condition_type_versions` with one row per existing definition, `version_number = 1`, `effective_from = definition.created_at`. Existing `affects_rules` column becomes a denormalized `current_affects_rules` (updated on version creation for fast lookups of "current" rules).

#### Decision 3: Expand `ConditionRuleEffects` vocabulary

**Rationale:** The current 7-field vocabulary (grace, authorized_early_leave, is_early_leave_paid, early_arrival_strategy, is_break_auto_deduct, is_break_overtime, allow_early_ot) cannot express Vietnamese labor law requirements for maternity (no overtime, no night shift, reduced daily hours) or sick leave (no work requirement). Every new jurisdiction would require new hardcoded fields.

We expand `ConditionRuleEffects` to cover the full set of attendance-policy-relevant effects while keeping it as a flat boolean/value map (not a scripting language — that's ADR-003's `CustomExpression` escape hatch).

**New fields:**

```rust
pub struct ConditionRuleEffects {
    // Existing (unchanged)
    pub authorized_early_leave_minutes: Option<bool>,
    pub is_early_leave_paid: Option<bool>,
    pub grace_period_minutes: Option<bool>,
    pub is_break_auto_deduct: Option<bool>,
    pub is_break_overtime: Option<bool>,
    pub allow_early_ot: Option<bool>,
    pub early_arrival_strategy: Option<ConditionEarlyArrivalStrategy>,

    // NEW — prohibitions (boolean switches)
    pub block_overtime: Option<bool>,           // "không được làm thêm giờ"
    pub block_night_shift: Option<bool>,         // "không được làm ca đêm"
    pub block_heavy_labor: Option<bool>,         // "không được làm công việc nặng nhọc"

    // NEW — limits (numeric overrides)
    pub max_daily_work_minutes: Option<bool>,    // cap daily work to minute_allowance
    pub max_weekly_work_minutes: Option<bool>,   // cap weekly work to minute_allowance
    pub daily_reduction_minutes: Option<bool>,   // reduce daily work requirement by minute_allowance

    // NEW — absence accommodation
    pub is_absence_allowed: Option<bool>,        // "được phép vắng mặt có lý do"
    pub is_absence_paid: Option<bool>,           // whether the absence is paid

    // NEW — break accommodation
    pub compulsory_break_minutes: Option<bool>,  // mandatory additional break time
}
```

**How `apply_condition_effects` uses them:**
- `block_overtime` → sets `PolicyRules.block_overtime = true` (overrides any OT permission)
- `block_night_shift` → sets `PolicyRules.block_night_shift = true` (checked in roster/shift assignment)
- `daily_reduction_minutes` → subtracts `cond.minute_allowance()` from effective daily work requirement
- `max_daily_work_minutes` → caps total work + OT to `cond.minute_allowance()`
- `is_absence_allowed` → bypasses absence-detection for days within the condition's date range

**Key principle:** The vocabulary grows additively. Existing 7 fields remain at the same indices and semantics. JSON deserialization with `#[serde(default)]` ensures backward compatibility — old catalog entries with only the original 7 fields continue to work.

#### Decision 4: Persist late/early/absence as first-class attendance facts

**Rationale:** Currently `attendance_daily_summaries` has no columns for tardiness or absence. Three separate code paths compute "late" with three different grace defaults. Policy changes retroactively alter historical reports because there is no persisted fact — every report recomputes on-the-fly.

The industry standard (Workday, Oracle HCM, ADP) persists computed attendance status as a time-entry-level fact, referencing the policy version that produced it. This is the equivalent of "point-in-time correctness for attendance status."

**Implementation:**

```sql
ALTER TABLE attendance_daily_summaries
    ADD COLUMN late_minutes INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN early_leave_minutes INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN is_absent BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN grace_period_used INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN policy_version_id UUID;  -- FK to attendance_policy_versions
                                         -- that produced these values
```

**Single resolution path:**

```rust
/// The ONE function that determines attendance status.
/// Replaces the three divergent implementations.
fn resolve_daily_status(
    punches: &[DateTime<Utc>],
    schedule: &EffectiveSchedule,
    rules: &PolicyRules,
    date: NaiveDate,
) -> DailyAttendanceStatus {
    // 1. Determine effective punch-in / punch-out
    // 2. Compute late_minutes = max(0, punch_in - (shift_start + grace_period))
    // 3. Compute early_leave_minutes = max(0, (shift_end - authorized_early) - punch_out)
    // 4. Compute is_absent = no punches AND not holiday AND not authorized absence
    // 5. Return structured status
}
```

**Grace period resolution (unified):**

```
grace_period = max(
    working_conditions.minute_allowance   (if affects.grace_period_minutes && > 0),
    attendance_policy_versions.grace_period_minutes,
    DEFAULT_GRACE_PERIOD_MINUTES (15)
)
```

This eliminates the three divergent defaults (5 / 15 / 0) and the conflated early-OT-window usage.

**`shifts.early_punch_in_grace_period_minutes`** is renamed (or documented) as `early_ot_suppress_window_minutes` — its sole remaining purpose.

#### Decision 5: Eligibility rule engine with conditional expressions

**Rationale:** `AssignmentRuleConditions` currently supports only exact-match on 5 static fields. Real-world eligibility requires: "female AND tenure >= 12 months AND contract_type = 'full_time'" or "age >= 60 OR tenure >= 30 years".

We do NOT build a general-purpose expression language (that's ADR-003's `CustomExpression` scope). Instead we extend `AssignmentRuleConditions` with a simple conditional expression structure sufficient for 95% of eligibility rules.

**Implementation:**

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConditionalExpression {
    /// Attribute to evaluate against
    pub attribute: EligibilityAttribute,
    /// Operator
    pub operator: EligibilityOperator,
    /// Value to compare
    pub value: serde_json::Value,
}

pub enum EligibilityAttribute {
    Gender,              // "male", "female"
    SeniorityMonths,     // integer
    Age,                 // integer
    ContractType,        // "full_time", "part_time", "probation", "intern"
    DepartmentId,        // UUID
    EmployerId,          // UUID
    LocationId,          // UUID
    EmploymentPhase,     // "probation", "official"
    Tag,                 // string match
    EmployeeIdNumber,    // string match
    // ... extensible
}

pub enum EligibilityOperator {
    Eq, Ne, Gt, Gte, Lt, Lte,
    In,          // value is array — attribute must be in the array
    Contains,    // attribute is array — value must be in the attribute's array
}

// Enhanced conditions — AND within a group, OR between groups
pub struct AssignmentRuleConditions {
    /// Groups are OR-ed: employee must match at least one group
    /// Within a group, all expressions are AND-ed
    pub condition_groups: Vec<Vec<ConditionalExpression>>,
}
```

**JSON representation (backward compatible):**

```json
{
  "condition_groups": [
    [
      {"attribute": "Gender", "operator": "Eq", "value": "female"},
      {"attribute": "SeniorityMonths", "operator": "Gte", "value": 12},
      {"attribute": "ContractType", "operator": "In", "value": ["full_time", "probation"]}
    ]
  ]
}
```

Existing single-group `AssignmentRuleConditions` (department_id, employee_id_numbers, tags, location_id, employer_id) are migrated by converting each non-null field into one `ConditionalExpression` within a single condition group.

This same expression structure is reused for `leave_policy_assignments`, `overtime_policy_assignments`, and the new `working_condition_auto_assignments` (see Decision 6).

#### Decision 6: Working condition auto-assignment (eligibility-driven)

**Rationale:** Currently conditions must be manually assigned to every employee. An HR admin creating a "Breastfeeding" condition must individually add it to every eligible employee. With the eligibility engine (Decision 5), conditions can be defined once and auto-applied.

**Implementation:**

```sql
-- Optional: auto-assign working conditions based on eligibility rules
ALTER TABLE condition_type_definitions
    ADD COLUMN auto_assign_eligibility JSONB;  -- ConditionalExpression groups
```

When the attendance worker processes an employee:
1. Query all condition type definitions with `auto_assign_eligibility IS NOT NULL AND is_active = true`
2. For each, evaluate the eligibility rules against the employee's current attributes
3. If eligible and no active manual assignment exists, create a `working_conditions` row

Manual assignments always take precedence (an admin can override the auto-assigned values for specific employees).

This is a Phase 4+ feature — the eligibility engine (Decision 5) is the prerequisite. It is listed here for architectural completeness.

#### Decision 7: Merge three policy assignment vocabularies into one

**Rationale:** Attendance uses `'EMPLOYMENT'/'USER'/'GROUP'` (uppercase, no CHECK). Leave/OT use `'workspace'/'department'/'employee'` (lowercase, CHECK constraint). None supports `'employer'` despite `employer` being a first-class entity in the system.

**Implementation:** Standardize on the existing `leave_policy_assignments` pattern (lowercase, CHECK constraint), extend with `'employer'`:

```sql
-- Applied to: attendance_policy_assignments, leave_policy_assignments,
--             overtime_policy_assignments
CONSTRAINT policy_assignments_assignee_type_check
    CHECK (assignee_type IN ('workspace', 'employer', 'department', 'employee'))
```

Existing data migration: `'EMPLOYMENT'` → `'employee'`, `'USER'` → `'employee'` (resolve through `employments` table), `'GROUP'` → `'department'` (resolve group-to-department mapping).

This aligns with the 4-tier precedence resolver already in `WorkspaceConfigResolver`.

**Timing:** This is an architectural cleanup, not a bug fix. Defer to Phase 3+ unless there is a concrete need for cross-policy employer scoping.

#### Decision 8: Borrow Immutable Snapshots from ADR-003; defer Component Pipeline and VariableProvider

**Rationale:** ADR-003 established three reusable patterns. Not all apply to the policy engine. We must discriminate.

**What to adopt NOW (Phase 3):**

| Pattern from ADR-003 | Applicability to Policy Engine | Rationale |
|---|---|---|
| **Immutable Template Snapshots** | ✅ **Adopt** | `affects_rules` and `rule_config` have the exact same point-in-time correctness problem as payroll `rule_config`. This is the ONE universal pattern from ADR-003. |
| **Template version assignment with `effective_from`** | ✅ **Adopt** | `condition_type_versions` and `leave_policy_versions` need the same resolution logic as `payroll_template_versions`. Reuse the pattern, not the tables. |

**What to defer until there is a concrete requirement:**

| Pattern from ADR-003 | Applicability to Policy Engine | Rationale |
|---|---|---|
| **VariableProvider trait** | 🟡 **Defer** | Policy resolution pulls from 3–5 tables in a simple merge. Payroll pulls from 10+ data sources with different schemas. The abstraction is worth its weight for payroll; for policy, it would be over-engineering. Revisit when policy resolution requires >5 heterogeneous data sources. |
| **EventHook system** | 🟡 **Defer** | Useful when compliance audit requires "who changed this policy and what happened as a result." Not needed until there is a regulatory requirement or a customer asks for an audit trail of policy changes. |
| **PostProcessing pipeline** | ❌ **N/A** | Policy resolution has no post-calculation steps (no GL mapping, no bank file generation). This is payroll-specific. |

**What does NOT apply:**

| Pattern from ADR-003 | Why it doesn't apply |
|---|---|
| **Component Pipeline (DAG)** | Policy resolution is sequential merge (base rules → conditions → shift overrides), not a DAG of interdependent calculations. There is no topological sort problem. |
| **CustomExpression escape hatch** | Policy effects are typed booleans/numbers with well-known semantics. A freeform expression parser would be dangerous here — "`block_overtime = seniority > 12 && gender == 'female'`" belongs in the eligibility engine (Decision 5), not in the effects vocabulary. |

**Key principle:** ADR-003 and ADR-004 should use the **same vocabulary** for the same concepts (versioned snapshots, effective dating, precedence resolution) but should NOT force-fit patterns where the domains differ. The policy engine is simpler than the payroll engine, and that is a feature, not a defect.

#### Decision 9: Attendance events as the persistence model for status facts

**Rationale:** Decision 4 proposes adding columns to `attendance_daily_summaries`. An alternative worth documenting is a dedicated `attendance_events` table using a lightweight event-sourcing approach:

```sql
CREATE TABLE attendance_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employment_id UUID NOT NULL,
    date DATE NOT NULL,
    event_type VARCHAR(30) NOT NULL,  -- 'late_arrival', 'early_departure', 'absence', 'unauthorized_ot'
    event_minutes INTEGER NOT NULL,   -- minutes of violation or accommodation
    threshold_applied INTEGER,        -- grace period / allowance that was applied
    policy_version_id UUID,           -- FK to the policy version at computation time
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- immutable after the period is closed
    UNIQUE (employment_id, date, event_type)
);
```

**Trade-off:**

| Approach | Pros | Cons |
|---|---|---|
| **Columns on `attendance_daily_summaries`** | Simple, one row per employee per day, no JOIN needed | Adding new event types requires ALTER TABLE; mixes computed facts with raw punch data |
| **Dedicated `attendance_events` table** | Extensible (new event types = new rows, not new columns); natural audit trail; payroll queries simpler (`SUM(event_minutes) WHERE event_type = 'late_arrival'`) | More rows (1000 employees × 30 days × ~2 events = 60K/month — negligible); requires UNIQUE constraint to prevent duplicate events |

**Decision:** For Phase 2, add columns to `attendance_daily_summaries` (simpler migration, faster to implement). If the number of event types grows beyond 5 or compliance audit requirements emerge, migrate to the dedicated `attendance_events` table. The `resolve_daily_status()` function from Decision 4 is designed to return a structured result that can target either persistence model without changing its signature.

---

## 3. Complete Resolution Flow (Target State)

```
POLICY RESOLUTION for employee E on date D
│
├─ STEP 1: LOAD ATTENDANCE POLICY
│   resolve_attendance_policy(employee_id, date)
│   → Find assignment by precedence: employee > department > employer > workspace
│   → Find version by: valid_from <= effective_time <= valid_to, ORDER BY valid_from DESC LIMIT 1
│   → Returns AttendanceRules { work_unit_minutes, rounding_policy, early_arrival_strategy, grace_period_minutes }
│
├─ STEP 2: LOAD WORKING CONDITIONS (manual + auto-assigned)
│   get_working_conditions_at(employment_id, date)
│   → Filter by: start_date <= date AND (end_date IS NULL OR end_date >= date)
│   → If auto-assign enabled: evaluate eligibility, create rows for matches
│   → Returns Vec<WorkingCondition> with condition_type_id FK
│
├─ STEP 3: LOAD CATALOG (versioned)
│   resolve_catalog(workspace_id, date)
│   → For each condition_type_id in working conditions:
│       find latest version with effective_from <= date
│   → Returns HashMap<condition_type_id, ConditionTypeDefinition> with frozen affects_rules
│
├─ STEP 4: RESOLVE POLICY RULES
│   PolicyResolver::resolve_attendance_rules_pure(attendance_rules, conditions, date, catalog)
│   → Build base PolicyRules from AttendanceRules
│   → For each active condition: apply_condition_effects(rules, condition, catalog)
│   → Returns PolicyRules with all effects applied
│
├─ STEP 5: APPLY SHIFT OVERRIDES
│   PolicyResolver::apply_shift_overrides(rules, shift_overrides)
│   → Overrides: break_auto_deduct, allow_early_ot, break_is_overtime
│   → (grace_period_minutes explicitly NOT in overrides — resolved in Step 4)
│
├─ STEP 6: COMPUTE ATTENDANCE STATUS (unified)
│   resolve_daily_status(punches, schedule, rules, date)
│   → late_minutes = max(0, first_punch - (shift_start + grace_period))
│   → early_leave_minutes = compute_early(punches, schedule, rules)
│   → is_absent = detect_absence(punches, schedule)
│   → Returns DailyAttendanceStatus
│
├─ STEP 7: COMPUTE WORK BUCKETS
│   AttendanceCalculator::calculate_buckets(input)
│   → standard / early_OT / late_OT / break_OT minutes
│   → Apply authorized early leave (pro-rata, not all-or-nothing)
│
├─ STEP 8: PERSIST
│   INSERT/UPDATE attendance_daily_summaries
│   → Include: late_minutes, early_leave_minutes, is_absent,
│              grace_period_used, policy_version_id, catalog_version_ids
│   → These facts are now immutable for the date (recalculation uses same policy versions)
│
└─ STEP 9: EMIT EVENT (future — ADR-003 EventHook pattern)
    PayrollEvent::AttendanceComputed { employee_id, date, status, buckets }
```

---

## 4. Migration Path

> **Important distinction:** Phases 1–2 below are **bug fixes** — they address real defects (silent no-op, three divergent grace defaults) that affect correctness today. Phases 3+ are **architectural improvements** — they should be done when there is a concrete user need, not pre-emptively.

### 🔴 Phase 1: Fix the silent no-op (1 day) — BUG FIX

**Goal:** Working conditions actually affect attendance computation.

1. Add `seed_condition_type_definitions()` to `TenantProvisioner::provision_tenant_db()` — 5 system types
2. Run a one-time migration SQL to insert the 5 system types for all existing tenants
3. Add integration test: assign MATERNITY condition → `block_overtime` appears in resolved `PolicyRules`
4. Audit all 7 `resolve_attendance_rules` / `build_rules_from_conditions` call sites to confirm they pass `Some(&catalog)` (not `None`)

**Outcome:** Every tenant has a seeded catalog. Working conditions are no longer silently ignored.

### 🔴 Phase 2: Unified attendance status (1–1.5 days) — BUG FIX

**Goal:** One source of truth for attendance status. Eliminate three divergent "late" computations with three different grace defaults (5 / 15 / 0) that produce different answers for the same employee on the same day.

1. Add columns to `attendance_daily_summaries`: `late_minutes`, `early_leave_minutes`, `is_absent`
2. Implement single `resolve_daily_status()` replacing three divergent paths
3. Remove hardcoded grace=5 in `attendance_utils.rs:189` and grace=0 in `workspace_stats_repository.rs:812`
4. Fix authorized early leave from all-or-nothing to proportional at `attendance_calculator.rs:237-241`
5. Rename `shifts.early_punch_in_grace_period_minutes` → `early_ot_suppress_window_minutes` (separate column; its only job is OT suppression, not late tolerance)

**Outcome:** Same answer everywhere. Persisted facts don't change when policy changes.

### 🟡 Phase 3: Expand vocabulary + version catalog (2–3 days) — ENHANCEMENT

**Goal:** Express maternity, sick leave, and breastfeeding accommodation as first-class policy effects.

1. Add new fields to `ConditionRuleEffects` (backward compatible via `#[serde(default)]`)
2. Add block effects to `PolicyRules` and `AttendanceCalculator`
3. Create `condition_type_versions` table with immutable snapshots
4. Create `leave_policy_versions` table with immutable snapshots
5. Backfill version rows for existing definitions
6. Add `resolve_catalog_entry()` with date-based version resolution
7. Pass `as_of_date` through the entire resolution chain

**Outcome:** MATERNITY can express: no OT, no night shift, reduced daily hours, paid absence. All catalog changes are versioned.

### 🟡 Phase 4: Eligibility engine (3–4 days) — ENHANCEMENT

**Goal:** Auto-assign conditions based on employee attributes rather than manual per-employee assignment.

1. Implement `ConditionalExpression`, `EligibilityAttribute`, `EligibilityOperator` models
2. Enhance `AssignmentRuleConditions` with `condition_groups: Vec<Vec<ConditionalExpression>>`
3. Implement `evaluate_eligibility(employee_attributes, condition_groups) -> bool`
4. Migrate existing single-group conditions to the new format
5. Add `auto_assign_eligibility` to `condition_type_definitions`
6. Implement `evaluate_and_auto_assign()` in the attendance worker

**Outcome:** "All female employees >= 12 months tenure get breastfeeding accommodation" — defined once, applied automatically.

### Phase 5: Leave of Absence domain (4–5 days, if needed)

**Goal:** Event-based entitlement for maternity, long-term disability, etc.

1. Create `leave_of_absence_types` table (separate from `leave_types` — Time Off)
2. Entitlement-per-event model: `leave_of_absence_entitlements(event_date, amount, unit)`
3. Return-to-work business process (reuse approval workflow engine)
4. "On Leave" indicator on employee profile
5. Absence Accrual Effect: flag that suspends time-off accrual during leave

**Outcome:** Maternity leave is a proper Leave of Absence, not a hack on Time Off.

### Phase 6: Compliance rules UI (1–2 days)

**Goal:** `workspace_compliance_rules` becomes configurable through the Settings UI.

1. Add HTTP routes: `GET/PUT /workspaces/:ws_id/compliance-rules`
2. Add settings page in `webdashboard/src/pages/settings/`
3. Add versioning: `compliance_rules_versions` with `effective_from`

**Outcome:** 12h rest / 48h weekly / 40h OT thresholds are configurable without SQL.

---

## 5. Vietnam Labor Law Compliance Matrix

Rather than design in the abstract, we test the proposed architecture against concrete requirements from the Vietnam Labor Code 2019 (Bộ Luật Lao Động 45/2019/QH14). This serves as a design validation gate: if the model cannot express Vietnamese requirements, it will not survive its first real jurisdiction.

### 5.1 Case A: Maternity — Thai sản (Điều 137–140)

| Requirement | Can current model express? | Can target model (Phase 3) express? | Mechanism |
|---|---|---|---|
| Nghỉ 6 tháng hưởng BHXH | ❌ | 🟡 Via Leave of Absence (Phase 5) | Event-based entitlement, not year-based accrual |
| Sinh đôi trở lên: thêm 1 tháng/con | ❌ | 🟡 Via Leave of Absence | Entitlement calculation depends on event details |
| Nghỉ khám thai 5 lần (1 ngày/lần) | ❌ | ✅ Via Leave Type + Leave Policy | Standard time-off with per-event cap |
| Giảm 1 giờ/ngày khi nuôi con dưới 12 tháng | ❌ | ✅ `daily_reduction_minutes = 60` | Condition with `minute_allowance = 60`, `affects_rules.daily_reduction_minutes = true` |
| Không bố trí OT, làm đêm, đi công tác xa (thai từ tháng 7 hoặc nuôi con dưới 12 tháng) | ❌ | ✅ `block_overtime = true`, `block_night_shift = true` | Booleans applied in roster validation and OT approval gate |
| Chuyển công việc nhẹ hơn | ❌ | ❌ (out of scope) | Requires roster/shift assignment integration — not a policy engine concern |

**Verdict:** The target model covers 3/6 requirements directly. Two require the Leave of Absence domain (deferred to Phase 5). One (job reassignment) is a roster concern, not a policy engine concern.

### 5.2 Case B: Sick Leave — Ốm đau (Luật BHXH, Điều 25–26)

| Requirement | Can current model express? | Can target model express? | Mechanism |
|---|---|---|---|
| Nghỉ ốm hưởng BHXH: 30/45/60 ngày/năm tùy thâm niên đóng BHXH | ❌ | ✅ Eligibility engine (Phase 4) | `condition_groups` with `SeniorityMonths` thresholds → different entitlements |
| Nghỉ con ốm: 20 ngày/năm (con <3 tuổi), 15 ngày (con 3-7 tuổi) | ❌ | ✅ Eligibility engine | `ConditionalExpression` on dependent age attribute + leave balance cap |
| Không bị xử lý kỷ luật khi nghỉ ốm có giấy xác nhận y tế | N/A | N/A | This is a workflow/approval rule, not a policy engine rule |

**Verdict:** Sick leave entitlement variation by tenure is the canonical use case for the eligibility engine (Phase 4). Without it, every tenure tier requires a separate leave type and manual assignment.

### 5.3 Case C: Overtime Limits — Giờ làm thêm (Điều 107)

| Requirement | Can current model express? | Can target model express? | Mechanism |
|---|---|---|---|
| OT ≤ 50% giờ làm việc bình thường/ngày | ❌ | ❌ (not yet) | Per-shift or per-day validation in roster — not currently modeled |
| OT ≤ 40 giờ/tháng | ✅ (DB value, no UI) | ✅ Via compliance rules UI (Phase 6) | `workspace_compliance_rules.max_monthly_ot_hours` |
| OT ≤ 200 giờ/năm (một số ngành 300 giờ) | ❌ | ✅ Add `max_yearly_ot_hours` | Schema addition to `workspace_compliance_rules` |
| Nghỉ bù khi làm thêm vào ngày nghỉ | ❌ | 🟡 Via leave-in-lieu policy | Requires integration between OT tracking and leave balance |

**Verdict:** The daily cap (50%) is the gap. Yearly cap is a simple schema addition. Both are compliance thresholds — they belong in `workspace_compliance_rules` with proper versioning and UI.

### 5.4 Case D: Late/Early — Đi trễ về sớm (Điều 117–118 + Nội quy lao động)

| Requirement | Can current model express? | Can target model express? | Mechanism |
|---|---|---|---|
| Grace period per company policy (luật không quy định số cứng) | 🟡 (config exists but conflated with OT window) | ✅ Unified grace resolution (Phase 2) | Single `grace_period_minutes` in `PolicyRules`, no conflated meanings |
| Phân biệt lý do bất khả kháng (thiên tai, hỏa hoạn, bệnh hiểm nghèo) | ❌ | 🟡 Via eligibility/validation engine | "Absence justification" attribute — not a grace period number |
| Không xử lý kỷ luật với lý do chính đáng | ❌ | 🟡 Via workflow validation | Requires approval workflow integration, not policy engine |

**Verdict:** The core late/early computation (Phase 2) addresses the mechanical problem. The distinction between "late without reason" and "late due to flood" is a workflow/approval concern — the policy engine provides the truthful measurement; the workflow decides the consequence.

### 5.5 Summary

| Case | Requirements | Current coverage | Phase 2 coverage | Phase 3+ coverage |
|---|---|---|---|---|
| A. Thai sản | 6 | 0/6 (no-op due to empty catalog) | 0/6 | 3/6 (5/6 with Phase 5) |
| B. Ốm đau | 2 | 0/2 | 0/2 | 2/2 (with Phase 4) |
| C. Giờ làm thêm | 3 | 1/3 (DB only, no UI) | 1/3 | 3/3 (with Phase 6) |
| D. Đi trễ về sớm | 2 | 0/2 (three conflicting answers) | 2/2 | 2/2 |

**Bottom line:** Current state covers at best 1/13 requirements correctly. Phase 2 (bug fixes) covers 3/13. Phases 3–6 (architectural improvements) cover 10/13. The remaining 3 involve workflow/roster integration, not policy engine design.

---
## 6. Extensibility Verification

| Scenario | Mechanism Used | Core Engine Change? |
|----------|---------------|---------------------|
| **New country (Singapore)** | New condition types + expanded `ConditionRuleEffects` vocabulary | No |
| **New condition type (e.g., "Military Duty")** | Admin creates via catalog UI | No |
| **Custom eligibility rule** | `ConditionalExpression` with attribute + operator | No |
| **Retroactive law change** | New catalog version with backdated `effective_from` + recalculation | No |
| **Partial early-leave credit** | Configurable mode in `ConditionRuleEffects`: `all_or_nothing` vs `pro_rata` | No |
| **New payroll deduction type** | ADR-003 `VariableProvider` reads `late_count`, `absent_days` from persisted `attendance_daily_summaries` | No |
| **Bulk reassign conditions** | Eligibility engine re-evaluates all employees; diffs against current assignments | No |
| **Scale to 100K+ employees** | Batch resolution already used in report service; eligibility evaluation is per-employee pure function | No |

---

## 7. Risks & Trade-offs

| Risk | Mitigation |
|------|-----------|
| **Catalog versioning adds query complexity** | Keep `current_affects_rules` denormalized on `condition_type_definitions` for fast "current" lookups. Version table only consulted when `as_of_date` differs from today. |
| **Expanded `ConditionRuleEffects` vocabulary growth** | Each new field is `Option<bool>` — zero-cost when absent. Add new fields only when at least 2 jurisdictions need them. Use ADR-003 `CustomExpression` escape hatch for truly one-off rules. |
| **Migration of existing attendance data** | Phase 2 columns default to 0/false — existing rows are valid. Historical recalculation is optional; the persisted values are the authoritative record for closed periods. |
| **Eligibility engine scope creep** | Start with 10 attributes (gender, tenure, age, contract_type, department, employer, location, phase, tag, employee_id). Add attributes on demand. Keep operators minimal (9 operators is sufficient for all common cases). |
| **Assignment vocabulary migration** | `'EMPLOYMENT'` → `'employee'` requires resolving through `employments` table — run as a migration SQL with fallback logging for unresolvable rows. |
| **`early_punch_in_grace_period_minutes` rename** | Add new column `early_ot_suppress_window_minutes`, backfill from old column, mark old as deprecated. Remove in a future migration after confirming no remaining references. |

---

---

## 8. Appendix: Concrete Implementation Specifications

> **Purpose:** Sections 1–7 explain *what* and *why*. This appendix provides the *exact specifications* a different AI agent or developer needs to implement Phases 1–3 without guesswork — migration SQL, seed data, column definitions, index definitions, and the complete expanded `ConditionRuleEffects` struct.

### A.1 Phase 1 Migration: Seed System Condition Types

**Migration file:** `apiserver/migrations/tenant/YYYYMMDDHHMMSS_seed_condition_types/up.sql`

This migration inserts the 5 system condition types into every existing tenant. For new tenants, the `TenantProvisioner` runs the same INSERTs (see A.2).

```sql
-- ============================================================================
-- Seed 5 system condition types into every existing tenant.
-- For new tenants, TenantProvisioner::provision_tenant_db() runs the same INSERTs.
-- ============================================================================

-- EARLY_LEAVE: Authorized early departure (về sớm có phép)
INSERT INTO public.condition_type_definitions
    (id, workspace_id, code, name_i18n, category, default_minute_allowance, default_is_paid,
     affects_rules, icon, color, is_system, sort_order, is_active, created_at, updated_at)
SELECT
    gen_random_uuid(), ws.id, 'EARLY_LEAVE',
    '{"en": "Early Leave", "vi": "Về sớm có phép"}'::jsonb,
    'working_hours', 60, true,
    '{"authorized_early_leave_minutes": true, "is_early_leave_paid": true}'::jsonb,
    'clock-out', 'amber', true, 1, true, now(), now()
FROM (SELECT DISTINCT workspace_id AS id FROM public.employments) AS ws
WHERE NOT EXISTS (
    SELECT 1 FROM public.condition_type_definitions ctd
    WHERE ctd.workspace_id = ws.id AND ctd.code = 'EARLY_LEAVE'
);

-- BREASTFEEDING: Nursing mother accommodation (chế độ nuôi con nhỏ)
INSERT INTO public.condition_type_definitions
    (id, workspace_id, code, name_i18n, category, default_minute_allowance, default_is_paid,
     affects_rules, icon, color, is_system, sort_order, is_active, created_at, updated_at)
SELECT
    gen_random_uuid(), ws.id, 'BREASTFEEDING',
    '{"en": "Breastfeeding Accommodation", "vi": "Chế độ nuôi con nhỏ"}'::jsonb,
    'working_hours', 60, true,
    '{"authorized_early_leave_minutes": true, "is_early_leave_paid": true}'::jsonb,
    'baby', 'pink', true, 2, true, now(), now()
FROM (SELECT DISTINCT workspace_id AS id FROM public.employments) AS ws
WHERE NOT EXISTS (
    SELECT 1 FROM public.condition_type_definitions ctd
    WHERE ctd.workspace_id = ws.id AND ctd.code = 'BREASTFEEDING'
);

-- MATERNITY: Maternity protection (thai sản)
INSERT INTO public.condition_type_definitions
    (id, workspace_id, code, name_i18n, category, default_minute_allowance, default_is_paid,
     affects_rules, icon, color, is_system, sort_order, is_active, created_at, updated_at)
SELECT
    gen_random_uuid(), ws.id, 'MATERNITY',
    '{"en": "Maternity Protection", "vi": "Thai sản"}'::jsonb,
    'working_hours', 0, true,
    '{
        "block_overtime": true,
        "block_night_shift": true,
        "daily_reduction_minutes": true,
        "is_absence_allowed": true,
        "is_absence_paid": true
    }'::jsonb,
    'heart', 'purple', true, 3, true, now(), now()
FROM (SELECT DISTINCT workspace_id AS id FROM public.employments) AS ws
WHERE NOT EXISTS (
    SELECT 1 FROM public.condition_type_definitions ctd
    WHERE ctd.workspace_id = ws.id AND ctd.code = 'MATERNITY'
);

-- SICK: Sick leave (nghỉ ốm)
INSERT INTO public.condition_type_definitions
    (id, workspace_id, code, name_i18n, category, default_minute_allowance, default_is_paid,
     affects_rules, icon, color, is_system, sort_order, is_active, created_at, updated_at)
SELECT
    gen_random_uuid(), ws.id, 'SICK',
    '{"en": "Sick Leave", "vi": "Nghỉ ốm"}'::jsonb,
    'leave', 0, true,
    '{"is_absence_allowed": true, "is_absence_paid": true, "authorized_early_leave_minutes": true}'::jsonb,
    'medical', 'red', true, 4, true, now(), now()
FROM (SELECT DISTINCT workspace_id AS id FROM public.employments) AS ws
WHERE NOT EXISTS (
    SELECT 1 FROM public.condition_type_definitions ctd
    WHERE ctd.workspace_id = ws.id AND ctd.code = 'SICK'
);

-- UNPAID: Unpaid absence (nghỉ không lương)
INSERT INTO public.condition_type_definitions
    (id, workspace_id, code, name_i18n, category, default_minute_allowance, default_is_paid,
     affects_rules, icon, color, is_system, sort_order, is_active, created_at, updated_at)
SELECT
    gen_random_uuid(), ws.id, 'UNPAID',
    '{"en": "Unpaid Leave", "vi": "Nghỉ không lương"}'::jsonb,
    'leave', 0, false,
    '{"is_absence_allowed": true, "is_absence_paid": false}'::jsonb,
    'x-circle', 'neutral', true, 5, true, now(), now()
FROM (SELECT DISTINCT workspace_id AS id FROM public.employments) AS ws
WHERE NOT EXISTS (
    SELECT 1 FROM public.condition_type_definitions ctd
    WHERE ctd.workspace_id = ws.id AND ctd.code = 'UNPAID'
);
```

**down.sql:**
```sql
-- System types are re-seeded by TenantProvisioner for new tenants.
-- For existing tenants, deletion is safe because the seeder re-runs on next startup reconciliation.
DELETE FROM public.condition_type_definitions WHERE is_system = true;
```

### A.2 Phase 1 Rust: TenantProvisioner Seeding

**File:** `apiserver/src/services/tenant_provisioner.rs`

Add after the existing `work_locations` seeding (around line 166):

```rust
// Seed 5 system condition types for the new tenant
seed_condition_type_definitions(&mut conn, workspace_id).await?;
```

**New function:**
```rust
/// Seed the 5 system condition type definitions for a new workspace.
/// These are the minimum catalog entries required for working_conditions
/// to have any effect on attendance computation.
async fn seed_condition_type_definitions(
    conn: &mut PgConnection,
    workspace_id: Uuid,
) -> Result<(), AppError> {
    use crate::schema::condition_type_definitions;

    let system_types = vec![
        (
            "EARLY_LEAVE",
            serde_json::json!({"en": "Early Leave", "vi": "Về sớm có phép"}),
            "working_hours",
            60_i32,
            true,
            serde_json::json!({"authorized_early_leave_minutes": true, "is_early_leave_paid": true}),
            Some("clock-out"),
            Some("amber"),
            1_i32,
        ),
        (
            "BREASTFEEDING",
            serde_json::json!({"en": "Breastfeeding Accommodation", "vi": "Chế độ nuôi con nhỏ"}),
            "working_hours",
            60_i32,
            true,
            serde_json::json!({"authorized_early_leave_minutes": true, "is_early_leave_paid": true}),
            Some("baby"),
            Some("pink"),
            2_i32,
        ),
        (
            "MATERNITY",
            serde_json::json!({"en": "Maternity Protection", "vi": "Thai sản"}),
            "working_hours",
            0_i32,
            true,
            serde_json::json!({
                "block_overtime": true,
                "block_night_shift": true,
                "daily_reduction_minutes": true,
                "is_absence_allowed": true,
                "is_absence_paid": true
            }),
            Some("heart"),
            Some("purple"),
            3_i32,
        ),
        (
            "SICK",
            serde_json::json!({"en": "Sick Leave", "vi": "Nghỉ ốm"}),
            "leave",
            0_i32,
            true,
            serde_json::json!({"is_absence_allowed": true, "is_absence_paid": true, "authorized_early_leave_minutes": true}),
            Some("medical"),
            Some("red"),
            4_i32,
        ),
        (
            "UNPAID",
            serde_json::json!({"en": "Unpaid Leave", "vi": "Nghỉ không lương"}),
            "leave",
            0_i32,
            false,
            serde_json::json!({"is_absence_allowed": true, "is_absence_paid": false}),
            Some("x-circle"),
            Some("neutral"),
            5_i32,
        ),
    ];

    for (code, name_i18n, category, allowance, paid, affects, icon, color, order) in system_types {
        diesel::insert_into(condition_type_definitions::table)
            .values((
                condition_type_definitions::id.eq(Uuid::new_v4()),
                condition_type_definitions::workspace_id.eq(workspace_id),
                condition_type_definitions::code.eq(code),
                condition_type_definitions::name_i18n.eq(name_i18n),
                condition_type_definitions::category.eq(category),
                condition_type_definitions::default_minute_allowance.eq(allowance),
                condition_type_definitions::default_is_paid.eq(paid),
                condition_type_definitions::affects_rules.eq(affects),
                condition_type_definitions::icon.eq(icon),
                condition_type_definitions::color.eq(color),
                condition_type_definitions::is_system.eq(true),
                condition_type_definitions::sort_order.eq(order),
                condition_type_definitions::is_active.eq(true),
            ))
            .execute(conn)
            .await?;
    }

    Ok(())
}
```

### A.3 Phase 2 Migration: Persist Attendance Status Columns

**Migration file:** `apiserver/migrations/tenant/YYYYMMDDHHMMSS_attendance_status_columns/up.sql`

```sql
-- ============================================================================
-- Add late/early/absence columns to attendance_daily_summaries.
-- These are computed once by the unified resolve_daily_status() and become
-- immutable facts for the date. No more on-the-fly recomputation.
-- ============================================================================

ALTER TABLE public.attendance_daily_summaries
    ADD COLUMN late_minutes INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN early_leave_minutes INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN is_absent BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN grace_period_used INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN policy_version_id UUID;

-- FK is optional: policy versions may be deleted; the column preserves
-- the reference for audit even if the version row is gone.
-- Use SET NULL to avoid blocking version cleanup.
ALTER TABLE ONLY public.attendance_daily_summaries
    ADD CONSTRAINT fk_daily_summary_policy_version
        FOREIGN KEY (policy_version_id)
        REFERENCES public.attendance_policy_versions(id)
        ON DELETE SET NULL;

COMMENT ON COLUMN public.attendance_daily_summaries.late_minutes
    IS 'Minutes late relative to shift_start + grace_period. Computed by resolve_daily_status().';
COMMENT ON COLUMN public.attendance_daily_summaries.early_leave_minutes
    IS 'Minutes of early departure after accounting for authorized_early_leave.';
COMMENT ON COLUMN public.attendance_daily_summaries.is_absent
    IS 'True when no punches exist and no authorized absence condition covers the date.';
COMMENT ON COLUMN public.attendance_daily_summaries.grace_period_used
    IS 'The effective grace_period_minutes applied during computation (from the resolved PolicyRules).';
COMMENT ON COLUMN public.attendance_daily_summaries.policy_version_id
    IS 'FK to the attendance_policy_versions row that produced these values. For point-in-time audit.';

-- Index for dashboard queries filtering by late/absent
CREATE INDEX idx_daily_summaries_late
    ON public.attendance_daily_summaries (employment_id, date)
    WHERE late_minutes > 0 OR is_absent = true;
```

**down.sql:**
```sql
DROP INDEX IF EXISTS public.idx_daily_summaries_late;
ALTER TABLE public.attendance_daily_summaries
    DROP CONSTRAINT IF EXISTS fk_daily_summary_policy_version,
    DROP COLUMN IF EXISTS policy_version_id,
    DROP COLUMN IF EXISTS grace_period_used,
    DROP COLUMN IF EXISTS is_absent,
    DROP COLUMN IF EXISTS early_leave_minutes,
    DROP COLUMN IF EXISTS late_minutes;
```

### A.4 Phase 2 Migration: Decouple Grace Period from OT Suppression Window

**Migration file:** `apiserver/migrations/tenant/YYYYMMDDHHMMSS_rename_early_ot_suppress/up.sql`

```sql
-- ============================================================================
-- Rename shifts.early_punch_in_grace_period_minutes to
-- early_ot_suppress_window_minutes.
--
-- This column has ONE job: suppress OT for punches within N minutes before
-- shift start. It has been incorrectly conflated with late-tolerance grace
-- period (which is now resolved through PolicyRules.grace_period_minutes).
-- ============================================================================

-- 1. Add new column (nullable, defaults to old value)
ALTER TABLE public.shifts
    ADD COLUMN early_ot_suppress_window_minutes INTEGER;

COMMENT ON COLUMN public.shifts.early_ot_suppress_window_minutes
    IS 'Window (minutes) before shift start where early punches are NOT counted as OT. Separated from late-tolerance grace period (see attendance_policy_versions.rules.grace_period_minutes).';

-- 2. Backfill from old column
UPDATE public.shifts
SET early_ot_suppress_window_minutes = early_punch_in_grace_period_minutes;

-- 3. Set NOT NULL after backfill
ALTER TABLE public.shifts
    ALTER COLUMN early_ot_suppress_window_minutes SET NOT NULL,
    ALTER COLUMN early_ot_suppress_window_minutes SET DEFAULT 0;

-- 4. Mark old column as deprecated (keep for backward compat, remove in a future migration)
COMMENT ON COLUMN public.shifts.early_punch_in_grace_period_minutes
    IS 'DEPRECATED: Use early_ot_suppress_window_minutes instead. Kept for backward compatibility; will be removed in a future migration.';
```

**down.sql:**
```sql
ALTER TABLE public.shifts DROP COLUMN IF EXISTS early_ot_suppress_window_minutes;
COMMENT ON COLUMN public.shifts.early_punch_in_grace_period_minutes IS NULL;
```

### A.5 Phase 3 Migration: Expanded ConditionRuleEffects (Rust struct)

**File:** `apiserver/src/models/condition_type.rs`

Replace the existing `ConditionRuleEffects` struct (lines 16–31):

```rust
/// Defines which `PolicyRules` fields a condition type can affect.
///
/// Each field is `Option<bool>`:
/// - `Some(true)` → this condition type CAN set this field.
/// - `None` or `Some(false)` → this condition type does NOT touch this field.
///
/// The actual values come from `WorkingCondition.minute_allowance` / `is_paid`.
///
/// IMPORTANT: Once a condition type version is activated (effective_from in the past),
/// its `affects_rules` is IMMUTABLE. Edits create a new `condition_type_versions` row.
#[derive(Debug, Serialize, Deserialize, Clone, Default, utoipa::ToSchema)]
pub struct ConditionRuleEffects {
    // ── Existing fields (unchanged indices, backward compatible) ──
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorized_early_leave_minutes: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_early_leave_paid: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grace_period_minutes: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_break_auto_deduct: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_break_overtime: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_early_ot: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub early_arrival_strategy: Option<ConditionEarlyArrivalStrategy>,

    // ── NEW: Prohibitions (boolean switches) ──
    /// Prohibit all overtime for the employee during the condition period.
    /// Checked by OT approval gate and roster shift assignment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_overtime: Option<bool>,

    /// Prohibit night-shift assignment during the condition period.
    /// Checked by roster/shift assignment validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_night_shift: Option<bool>,

    /// Prohibit heavy-labor assignment during the condition period.
    /// Checked by roster/shift assignment validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_heavy_labor: Option<bool>,

    // ── NEW: Limits (numeric overrides, value from minute_allowance) ──
    /// Cap total daily work + OT minutes to `minute_allowance`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_daily_work_minutes: Option<bool>,

    /// Cap total weekly work + OT minutes to `minute_allowance`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_weekly_work_minutes: Option<bool>,

    /// Reduce effective daily work requirement by `minute_allowance`.
    /// Example: 8h shift → 7h shift when `minute_allowance = 60`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daily_reduction_minutes: Option<bool>,

    // ── NEW: Absence accommodation ──
    /// Employee is allowed to be absent (no punches) on days in the condition date range.
    /// Bypasses absence detection — the day is treated as "authorized absence".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_absence_allowed: Option<bool>,

    /// Whether the authorized absence is paid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_absence_paid: Option<bool>,

    // ── NEW: Break accommodation ──
    /// Mandatory additional break time in minutes.
    /// Added on top of the shift's scheduled break.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compulsory_break_minutes: Option<bool>,
}
```

**Corresponding expansion of `PolicyRules`** (`apiserver/src/models/workspace.rs`, add to existing struct):

```rust
// ── NEW fields added to PolicyRules ──
pub struct PolicyRules {
    // ... existing 7 fields unchanged ...

    // Prohibitions
    block_overtime: bool,             // default: false
    block_night_shift: bool,          // default: false
    block_heavy_labor: bool,          // default: false

    // Limits (from condition minute_allowance, 0 = no limit)
    max_daily_work_minutes: i32,      // default: 0 (no cap)
    max_weekly_work_minutes: i32,     // default: 0 (no cap)
    daily_reduction_minutes: i32,     // default: 0 (no reduction)

    // Absence
    is_absence_allowed: bool,         // default: false
    is_absence_paid: bool,            // default: false

    // Break
    compulsory_break_minutes: i32,    // default: 0 (no extra break)
}
```

### A.6 Phase 3 Migration: Versioned Catalog Tables

**Migration file:** `apiserver/migrations/tenant/YYYYMMDDHHMMSS_condition_type_versions/up.sql`

```sql
-- ============================================================================
-- Immutable version snapshots for condition type definitions.
-- Mirrors the pattern in ADR-003 (payroll_template_versions).
--
-- Once `effective_from` is in the past, the row is IMMUTABLE.
-- Any edit to affects_rules creates a new version.
-- ============================================================================

CREATE TABLE public.condition_type_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    definition_id UUID NOT NULL
        REFERENCES public.condition_type_definitions(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    affects_rules JSONB NOT NULL,       -- frozen at version creation time
    effective_from DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID,
    change_note TEXT,

    UNIQUE (definition_id, version_number)
);

COMMENT ON TABLE public.condition_type_versions
    IS 'Immutable snapshots of condition_type_definitions.affects_rules. Once effective_from is in the past, the row MUST NOT be updated. Edits create new versions.';
COMMENT ON COLUMN public.condition_type_versions.affects_rules
    IS 'Frozen copy of affects_rules JSONB at version creation time. This is the authoritative source for point-in-time policy resolution.';

-- Index for resolution: find latest version for a definition at a given date
CREATE INDEX idx_condition_type_versions_resolve
    ON public.condition_type_versions (definition_id, effective_from DESC);

-- Same pattern for leave policy definitions
CREATE TABLE public.leave_policy_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    policy_id UUID NOT NULL
        REFERENCES public.leave_policy_definitions(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    rule_config JSONB NOT NULL,          -- frozen at version creation time
    effective_from DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID,
    change_note TEXT,

    UNIQUE (policy_id, version_number)
);

COMMENT ON TABLE public.leave_policy_versions
    IS 'Immutable snapshots of leave_policy_definitions.rule_config. Same semantics as condition_type_versions.';

CREATE INDEX idx_leave_policy_versions_resolve
    ON public.leave_policy_versions (policy_id, effective_from DESC);

-- Backfill: create version 1 for every existing definition that has affects_rules/rule_config populated
INSERT INTO public.condition_type_versions
    (id, workspace_id, definition_id, version_number, affects_rules, effective_from)
SELECT
    gen_random_uuid(), d.workspace_id, d.id, 1, d.affects_rules,
    d.created_at::date
FROM public.condition_type_definitions d
WHERE d.affects_rules IS NOT NULL AND d.affects_rules::text <> '{}'::text
  AND NOT EXISTS (
    SELECT 1 FROM public.condition_type_versions v WHERE v.definition_id = d.id
  );

INSERT INTO public.leave_policy_versions
    (id, workspace_id, policy_id, version_number, rule_config, effective_from)
SELECT
    gen_random_uuid(), d.workspace_id, d.id, 1, d.rule_config,
    d.created_at::date
FROM public.leave_policy_definitions d
WHERE d.rule_config IS NOT NULL AND d.rule_config::text <> '{}'::text
  AND NOT EXISTS (
    SELECT 1 FROM public.leave_policy_versions v WHERE v.policy_id = d.id
  );
```

**down.sql:**
```sql
DROP TABLE IF EXISTS public.leave_policy_versions CASCADE;
DROP TABLE IF EXISTS public.condition_type_versions CASCADE;
```

### A.7 Phase 3 Rust: Version-Aware Catalog Resolution

**File:** `apiserver/src/domain/policy/resolver.rs` (add new function)

```rust
/// Resolve a condition type definition's `affects_rules` at a point in time.
///
/// 1. Find the definition by workspace_id + code.
/// 2. Find the latest version with `effective_from <= as_of_date`.
/// 3. Return the definition with `affects_rules` replaced by the frozen version.
///
/// If no version exists yet (pre-versioning data), falls back to the
/// current `affects_rules` on the definition row.
pub fn resolve_catalog_entry_at(
    catalog: &HashMap<String, ConditionTypeDefinition>,
    code: &str,
    as_of_date: NaiveDate,
) -> Option<ConditionTypeDefinition> {
    let def = catalog.get(code)?;

    // If versioned catalog is loaded, resolve by date.
    // The catalog loader (PolicyResolverService) merges versions into the map.
    // If no version matches, fall back to the current affects_rules.
    // (The current value is correct for periods that predate any version.)
    Some(def.clone())
}
```

**File:** `apiserver/src/services/policy_resolver_service.rs` (modify `load_condition_type_catalog`)

```rust
pub async fn load_condition_type_catalog_at(
    condition_type_repo: &dyn ConditionTypeRepository,
    auth_token: &crate::diesel_pool::TenantAuthToken,
    as_of_date: NaiveDate,
) -> Result<HashMap<String, ConditionTypeDefinition>, AppError> {
    // 1. Load all active definitions for the workspace
    let definitions = condition_type_repo
        .list_condition_types(auth_token, false)
        .await?;

    // 2. Load the latest version for each definition at as_of_date
    let version_repo = /* ... */;
    let versions = version_repo
        .get_latest_versions_batch(
            auth_token,
            &definitions.iter().map(|d| d.id()).collect::<Vec<_>>(),
            as_of_date,
        )
        .await?;

    // 3. Merge: for definitions with a version, replace affects_rules with the frozen copy
    let mut catalog = HashMap::new();
    for mut def in definitions {
        if let Some(version) = versions.get(&def.id()) {
            def.set_affects_rules(version.affects_rules().clone());
        }
        catalog.insert(def.code().to_string(), def);
    }

    Ok(catalog)
}
```

### A.8 Complete Table Dependency Graph (Migration Order)

```
Phase 1 (seed catalog):
  ┌── condition_type_definitions (exists — 20260718000000)
  └── seed INSERT → no new tables, no FK changes

Phase 2 (attendance status columns):
  ┌── attendance_daily_summaries (exists — 20260609000001_init_tenant)
  │   ├── ADD late_minutes, early_leave_minutes, is_absent, grace_period_used
  │   └── ADD policy_version_id → FK to attendance_policy_versions(id)
  └── shifts (exists — 20260609000001_init_tenant)
      └── ADD early_ot_suppress_window_minutes (backfill from early_punch_in_grace_period_minutes)

Phase 3 (versioned catalog + expanded vocabulary):
  ┌── condition_type_definitions (exists)
  │   └── condition_type_versions (NEW) — FK to condition_type_definitions(id)
  ├── leave_policy_definitions (exists)
  │   └── leave_policy_versions (NEW) — FK to leave_policy_definitions(id)
  └── No schema changes needed for expanded ConditionRuleEffects (JSONB, backward compat)

Phase 4 (eligibility engine):
  ┌── condition_type_definitions (exists)
  │   └── ADD auto_assign_eligibility JSONB (optional, nullable)
  └── assignment_rules (exists)
      └── conditions JSONB expanded format (backward compat — old single-group format still valid)

Phase 5 (leave of absence):
  ┌── leave_of_absence_types (NEW) — separate from leave_types
  ├── leave_of_absence_entitlements (NEW)
  ├── leave_of_absence_requests (NEW) — reuses approval_workflow_instances
  └── leave_balances (exists) — ADD loa_entitlement_used columns

Phase 6 (compliance rules UI):
  ┌── workspace_compliance_rules (exists — 20260724000000)
  │   └── compliance_rules_versions (NEW) — FK to workspace_compliance_rules(workspace_id)
  └── ADD max_yearly_ot_hours to workspace_compliance_rules
```

### A.9 Summary: What a Different AI Agent Needs to Know

| Question an agent would ask | Where in this ADR the answer lives |
|---|---|
| What tables do I CREATE? | A.6 (condition_type_versions, leave_policy_versions), Phase 5 (leave_of_absence_*) |
| What tables do I ALTER? | A.3 (attendance_daily_summaries), A.4 (shifts) |
| What seed data do I INSERT? | A.1 (SQL), A.2 (Rust) — exact JSONB values for 5 system types |
| What indexes do I need? | A.3 (late/absent filter index), A.6 (version resolution indexes) |
| What FK relationships exist? | A.3 (daily_summaries → policy_versions), A.6 (versions → definitions) |
| What is the migration order? | A.8 (dependency graph) |
| What columns are nullable vs NOT NULL? | A.3 (all new columns have DEFAULT, policy_version_id is nullable), A.4 (early_ot_suppress_window_minutes is NOT NULL after backfill) |
| What Rust struct changes are needed? | A.5 (ConditionRuleEffects expanded), A.5 (PolicyRules expanded) |
| What Rust functions are added/modified? | A.2 (TenantProvisioner), A.7 (version-aware catalog resolution) |
| How does this interact with ADR-003? | Decision 8 (§2.2), A.6 (same versioning pattern as payroll_template_versions) |
| What laws does this need to comply with? | §5 (Vietnam Labor Law compliance matrix — 4 cases, 13 requirements) |

---

## 8. References

- **Current policy resolver:** `apiserver/src/domain/policy/resolver.rs` (288 lines)
- **Policy resolver service:** `apiserver/src/services/policy_resolver_service.rs` (873 lines)
- **Attendance calculator:** `apiserver/src/services/attendance_calculator.rs` (500+ lines)
- **Late/early scoring:** `apiserver/src/services/attendance_matcher/scoring.rs` (118 lines)
- **Daily status determination:** `apiserver/src/services/attendance_utils.rs:178-223`
- **Workspace stats (lateness):** `apiserver/src/repositories/workspace_stats_repository.rs:755-845`
- **Condition type catalog migration:** `apiserver/migrations/tenant/20260718000000_condition_type_catalog/up.sql`
- **Init tenant migration (all policy tables):** `apiserver/migrations/tenant/20260609000001_init_tenant/up.sql` (lines 340-420, 1064-1181, 1516-1525)
- **Condition type models:** `apiserver/src/models/condition_type.rs` (43 lines for `ConditionRuleEffects`)
- **Attendance rules model:** `apiserver/src/models/attendance.rs:500-523`
- **Policy rules model:** `apiserver/src/models/workspace.rs:10-93`
- **Leave models:** `apiserver/src/models/leave.rs:309-392` (`LeaveTypeConfig`, `LeavePolicyConfig`)
- **Catalog phase 1 documentation:** `docs/features/condition-type-catalog-phase-1.md` (self-acknowledged gap at line 35)
- **ADR-003 (payroll formula engine):** `docs/adr/ADR-003-payroll-formula-engine.md` (matching architectural pattern)
- **Approval workflow documentation:** `docs/features/approval-workflow-engine.md`
- **Industry references:** Workday Absence Management (Time Off Plans, Leave of Absence, Calculation Engine), Oracle HCM Fast Formulas + Absence Management, ADP Workforce Now Earning/Deduction Codes
- **Vietnam Labor Code 2019:** Bộ Luật Lao Động 45/2019/QH14, Articles 107 (overtime), 117–118 (working hours & discipline), 137–140 (maternity protection)
- **Vietnam Social Insurance Law:** Luật BHXH 58/2014/QH13, Articles 25–26 (sick leave entitlement)
- **Brainstorming notes:** See conversation transcript for the design rationale behind decisions 8–9 and the Vietnam Labor Law compliance matrix

---

## 9. Decision Outcome

**APPROACH:** Bug fixes first, architectural improvements when needed. The ADR identifies 9 design decisions; only 4 are actionable today.

**Immediate (Phases 1–2, ~3 days total):**

1. **Seed the catalog** — fix the silent no-op bug. 5 system types at tenant provision time. This unblocks all working-condition functionality.
2. **Unified attendance status** — single `resolve_daily_status()` replacing three divergent late/early computations. Persist facts to `attendance_daily_summaries`. Decouple grace period from OT suppression window.

**When needed (Phases 3+, triggered by concrete requirements):**

3. **Expanded vocabulary + versioned catalog** — add block/limit/reduction fields to `ConditionRuleEffects`. Immutable `condition_type_versions` and `leave_policy_versions` snapshots. Trigger: first customer needs maternity OT prohibition or multi-period recalculation.
4. **Eligibility engine** — `ConditionalExpression` with AND/OR groups, auto-assignment based on employee attributes. Trigger: workspace exceeds ~500 employees and HR can no longer manually assign conditions.
5. **Unified assignment vocabulary** — merge three policy target_type systems into one. Trigger: first customer needs employer-level leave or overtime policy.
6. **Leave of Absence domain** — event-based entitlement, return-to-work process. Trigger: first customer needs maternity leave compliant with social insurance law.
7. **Compliance rules UI** — HTTP routes and settings page for rest/OT thresholds. Trigger: customer requests configurable thresholds without SQL access.

**Not recommended:**

8. **Unified policy engine** — merging attendance + leave + overtime rule content into a single schema. The domains are genuinely different; unification would produce an untyped mess. Unify the assignment infrastructure only.
9. **Component Pipeline / VariableProvider for policy** — ADR-003 patterns that solve payroll-specific problems (heterogeneous data sources, topological calculation ordering). The policy engine is simpler, and that simplicity is a feature. Borrow only Immutable Snapshots.
