# Implementation Plan: Auth & Workspace Navigation Guards

Implement a robust navigation flow that enforces user authentication and workspace existence before allowing access to the application dashboard.

## Approach
- **AuthGuard**: A functional component that checks for `userId` in `localStorage`. Redirects to `/login` if missing.
- **WorkspaceGuard**: A component that fetches the user's workspaces using `useWorkspaces`. Redirects to `/onboarding` if the list is empty.
- **Routing**: Update `App.tsx` to use a nested route structure with these guards as wrappers.

## Steps

### 1. Create New Pages (15 min)
- **LoginPage**: A simple page for authentication (moving the Dev Fast Login logic here).
- **OnboardingPage**: A page prompting the user to create their first workspace.
- Files:
  - `webdashboard/src/pages/LoginPage.tsx`
  - `webdashboard/src/pages/OnboardingPage.tsx`

### 2. Implement Guard Components (15 min)
- Create `webdashboard/src/components/AuthGuard.tsx`
- Create `webdashboard/src/components/WorkspaceGuard.tsx`

### 3. Update Routing Logic (20 min)
- Modify `webdashboard/src/App.tsx` to include new routes and wrap protected content.
- Structure:
```tsx
<Routes>
  <Route path="/login" element={<LoginPage />} />
  <Route element={<AuthGuard />}>
    <Route path="/onboarding" element={<OnboardingPage />} />
    <Route element={<WorkspaceGuard />}>
       <Route path="/" element={<Layout />}>...</Route>
    </Route>
  </Route>
</Routes>
```

### 4. Refining Layout & Switcher (10 min)
- Ensure `WorkspaceSwitcher` doesn't break if no workspace is selected yet (though `WorkspaceGuard` should prevent this).
- Automatically set the first workspace as active if `active_workspace_id` is null but workspaces exist.

## Timeline
| Phase | Duration |
|-------|----------|
| Page Creation | 15 min |
| Guards Implementation | 15 min |
| Routing Integration | 20 min |
| **Total** | **50 min** |

## Rollback Plan
- Revert `App.tsx` to the previous flat route structure.
- Keep created files (they won't affect the app if not imported).

## Security Checklist
- [x] Client-side Auth checks (localStorage).
- [x] Server-side Workspace validation (React Query).
- [x] Handle loading states to prevent unauthorized "flickers".
