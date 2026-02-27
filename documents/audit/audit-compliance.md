# LeeAttend — Compliance Assessment

## Regulatory Context

LeeAttend processes employee attendance, personal information, and payroll data for Vietnamese enterprises. Relevant regulations include:

| Regulation | Relevance |
|---|---|
| Vietnamese Labor Code (2019) | Overtime limits, holiday pay multipliers, leave entitlements |
| Personal Data Protection Decree (13/2023/ND-CP) | Employee PII handling |
| Cybersecurity Law (24/2018/QH14) | Data localization requirements |
| GDPR (if international users) | Data protection principles |

## Labor Law Compliance

### Overtime Regulations

| Requirement | Implementation Status |
|---|---|
| Holiday OT at 300% rate | ✅ Default `default_holiday_ot_multiplier = 3.0` |
| Configurable per-holiday OT multiplier | ✅ `ot_multiplier` field on Holiday model |
| OT tracking and reporting | ✅ `AttendanceDailySummary` with system-detected OT |
| Employee OT claim workflow | ✅ Claim → Approve/Reject workflow |
| Daily/monthly OT limits | ⚠️ Not enforced (tracking only) |

### Leave Entitlements

| Requirement | Status |
|---|---|
| Annual leave (12+ days) | ✅ Configurable leave types and policies |
| Seniority-based extra leave | ✅ `seniority_bonus` in leave policy accrual rules |
| Sick leave | ✅ Configurable leave types |
| Leave balance tracking | ✅ Per-employee balance with history |

### Holiday Calendar

| Requirement | Status |
|---|---|
| National holidays observed | ✅ Holiday calendar with `national` type |
| Holiday pay for non-working days | ✅ `is_paid` flag on holidays |
| Accurate status reporting | ✅ `holiday` vs `holiday_work` statuses |

## Data Protection Compliance

### Personal Data Inventory

| Data Category | Examples | Storage |
|---|---|---|
| Employee PII | Full name, ID number, hire date | PostgreSQL (encrypted at rest depends on hosting) |
| Biometric data references | Device user ID (not raw biometrics) | PostgreSQL |
| Contact information | Emergency contacts | PostgreSQL |
| Attendance records | Punch times, locations | PostgreSQL |
| Financial data | Salary, payslips | PostgreSQL |
| Authentication | Password hashes, JWT tokens | PostgreSQL / In-memory |

### Data Protection Controls

| Control | Status |
|---|---|
| Data minimization | ✅ Only necessary fields collected |
| Purpose limitation | ✅ Data used only for attendance/payroll |
| Access control | ✅ RBAC with workspace isolation |
| Password protection | ✅ bcrypt hashing |
| Data retention policy | ⚠️ Not explicitly configured (no auto-deletion) |
| Data export capability | ✅ Excel export available |
| Data deletion capability | ⚠️ Cascading deletes via FK constraints, no self-service deletion |
| Consent management | ⚠️ Not implemented (employer-managed system) |

## Testing and Quality Compliance

| Control | Status |
|---|---|
| Automated testing | ✅ CI via Docker containers |
| Compile-time verification | ✅ sqlx macros, Rust strict linting |
| Code quality | ✅ `warnings = deny`, `unwrap_used = deny` |
| Integration tests | ✅ Holiday CRUD, reports, OT, attendance |
| Change management | ✅ 89 versioned DB migrations |

## Compliance Gaps

| ID | Gap | Severity | Description |
|---|---|---|---|
| CMP-001 | No data retention policy | Medium | No automated data cleanup or archival |
| CMP-002 | No audit logging | Medium | User actions not logged for compliance trails |
| CMP-003 | No OT limit enforcement | Low | OT hours tracked but daily/monthly limits not enforced |
| CMP-004 | No data deletion workflow | Medium | Employees cannot request data deletion |
| CMP-005 | No encryption at rest | Low | Depends on hosting provider's disk encryption |
