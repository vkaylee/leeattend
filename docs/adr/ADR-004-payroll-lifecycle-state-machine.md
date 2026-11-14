# ADR-004: Payroll Period Lifecycle State Machine

| Property       | Value                                                    |
|----------------|----------------------------------------------------------|
| **Status**     | Accepted — additive lifecycle implementation            |
| **Date**       | 2026-11-14                                               |
| **Author**     | AI + Lee                                                 |
| **Depends on** | [ADR-003](ADR-003-payroll-formula-engine.md)             |
| **Domain**     | Payroll Lifecycle & Governance                           |

---

## Implementation Reality (2026-11-14)

This ADR's original multi-stage `in_progress/verified/approved/committed/closed` model is future-state guidance. Current Go/PostgreSQL implementation keeps existing payroll period compatibility: `open → locked`, with optimistic version checks and audit rows for lock/reopen actions.

Formula/input lifecycles are separate and shipped:

- Input batch: `submitted → approved → locked`.
- Input row: `accepted | warning | rejected | corrected`.
- Formula version: `draft → published → retired`.
- Locked input batches are the only KPI calculation source.
- Locked/finalized payroll cannot be directly mutated; correction history is used before lock, and calculation snapshots link to payslips.
- Every successful input/formula transition emits a sanitized audit event. Authorization runs in handlers before repository calls.

Do not implement the richer state machine by changing current statuses in place. Add migration-safe states, permissions, APIs, tests, and UI together. Historical payroll results must remain unchanged after later formula publication.

---

## 1. Context & Problem Statement

### 1.1 Current State

The system today has a single payslip status: `"draft"` — hardcoded in `FinancialCalculator::assemble_payslip()` and never changed. There is:

- **No approval flow** for payroll periods
- **No locking mechanism** — a payslip can be recalculated at any time
- **No audit trail** — no record of who approved what, when
- **No segregation of duties** — the person who calculates can also approve

Permissions exist in the system (`payroll:read`, `payroll:approve`, `payroll:disburse`, `payroll:sensitive:read`) but are **not enforced** because the lifecycle states that would gate them don't exist.

### 1.2 Driving Forces

1. **Financial controls**: Payroll is the single largest expense for most companies. Errors mean real money lost. Every enterprise payroll system enforces a multi-stage approval process.
2. **Regulatory compliance**: In Vietnam, payroll data feeds into mandatory social insurance filings (BHXH) and tax declarations (TNCN). Once filed, the data is legally binding.
3. **Data integrity**: Attendance records, leave requests, and salary changes must be FROZEN for a period once payroll is committed — otherwise the same data could change mid-approval.
4. **Audit requirements**: ISO 27001, SOC 2, and Vietnamese accounting standards all require an immutable audit trail of who did what to payroll data.

### 1.3 Industry Reference

| System | Lifecycle Model |
|--------|----------------|
| Oracle HCM | Payroll Flow: Start → Calculate → Verify → Approve → Transfer → Close |
| ADP Workforce Now | Payroll Cycle: Preview → Process → Approve → Commit → Archive |
| Workday | Payroll Period: Open → In Progress → Calculated → Approved → Completed → Closed |
| SAP SuccessFactors | Payroll Control Center: Schedule → Run → Check → Approve → Post |

All four share the same pattern: **state machine with irreversible transitions after commit.**

---

## 2. Architecture Decision

### 2.1 Core Principle: Two-Level Lifecycle

Payroll governance operates at TWO levels, not one:

