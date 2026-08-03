# ADR-009: GL Accounting Integration

| Property       | Value                                                       |
|----------------|-------------------------------------------------------------|
| **Status**     | Proposed                                                    |
| **Date**       | 2026-08-03                                                  |
| **Author**     | AI + Lee                                                    |
| **Depends on** | [ADR-003](ADR-003-payroll-formula-engine.md), [ADR-004](ADR-004-payroll-lifecycle-state-machine.md), [ADR-008](ADR-008-employer-contributions.md) |
| **Domain**     | Payroll — Accounting Integration                            |

---

## 1. Context

### 1.1 The Void After Net Pay

The engine calculates net pay. Employer contributions are computed. Then what?

In every enterprise, payroll is the largest monthly expense and MUST flow into the general ledger. Without GL integration, payroll is a black box: "We calculated net pay, now accounting has to manually enter journals."

### 1.2 Vietnamese Chart of Accounts (Circular 200/2014/TT-BTC)

| Account | Name | Payroll Usage |
|---------|------|--------------|
| **334** | Phải trả người lao động | Wages payable (credit = net pay + employee deductions owed) |
| **3382** | Kinh phí công đoàn | Trade union fee payable |
| **3383** | Bảo hiểm xã hội | Social insurance payable |
| **3384** | Bảo hiểm y tế | Health insurance payable |
| **3386** | Bảo hiểm thất nghiệp | Unemployment insurance payable |
| **3335** | Thuế TNCN | Personal income tax payable |
| **622** | Chi phí nhân công trực tiếp | Direct labor cost |
| **627** | Chi phí sản xuất chung | Manufacturing overhead (indirect labor) |
| **642** | Chi phí quản lý doanh nghiệp | G&A (admin, HR, management) |
| **112** | Tiền gửi ngân hàng | Bank deposit (credit for net pay transfer) |

### 1.3 Complete Journal for One Employee

```
Nguyễn Văn A, Department: Engineering (622), February 2026
Gross: 15,000,000 VND

┌──────────────────────────────────────────────────────────────┐
│ DR  622  (Chi phí lương)              15,000,000             │
│ DR  622  (BHXH employer 17.5%)         2,625,000             │
│ DR  622  (BHYT employer 3%)              450,000             │
│ DR  622  (BHTN employer 1%)              150,000             │
│ DR  622  (CĐ phí employer 2%)            300,000             │
│                                                              │
│     CR  334  (Phải trả NLĐ)                       12,925,000 │  ← net pay
│     CR  334  (BHXH employee giữ lại)               1,200,000 │
│     CR  334  (BHYT employee giữ lại)                 225,000 │
│     CR  334  (BHTN employee giữ lại)                 150,000 │
│     CR  334  (PIT giữ lại)                            500,000 │
│     CR  3382 (CĐ phí)                                 300,000 │
│     CR  3383 (BHXH tổng: 1,200K + 2,625K)          3,825,000 │
│     CR  3384 (BHYT tổng: 225K + 450K)                 675,000 │
│     CR  3386 (BHTN tổng: 150K + 150K)                 300,000 │
│     CR  3335 (Thuế TNCN)                               500,000 │
│                                                              │
│  TOTAL DR: 18,525,000    TOTAL CR: 18,525,000  ✓             │
└──────────────────────────────────────────────────────────────┘
```

---

## 2. Architecture Decision

### 2.1 GL Accounting as a PostProcessStep

GL accounting fits perfectly into ADR-003's PostProcessStep:

```rust
struct GlAccountingStep {
    mapping_config: GlMappingConfig,
}

impl PostProcessStep for GlAccountingStep {
    fn step_name(&self) -> &str { "gl_accounting" }

    fn execute(
        &self,
        payslip: &Payslip,
        component_results: &[(VariableRef, Decimal)],
        employer_results: &[(VariableRef, Decimal)],
        ctx: &PayrollContext,
    ) -> Result<PostProcessOutput> {
        let mut entries = Vec::new();

        // For each component output, map to GL accounts
        for (var, amount) in component_results {
            let mapping = self.mapping_config.resolve(var, ctx)?;
            entries.push(GlEntry {
                debit_account: mapping.expense_account,
                credit_account: mapping.liability_account,
                amount: *amount,
                dimensions: extract_dimensions(ctx, &mapping),
                description: format!("{} - {}", var, ctx.period_label),
                entry_type: GlEntryType::Component,
            });
        }

        // Employer contributions
        for (var, amount) in employer_results {
            let mapping = self.mapping_config.resolve(var, ctx)?;
            entries.push(GlEntry {
                debit_account: mapping.expense_account,
                credit_account: mapping.liability_account,
                amount: *amount,
                dimensions: extract_dimensions(ctx, &mapping),
                description: format!("Employer {} - {}", var, ctx.period_label),
                entry_type: GlEntryType::EmployerContribution,
            });
        }

        // Net pay settlement
        entries.push(GlEntry {
            debit_account: "334",
            credit_account: "112",
            amount: payslip.net_pay_amount(),
            description: format!("Net pay transfer - {}", ctx.period_label),
            entry_type: GlEntryType::Settlement,
        });

        Ok(PostProcessOutput::GlJournal(GlJournal {
            period: (payslip.period_start(), payslip.period_end()),
            employment_id: payslip.employment_id(),
            entries,
            total_debit: entries.iter().map(|e| e.amount).sum(),
            total_credit: entries.iter().map(|e| e.amount).sum(),
        }))
    }
}
```

### 2.2 GL Mapping: Three Tiers

