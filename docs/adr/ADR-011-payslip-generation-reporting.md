# ADR-011: Payslip Generation & Payroll Reporting

| Property       | Value                                                       |
|----------------|-------------------------------------------------------------|
| **Status**     | Proposed                                                    |
| **Date**       | 2026-08-03                                                  |
| **Author**     | AI + Lee                                                    |
| **Depends on** | [ADR-003](ADR-003-payroll-formula-engine.md), [ADR-004](ADR-004-payroll-lifecycle-state-machine.md), [ADR-010](ADR-010-off-cycle-payroll-run-types.md) |
| **Domain**     | Payroll — Output & Distribution                             |

---

## 1. Context

### 1.1 The "So What" Layer

The engine calculates. The lifecycle approves. Now the question is: **who sees what, in what format, through what channel?**

| Output | Audience | Format | Channel | Trigger |
|--------|----------|--------|---------|---------|
| **Payslip** | Individual employee | PDF | Portal download, email | Period committed |
| **Payroll Summary** | HR Manager, CFO | PDF/Excel | Dashboard, email | Period committed |
| **Department Breakdown** | Department heads | Table/chart | Dashboard | Period committed |
| **Statutory Reports** | Government agencies | Prescribed form | Manual download | Period committed + filing deadline |
| **Audit Trail Report** | Auditors | PDF/CSV | On-demand | Ad-hoc |

### 1.2 Industry Standard

| System | Payslip Delivery | Report Engine |
|--------|:---------------:|--------------|
| ADP | Employee portal + email PDF | 50+ pre-built reports + custom report builder |
| Oracle HCM | Self-service portal + PDF | BI Publisher (template-based) |
| Workday | Portal (HTML) + PDF download | Custom Reports (drag-drop builder) |
| SAP SF | Email + portal + mobile app | Ad-hoc + scheduled reports |

### 1.3 Vietnamese Requirements

Vietnamese labor law requires:
- Payslip must show: gross salary, all deductions (by type), net pay (Decree 145/2020/NĐ-CP)
- Employer must provide payslip to employee each pay period
- Social insurance contribution report (Form C12) filed monthly/quarterly
- PIT declaration (Form 02/KK-TNCN) filed monthly/quarterly
- Annual PIT finalization (Form 05/KK-TNCN) filed by March 30

---

## 2. Architecture: Reporting is a PostProcessStep

Reporting is a **PostProcessStep** (ADR-003) that runs when a period is committed:

```rust
struct PayslipGenerationStep {
    template_engine: Arc<dyn PayslipTemplateEngine>,
    storage: Arc<dyn DocumentStorage>,
}

impl PostProcessStep for PayslipGenerationStep {
    fn step_name(&self) -> &str { "payslip_generation" }

    fn execute(
        &self,
        payslip: &Payslip,
        component_results: &[(VariableRef, Decimal)],
        ctx: &PayrollContext,
    ) -> Result<PostProcessOutput> {
        // 1. Prepare data context (unmasked — this is for the employee themselves)
        let data = PayslipViewModel::from(payslip, component_results, ctx);

        // 2. Render PDF from template
        let pdf_bytes = self.template_engine.render("payslip_standard", &data)?;

        // 3. Store PDF in document storage (immutable once generated)
        let doc_id = self.storage.store(
            workspace_id = ctx.workspace_id,
            employment_id = payslip.employment_id(),
            period = (payslip.period_start(), payslip.period_end()),
            document_type = "payslip",
            content = pdf_bytes,
        )?;

        // 4. Queue for distribution
        Ok(PostProcessOutput::PayslipDocument(PayslipDocument {
            payslip_id: payslip.id(),
            document_id: doc_id,
            distribution_channel: self.resolve_channels(ctx.config),
            status: DistributionStatus::Pending,
        }))
    }
}
```

### 2.3 Payslip Data Model

The `PayslipViewModel` is what the template receives:

```rust
struct PayslipViewModel {
    // Header
    company_name: String,
    company_address: String,
    company_logo_url: Option<String>,
    period_label: String,           // "Tháng 02/2026"
    generated_date: NaiveDate,

    // Employee
    employee_name: String,
    employee_code: String,
    department: String,
    position: String,
    employment_start_date: NaiveDate,

    // Earnings (grouped by category)
    earnings: Vec<EarningLine>,
    // Deductions
    deductions: Vec<DeductionLine>,

    // Totals
    gross_income: Decimal,
    total_deductions: Decimal,
    net_income: Decimal,

    // Year-to-date
    ytd_gross: Decimal,
    ytd_pit_paid: Decimal,
    ytd_bhxh_paid: Decimal,

    // Footer
    tax_code: String,               // Mã số thuế công ty
    legal_representative: String,
    chief_accountant: String,
    signature_blocks: SignatureConfig,

    // Digital
    document_ref: String,           // Unique payslip reference number
    verification_code: String,      // For online verification
}

struct EarningLine {
    label: String,                  // "Lương cơ bản", "Phụ cấp đi lại"
    amount: Decimal,
    category: EarningCategory,      // BasicSalary, Allowance, Overtime, Bonus, Other
}

struct DeductionLine {
    label: String,                  // "BHXH (8%)", "Thuế TNCN"
    amount: Decimal,
    category: DeductionCategory,    // Insurance, Tax, UnionFee, Advance, Other
    note: Option<String>,           // "Đã bao gồm giảm trừ gia cảnh 11tr"
}
```

### 2.4 Template Engine

The payslip template is an HTML/CSS document with variable interpolation:

