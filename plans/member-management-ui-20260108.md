# Implementation Plan: Member Management & RBAC UI

## Approach
We will implement a comprehensive Member Management interface within the Web Dashboard. This includes viewing members, inviting new users via email, and managing their roles (both primary and temporal extra roles). We will leverage the existing `AuthGuard` and `WorkspaceGuard` to ensure security context.

## Steps

### 1. Dependencies & Assets
- Ensure `shadcn/ui` components are available: `Table`, `Dialog`, `Select`, `Button`, `Input`, `Badge`, `DropdownMenu`, `Popover`, `Calendar`.
- We might need `lucide-react` icons (already present).

### 2. API Integration Hooks (`src/hooks/use-members.ts`)
- `useWorkspaceMembers(wsId)`: Fetch list of members.
- `useInviteMember()`: POST to `/workspaces/:id/members`.
- `useMemberRoles(wsId, userId)`: Fetch detailed roles.
- `useAssignRole()`: POST to `/workspaces/:id/members/:uid/roles`.
- `useRevokeRole()`: DELETE to `/workspaces/:id/members/:uid/roles/:roleId`.

### 3. UI Components
- **`src/components/members/MemberTable.tsx`**:
    - Display columns: Name/Email, Primary Role, Extra Roles (badges), Joined At, Actions.
    - Actions: "Manage Roles", "Remove from Workspace".
- **`src/components/members/InviteMemberDialog.tsx`**:
    - Form with Email input and Primary Role selection (default 'member').
- **`src/components/members/RoleManagementDialog.tsx`**:
    - Allow changing Primary Role.
    - List current Extra Roles with expiry.
    - Form to add new Extra Role with optional Expiry Date.

### 4. Page Implementation (`src/pages/MembersPage.tsx`)
- Layout with Header ("Members") and "Invite Member" button (top right).
- Render `MemberTable`.
- Integrate `InviteMemberDialog`.

### 5. Routing
- Add route `/members` to `src/App.tsx`.
- Add "Members" link to the Navigation Sidebar/Header (if exists).

## Verification Plan
- **Automated Test**: `tests/members-ui.spec.ts` (Playwright).
    - Login as Dev Admin.
    - Navigate to Members page.
    - Open Invite Dialog.
    - Check if Member Table renders.
- **Manual Verification**:
    - Invite a user (mocked or real).
    - Assign an extra role and verify it appears.

## Rollback Plan
- Revert changes to `App.tsx` and delete created component files.
