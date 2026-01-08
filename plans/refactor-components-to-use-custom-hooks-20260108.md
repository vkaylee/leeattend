# Implementation Plan: Refactor to Custom Hooks with React Query

## Approach
- Migrate from manual `useEffect` + `useState` data fetching to `@tanstack/react-query`.
- Create domain-specific hooks (`useAgents`, `useDevices`, `useWorkspaces`, `useProfile`) to centralize API logic.
- **Why**: 
    - Automatic caching and background refetching.
    - Simplified loading/error state management.
    - Better DX with devtools and consistent patterns.
    - Easy cache invalidation after mutations (e.g., refreshing list after adding an agent).

## Steps

1. **Setup QueryClient** (10 min)
   - Wrap `App.tsx` with `QueryClientProvider`.
   - Configure default options (e.g., `retry: 1`, `staleTime: 5min`).

2. **Implement Base Hooks** (30 min)
   - `src/hooks/use-agents.ts`:
     - `useAgentsQuery()`
     - `useCreateAgentMutation()`
     - `useUpdateAgentTrustMutation()`
     - `useDeleteAgentMutation()`
   - `src/hooks/use-devices.ts`:
     - `useDevicesQuery()`
     - `useCreateDeviceMutation()`
     - `useDeleteDeviceMutation()`

3. **Refactor Components** (40 min)
   - Replace `loadData` and local states in `AgentsPage.tsx` with `useAgentsQuery()` and mutation hooks.
   - Repeat for `DevicesPage.tsx`, `DashboardPage.tsx`, etc.

4. **Integration & Cleanup** (10 min)
   - Remove unused states and `useEffect` blocks.
   - Verify cache invalidation triggers (using `queryClient.invalidateQueries`).

## Timeline
| Phase | Duration |
|-------|----------|
| Setup | 10 min |
| Hooks Creation | 30 min |
| Refactoring UI | 40 min |
| Testing | 10 min |
| **Total** | **1.5 hours** |

## Rollback Plan
- Revert components to previous `useEffect` implementations.
- `git checkout src/pages/`

## Security Checklist
- [x] Headers (X-Workspace-Id, X-User-Id) are already handled in `lib/api.ts`.
- [ ] Ensure error messages from API are correctly displayed in UI via `mutation.error`.
