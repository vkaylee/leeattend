# Task Plan: Refactoring Core Domain (Workspace & Employees)

## 🛡️ Rule Compliance Check
- **Rule 4.8 / Execution Protocol:** I will strictly use the hermetic toolbox script `./scripts/_npm.sh run lint` to execute linters and never run `npm` directly on the host. I will ensure all background commands are PTY-safe by appending `</dev/null`.
- **Rule 4.2 / Clean Code Standards:** I will break down large "God components" (e.g., `employee-table.tsx`, `role-card.tsx`, `permissions-matrix.tsx`) into smaller, modular sub-components, reducing complexity below 10 and line count below 100 per function.
- **Rule 5.1 / Project Type Routing:** As the `@frontend-specialist`, I will ensure all React best practices are followed, applying proper TypeScript definitions aligned with backend models and removing all `any` and unsafe accesses. I will adhere to the Anti-Collision Protocol and only modify files in `src/pages/workspace/**` and `src/pages/employees/**`.

## Objectives

Systematically eliminate all technical debt in the **Core Domain**:
1. `webdashboard/src/pages/workspace/**`
2. `webdashboard/src/pages/employees/**`

### Key Problems Identified:
Based on the initial linting pass:
1. **`employee-table.tsx`**: Too large (158 lines), high complexity (14), extensive use of `any` leading to unsafe-member-access and leaked-render issues.
2. **`employees-page.tsx`**: Extremely large (268 lines), very high complexity (18), missing types, and import sorting issues.
3. **`permissions-matrix.tsx`**: Large (145 lines), high complexity (16), multiple unsafe accesses and any.
4. **`role-card.tsx`**: Large (103 lines), complexity (13), and type issues.
5. **`workspace-overview-page.tsx`**: Large (125 lines), high complexity (14).
6. **Various files**: Import sorting (`perfectionist`), void expression, and leaked render issues.

## Execution Plan

### Phase 1: `webdashboard/src/pages/employees` Refactor
1. **`employee-table.tsx`**:
   - Extract the row rendering logic into an `EmployeeTableRow` component.
   - Extract the column headers into a separate variable or component.
   - Replace `any` with the proper `User` or `Employee` type from the API models (by defining it locally or importing the existing ones if available in `src/lib/api/types`).
2. **`employees-page.tsx`**:
   - Extract the complex modals (Edit, Delete, Attendance, etc.) into separate files under `src/pages/employees/components/` or extract hook logic into `src/pages/employees/hooks/`.
   - Fix import sorting and type safety.
   
### Phase 2: `webdashboard/src/pages/workspace` Refactor
1. **`role-card.tsx`**:
   - Break out the actions dropdown and role detail rendering.
   - Type the `role` prop properly using local or API types instead of `any`.
2. **`permissions-matrix.tsx`**:
   - Extract table cell logic and category grouping into separate helper functions or components.
   - Add proper typings for permissions mapping.
3. **`workspace-overview-page.tsx`**:
   - Extract sections (stats, metrics, recent activity) into standalone components under `src/pages/workspace/components/`.
4. **Hooks and Pages**:
   - Fix perfectionist sorting issues across hooks and wrapper pages.
   
### Phase 3: Validation
- Run local linting on the target folders: `./scripts/_npm.sh run lint -- src/pages/workspace src/pages/employees`
- Ensure zero errors.
- Run the full test suite: `./test.sh --web </dev/null`
