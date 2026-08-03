# ADR-013: Bank Payment File Generation

| Property       | Value                                                       |
|----------------|-------------------------------------------------------------|
| **Status**     | Proposed                                                    |
| **Date**       | 2026-08-03                                                  |
| **Author**     | AI + Lee                                                    |
| **Depends on** | [ADR-003](ADR-003-payroll-formula-engine.md), [ADR-004](ADR-004-payroll-lifecycle-state-machine.md), [ADR-010](ADR-010-off-cycle-payroll-run-types.md) |
| **Domain**     | Payroll — Payment Processing                                |

---

## 1. Context

### 1.1 The Final Mile

After calculation, approval, and commit — the money must actually move. In Vietnam, this means generating bank-specific payment files that are uploaded to corporate internet banking portals.

### 1.2 The Fragmented Banking Landscape

Every Vietnamese bank has a different file format for bulk salary payments:

| Bank | Format | Encoding | Delimiter | Header Row? |
|------|--------|----------|-----------|:-----------:|
| Vietcombank | CSV | UTF-8 | Comma | Yes |
| Techcombank | Fixed-width text | UTF-8 | Positional | No |
| BIDV | XML | UTF-8 | — | — |
| Vietinbank | CSV | UTF-8 | Semicolon | Yes |
| ACB | CSV | UTF-8 | Comma | No |
| MB Bank | Fixed-width text | UTF-8 | Positional | Yes |

And they change formats periodically — without notice. This is a plugin architecture problem.

### 1.3 Vietnamese Banking Requirements

- **Beneficiary name must match bank account name exactly** (diacritics preserved)
- **Nội dung chuyển khoản** (payment description) is critical for audit
- Some banks limit description to 100 characters
- Corporate accounts often require **batch approval** (multiple signatories)
- Some companies split payment: part bank transfer, part cash

---

## 2. Architecture Decision

### 2.1 Bank Format Plugins

Bank file generation is a PostProcessStep (ADR-003), but the format logic is delegated to **pluggable format handlers**:

```rust
/// Each bank implements this trait to generate its specific file format.
trait BankPaymentFormat: Send + Sync {
    /// Unique identifier: "vietcombank_csv", "techcombank_fixed", etc.
    fn format_id(&self) -> &str;
    fn bank_name(&self) -> &str;

    /// File extension for the generated file
    fn file_extension(&self) -> &str;       // ".csv", ".txt", ".xml"

    /// Generate the file content from a list of payment instructions
    fn generate(
        &self,
        batch: &PaymentBatch,
        instructions: &[BankPaymentInstruction],
    ) -> Result<Vec<u8>, BankFormatError>;

    /// Validate instructions BEFORE generation (catch errors early)
    fn validate(&self, instructions: &[BankPaymentInstruction]) -> Result<(), Vec<BankValidationError>>;

    /// Bank-specific configuration schema (JSON Schema)
    fn config_schema(&self) -> serde_json::Value;
}

struct BankPaymentInstruction {
    beneficiary_name: String,       // "NGUYỄN VĂN A"
    beneficiary_account: String,    // "1234567890"
    bank_code: String,              // "VCB", "TCB", "BIDV"...
    bank_branch: Option<String>,    // "CN Hà Nội"
    amount: Decimal,                // 12,925,000
    description: String,            // "Luong T02/2026 Nguyen Van A"
    reference: String,              // Unique payment reference for reconciliation
}
```

### 2.2 Plugin Registry

```rust
struct BankFormatRegistry {
    formats: HashMap<String, Arc<dyn BankPaymentFormat>>,
}

impl BankFormatRegistry {
    fn register(&mut self, format: Arc<dyn BankPaymentFormat>) {
        self.formats.insert(format.format_id().to_string(), format);
    }

    fn get(&self, format_id: &str) -> Option<&Arc<dyn BankPaymentFormat>> {
        self.formats.get(format_id)
    }

    fn list_formats(&self) -> Vec<BankFormatInfo> {
        self.formats.iter().map(|(id, f)| BankFormatInfo {
            id: id.clone(),
            bank_name: f.bank_name().to_string(),
            file_extension: f.file_extension().to_string(),
        }).collect()
    }
}
```

### 2.3 Example: Vietcombank Format

