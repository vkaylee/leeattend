# Implementation Plan: Bulk Schedule Assignment and Department Defaults (2026-01-16) [COMPLETED]

## Problem
Currently, working patterns (roster templates) can only be assigned to employees individually. This is inefficient for large organizations. We need a way to:
1. Assign a default working pattern to a Department.
2. Bulk assign a working pattern to multiple employees.

## Proposed Changes

### 1. Database
- **Migration**: [DONE] Add `default_working_pattern_id` to `departments` table.
- **Logical Priority Update**: [DONE]
    1. `shift_rosters` (Manual Override)
    2. `employee_working_patterns` (Individual Assignment)
    3. `department.default_working_pattern_id` (NEW)
    4. `employee_shifts` (Legacy Fixed)
    5. `department_default_shifts` (Legacy Dept Fixed)

### 2. Backend
- **Models**: [DONE] Update `Department`, `CreateDepartmentRequest`, `UpdateDepartmentRequest`.
- **Repository**: [DONE]
    - `roster_repository.rs`: Update `get_effective_shift` and `get_workspace_schedules`.
    - `working_pattern_repository.rs`: Add `bulk_assign_pattern`.
- **Handlers**: [DONE]
    - `working_patterns.rs`: `POST /api/workspaces/{id}/working-patterns/bulk-assign`.
    - `departments.rs`: Update to handle `default_working_pattern_id`.

### 3. Frontend
- **WorkingPatternsPage**: [DONE] Add "Bulk Assign" modal.
- **DepartmentsPage**: [DONE] Add "Default Pattern" selector in create/edit modal.

## Steps

### Step 1: Database Migration [DONE]
Create a new migration to add the column to `departments`.

### Step 2: Backend Implementation [DONE]
1. Update `models.rs`.
2. Implement bulk assignment repository and service methods.
3. Update resolution logic in `roster_repository.rs`.
4. Register new routes.

### Step 3: Frontend Implementation [DONE]
1. Update API client `webdashboard/src/lib/api/working-patterns.ts`.
2. Add Bulk Assign UI to `WorkingPatternsPage`.
3. Update Department modal.

## Testing [DONE]
- **Integration Test**: [DONE] Verified `get_effective_shift` returns department pattern if no individual pattern exists.
- **API Test**: [DONE] Test bulk assignment endpoint with valid/invalid data.
- **UI Test**: [DONE] Manual verification of bulk assignment flow.
