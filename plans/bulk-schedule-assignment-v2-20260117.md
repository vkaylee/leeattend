# Implementation Plan: Enterprise Bulk Schedule Assignment & Verification (2026-01-17)

## Approach
- **Why this solution**: The existing implementation is functional but needs "Enterprise-grade" robustness. This includes better error reporting for bulk operations, transaction safety, and a more intuitive UI for selecting target employees.
- **Alternatives considered**: Raw SQL execution (Faster development but bypasses business logic and auditing).

## Steps

### 1. Backend: Robust Bulk Processing (30 min)
- **Files to modify**: `apiserver/src/repositories/working_pattern_repository.rs`
- **Goal**: Implement a repository method that uses a single database transaction for bulk inserts/updates and returns a structured result (count of successes, list of failed IDs).
```rust
pub struct BulkAssignResponse {
    pub success_count: usize,
    pub failed_employees: Vec<Uuid>,
}
```

### 2. Backend: Advanced Conflict Validation (20 min)
- **Goal**: Before assigning a new pattern, check for overlapping active patterns for each employee to prevent data corruption.

### 3. Frontend: Enhanced Selection UI (40 min)
- **Files to modify**: `webdashboard/src/pages/WorkingPatterns/BulkAssignModal.tsx`
- **Goal**: Use the `MultiSelectEmployeeSearch` component (if available) or add filters (Department, Job Title) to the assignment modal.
- **Feedback**: Show a progress indicator or a detailed result summary toast.

### 4. Integration & Testing (30 min)
- **Files to create**: `apiserver/tests/bulk_assignment_integration_test.rs`
- **Coverage**: 
    - Test assigning to 100+ employees.
    - Test partial failure (e.g., one employee ID is invalid).
    - Verify `get_effective_shift` priority after bulk assignment.

## Timeline
| Phase | Duration |
|-------|----------|
| Backend Optimization | 30 min |
| Conflict Validation | 20 min |
| Frontend UX Polish | 40 min |
| Integration Testing | 30 min |
| **Total** | **2 hours** |

## Rollback Plan
1. **Database**: All bulk operations are wrapped in SQL Transactions. If one fails, the entire batch rolls back.
2. **Code**: Revert to the stable `COMPLETED` version from 2026-01-16.

## Security Checklist
- [ ] **Auth**: Verify `ctx.is_admin_or_owner()` on the bulk assign endpoint.
- [ ] **Validation**: Maximum 500 employees per request to prevent OOM/Timeout.
- [ ] **Audit**: Ensure the action is logged in the system audit logs (if implemented).
