# Privacy & Data Governance Contract (P1-11)

**Parent ledger task:** P1-11  
**Scope:** Data classification, PII masking, retention schedules, audit event logging, and right-to-erasure behavior for `apiserver` Rust-to-Go migration.  
**Governing Standard:** SOC 2 CC6.5 / ISO 27001 A.5.12-5.13 / GDPR Art. 17 / PDPA.

---

## 1. Data Classification Tiers

Every field in the `apiserver` domain is classified into one of four tiers:

| Tier | Classification | Storage Rules | Logging Rules | API Response Rules | Example Fields |
|---|---|---|---|---|---|
| **T1** | **Restricted** | Encrypted at rest (AES-256), salted Argon2id/bcrypt for credentials. Never cached. | **STRICTLY PROHIBITED.** Must never appear in any log level. | Never returned in any API response. Excluded via `#[serde(skip_serializing)]` / omit in Go JSON tags. | `users.password_hash`<br>`agents.password`<br>`JWT_SECRET`<br>`ess.bank_account_number`<br>`signature.signature_image_base64` |
| **T2** | **Confidential** | Encrypted at rest. Workspace-isolated. Role-gated. | **MUST BE MASKED** using canonical masking functions if logged. | Returned ONLY to authorized roles (`HR_ADMIN`, `OWNER`, self via ESS). NEVER in collection/list endpoints; detail views only. | `users.email`<br>`ess.phone`<br>`ess.address`<br>`ess.national_id`<br>`ess.bank_name`<br>`ess.bank_branch`<br>`employee_salary_components.amount`<br>`payslips.applied_base_salary`<br>`emergency_contacts.phone` |
| **T3** | **Internal** | Standard PostgreSQL storage with workspace tenant isolation. | Masked in external-facing logs; permissible in debug audit trails with workspace scope. | Standard RBAC authenticated users within the same workspace. | `employees.first_name`<br>`employees.last_name`<br>`attendance_logs.punch_time`<br>`shifts.start_time`<br>`departments.name` |
| **T4** | **Public** | Standard storage. | Permissible in logs. | Authenticated workspace members. | `workspaces.name`<br>`workspaces.slug`<br>`policy_definitions.name` |

---

## 2. Canonical Masking Rules (Rust Reference: `apiserver/src/utils/masking.rs`)

The Go implementation MUST match the exact output of Rust's `apiserver/src/utils/masking.rs`:

### 2.1 Email Masking (`mask_email`)
- If input has no `@` or `!= 2` parts:
  - Length $\le 2$: returns `"***"`
  - Length $> 2$: returns `{first_2_chars}***`
- If input is valid `{local}@{domain}`:
  - Local length $\le 2$: `"***@" + domain`
  - Local length $3..4$: `{local[..1]}***@" + domain`
  - Local length $> 4$: `{local[..3]}***@" + domain`
- **Golden test vectors:**
  - `john.doe@example.com` $\rightarrow$ `joh***@example.com`
  - `lee@gmail.com` $\rightarrow$ `l***@gmail.com`
  - `me@test.com` $\rightarrow$ `***@test.com`
  - `a@b.com` $\rightarrow$ `***@b.com`
  - `longemailaddress@corp.com` $\rightarrow$ `lon***@corp.com`

### 2.2 Phone Masking (`mask_phone`)
- Length $\le 6$: returns `"***"`
- Length $> 6$: returns `{first_3_digits}` + `*` repeated `(len - 6)` times + `{last_3_digits}`
- **Golden test vectors:**
  - `0987654321` $\rightarrow$ `098****321`
  - `12345` $\rightarrow$ `***`
  - `1234567` $\rightarrow$ `123*567`

### 2.3 Bank Account Masking (`mask_bank_account`)
- Length $\le 4$: returns `"****"`
- Length $> 4$: returns `*` repeated `(len - 4)` times + `{last_4_digits}`
- **Golden test vectors:**
  - `1234567890` $\rightarrow$ `******7890`
  - `123` $\rightarrow$ `****`
  - `12345` $\rightarrow$ `*2345`

---

## 3. Data Retention Schedule (Housekeeping Worker)

Reference: `apiserver/src/worker/handlers/housekeeping.rs` and `.agent/rules/data-governance.md`:

| Entity Type | Retention Period | Purge Method | Legal / Regulatory Basis |
|---|---|---|---|
| **Audit Logs** | 3 years (1095 days) | Automated batch delete via `housekeeping` background worker | SOC 2 CC7.3 |
| **Attendance Records** | 5 years after employee termination | Soft-delete $\rightarrow$ hard purge after retention window | Labor law record-keeping |
| **Leave Requests** | 5 years after employee termination | Soft-delete $\rightarrow$ hard purge | Labor law |
| **Employee PII** | 1 year after termination (or immediate on erasure) | Anonymization $\rightarrow$ purge | GDPR Art. 17 / PDPA |
| **Session Tokens / JWTs** | Configurable (default 24 hours) | Redis key TTL expiration | Security best practice |
| **Failed Login Attempts** | 90 days | Automated Redis key expiry / table purge | Security monitoring |
| **Temporary Uploads** | 7 days | Periodic cron S3 / disk cleanup | Storage hygiene |
| **Soft Delete Grace Period** | 30 days before permanent hard purge | Background worker soft-delete sweep | Accidental deletion recovery |

---

## 4. Right to Erasure (GDPR Art. 17 / PDPA)

When an erasure request is executed for an employee or user:
1. **Scope:** All T1 and T2 fields must be scrubbed across PostgreSQL tables (`users`, `employees`, `ess_profiles`, `emergency_contacts`, `payroll_records`).
2. **Anonymization Invariant:** Foreign keys and non-PII historical operational logs (e.g., aggregate hours, timesheet segment totals) MUST retain referential integrity.
   - PII strings replaced with deterministic hashes or sentinel values:
     - `email` $\rightarrow$ `deleted_<hash>@anonymized.local`
     - `full_name` $\rightarrow$ `DELETED_USER_<hash>`
     - `phone` $\rightarrow$ `NULL` or sentinel `[REDACTED]`
     - `national_id` $\rightarrow$ `NULL` or sentinel `[REDACTED]`
     - `address` $\rightarrow$ `NULL`
     - `bank_account_number` $\rightarrow$ `NULL`
3. **Audit Trail:** Erasure event MUST be written to `audit_logs` capturing `actor_id`, `target_user_id`, timestamp, and action `"USER_DATA_ERASURE"`, but **WITHOUT** containing any erased PII data.
4. **Execution SLA:** Maximum 30 calendar days from request.

---

## 5. Exports & Auditing

- Any endpoint producing CSV, XLSX, or JSON export of T2+ data (`/api/v1/employees/export`, `/api/v1/payroll/export`, `/api/v1/reports/*`) MUST:
  1. Record an audit log entry in table `audit_logs`:
     - `actor_id` (UUID of requesting user)
     - `action` (e.g., `"EXPORT_PAYROLL_CSV"`)
     - `target_resource_id` (report or batch ID)
     - `details` (JSON object including `row_count`, `export_format`, `filter_params`)
     - `ip_address` and `user_agent` extracted from client context
  2. Verify calling user has role permission `export:read` or `payroll:export`.