**Tier 1: Static (default for small companies)**

One mapping per component type, used across the entire workspace:

```json
{
  "component_mappings": {
    "gross_salary":           { "expense": "622", "liability": "334" },
    "bhxh_employee":          { "expense": "—",   "liability": "3383" },
    "bhxh_employer":          { "expense": "622", "liability": "3383" },
    "bhyt_employee":          { "expense": "—",   "liability": "3384" },
    "bhyt_employer":          { "expense": "622", "liability": "3384" },
    "bhtn_employee":          { "expense": "—",   "liability": "3386" },
    "bhtn_employer":          { "expense": "622", "liability": "3386" },
    "union_fee_employer":     { "expense": "622", "liability": "3382" },
    "pit_deduction":          { "expense": "—",   "liability": "3335" },
    "overtime_pay":           { "expense": "622", "liability": "334" },
    "incentive":              { "expense": "622", "liability": "334" },
    "net_settlement":         { "expense": "334", "liability": "112" }
  }
}
```

**Tier 2: Dimensional (department × component)**

Override expense accounts by department:

```json
{
  "default": { "gross_salary": { "expense": "622" } },
  "overrides": [
    {
      "dimension": "department",
      "rules": {
        "Engineering": { "gross_salary": { "expense": "6221" } },
        "HR":          { "gross_salary": { "expense": "6421" } },
        "Admin":       { "gross_salary": { "expense": "6422" } },
        "Sales":       { "gross_salary": { "expense": "6411" } }
      }
    }
  ]
}
```

**Tier 3: Rule-based (enterprise)**

```json
{
  "rules": [
    {
      "conditions": {
        "department": {"in": ["Engineering", "R&D"]},
        "employment_type": "fulltime",
        "project_code": {"regex": "^PRD-"}
      },
      "mapping": {
        "gross_salary": { "expense": "6221-{project_code}", "liability": "3341" }
      }
    }
  ]
}
```

Tier 3 supports **account templates** with variable interpolation: `"6221-{project_code}-{department_code}"`.

### 2.3 When Are GL Entries Created?

Not during calculation — because payslips can be recalculated. Entries are created **at period commit** (ADR-004 lifecycle):

```
Period State:                   GL State:
─────────────                   ────────
InProgress / Calculated         (nothing — payslips are tentative)
Verified / Approved             (nothing — still can recalculate)
                              
COMMITTED  ───────────────────→ GL Journal created, status: "pending_posting"
                               (period frozen, entries are final for this version)

If RETRO after commit (ADR-005):
  Old journal → status: "reversed"
  New journal created for new version
```

### 2.4 Journal Versioning

```sql
CREATE TABLE payroll_gl_journals (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    period_id UUID NOT NULL REFERENCES payroll_periods(id),
    payslip_id UUID REFERENCES payslips(id),   -- NULL for workspace-level entries
    entry_type TEXT NOT NULL,
    debit_account TEXT NOT NULL,
    credit_account TEXT NOT NULL,
    amount DECIMAL NOT NULL,
    dimensions JSONB DEFAULT '{}',             -- {department_code, project_code, cost_center}
    description TEXT,
    status TEXT DEFAULT 'pending_posting'
        CHECK (status IN ('pending_posting','posted','reversed')),
    journal_version INTEGER NOT NULL DEFAULT 1, -- incremented on retro
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Index for GL system to pull
CREATE INDEX idx_payroll_gl_journals_posting
    ON payroll_gl_journals (workspace_id, status, created_at);
```

### 2.5 Batch vs Individual Journals

| Approach | When | Pros | Cons |
|----------|------|------|------|
| **Per-payslip** | Standard | Fine-grained, traceable | 5000 journals/month for 5000 employees |
| **Aggregated by account** | Large companies | Compact: 1 journal per account | Loses per-employee traceability |

Both are needed. The system generates **per-payslip** journals for audit, and can **aggregate** them into summary journals for posting:

```sql
-- Aggregate view
SELECT period_id, debit_account, credit_account, SUM(amount)
FROM payroll_gl_journals
WHERE workspace_id = $1 AND status = 'pending_posting'
GROUP BY period_id, debit_account, credit_account;
```

---

## 3. Integration with ADR-008 (Employer Contributions)

Employer contributions generate their own GL entries. The mapping:

| Component | DR (Expense) | CR (Liability) |
|-----------|-------------|----------------|
| `employer_bhxh` | 622/627/642 (same as gross salary) | 3383 |
| `employer_bhyt` | 622/627/642 | 3384 |
| `employer_bhtn` | 622/627/642 | 3386 |
| `employer_union_fee` | 622/627/642 | 3382 |

---

## 4. Risks & Trade-offs

| Risk | Mitigation |
|------|-----------|
| **GL account codes differ between clients** | Tier 1/2/3 mapping allows per-workspace customization |
| **Dr/Cr imbalance due to rounding** | A final "rounding difference" entry absorbs any discrepancy (typically < 1 VND) |
| **External GL system has different period close** | Journals can be exported as CSV/XML; no two-way sync assumed |
| **Recalculation creates journal churn** | Old journals are reversed (not deleted); GL auditor sees the full history |

---

## 5. References

- [ADR-003](ADR-003-payroll-formula-engine.md) — PostProcessStep trait
- [ADR-004](ADR-004-payroll-lifecycle-state-machine.md) — Period commit trigger
- [ADR-008](ADR-008-employer-contributions.md) — Employer contribution pipeline
- Circular 200/2014/TT-BTC — Vietnamese chart of accounts