```
┌─────────────────────────────────────────────────────────────┐
│                   PAYROLL GOVERNANCE MODEL                   │
│                                                              │
│  LEVEL 1: PAYROLL PERIOD (workspace-wide)                    │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ "Tháng 2/2026 cho toàn bộ workspace"                  │   │
│  │                                                        │   │
│  │ Open → InProgress → Verified → Approved → Committed    │   │
│  │                                          ↓              │   │
│  │                               [GL Posted] [BHXH Filed]  │   │
│  │                                          ↓              │   │
│  │                                       Closed            │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  LEVEL 2: INDIVIDUAL PAYSLIP (per employment)                │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ "Payslip của Nguyễn Văn A, tháng 2/2026"             │   │
│  │                                                        │   │
│  │ Draft → Calculated → Verified → Approved → Committed   │   │
│  │    ↓          ↓           ↓                              │   │
│  │ Voided   Superseded   Superseded                         │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**Why two levels?**

- Level 1 (Period) controls **process-wide governance**: "Has the payroll been filed with tax authorities?"
- Level 2 (Payslip) controls **individual record integrity**: "This specific payslip has been corrected"

You can retrofit individual payslips inside a committed period (via Delta pattern — see ADR-005), but you cannot casually reopen the entire period.

### 2.2 Period State Machine (Level 1)

```
                          ┌──────────┐
                          │   OPEN   │ ← Period created, no calculations done
                          └────┬─────┘
                               │ start_calculation()
                               ▼
                         ┌───────────┐
                         │ IN_PROGRESS│ ← Batch calculation running
                         └─────┬─────┘
                               │ all_calculations_complete()
                               ▼
                         ┌───────────┐
                         │ CALCULATED │ ← All payslips in Draft/Calculated
                         └─────┬─────┘
                               │ verify_all() ← optional, can skip
                               ▼
                         ┌───────────┐
                         │  VERIFIED  │ ← HR manager confirmed totals
                         └─────┬─────┘
                               │ approve() ← ⚠️ SEGREGATED: cannot be calculator
                               ▼
                         ┌───────────┐
                         │ APPROVED   │ ← Ready for payment
                         └─────┬─────┘
                               │ commit() ← ⚠️ IRREVERSIBLE: freezes data
                               ▼
                         ┌───────────┐        ┌──────────────┐
                         │ COMMITTED  │──────→│   CLOSED     │
                         └───────────┘        │(tax filed,   │
                              │               │ permanently  │
                              │               │ locked)      │
                              ▼               └──────────────┘
                    [Individual payslip retro
                     via Delta — see ADR-005]
```

### 2.3 Payslip State Machine (Level 2)

```
                    calculate()
                         │
                         ▼
                    ┌─────────┐
                    │  DRAFT   │ ← Just calculated, no review
                    └────┬────┘
                         │ verify() (optional, can batch verify)
                         ▼
                    ┌──────────┐
                    │ VERIFIED  │ ← Reviewed by manager
                    └────┬─────┘
                         │ approve()
                         ▼
                    ┌──────────┐
                    │ APPROVED  │ ← Ready for payment
                    └────┬─────┘
                         │ commit() (period-level trigger)
                         ▼
                    ┌───────────┐
                    │ COMMITTED  │ ← Payment issued, data frozen
                    └─────┬─────┘
                          │ period_close()
                          ▼
                    ┌───────────┐
                    │  CLOSED    │ ← Tax filed, permanent lock
                    └───────────┘

          ┌────────────┐          ┌──────────────┐
          │  VOIDED    │          │ SUPERSEDED   │
          │ (explicit  │          │ (replaced by │
          │  cancel)   │          │  new version)│
          └────────────┘          └──────────────┘
               ↑                        ↑
               │                        │
          From DRAFT/             From any state
          VERIFIED only           (retro recalculation)
