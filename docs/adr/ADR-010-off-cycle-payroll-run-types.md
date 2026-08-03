# ADR-010: Off-Cycle Payroll & Payroll Run Types

| Property       | Value                                                       |
|----------------|-------------------------------------------------------------|
| **Status**     | Proposed                                                    |
| **Date**       | 2026-08-03                                                  |
| **Author**     | AI + Lee                                                    |
| **Depends on** | [ADR-003](ADR-003-payroll-formula-engine.md), [ADR-004](ADR-004-payroll-lifecycle-state-machine.md), [ADR-006](ADR-006-accumulators-multi-period-context.md) |
| **Domain**     | Payroll — Run Management                                    |

---

## 1. Context

### 1.1 Beyond "One Month = One Payroll Run"

The common assumption that payroll runs exactly once per month per employee is wrong. Real-world payroll has multiple run types:

| Run Type | Example | Frequency | Uses Template? | Affects Accumulators? |
|----------|---------|-----------|:--------------:|:---------------------:|
| **Regular** | Monthly salary | Monthly | ✅ Default | ✅ All |
| **Bonus** | Tết bonus (2× monthly salary) | Annual / quarterly | ✅ Bonus template | ⚠️ PIT only |
| **Advance** | Tạm ứng lương giữa tháng | Ad-hoc | ✅ Advance template | ❌ |
| **Termination** | Final settlement on last day | Ad-hoc | ✅ Full template | ✅ All |
| **13th Month** | Tháng lương thứ 13 | Annual (Dec/Jan) | ✅ 13M template | ❌ |
| **Annual Settlement** | Quyết toán thuế cuối năm | Annual (Mar) | ✅ Settlement template | ✅ Reads all, writes none |

### 1.2 The Current ADR-003 Gap

ADR-003 assumes **one template per workspace per period**. But if a workspace has 3 runs in February (regular + advance + termination), we need **one template per run type**, not one per workspace.

---

## 2. Architecture Decision

### 2.1 Separate "Run" from "Period"

The current model conflates two concepts:

```
CURRENT (ADR-004):                    PROPOSED:
┌─────────────────────┐              ┌─────────────────────────┐
│ PayrollPeriod        │              │ PayrollPeriod            │
│ - year, month       │              │ - year, month           │
│ - status            │              │ - status                │
│ - template_version  │  ← REMOVE    │                          │
└─────────────────────┘              └──────────┬──────────────┘
                                                │ 1:N
                                     ┌──────────▼──────────────┐
                                     │ PayrollRun               │
                                     │ - run_type               │
                                     │ - template_version_id     │
                                     │ - run_date               │
                                     │ - status                 │
                                     │ - affects_accumulators   │
                                     │ - parent_run_id          │
                                     └──────────┬──────────────┘
                                                │ 1:N
                                     ┌──────────▼──────────────┐
                                     │ Payslip (existing)        │
                                     │ - run_id                  │
                                     └──────────────────────────┘
```

### 2.2 PayrollRun Model

```sql
CREATE TABLE payroll_runs (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    period_id UUID REFERENCES payroll_periods(id), -- NULL for true off-period runs

    run_type TEXT NOT NULL
        CHECK (run_type IN ('regular','bonus','advance',
                            'termination','thirteenth_month','annual_settlement')),

    run_label TEXT NOT NULL,                    -- Human-readable: "Thưởng Tết Âm Lịch 2026"
    template_version_id UUID NOT NULL
        REFERENCES payroll_template_versions(id),

    run_date DATE NOT NULL,                     -- When the run is executed (not period date)
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending','in_progress','calculated','verified',
                          'approved','committed','cancelled')),

    -- Configuration
    affects_accumulators BOOLEAN DEFAULT true,  -- Does this run write to accumulators?
    pit_handling TEXT DEFAULT 'merged'          -- 'merged' | 'separate'
        CHECK (pit_handling IN ('merged','separate')),

    -- Relations
    parent_run_id UUID REFERENCES payroll_runs(id),  -- advance → its regular run
    settlement_run_id UUID REFERENCES payroll_runs(id), -- annual_settlement source

    -- Metadata
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    committed_at TIMESTAMPTZ
);

-- Payslip now belongs to a run
ALTER TABLE payslips
    ADD COLUMN run_id UUID REFERENCES payroll_runs(id);
```

