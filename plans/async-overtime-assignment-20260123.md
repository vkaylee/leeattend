# Implementation Plan: Async Search for Overtime Policy Assignment

To handle 3000+ employees, we need to transition from loading all employees at once to an asynchronous search-as-you-type approach. This prevents browser lag and reduces network payload.

## Approach
- **Backend**: Update `get_all_employees` repository to include a `LIMIT` (e.g., 50) to prevent oversized responses.
- **Frontend**: Create a new `AsyncEmployeeSearch` component (or refactor existing `EmployeeSearch`) that uses `useQuery` with a debounced search term.
- **Integration**: Update `AssignOvertimePolicyModal` to use the new async component for both Employee and Department selection (if needed).

## Steps

1. **Backend Optimization** (15 min)
   - Modify `apiserver/src/repositories/employee_repository.rs` to add `LIMIT 50` to search queries.
   - Run `cargo sqlx prepare` to update the offline cache.

2. **Async Search Component** (30 min)
   - Update `webdashboard/src/components/common/EmployeeSearch.tsx` to support `async` mode.
   - Implement debouncing using `setTimeout` or a custom hook within the component.
   - Integrate with `searchEmployees` API from `lib/api/employees.ts`.

3. **Modal Integration** (15 min)
   - Replace the static `input` for Target ID in `AssignOvertimePolicyModal.tsx` with `EmployeeSearch` (for employees) and `Combobox` (for departments).
   - Ensure `department` selection also handles 3000+ if applicable (unlikely for depts, but should use a consistent search UI).

4. **Verification & Testing** (30 min)
   - Manually test search with various queries (name, employee ID).
   - Add/Update unit tests for `EmployeeSearch` to mock async API responses.
   - Verify that only the top 50 results are returned and UI remains responsive.

## Timeline
| Phase | Duration |
|-------|----------|
| Backend Optimization | 15 min |
| Async Search UI | 30 min |
| Modal Integration | 15 min |
| Testing | 30 min |
| **Total** | **1.5 hours** |

## Rollback Plan
- Revert changes to `AssignOvertimePolicyModal.tsx` and `EmployeeSearch.tsx`.
- Remove the `LIMIT` in `employee_repository.rs`.

## Security Checklist
- [x] Input validation (search query sanitization via SQL parameters).
- [x] Auth checks (MemberContext already verifies workspace access).
- [ ] Rate limiting (Considered, but debouncing on client handles most cases).
- [x] Error handling (Handle API failures in UI).