```

### 2.4 State Transition Rules

| From → To | Allowed Period State | Permission Required | Constraint |
|-----------|---------------------|---------------------|------------|
| Draft → Verified | InProgress, Calculated, Verified | `payroll:verify` | Must be assigned reviewer |
| Draft → Voided | InProgress, Calculated, Verified | `payroll:void` | Must provide reason |
| Verified → Draft | Verified | `payroll:unverify` | Segregated: unverify-er ≠ verifier |
| Verified → Approved | Verified, Approved | `payroll:approve` | **Segregated**: approver ≠ calculator |
| Approved → Verified | Approved | `payroll:unapprove` | Segregated, must provide reason |
| Any → Superseded | Any | Automatic (system) | Retro recalculation |
| Committed → Closed | Committed | `payroll:close` | Period reconciliation done |
| Closed → Committed | Closed | `payroll:reopen` + superadmin | ⛔ Extremely restricted |

### 2.5 Data Locking on Commit

When a **period** transitions to `COMMITTED`, the following data is **frozen for that period**:

| Data Entity | Lock Mechanism | Effect |
|------------|----------------|--------|
| `attendance_daily_summaries` | Snapshot hash per period | Cannot modify attendance within committed period |
| `leave_requests` | Status lock | Cannot approve/reject leave requests dated within committed period |
| `employee_salary_components` | Effective date cutoff | Cannot backdate salary changes into committed period |
| `overtime_requests` | Status lock | Cannot approve OT requests within committed period |
| `payroll_template_versions` | FK reference from payslip | Template version referenced by committed payslip is immutable |

The lock is implemented as **soft enforcement** (application-layer checks) rather than database triggers, because:
1. Database triggers are invisible to developers and hard to debug
2. Application-layer checks produce clear error messages: "Cannot modify leave request dated 2026-02-15: payroll period February 2026 is COMMITTED"

### 2.6 Segregation of Duties

```
┌─────────────────────────────────────────────────────┐
│              SEGREGATION OF DUTIES                   │
│                                                      │
│  Calculator ──→ Calculates payslip (Draft)           │
│  Verifier   ──→ Reviews for accuracy (Verified)      │
│  Approver   ──→ Authorizes payment (Approved)        │
│  Disburser  ──→ Executes payment (Committed)         │
│                                                      │
│  ⛔ RULES (enforced at application level):           │
│  • Calculator ≠ Approver                             │
│  • Calculator ≠ Disburser                            │
│  • Verifier ≠ Approver (same payslip)                │
│  • Same person CAN be Calculator + Verifier          │
│    (small companies — configurable per workspace)     │
│                                                      │
└─────────────────────────────────────────────────────┘
```

The segregation strictness is **configurable per workspace**:

| Mode | Calculator ≠ Approver | Calculator ≠ Disburser | Verifier ≠ Approver | Min Approvers |
|------|----------------------|------------------------|---------------------|---------------|
| `strict` | ✅ Required | ✅ Required | ✅ Required | 2 |
| `standard` (default) | ✅ Required | ✅ Required | ⚠️ Warning only | 1 |
| `relaxed` | ⚠️ Warning only | ✅ Required | — | 1 |
| `single_user` | — | — | — | 1 |

### 2.7 Database Schema Changes

```sql
-- Period-level lifecycle
CREATE TABLE payroll_periods (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    period_year INTEGER NOT NULL,
    period_month INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'open'
        CHECK (status IN ('open','in_progress','calculated','verified','approved','committed','closed')),
    calculator_id UUID,       -- who initiated the calculation
    verifier_id UUID,          -- who verified (can be NULL if skipped)
    approver_id UUID,          -- who approved
    committed_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (workspace_id, period_year, period_month)
);

-- Payslip-level lifecycle (extend existing payslips table)
ALTER TABLE payslips
    ADD COLUMN period_id UUID REFERENCES payroll_periods(id),
    ADD COLUMN status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft','verified','approved','committed','closed','voided','superseded')),
    ADD COLUMN superseded_by_id UUID REFERENCES payslips(id),
    ADD COLUMN voided_by UUID,
    ADD COLUMN voided_at TIMESTAMPTZ,
    ADD COLUMN void_reason TEXT,
    ADD COLUMN verified_by UUID,
    ADD COLUMN verified_at TIMESTAMPTZ,
    ADD COLUMN approved_by UUID,
    ADD COLUMN approved_at TIMESTAMPTZ;