### 2.3 Template per Run Type

Each run type uses a **different payroll template**, which selects different components:

| Run Type | Template Components | Excluded Components |
|----------|--------------------|---------------------|
| **Regular** | All: Salary, OT, Allowances, Insurance, PIT, Net | — |
| **Bonus** | GrossBonus, PIT (optional), NetBonus | Insurance, OT, Attendance |
| **Advance** | NetAdvance (fixed amount) | Everything else |
| **Termination** | All regular + LeaveEncashment + Severance + FinalPIT | — |
| **13th Month** | ThirteenthMonthPayout, PIT | Insurance, OT, Attendance |
| **Annual Settlement** | PITReconciliation (read-only accumulator scan) | Everything else |

Templates are just data — no code change needed. Admin selects which components are active in each template.

### 2.4 Accumulator Interaction per Run Type

| Run Type | Reads Accumulators | Writes Accumulators |
|----------|:------------------:|:-------------------:|
| **Regular** | ✅ (ytd_bhxh, ytd_pit) | ✅ (updates all accumulators) |
| **Bonus** | ✅ (ytd_pit for bracket) | ⚠️ Only ytd_pit_paid (if `pit_handling = 'separate'`) |
| **Advance** | ❌ | ❌ |
| **Termination** | ✅ | ✅ (final snapshot) |
| **13th Month** | ✅ (ytd_gross for average calc) | ❌ (doesn't affect insurance caps) |
| **Annual Settlement** | ✅ (reads ALL) | ❌ (read-only reconciliation) |

### 2.5 Advance → Regular Relationship

Advance is special: it creates a **receivable** that the regular run settles:

```
┌─────────────────────────────────────────────────────────────┐
│                 ADVANCE FLOW                                 │
│                                                              │
│  Feb 10: ADVANCE RUN                                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Employee: Nguyễn Văn A                                │   │
│  │ Advance amount: 5,000,000 VND                         │   │
│  │                                                        │   │
│  │ Output:                                                 │   │
│  │ • AdvanceRecord (receivable): employee owes 5,000,000  │   │
│  │ • Bank payment: 5,000,000 to employee                  │   │
│  │ • GL: Nợ 141 (Tạm ứng) / Có 112                       │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  Feb 28: REGULAR RUN                                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Regular net pay: 12,925,000                            │   │
│  │ Outstanding advance: -5,000,000                        │   │
│  │                                                        │   │
│  │ Take-home: 7,925,000                                   │   │
│  │                                                        │   │
│  │ AdvanceRecord: status = "settled"                       │   │
│  │ GL: Nợ 334 / Có 141 (tất toán tạm ứng)               │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

```sql
CREATE TABLE payroll_advances (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL REFERENCES payroll_runs(id),
    employment_id UUID NOT NULL,
    amount DECIMAL NOT NULL,
    settled_by_run_id UUID REFERENCES payroll_runs(id),
    status TEXT DEFAULT 'outstanding'
        CHECK (status IN ('outstanding','settled','written_off')),
    created_at TIMESTAMPTZ DEFAULT now()
);
```

The regular run's `VariableProvider` includes an `AdvanceProvider` that makes `outstanding_advance` available as a variable that the `AdvanceRecovery` component uses to deduct from net pay.

### 2.6 PIT Handling for Bonus Runs

Vietnamese tax law (Circular 111/2013/TT-BTC) allows two methods:

| Method | How | When |
|--------|-----|------|
| **Merged** (`pit_handling = 'merged'`) | Bonus added to regular monthly income → PIT calculated on total | Standard, simpler |
| **Separate** (`pit_handling = 'separate'`) | Bonus taxed at flat rate or progressive on its own | Large bonuses (Tết) |

```
MERGED (standard):
  Regular gross: 15,000,000 + Bonus: 30,000,000 = 45,000,000
  → PIT bracket determined by 45M total

SEPARATE:
  Regular PIT: calculated on 15,000,000 (bracket X)
  Bonus PIT: calculated on 30,000,000 (using separate bracket table)
  
  Bonus run creates a SEPARATE payslip with its own PIT calculation.
  PIT accumulator ytd_pit_paid is updated by BOTH runs.
```

### 2.7 Termination Run

The most complex type — it's the employee's final settlement:

```
TERMINATION RUN INCLUDES:
├─ Regular salary (prorated for days worked this month)
├─ Unused annual leave encashment
├─ Severance allowance (if applicable: ½ month salary per year of service)
├─ Job-loss allowance (if applicable)
├─ 13th month pro-rata (if company policy)
├─ Final PIT calculation (ytd reconciliation)
└─ Final insurance contributions up to termination date
```

The termination run uses a special template that includes all these additional components. It reads all accumulators for the final PIT reconciliation.

---

## 3. Period State with Multiple Runs

ADR-004 defines period-level states. With multiple runs, this gets more nuanced:

```
PayrollPeriod (February 2026)
│
├─ Run #1: ADVANCE (Feb 10)      → status: committed (independent)
├─ Run #2: BONUS (Feb 15)         → status: committed (independent)
├─ Run #3: TERMINATION (Feb 20)   → status: committed (independent)
└─ Run #4: REGULAR (Feb 28)       → status: committed
                                      │
                          Period status: committed
                          (when ALL runs committed or period close date reached)
```

Period-level status depends on the collective state of its runs:

| Period Status | Run States |
|---------------|-----------|
| `open` | No runs created yet |
| `in_progress` | At least one run in progress |
| `calculated` | All runs calculated |
| `verified` | All runs verified |
| `approved` | All runs approved |
| `committed` | All runs committed |
| `closed` | Period permanently closed |

A period can only transition when ALL its runs are at the required state.

---

## 4. Data Consistency Across Runs

Critical invariant: **accumulators must be consistent across runs.**

Scenario: A bonus run increases `ytd_pit_paid`. The regular run later reads `ytd_pit_paid` to compute remaining PIT. If the bonus run committed after the regular run, the regular run's PIT is stale.

Rule: **Runs within the same period must be committed in order.** Bonus MUST commit before regular, so the regular run sees the bonus's accumulator contribution.

```rust
impl PayrollPeriod {
    fn validate_run_commit_order(&self, run: &PayrollRun) -> Result<()> {
        // Regular run must be the LAST to commit
        if run.run_type == "regular" {
            let uncommitted = self.runs.iter()
                .filter(|r| r.run_type != "regular" && r.status != "committed");
            if uncommitted.count() > 0 {
                return Err(anyhow!("Cannot commit regular run: other runs still pending"));
            }
        }
        Ok(())
    }
}
```

---

## 5. Impact on ADR-004 Lifecycle

| Change | Rationale |
|--------|-----------|
| `PayrollPeriod` no longer owns `template_version_id` | Template is per-run, not per-period |
| Period state transitions check ALL runs | Aggregate state from constituent runs |
| New `PayrollRun` state machine | Each run has its own lifecycle (pending → committed) |
| Advance receivable table | Track cross-run financial relationships |

---

## 6. Risks & Trade-offs

| Risk | Mitigation |
|------|-----------|
| **Run commit order violation** | System enforces: non-regular runs must commit before regular |
| **Advance exceeds net pay** | Advance limit configurable (e.g., max 50% of estimated net); warning if advance > net |
| **Too many run types confuse users** | UI collapses into 3 visible categories: "Regular", "Bonus/13M", "Special (advance/termination)" |
| **Accumulator drift across runs** | Each run snapshots accumulators at commit; reconciliation query detects drift |

---

## 7. References

- [ADR-003](ADR-003-payroll-formula-engine.md) — Component pipeline, templates
- [ADR-004](ADR-004-payroll-lifecycle-state-machine.md) — Period lifecycle
- [ADR-006](ADR-006-accumulators-multi-period-context.md) — Accumulator model
- Circular 111/2013/TT-BTC — Vietnam PIT regulations (bonus taxation)
- Labor Code 2019 — Termination, severance, leave encashment
