# Payroll Data Inventory

Owner: Payroll platform
Runtime: Go `net/http` + PostgreSQL
Scope: one implicit organization
Last reviewed: 2026-11-14

| Table | Classification | Retention | Owner | Notes |
|---|---|---:|---|---|
| `payroll_metric_definitions` | T4/T3 | Configuration lifetime | Production/Payroll | Canonical metric code, unit, source, employee scope. No salary amount. |
| `payroll_input_batches` | T2 | Payroll legal retention | Production/HR | Source batch, period, lifecycle status, actor IDs, idempotency hash. |
| `payroll_inputs` | T2 | Payroll legal retention | Production/Payroll | Employment-linked metric values. Never log raw values. Locked rows feed KPI calculation. |
| `payroll_input_corrections` | T2 | Payroll legal retention | HR/Payroll | Old/new metric values, reason, correcting actor. Detail access requires payroll authorization. |
| `payroll_formula_rules` | T4 | Configuration lifetime | HR | Stable formula code/name/component mapping. |
| `payroll_formula_rule_versions` | T2/T4 | While referenced by payroll snapshots plus legal retention | HR/Payroll | Formula expressions and actor IDs. Published versions immutable. |
| `payroll_calculation_runs` | T2 | Payroll legal retention | Payroll/Finance | Input/output snapshots and safe calculation trace linked to employment/payslip. Never log raw snapshots. |
| `audit_logs` | T2 | Minimum 3 years | Compliance | Sanitized payroll lifecycle events. Details exclude salary, metric values, expressions, correction reasons, source references, and PII. |

## Access rules

- Handlers enforce permissions before repository/business logic.
- `payroll:read` reads formula history, inputs, and calculation runs according to role scope.
- `payroll:manage` creates/corrects formula and input data.
- `payroll:approve` approves input batches.
- `payroll:calculate` runs payroll.
- Employees do not receive organization-wide input batches or formula management data.

## Retention and export

Payroll data retention follows applicable labor/accounting requirements. Any export containing T2 data must be audit logged with actor, timestamp, and row count. Do not place raw payroll values in logs, screenshots, test artifacts, commits, or browser evidence.