```html
<!-- Template: payslip_standard.html -->
<html>
<head>
    <style>
        /* Company branding, fonts, colors */
    </style>
</head>
<body>
    <header>
        <img src="{{company_logo_url}}" />
        <h1>{{company_name}}</h1>
        <p>PHIẾU LƯƠNG - {{period_label}}</p>
    </header>

    <section class="employee-info">
        <p>Họ tên: <strong>{{employee_name}}</strong></p>
        <p>Mã NV: {{employee_code}}</p>
        <p>Phòng ban: {{department}}</p>
    </section>

    <table class="earnings">
        <tr><th>Khoản thu nhập</th><th>Số tiền (VND)</th></tr>
        {% for item in earnings %}
        <tr><td>{{item.label}}</td><td class="amount">{{item.amount|currency}}</td></tr>
        {% endfor %}
    </table>

    <table class="deductions">
        {% for item in deductions %}
        <tr><td>{{item.label}}</td><td class="amount">{{item.amount|currency}}</td></tr>
        {% endfor %}
    </table>

    <footer class="totals">
        <p>Tổng thu nhập: <strong>{{gross_income|currency}} VND</strong></p>
        <p>Tổng khấu trừ: <strong>{{total_deductions|currency}} VND</strong></p>
        <p class="net">Thực nhận: <strong>{{net_income|currency}} VND</strong></p>
    </footer>

    <section class="signatures">
        <div>Người lập</div>
        <div>Kế toán trưởng</div>
        <div>Giám đốc</div>
    </section>
</body>
</html>
```

**Template engine choice:** Use `tera` (Rust) or `handlebars-rust` — both are pure Rust, safe, and support the Jinja2-style syntax above. No need for a custom template engine.

### 2.5 Report Types

All reports are built on the same PostProcessStep pattern but operate on **aggregated data** after commit:

| Report | Data Source | Aggregation |
|--------|------------|-------------|
| **Payroll Summary** | All payslips + employer contributions | SUM per component, per department |
| **Department Cost** | All payslips grouped by department | SUM per department, per cost category |
| **Insurance Declaration (C12)** | All payslips, insurance components | Per employee, formatted as C12 template |
| **PIT Declaration (02/KK)** | All payslips, PIT component | Aggregated by tax bracket |
| **Bank Transfer Summary** | Net pay per employee | Per bank, per branch |
| **GL Export** | GL journals (ADR-009) | CSV/XML for accounting software |

```rust
trait PayrollReport {
    fn report_id(&self) -> &str;
    fn report_name(&self) -> &str;
    fn generate(
        &self,
        period: &PayrollPeriod,
        payslips: &[Payslip],
        ctx: &WorkspaceContext,
    ) -> Result<ReportOutput>;
}

enum ReportOutput {
    Pdf(Vec<u8>),
    Excel(Vec<u8>),
    Csv(String),
    Xml(String),          // For tax authority formats
}
```

### 2.6 Distribution

```sql
CREATE TABLE payroll_document_distributions (
    id UUID PRIMARY KEY,
    payslip_id UUID REFERENCES payslips(id),
    document_type TEXT NOT NULL,      -- 'payslip', 'summary_report'
    channel TEXT NOT NULL,            -- 'portal', 'email', 'manual'
    status TEXT DEFAULT 'pending'
        CHECK (status IN ('pending','sent','delivered','failed','viewed')),
    recipient_email TEXT,
    sent_at TIMESTAMPTZ,
    viewed_at TIMESTAMPTZ,            -- Track if employee opened it
    created_at TIMESTAMPTZ DEFAULT now()
);
```

Distribution channels are configurable per workspace:
- **Portal only:** Employee logs in to view/download payslip (default, most secure)
- **Email:** PDF sent to employee email (convenient, less secure — email is not encrypted)
- **Both:** PDF in portal + notification email (no attachment, just link)

---

## 3. Statutory Reports (Vietnam-Specific)

These are specialized `PayrollReport` implementations:

| Report | Legal Basis | Format | Frequency |
|--------|------------|--------|-----------|
| C12-HD (Labor usage) | Decree 145/2020 | XML/PDF | 6 months |
| 02/KK-TNCN (PIT declaration) | Circular 80/2021 | XML | Monthly |
| D01-TS (Insurance declaration) | Decision 1040/QĐ-BHXH | XML | Monthly |
| Labor report (semi-annual) | Decree 145/2020 | PDF | 6 months |

These are implemented as individual report structs that produce the legally-prescribed XML/PDF format. The system **generates the data but does not auto-submit** — HR downloads and uploads to government portals manually.

---

## 4. Risks & Trade-offs

| Risk | Mitigation |
|------|-----------|
| **Payslip PDF contains stale data if recalculation occurs after generation** | Payslips are regenerated on retro. Distribution status tracks which version employee received. |
| **Email distribution leaks PII** | Email contains NO attachment — only a notification with a secure portal link that requires authentication |
| **Template customization per client** | Template is part of workspace settings; admin uploads custom CSS/header. Default template ships with the product. |
| **Statutory form changes** | Report structs are versioned. When government changes a form, a new report version is deployed. |

---

## 5. References

- [ADR-003](ADR-003-payroll-formula-engine.md) — PostProcessStep
- [ADR-010](ADR-010-off-cycle-payroll-run-types.md) — Run types (bonus, termination payslips)
- Decree 145/2020/NĐ-CP — Vietnam labor law: payslip requirements
- Circular 80/2021/TT-BTC — Tax declaration forms
