# Plan: Approval List UI Refactor

## 1. Overview
The goal is to resolve Technical Debt #3 by extracting the duplicated UI logic from `LeaveRequestsPage` and `ShiftSwapsPage` into a reusable, generic `ApprovalListLayout` component. Both pages share identical UX patterns: a status tab bar, a data table with specific columns (employee, time, status, actions), and approval/rejection actions. 

By extracting this monolithic structure, we will adhere to the DRY principle, making future approval-based pages (e.g., Timesheet Approvals) trivial to implement.

## 2. Project Type
**WEB** (Frontend React SPA)

## 3. Success Criteria
- [x] `ApprovalListLayout` component (or template) is created.
- [x] `LeaveRequestsPage.tsx` is refactored to use the new abstraction and its LOC is reduced significantly.
- [x] `ShiftSwapsPage.tsx` is refactored to use the new abstraction and its LOC is reduced significantly.
- [x] Custom tab requirements are supported (e.g., `ShiftSwapsPage` has `pending_admin` while `LeaveRequestsPage` has `pending`).
- [x] No regression in UI/UX (tabs work, table renders correctly, actions execute mutations successfully).
- [x] Linting and type-checking pass with zero errors.

## 4. Tech Stack
- React (Vite)
- Tailwind CSS
- React Query (for data fetching and mutations)

## 5. File Structure
```text
webdashboard/src/
  components/
    templates/
      ApprovalListLayout.tsx   <-- New generic layout abstraction
  pages/
    LeaveRequestsPage.tsx      <-- Refactored
    ShiftSwapsPage.tsx         <-- Refactored
```

## 6. Task Breakdown

### Task 1: Create `ApprovalListLayout` Generic Component
- **Agent:** `frontend-specialist`
- **Skill:** `clean-code`, `frontend-design`
- **Priority:** P2
- **Dependencies:** None
- **INPUT:** Existing UI logic in `LeaveRequestsPage.tsx` (Tabs, Table Wrapper, Header).
- **OUTPUT:** New generic component `components/templates/ApprovalListLayout.tsx` that accepts generic props (`tabs`, `activeTab`, `onTabChange`, `tableHeaders`, `renderRow`, `isLoading`, `isEmpty`, `headerActions`, etc.).
- **VERIFY:** Component compiles successfully without type errors.

### Task 2: Refactor `LeaveRequestsPage`
- **Agent:** `frontend-specialist`
- **Skill:** `clean-code`
- **Priority:** P2
- **Dependencies:** Task 1
- **INPUT:** New `ApprovalListLayout` and `LeaveRequestsPage.tsx`.
- **OUTPUT:** Refactored `LeaveRequestsPage.tsx` consuming the generic layout and passing specific query hooks and table rows.
- **VERIFY:** Page renders successfully, tabs switch state, and table displays leave requests.

### Task 3: Refactor `ShiftSwapsPage`
- **Agent:** `frontend-specialist`
- **Skill:** `clean-code`
- **Priority:** P2
- **Dependencies:** Task 1
- **INPUT:** New `ApprovalListLayout` and `ShiftSwapsPage.tsx`.
- **OUTPUT:** Refactored `ShiftSwapsPage.tsx` consuming the generic layout. Must correctly map `pending_admin`, `approved`, and `rejected` tabs.
- **VERIFY:** Page renders successfully, tabs switch state, and table displays shift swaps.

### Task 4: Cleanup & Type Safety Verification
- **Agent:** `test-engineer` / `frontend-specialist`
- **Skill:** `clean-code`
- **Priority:** P2
- **Dependencies:** Task 2, Task 3
- **INPUT:** The entire frontend codebase.
- **OUTPUT:** Code is clean, DRY, and well-typed.
- **VERIFY:** Run `./scripts/_npm.sh run typecheck` and `./scripts/_npm.sh run lint`.

## 7. Phase X: Verification
- [x] Lint: `./scripts/_npm.sh run lint`
- [x] Type Check: `./scripts/_npm.sh run typecheck`
- [x] Build: `./scripts/_npm.sh run build`
- [x] Manual Check: Start dev server and navigate to both pages to verify UI renders perfectly.
