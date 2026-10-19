# P6-14 Salary Bases & Components Endpoints Evidence

Task 5.1 Salary bases & components (8 missing routes) completed in Go API, advancing contract route coverage from 318 to 326.

## Route Coverage Status

- Contract routes: **403**
- Go route registrations: **351** (+8)
- Contract routes covered: **326** (+8)
- Contract routes missing: **77** (-8)

---

## 8 Newly Implemented & Wired Routes

All 8 routes wired in `apiserver/main.go` and implemented in `internal/api/payroll.go` and `internal/repository/payroll.go`:

### 1. Salary Component Definitions (2)
1. `GET /api/v1/workspaces/{ws_id}/salary-components` — 200 OK `[]SalaryComponentDefinition`. Lists all salary component definitions scoped to the workspace (`salary_component_definitions`), ordered by name and code. Scoped to workspace members (`payroll:read` or MemberContext).
2. `POST /api/v1/workspaces/{ws_id}/salary-components` — 201 Created `SalaryComponentDefinition`. Creates a new salary component definition with optional `base_group_ids`. If base groups provided, validates workspace and system groups membership (403 on invalid base group); persists component and base associations into `salary_component_base_members`. Guarded by `payroll:components:manage`.

### 2. Employment Salary Components (2)
3. `GET /api/v1/workspaces/{ws_id}/employments/{id}/salary-components` — 200 OK `[]EmploymentSalaryComponentResponse`. Returns distinct-on effective components for the employment as of today in the workspace's configured timezone (fallback UTC). Joins `employee_salary_components` with `salary_component_definitions`. Guarded by workspace membership / `payroll:read`.
4. `POST /api/v1/workspaces/{ws_id}/employments/{id}/salary-components` — 201 Created `EmployeeSalaryComponent`. Assigns salary component to an employment with decimal amount and effective date (`YYYY-MM-DD`). Validates existence of component definition and employment within workspace. Guarded by `payroll:components:manage`.

### 3. Salary Bases & Membership (4)
5. `GET /api/v1/workspaces/{ws_id}/salary-bases` — 200 OK `SalaryBasesResponse`. Returns salary base groups (`salary_base_groups`) including system bases (`gross`, `net`, `bhxh`, `taxable` where `workspace_id IS NULL`) and workspace custom bases (`workspace_id = $ws`), along with all member associations (`salary_component_base_members`). Guarded by workspace membership / `payroll:read`.
6. `POST /api/v1/workspaces/{ws_id}/salary-bases` — 201 Created `SalaryBaseGroup`. Creates a custom workspace salary base group (`is_system = false`). Validates non-empty name and slug, maps slug conflicts to 409. Guarded by `payroll:components:manage`.
7. `POST /api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members` — 200 OK `{"success": true}`. Assigns a component to a custom salary base group. Enforces tenant boundary: rejects attempts to assign members to system bases or unowned groups (403 Forbidden). Guarded by `payroll:components:manage`.
8. `DELETE /api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members/{component_id}` — 200 OK `{"success": true}`. Unassigns a component from a workspace base group. Enforces tenant boundary via workspace filter on base group. Guarded by `payroll:components:manage`.

---

## Complete Salary Component & Base Route Map (8/8)

| # | Method | Path | Status | Handler | Auth Guard |
|---|---|---|---|---|---|
| 1 | GET | `/api/v1/workspaces/{ws_id}/salary-components` | 200 OK | `PayrollHandler.ListSalaryComponents` | `payroll:read` / Member |
| 2 | POST | `/api/v1/workspaces/{ws_id}/salary-components` | 201 Created | `PayrollHandler.CreateSalaryComponent` | `payroll:components:manage` |
| 3 | GET | `/api/v1/workspaces/{ws_id}/employments/{id}/salary-components` | 200 OK | `PayrollHandler.ListEmploymentSalaryComponents` | `payroll:read` / Member |
| 4 | POST | `/api/v1/workspaces/{ws_id}/employments/{id}/salary-components` | 201 Created | `PayrollHandler.AssignEmploymentSalaryComponent` | `payroll:components:manage` |
| 5 | GET | `/api/v1/workspaces/{ws_id}/salary-bases` | 200 OK | `PayrollHandler.ListSalaryBases` | `payroll:read` / Member |
| 6 | POST | `/api/v1/workspaces/{ws_id}/salary-bases` | 201 Created | `PayrollHandler.CreateSalaryBase` | `payroll:components:manage` |
| 7 | POST | `/api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members` | 200 OK | `PayrollHandler.AssignBaseMember` | `payroll:components:manage` |
| 8 | DELETE | `/api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members/{component_id}` | 200 OK | `PayrollHandler.UnassignBaseMember` | `payroll:components:manage` |

---

## Key Implementation Details

1. **Typed Decimal Values**: Numeric fields use `shopspring/decimal.Decimal` across all models and scanning logic. Zero `float64` for amounts.
2. **Workspace Timezone Resolution**: Employment salary components resolution loads timezone from `workspaces` table via `timezone.ParseWorkspaceTZ`, defaulting to UTC if missing/invalid, ensuring consistent date boundaries.
3. **Tenant & System Base Group Isolation**:
   - `salary_base_groups` lists system groups (`workspace_id IS NULL`) plus workspace groups (`workspace_id = $ws`).
   - `AssignBaseMember` and `UnassignBaseMember` enforce that only workspace-owned base groups can have members modified; system groups reject mutation with 403 Forbidden.
4. **Unit Tests**:
   - `TestPayrollSalaryComponentsEndpoints`: Component listing, creation (201), validation (400), and permission guard (403).
   - `TestPayrollEmploymentSalaryComponentsEndpoints`: Effective component resolution by employment ID with decimal amount verification, assignment (201), and date validation (400).
   - `TestPayrollSalaryBasesEndpoints`: Listing groups and members, custom base creation (201), member assignment (200), rejection of system base assignment (403), and unassignment (200).
