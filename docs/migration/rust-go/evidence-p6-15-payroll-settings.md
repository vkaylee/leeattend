# P6-15 Payroll Settings, Employee Config & Incentive Policies Evidence

Task 5.2 Payroll settings & configs (9 missing routes) completed in Go API, advancing contract route coverage from 326 to 335.

## Route Coverage Status

- Contract routes: **403**
- Go route registrations: **360** (+9)
- Contract routes covered: **335** (+9)
- Contract routes missing: **68** (-9)

---

## 9 Newly Implemented & Wired Routes

All 9 routes wired in `apiserver/main.go` and implemented in `internal/api/payroll.go` and `internal/repository/payroll.go`:

### 1. Workspace Payroll Settings (3)
1. `GET /api/v1/workspaces/{ws_id}/payroll-settings` — 200 OK `PayrollSettings`. Returns active settings resolved from `workspace_config_entries` (`scope_type = 'workspace'`) as of today in workspace timezone. 404 if no settings exist. Scoped to workspace members.
2. `PATCH /api/v1/workspaces/{ws_id}/payroll-settings` — 200 OK `PayrollSettings`. Partially updates workspace payroll settings at effective date (defaulting to today in workspace timezone). Merges unspecified fields with existing snapshot; supports clearing max overtime fields with explicit null. Scoped to workspace members.
3. `GET /api/v1/workspaces/{ws_id}/payroll-settings/history` — 200 OK `[]PayrollSettings`. Resolves historical SCD timeline of payroll settings grouped by distinct `effective_from` dates descending. Scoped to workspace members.

### 2. Employee Payroll Config (2)
4. `GET /api/v1/workspaces/{ws_id}/employees/{id}/payroll-config` — 200 OK `[]EmployeePayrollConfig`. Returns SCD history list of standard working days and daily hours for employee's active employment. Validates employee exists (404) and has active employment (400). Scoped to workspace members.
5. `POST /api/v1/workspaces/{ws_id}/employees/{id}/payroll-config` — 201 Created `EmployeePayrollConfig`. Upserts standard working days and daily hours into `workspace_config_entries` (`scope_type = 'employment'`). Validates date format, non-negative decimals, employee existence (404), and active employment (400). Scoped to workspace members.

### 3. Incentive Policies (4)
6. `GET /api/v1/workspaces/{ws_id}/payroll/incentive-policies` — 200 OK `[]PayrollIncentivePolicyResponse`. Lists active, non-deleted incentive policies (`payroll_incentive_policies`) along with their tiers (`payroll_incentive_tiers`) ordered by priority ascending. Scoped to workspace members.
7. `POST /api/v1/workspaces/{ws_id}/payroll/incentive-policies` — 201 Created `PayrollIncentivePolicy`. Creates new incentive policy with active state and persists nested tiers. Validates non-empty name and valid effective date. Scoped to workspace members.
8. `DELETE /api/v1/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}` — 204 No Content. Soft-deletes policy setting `deleted_at = NOW()`. Scoped to workspace members.
9. `POST /api/v1/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}/clone` — 201 Created `PayrollIncentivePolicy`. Creates a clone of existing policy with `"{name} (Copy)"`, inactive status (`is_active = false`), effective date set to current date, and clones all tiers with fresh IDs. 404 if original policy missing or deleted. Scoped to workspace members.

---

## Complete Payroll Settings & Policies Route Map (9/9)

| # | Method | Path | Status | Handler | Auth Guard |
|---|---|---|---|---|---|
| 1 | GET | `/api/v1/workspaces/{ws_id}/payroll-settings` | 200 OK | `PayrollHandler.GetPayrollSettings` | `MemberContext` / IsMember |
| 2 | PATCH | `/api/v1/workspaces/{ws_id}/payroll-settings` | 200 OK | `PayrollHandler.UpdatePayrollSettings` | `MemberContext` / IsMember |
| 3 | GET | `/api/v1/workspaces/{ws_id}/payroll-settings/history` | 200 OK | `PayrollHandler.GetPayrollSettingsHistory` | `MemberContext` / IsMember |
| 4 | GET | `/api/v1/workspaces/{ws_id}/employees/{id}/payroll-config` | 200 OK | `PayrollHandler.GetEmployeePayrollConfig` | `MemberContext` / IsMember |
| 5 | POST | `/api/v1/workspaces/{ws_id}/employees/{id}/payroll-config` | 201 Created | `PayrollHandler.SetEmployeePayrollConfig` | `MemberContext` / IsMember |
| 6 | GET | `/api/v1/workspaces/{ws_id}/payroll/incentive-policies` | 200 OK | `PayrollHandler.GetIncentivePolicies` | `MemberContext` / IsMember |
| 7 | POST | `/api/v1/workspaces/{ws_id}/payroll/incentive-policies` | 201 Created | `PayrollHandler.CreateIncentivePolicy` | `MemberContext` / IsMember |
| 8 | DELETE | `/api/v1/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}` | 204 No Content | `PayrollHandler.DeleteIncentivePolicy` | `MemberContext` / IsMember |
| 9 | POST | `/api/v1/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}/clone` | 201 Created | `PayrollHandler.CloneIncentivePolicy` | `MemberContext` / IsMember |

---

## Key Implementation Details

1. **Typed Decimals**: `shopspring/decimal.Decimal` used across all models and scanning logic for standard days, daily hours, overtime caps, and incentive payout amounts. Zero `float64` in struct models.
2. **Workspace Timezone Resolution**: Settings effective dates use `workspaces.timezone` parsed via `timezone.ParseWorkspaceTZ`, defaulting to UTC.
3. **SCD Snapshots**:
   - `workspace_config_entries` EAV entries resolved at `effective_from <= date` boundaries.
   - Null handling via double-pointer `**decimal.Decimal` distinguishes omitted fields from explicit JSON `null` field resets.
4. **Active Employment Validation**: Employee payroll configs validate employee existence in workspace (404 `error.employee_not_found`) and resolve active employment (400 `error.no_active_employment`).
5. **Incentive Policy Deep Cloning**: Clones policy row with `"(Copy)"` suffix, `is_active = false`, and clones all tier rows with newly generated UUIDs.
6. **Unit Tests**:
   - `TestPayrollSettingsEndpoints`: 200 settings lookup, 404 missing workspace settings, 403 permission guard, 200 patch with partial fields, 400 invalid date, 200 history timeline.
   - `TestEmployeePayrollConfigEndpoints`: 200 config lookup, 404 missing employee, 400 no active employment, 201 upsert config, 400 invalid date / negative values, 403 permission guard.
   - `TestIncentivePoliciesEndpoints`: 200 list with tiers, 201 create with tiers, 400 invalid payloads, 204 delete, 201 clone with `(Copy)` name and inactive status, 404 missing policy clone.
