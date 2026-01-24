# Implementation Plan: Employees Page Pagination

## Approach
- Server-side pagination to ensure scalability and fix the "50-limit" visibility bug.
- Use the existing `PaginationControls` UI component for consistency.

## Steps

### 1. Backend Models & Repository
- Update `EmployeeSearchQuery` in `apiserver/src/models.rs`.
- Update `employee_repository::get_all_employees` in `apiserver/src/repositories/employee_repository.rs` to support `LIMIT` and `OFFSET` and return total count.

### 2. Backend Service & Handler
- Update `EmployeeService` in `apiserver/src/services/employee_service.rs`.
- Update `employees::get_employees` in `apiserver/src/handlers/employees.rs` to return `PaginatedResponse<Employee`.

### 3. Frontend API Layer
- Update `webdashboard/src/lib/api.ts` types and `searchEmployees` function.

### 4. Frontend UI Integration
- Update `webdashboard/src/pages/EmployeesPage.tsx` with pagination state and `PaginationControls`.

### 5. Verification
- Run backend tests and check UI functionality.
