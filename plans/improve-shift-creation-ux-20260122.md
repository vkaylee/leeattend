# Implementation Plan: Improve Shift Creation UX

Automatically close the "Create Shift" modal and switch to the "DIRECTORY" tab after a shift is successfully created.

## Approach
- Update the `onSuccess` callback of `createShiftMutation` in `ShiftsPage.tsx`.
- Call `setIsCreateModalOpen(false)` to close the modal.
- Call `setActiveTab("directory")` to navigate the user to the list of shift types.

## Steps
1. **Core Implementation** (5 min)
   - Modify `webdashboard/src/pages/ShiftsPage.tsx`:
     - Update `createShiftMutation`'s `onSuccess` handler.

2. **Verification** (5 min)
   - Manually verify the UI flow.
   - Run existing frontend tests to ensure no regressions in tab state management.

## Timeline
| Phase | Duration |
|-------|----------|
| Implementation | 5 min |
| Testing | 5 min |
| **Total** | **10 min** |

## Rollback Plan
- Revert changes in `webdashboard/src/pages/ShiftsPage.tsx`.

## Security Checklist
- [x] Input validation (Existing)
- [x] Auth checks (Existing)
- [x] Error handling (Existing)