```rust
struct VietcombankCsvFormat;

impl BankPaymentFormat for VietcombankCsvFormat {
    fn format_id(&self) -> &str { "vietcombank_csv" }
    fn bank_name(&self) -> &str { "Vietcombank" }
    fn file_extension(&self) -> &str { "csv" }

    fn generate(
        &self,
        batch: &PaymentBatch,
        instructions: &[BankPaymentInstruction],
    ) -> Result<Vec<u8>, BankFormatError> {
        let mut csv = String::new();

        // VCB header
        csv.push_str("STT,Tên người thụ hưởng,Số tài khoản,Số tiền,Nội dung\n");

        for (i, instr) in instructions.iter().enumerate() {
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                i + 1,
                instr.beneficiary_name,
                instr.beneficiary_account,
                instr.amount.round_dp(0),
                instr.description,
            ));
        }

        // Write with BOM for Excel compatibility with Vietnamese characters
        let mut bytes = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
        bytes.extend(csv.as_bytes().to_vec());
        Ok(bytes)
    }

    fn validate(&self, instructions: &[BankPaymentInstruction]) -> Result<(), Vec<BankValidationError>> {
        let mut errors = Vec::new();
        for (i, instr) in instructions.iter().enumerate() {
            // VCB: account number must be numeric, max 15 digits
            if !instr.beneficiary_account.chars().all(|c| c.is_ascii_digit()) {
                errors.push(BankValidationError {
                    index: i,
                    field: "beneficiary_account",
                    message: "VCB requires numeric account numbers".into(),
                });
            }
            // VCB: description max 100 chars
            if instr.description.chars().count() > 100 {
                errors.push(BankValidationError {
                    index: i,
                    field: "description",
                    message: "VCB limits description to 100 characters".into(),
                });
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    fn config_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "corporate_account": {"type": "string", "description": "Tài khoản nguồn"},
                "payment_method": {"enum": ["fast", "standard", "batch"]}
            }
        })
    }
}
```

### 2.4 Payment Batch Model

```sql
CREATE TABLE payroll_payment_batches (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    period_id UUID REFERENCES payroll_periods(id),
    run_id UUID REFERENCES payroll_runs(id),

    batch_label TEXT NOT NULL,            -- "Lương T02/2026 - Đợt 1"
    bank_format_id TEXT NOT NULL,         -- "vietcombank_csv"

    -- Source account
    source_account TEXT NOT NULL,         -- "1234567890" (corporate account)
    source_bank TEXT NOT NULL,            -- "VCB"

    -- Status
    status TEXT DEFAULT 'draft'
        CHECK (status IN ('draft','generated','uploaded','confirmed','reconciled')),
    
    -- Totals
    total_amount DECIMAL NOT NULL,
    instruction_count INTEGER NOT NULL,
    
    -- File
    file_content BYTEA,                   -- Generated file content
    file_name TEXT,                       -- "VCB_Luong_T02_2026.csv"
    file_hash TEXT,                       -- SHA-256 for integrity

    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    generated_at TIMESTAMPTZ,
    uploaded_at TIMESTAMPTZ
);

CREATE TABLE payroll_payment_instructions (
    id UUID PRIMARY KEY,
    batch_id UUID NOT NULL REFERENCES payroll_payment_batches(id),
    payslip_id UUID NOT NULL REFERENCES payslips(id),  -- traceability
    employment_id UUID NOT NULL,

    beneficiary_name TEXT NOT NULL,
    beneficiary_account TEXT NOT NULL,
    bank_code TEXT NOT NULL,
    bank_branch TEXT,
    amount DECIMAL NOT NULL,
    description TEXT NOT NULL,
    reference TEXT NOT NULL,              -- unique, for reconciliation
    
    -- Split payment support
    is_split BOOLEAN DEFAULT false,
    parent_instruction_id UUID REFERENCES payroll_payment_instructions(id),
    
    status TEXT DEFAULT 'pending'
        CHECK (status IN ('pending','sent','cleared','failed'))
);
```

### 2.5 Split Payments

Some employees receive salary through multiple channels:

```
Nguyễn Văn A: Net = 12,925,000
  ├─ Bank transfer (Vietcombank):  10,000,000
  └─ Cash:                          2,925,000
```

Split payment configuration is per employee:

