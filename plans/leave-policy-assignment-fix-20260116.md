# ## Implementation Plan: Leave Policy Assignment Enhancement & Fix

### ## Approach
- **Fix Payload Mismatch:** Align Frontend and Backend data structures for `AssignLeavePolicyRequest`.
- **Quick Assignment:** Update Frontend UI to support 'Workspace' (All) and 'Department' assignment types.
- **Data Integrity:** Add `leave_type` to `leave_policy_definitions` to correctly categorize policies and manage overlaps (e.g., distinct policies for 'vacation' vs 'sick').
- **Auto-Close Logic:** Implement backend logic to automatically set `effective_to` on existing assignments when a new one is created for the same category and entity, preventing overlapping active policies.

### ## Steps

1. **Database Migration** (5 min)
   - Create a new migration to add `leave_type` column to `leave_policy_definitions`.
   ```sql
   ALTER TABLE leave_policy_definitions ADD COLUMN leave_type VARCHAR(50) NOT NULL DEFAULT 'vacation';
   ```

2. **Backend Model & Repository Updates** (15 min)
   - Update `LeavePolicyDefinition` struct in `apiserver/src/models.rs`.
   - Update repository methods in `apiserver/src/repositories/leave_policy_repository.rs` to handle `leave_type`.
   - Update `AssignLeavePolicyRequest` to ensure it matches desired frontend payload.

3. **Backend Service Logic (Auto-Close)** (20 min)
   - Modify `assign_leave_policy` handler or add a service method to:
     - Check for existing assignments for the same `assignee_type` and `assignee_id` that overlap with the new `effective_from`.
     - Update `effective_to` of those existing assignments.

4. **Frontend API Synchronization** (5 min)
   - Update `webdashboard/src/lib/api/types.ts` and `leaves.ts` to match the backend `AssignLeavePolicyRequest`.

5. **Frontend UI Enhancement** (30 min)
   - Modify `LeaveRequestsPage.tsx`:
     - Add `assigneeType` state.
     - Add dropdown in Assignment Modal to select type (Employee, Department, Workspace).
     - Conditional rendering: Show Employee/Department selection based on type.
     - Fetch departments for selection.

6. **Testing & Verification** (15 min)
   - Run existing E2E tests.
   - Add a new test case for Department/Workspace assignment.

### ## Timeline
| Phase | Duration |
|-------|----------|
| DB Migration | 5 min |
| Backend Core | 35 min |
| Frontend API/UI | 35 min |
| Testing | 15 min |
| **Total** | **~1.5 hours** |

### ## Rollback Plan
- Revert DB migration using `sqlx migrate revert`.
- Revert code changes via Git.

### ## Security Checklist
- [x] RBAC: Ensure `assign_leave_policy` requires admin permissions.
- [x] Input Validation: Validate `assignee_id` exists for 'employee' and 'department' types.
- [x] Data Integrity: Prevent overlapping assignments via DB constraints and app logic.
