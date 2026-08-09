# React Query Key Migration

This checklist tracks the workspace-scoped React Query migration. The current change completes the `departments` and `employees` resource namespaces; unrelated resource domains remain separate until their own migration is scheduled.

## Completed namespaces

| Namespace | Query-key factory | Workspace query wrapper | Workspace mutation wrapper | Prefix behavior | Manual cache check | Lint warnings |
|---|---|---|---|---|---|---|
| Departments | `departmentKeys` in `webdashboard/src/lib/query-keys.ts` | Migrated department list, lookup, org-chart, assignment, attendance, leave, shift, report, payroll, settings, and approval consumers | Department create/update/delete flows use `departmentKeys.all(wsId)` | `departmentKeys.all(wsId)` prefixes list/detail/lookup descendants | Pending runtime verification | 276 legacy literal-key warnings remain globally; no migrated department literal keys remain |
| Employees | `employeeKeys` in `webdashboard/src/lib/query-keys.ts` | Migrated all-employees, infinite search, detail lookup, payroll/report search, employer/attendance/bulk-picker lookups | Employee create/update/terminate/rehire/undo/location/transfer flows invalidate `employeeKeys.all(wsId)` and retain dedicated relationship invalidations | `employeeKeys.all(wsId)` prefixes list/search/detail/lookup descendants | Pending runtime verification | 276 legacy literal-key warnings remain globally; remaining warnings are unrelated resource domains/relationship caches |

## Required manual cache verification

### Departments

- Open the departments list and organization chart, switch workspaces, and confirm the new workspace never renders the previous workspace's departments while loading.
- Create, edit, and delete a department from the departments page and organization chart. Confirm the department list, org chart, assignment dropdowns, and compatible lookups refresh without a full reload.
- Start with no selected workspace and confirm the existing select-workspace UI, skeleton, empty, and error states remain intact. Confirm no request contains an empty workspace ID.

### Employees

- Open the employee list with search, sort, department, status, and custom-field filters. Switch workspaces and confirm filtered data is isolated and refetches under the new workspace prefix.
- Create and edit an employee, then terminate, rehire, and undo termination. Confirm employee list/search/lookup data and selected employee employment/department history update as before.
- Exercise the organization chart, payroll/report search, attendance policy assignment, working-pattern bulk picker, and employer page. Confirm each uses only the intended workspace-scoped employee prefix.
- Start with no selected workspace and confirm no employee request or mutation is sent with an empty workspace ID.

## Remaining namespace checklist

| Namespace | Factory | Query wrapper | Mutation wrapper | Invalidation-prefix test | Manual cache check | Lint warnings |
|---|---|---|---|---|---|---|
| Attendance | TODO | TODO | TODO | TODO | TODO | TODO |
| Leave | TODO | TODO | TODO | TODO | TODO | TODO |
| Payroll | TODO | TODO | TODO | TODO | TODO | TODO |
| Shifts | TODO | TODO | TODO | TODO | TODO | TODO |
| Reports | TODO | TODO | TODO | TODO | TODO | TODO |
| Workspace/settings | TODO | TODO | TODO | TODO | TODO | TODO |
| Devices/connectivity | TODO | TODO | TODO | TODO | TODO | TODO |
| Documents/files | TODO | TODO | TODO | TODO | TODO | TODO |
| Employment/contracts/compensation | TODO | TODO | TODO | TODO | TODO | TODO |

## Verification record

- Focused query-key tests: passed with `./leedevkit test web --unit-only --pattern 'query-keys|use-workspace-query|employees|departments'`.
- Frontend lint/typecheck: passed with `./leedevkit test web --lint-only`.
- Query-key guard baseline: 276 warning-level legacy literal-key findings remain outside the completed namespace migration; the warning rule is intentionally non-blocking.
- Full frontend suite: passed with `./leedevkit test web` (unit, Playwright integration/E2E, i18n, typecheck, and lint; 5941/5941 checks passed). Manual browser cache verification remains pending.
