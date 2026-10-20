# P6-16 Payroll Rules, Tiers, Assignments & Calculation Execution Evidence

Task 5.3 Payroll rules, calculations, and execution (10 routes) completed in Go API, advancing contract route coverage from 335 to 345 (missing routes reduced from 68 to 58).

## Route Coverage Status

- Contract routes: **403**
- Go route registrations: **370** (+10)
- Contract routes covered: **345** (+10)
- Contract routes missing: **58** (-10)

---

## 10 Newly Implemented & Wired Routes

All 10 routes wired in `apiserver/main.go` and implemented in `internal/api/payroll.go` and `internal/repository/payroll.go`:

### 1. Payroll Rules (2)
1. `GET /api/v1/workspaces/{ws_id}/payroll-rules` — 200 OK `[]PayrollRuleDefinition`. Lists rule definitions (`payroll_rule_definitions`) ordered by display order and creation date descending. Scoped to workspace members.
2. `POST /api/v1/workspaces/{ws_id}/payroll-rules` — 201 Created `PayrollRuleDefinition`. Creates rule definition with unique constraint check on `(workspace_id, code)` (409 Conflict). Validates name and code. Scoped to workspace members.

### 2. Rule Tiers (3)
3. `GET /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers` — 200 OK `[]PayrollRuleTier`. Lists tiers for rule ordered by `tier_order` ascending. 404 if rule does not exist. Scoped to workspace members.
4. `POST /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers` — 201 Created `PayrollRuleTier`. Adds calculation tier. Validates `tier_order >= 0`, `min_amount >= 0`, and `max_amount > min_amount` (400 Validation on violation). 409 Conflict on duplicate tier order. Scoped to workspace members.
5. `DELETE /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers/{tier_id}` — 200 OK `{"success": true}`. Deletes tier by ID for rule. Scoped to workspace members.

### 3. Rule Assignments (3)
6. `GET /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments` — 200 OK `[]PayrollRuleAssignment`. Lists assignments for rule ordered by creation date descending. 404 if rule does not exist. Scoped to workspace members.
7. `POST /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments` — 201 Created `PayrollRuleAssignment`. Assigns rule to workspace, department, or employee. Validates assignee type, required assignee ID for department/employee, and date formats. Checks GIST range exclusion constraint (409 Conflict on overlapping periods for same assignee). Scoped to workspace members.
8. `DELETE /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments/{assignment_id}` — 200 OK `{"success": true}`. Deletes assignment by ID. Scoped to workspace members.

### 4. Payroll Calculation Execution (2)
9. `POST /api/v1/workspaces/{ws_id}/employees/{id}/calculate-payroll` — 200 OK `FullPayslipResponse`. Executes single-employee payroll calculation pipeline for specified `year` and `month` query parameters. Validates employee existence (404), active employment (400), and date parameters (400). Requires `payroll:calculate` (`PermissionPayrollCalculate`).
10. `POST /api/v1/workspaces/{ws_id}/payroll/calculate-batch` — 200 OK `BatchCalculatePayrollResponse`. Executes batch payroll calculation for up to 500 employment IDs (accepting `employee_ids` alias). Validates non-empty batch and batch size <= 500 (400 Validation). Deduplicates IDs and returns per-item `success` or `error` results. Requires `payroll:calculate` (`PermissionPayrollCalculate`).

---

## Complete Payroll Execution Route Map (10/10)

