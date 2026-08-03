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

### 1.2 Driving Forces

1. **Correctness baseline**: A working-condition engine that silently no-ops is a data-integrity risk, not a missing feature. This is the #1 priority to fix.
2. **Single source of truth for attendance status**: Late/absence must be a persisted, policy-resolved, versioned fact, not three runtime computations with three defaults.
3. **Multi-country compliance**: Vietnam labor law has specific maternity, breastfeeding, sick-leave, and disability accommodation rules. The policy engine must express these without code changes per jurisdiction.
4. **Point-in-time correctness**: Any formula, rule, or catalog entry that affects payroll-adjacent attendance computation MUST be immutable after a period is closed. Editing in-place is a regulatory risk.
5. **Enterprise multi-tenant**: Each workspace defines its own condition types, leave policies, and overtime rules. The engine must support this with a consistent, typed vocabulary.
6. **Alignment with ADR-003**: The payroll formula engine ADR already established Component Pipeline + Immutable Snapshots + VariableProvider + EventHooks as the architectural pattern for this codebase. The attendance policy engine should adopt the same patterns.

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

## 5. Extensibility Verification

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

## 6. Risks & Trade-offs

| Risk | Mitigation |
|------|-----------|
| **Catalog versioning adds query complexity** | Keep `current_affects_rules` denormalized on `condition_type_definitions` for fast "current" lookups. Version table only consulted when `as_of_date` differs from today. |
| **Expanded `ConditionRuleEffects` vocabulary growth** | Each new field is `Option<bool>` — zero-cost when absent. Add new fields only when at least 2 jurisdictions need them. Use ADR-003 `CustomExpression` escape hatch for truly one-off rules. |
| **Migration of existing attendance data** | Phase 2 columns default to 0/false — existing rows are valid. Historical recalculation is optional; the persisted values are the authoritative record for closed periods. |
| **Eligibility engine scope creep** | Start with 10 attributes (gender, tenure, age, contract_type, department, employer, location, phase, tag, employee_id). Add attributes on demand. Keep operators minimal (9 operators is sufficient for all common cases). |
| **Assignment vocabulary migration** | `'EMPLOYMENT'` → `'employee'` requires resolving through `employments` table — run as a migration SQL with fallback logging for unresolvable rows. |
| **`early_punch_in_grace_period_minutes` rename** | Add new column `early_ot_suppress_window_minutes`, backfill from old column, mark old as deprecated. Remove in a future migration after confirming no remaining references. |

---

## 7. References

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

---

## 8. Decision Outcome

**APPROACH:** Four-dimensional refactor of the attendance policy engine:

1. **Seed the catalog** — fix the silent no-op bug by seeding 5 system condition types at tenant provision time
2. **Unified attendance status** — single `resolve_daily_status()` function, persist late/early/absence to `attendance_daily_summaries`, eliminate three divergent grace defaults
3. **Expanded vocabulary + versioned catalog** — add block/limit/reduction fields to `ConditionRuleEffects`, immutable `condition_type_versions` and `leave_policy_versions` snapshots
4. **Eligibility engine** — enhanced `AssignmentRuleConditions` with conditional expressions, auto-assignment of working conditions based on employee attributes

**RATIONALE:** This architecture:

1. Fixes a **production bug** (empty catalog → all working conditions no-op) before adding features
2. Aligns attendance policy resolution with the same Component Pipeline + Immutable Snapshots pattern established by ADR-003 for payroll
3. Meets **Vietnamese labor law requirements** for maternity, breastfeeding, and sick leave accommodation through an expanded-but-typed vocabulary
4. Eliminates **three divergent late-computation paths** with three different grace defaults, producing a single source of truth
5. Provides **point-in-time correctness** for all attendance status through persisted facts with policy version references
6. Enables **auto-assignment at scale** through an eligibility rule engine shared across assignment rules and working conditions
7. Builds incrementally: Phase 1 (bug fix, 1–2 days) delivers immediate value independent of later phases
