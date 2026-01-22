# Implementation Plan: Implement Offset-based Pagination for Leave Balances with Server-side Search

## Approach
We will implement **Offset-based Pagination** (Approach A from Brainstorm) for the Leave Balances page. This is the standard pattern for Admin Dashboards, allowing users to jump to specific pages and see total record counts.

**Why this solution:**
- **User Expectation:** HR admins expect standard table navigation (Page 1, 2, 3...) rather than infinite scroll.
- **Scalability:** Moves filtering and slicing logic from Client (browser memory) to Server (Postgres efficient query), preventing browser crashes with large employee counts.
- **Maintainability:** Standard SQL `LIMIT` and `OFFSET` is easy to maintain and debug.

**Alternatives considered:**
- **Client-side Pagination:** Rejected because it doesn't solve the initial payload size issue or backend database load fetching all rows.
- **Cursor-based Pagination:** Rejected because it complicates "jump to page" functionality which is desirable for admin tables.

## Steps

1.  **Define DTOs & Models** (15 min)
    -   Create `PaginationParams` struct for query parameters (`page`, `limit`, `q`).
    -   Create `PaginatedResponse<T>` generic struct for API responses.
    -   Update `get_workspace_leave_balances` handler signature.
    -   Files: `apiserver/src/models.rs`, `apiserver/src/handlers/leaves.rs`

2.  **Backend Implementation** (30 min)
    -   Modify `LeaveService::get_workspace_balances` to accept pagination/search args.
    -   Update `LeaveRepository` (or inline SQL in Service) to use `COUNT(*) OVER()` for total items and `LIMIT/OFFSET` for data.
    -   Implement case-insensitive search logic for `employee_name` or `leave_type`.
    -   Files: `apiserver/src/services/leave_service.rs`, `apiserver/src/repositories/leave_repository.rs` (if exists) or service directly.

3.  **Frontend API Client Update** (15 min)
    -   Update `fetchWorkspaceLeaveBalances` in `webdashboard/src/lib/api/leaves.ts` to accept `page`, `limit`, `search`.
    -   Update types to handle the new `PaginatedResponse` structure (or `items` array + metadata).
    -   Files: `webdashboard/src/lib/api/leaves.ts`

4.  **Frontend UI Implementation** (45 min)
    -   Refactor `LeaveBalancesPage.tsx` to use server-side state instead of client-side filtering.
    -   Add `Pagination` controls (Prev, Next, Page numbers).
    -   Debounce the Search input to prevent excessive API calls.
    -   Files: `webdashboard/src/pages/LeaveBalancesPage.tsx`

5.  **Testing** (20 min)
    -   Add backend integration test for paginated results (page 1, page 2, search filtering).
    -   Verify frontend manually (mock data or dev env).
    -   Files: `apiserver/tests/leave_api_tests.rs`

## Timeline
| Phase | Duration |
|-------|----------|
| Define DTOs | 15 min |
| Backend Logic | 30 min |
| Frontend API | 15 min |
| Frontend UI | 45 min |
| Testing | 20 min |
| **Total** | **~2 hours** |

## Rollback Plan
1.  Revert `LeaveBalancesPage.tsx` to use the old `fetchWorkspaceLeaveBalances` signature.
2.  Revert backend `LeaveService` to remove `LIMIT/OFFSET`.

## Security Checklist
-   [ ] **Input validation**: Ensure `limit` is capped (e.g., max 100) to prevent DoS.
-   [ ] **SQL Injection**: Use parameterized queries for `limit`, `offset`, and `search` (sqlx handles this, but verify logic).
-   [ ] **Auth checks**: Ensure `ctx.is_admin_or_owner()` remains enforced.