-- Audit trail
CREATE TABLE payroll_period_audit_log (
    id UUID PRIMARY KEY,
    period_id UUID NOT NULL REFERENCES payroll_periods(id),
    actor_id UUID NOT NULL,
    action TEXT NOT NULL,          -- 'calculate','verify','approve','commit','close','reopen'
    from_status TEXT,
    to_status TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',   -- e.g., {"payslip_count": 125, "total_net": ...}
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Payslip-level audit
CREATE TABLE payroll_payslip_audit_log (
    id UUID PRIMARY KEY,
    payslip_id UUID NOT NULL REFERENCES payslips(id),
    actor_id UUID NOT NULL,
    action TEXT NOT NULL,          -- 'create','recalculate','verify','approve','void','supersede'
    from_status TEXT,
    to_status TEXT NOT NULL,
    reason TEXT,                   -- mandatory for void/supersede
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### 2.8 Enforcing Lifecycle in ADR-003 Component Pipeline

The lifecycle gates the calculation pipeline at specific points:

```rust
impl PayrollLifecycleService {
    /// Called before batch calculation starts
    pub fn start_period_calculation(
        period: &mut PayrollPeriod,
        calculator_id: Uuid,
    ) -> Result<(), LifecycleError> {
        match period.status() {
            "open" | "in_progress" => {
                period.transition_to("in_progress", calculator_id)?;
                Ok(())
            }
            "calculated" | "verified" => {
                // Recalculation allowed but must confirm
                // (triggers retro — see ADR-005)
                Ok(())
            }
            "committed" | "closed" => {
                Err(LifecycleError::PeriodLocked)
            }
            _ => Err(LifecycleError::InvalidTransition)
        }
    }

    /// Called before persisting an individual payslip
    pub fn validate_payslip_persist(
        period: &PayrollPeriod,
        existing_payslip: Option<&Payslip>,
    ) -> Result<(), LifecycleError> {
        match period.status() {
            "in_progress" | "calculated" | "verified" => Ok(()),
            "approved" => {
                // Can recalculate individual slips but via Delta pattern
                // (see ADR-005)
                Ok(())
            }
            "committed" | "closed" => {
                // Only via Retro Engine (see ADR-005)
                Err(LifecycleError::PeriodLockedForDirectEdit)
            }
            _ => Err(LifecycleError::InvalidTransition)
        }
    }
}
```

---

## 3. Integration with Existing System

### 3.1 Existing Permissions

The system already seeds these permissions in `system/20260609000000_init_system/up.sql`:

| Permission | Role Assignment | Lifecycle Mapping |
|-----------|----------------|-------------------|
| `payroll:read` | Owner, HR Manager, Accountant | Read-only access to all payslips |
| `payroll:approve` | Owner, HR Manager | `approve()` transition |
| `payroll:disburse` | Owner | `commit()` transition |
| `payroll:sensitive:read` | Owner, HR Manager | View unmasked PII/salary |
| `payroll:calculate` | Owner, HR Manager | `calculate()` trigger |
| `payroll:settings:manage` | Owner, HR Manager | Configure payroll params |
| `payroll:components:manage` | Owner, HR Manager | Manage salary components |
| `payroll:incentives:manage` | Owner, HR Manager | Manage incentive policies |
| `ess:payslips` | Employee | View own payslips only |

### 3.2 New Permissions Required

| Permission | Purpose |
|-----------|---------|
| `payroll:verify` | Transition payslip/period to Verified |
| `payroll:unverify` | Revert payslip/period to Draft |
| `payroll:unapprove` | Revert payslip/period to Verified |
| `payroll:void` | Void a payslip |
| `payroll:close` | Close a committed period permanently |
| `payroll:reopen` | Reopen a closed period (superadmin only) |
| `payroll:override_segregation` | Bypass segregation of duties (superadmin only) |

---

## 4. Risks & Trade-offs

| Risk | Mitigation |
|------|-----------|
| **Process too rigid for small companies** | Segregation strictness is configurable per workspace: `single_user` mode skips all checks |
| **Locked data breaks other modules** | Data locking is soft (app-layer), not hard (DB trigger). Errors are descriptive, not silent. |
| **Period state drifts from payslip states** | Period transition `all_calculations_complete()` validates that all payslips are at expected states |
| **Superadmin bypass creates audit gap** | All bypasses are logged in audit trail with explicit reason |
| **Too many states confuse users** | Frontend collapses into 3 visible stages: "Calculation", "Review", "Done" |

---

## 5. References

- [ADR-003: Payroll Formula Engine](ADR-003-payroll-formula-engine.md) — Core calculation pipeline
- [ADR-005: Retroactive Processing](ADR-005-retroactive-processing.md) — How recalculations interact with lifecycle
- Current permissions: `apiserver/migrations/system/20260609000000_init_system/up.sql` lines 351-548
- Industry: Oracle HCM Payroll Flow, ADP Workforce Now Payroll Cycle, Workday Payroll Period Management