| # | Method | Path | Status | Handler | Auth Guard |
|---|---|---|---|---|---|
| 1 | GET | `/api/v1/workspaces/{ws_id}/payroll-rules` | 200 OK | `PayrollHandler.ListPayrollRules` | `MemberContext` / IsMember |
| 2 | POST | `/api/v1/workspaces/{ws_id}/payroll-rules` | 201 Created | `PayrollHandler.CreatePayrollRule` | `MemberContext` / IsMember |
| 3 | GET | `/api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers` | 200 OK | `PayrollHandler.ListPayrollRuleTiers` | `MemberContext` / IsMember |
| 4 | POST | `/api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers` | 201 Created | `PayrollHandler.CreatePayrollRuleTier` | `MemberContext` / IsMember |
| 5 | DELETE | `/api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers/{tier_id}` | 200 OK | `PayrollHandler.DeletePayrollRuleTier` | `MemberContext` / IsMember |
| 6 | GET | `/api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments` | 200 OK | `PayrollHandler.ListPayrollRuleAssignments` | `MemberContext` / IsMember |
| 7 | POST | `/api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments` | 201 Created | `PayrollHandler.AssignPayrollRule` | `MemberContext` / IsMember |
| 8 | DELETE | `/api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments/{assignment_id}` | 200 OK | `PayrollHandler.DeletePayrollRuleAssignment` | `MemberContext` / IsMember |
| 9 | POST | `/api/v1/workspaces/{ws_id}/employees/{id}/calculate-payroll` | 200 OK | `PayrollHandler.CalculatePayroll` | `payroll:calculate` (`PermissionPayrollCalculate`) |
| 10 | POST | `/api/v1/workspaces/{ws_id}/payroll/calculate-batch` | 200 OK | `PayrollHandler.CalculateBatchPayroll` | `payroll:calculate` (`PermissionPayrollCalculate`) |

---

## Key Implementation Details

1. **In-Process Pure Go Domain Calculation Pipeline**:
   - `domain/payroll.MonthBoundaries(year, month)`: Validates year (2000-2100) and month (1-12), computes start and end dates.
   - `domain/payroll.BuildSegments(startDate, endDate, salaryChanges, deptChanges)`: Splits pay period into contiguous segments across mid-month compensation or department transitions.
   - `domain/payroll/rates.CalculateDailyRate`: Computes daily base rate from base salary and standard working days.
   - `domain/payroll/rates.CalculateHourlyRate` & `domain/payroll/rates.ApplyOTCap`: Computes hourly rate, overtime earnings, and enforces monthly overtime hour cap from workspace settings.
   - `domain/payroll/incentives.EvaluatePolicy`: Evaluates active incentive policies against attendance metrics and resolves payout tiers.
   - `domain/payroll/rules.EvaluateRules`: Evaluates statutory deductions (`BHXH`, `BHYT`, `BHTN`, `PIT`) across Percentage, Fixed, and Tiered rules using base maps (`gross`, `base_pay`, `bhxh_base`).
   - `domain/payroll.AssemblePayslip`: Assembles `net_pay = base + ot + holiday_premium + incentive - deductions`.

2. **Persistence & Idempotency**:
   - Re-calculation cleans up existing child records (`payslip_segments`, `payslip_incentives`, `payslip_rule_evaluations`) and updates `payslips` in place.
   - Initial calculation generates UUIDs and inserts `payslips` along with all child rows in tenant DB.

3. **Typed Decimals**:
   - `shopspring/decimal.Decimal` used across all numeric rates, hours, days, amounts, and deduction tiers. Zero `float64` in domain and DTO models.

4. **Batch Processing**:
   - Deduplicates employment IDs while preserving input order.
   - Limits batch size to 1..500 employees.
   - Captures per-employment success/failure without aborting remaining calculations in batch.

5. **Unit Tests (`internal/api/payroll_test.go`)**:
   - `TestPayrollRulesEndpoints`: 200 OK list rules, 403 Forbidden without membership, 201 Created rule, 400 Validation on invalid input, 409 Conflict on duplicate code.
   - `TestPayrollRuleTiersEndpoints`: 200 OK list tiers, 404 rule not found, 201 Created tier, 400 Validation when max <= min, 200 OK delete tier, 404 tier not found.
   - `TestPayrollRuleAssignmentsEndpoints`: 200 OK list assignments, 404 rule not found, 201 Created assignment, 400 Validation on invalid type/dates, 409 Conflict on overlap exclusion, 200 OK delete assignment, 404 assignment not found.
   - `TestPayrollCalculationEndpoints`: 200 OK calculate single employee payslip, 403 Forbidden without `payroll:calculate`, 400 Validation on invalid date range, 404 employee not found, 400 no active employment, 200 OK calculate batch with mixed results, 400 Validation on empty list, 403 Forbidden without `payroll:calculate`.