```sql
CREATE TABLE employee_payment_methods (
    id UUID PRIMARY KEY,
    employment_id UUID NOT NULL,
    priority INTEGER NOT NULL DEFAULT 1,
    payment_type TEXT NOT NULL        -- 'bank_transfer', 'cash'
        CHECK (payment_type IN ('bank_transfer','cash')),
    bank_account TEXT,
    bank_code TEXT,
    -- For bank_transfer: percentage OR fixed amount
    allocation_type TEXT NOT NULL     -- 'percentage', 'fixed'
        CHECK (allocation_type IN ('percentage','fixed')),
    allocation_value DECIMAL NOT NULL, -- 80 (for 80%) or 10_000_000 (fixed)
    is_active BOOLEAN DEFAULT true,
    effective_from DATE,
    effective_to DATE
);
```

### 2.6 Payment Workflow

```
┌─────────────────────────────────────────────────────────────┐
│                    PAYMENT WORKFLOW                           │
│                                                              │
│  STEP 1: CREATE BATCH                                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Select: Bank = Vietcombank, Format = "vietcombank_csv" │   │
│  │ Select: Period = Tháng 2/2026, Run = Regular          │   │
│  │ System: Populates instructions from committed payslips │   │
│  │ System: Validates (account number format, amount > 0) │   │
│  │ Admin: Reviews and confirms batch                     │   │
│  └──────────────────────────────────────────────────────┘   │
│                           │                                  │
│                           ▼                                  │
│  STEP 2: GENERATE FILE                                        │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ System: Calls BankPaymentFormat::generate()           │   │
│  │ Output: VCB_Luong_T02_2026.csv (downloadable)         │   │
│  │ Hash: SHA-256 stored for integrity verification      │   │
│  └──────────────────────────────────────────────────────┘   │
│                           │                                  │
│                           ▼                                  │
│  STEP 3: UPLOAD TO BANK                                       │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Admin: Downloads file, uploads to VCB iBanking        │   │
│  │ Admin: Marks batch as "uploaded" in system            │   │
│  │ (Manual step — cannot be automated for security)      │   │
│  └──────────────────────────────────────────────────────┘   │
│                           │                                  │
│                           ▼                                  │
│  STEP 4: CONFIRMATION                                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Admin: After bank executes, marks batch "confirmed"   │   │
│  │ System: Updates payment instruction statuses          │   │
│  └──────────────────────────────────────────────────────┘   │
│                           │                                  │
│                           ▼                                  │
│  STEP 5: RECONCILIATION                                       │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Match bank statement entries against instructions     │   │
│  │ Flag: cleared, failed, returned                       │   │
│  │ Generate reconciliation report                        │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 2.7 Description Template

Payment description is configurable:

```json
{
    "description_template": "Luong T{month}/{year} {employee_name} - {employee_code}",
    "max_length": 100,
    "truncation_strategy": "end"  
}
```

Variable interpolation uses the same context as payslip templates (ADR-011).

---

## 3. Integration with Post-Processing Pipeline

```
Post-processing sequence (ADR-003):

1. Anomaly Detection (ADR-007)
2. Employer Contributions (ADR-008)
3. GL Accounting (ADR-009)
4. Payslip Generation (ADR-011)
5. Bank Payment Generation (THIS ADR)  ← NEW
   ├─ Create payment batch
   ├─ Split payments per employee config
   ├─ Generate bank file (per format)
   └─ Store file + hash
```

Bank payment is the LAST step because it requires all previous steps to be complete: net pay must be final, all overrides applied, all deductions settled.

---

## 4. Risks & Trade-offs

| Risk | Mitigation |
|------|-----------|
| **Bank changes format without notice** | Format plugins are versioned. Old batches preserve the format used at generation time. New batches use updated format. |
| **Error in bank file → wrong payments** | Validate BEFORE generation (account format, name matching). Validate AFTER generation (hash). Dry-run possible. |
| **Split payment inconsistency** | If bank + cash ≠ net pay → batch creation fails with clear error |
| **Vietnamese character encoding in bank files** | Each format specifies encoding. UTF-8 with BOM is standard. |
| **Retro changes net pay after bank file generated** | Retro event invalidates the payment batch. New batch must be generated for the delta. |

---

## 5. References

- [ADR-003](ADR-003-payroll-formula-engine.md) — PostProcessStep
- [ADR-004](ADR-004-payroll-lifecycle-state-machine.md) — Period commit as trigger
- [ADR-011](ADR-011-payslip-generation-reporting.md) — Template engine reuse for descriptions
- State Bank of Vietnam — Corporate bulk payment regulations
