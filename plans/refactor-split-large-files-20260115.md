# Implementation Plan: Refactor Split Large Files

**Goal:** Decompose monolithic files (`handlers.rs`, `api.ts`, `ShiftsPage.tsx`) into maintainable, domain-specific modules.

## Approach
**Functional Splitting**:
- **Backend:** Break `handlers.rs` into a `handlers/` directory with modules corresponding to business domains (Auth, Workspace, Employees, etc.). Common extractors go to `common.rs`.
- **Frontend API:** Break `api.ts` into a `lib/api/` directory. Create a `types.ts` for interfaces and separate files for API functions. Use an `index.ts` barrel file to maintain backward compatibility for imports.
- **Frontend UI:** Extract major sub-components from `ShiftsPage.tsx` into `src/components/shifts/` to reduce complexity and improve readability.

## Steps

### Phase 1: Backend (`apiserver/src/handlers.rs`)

1.  **Preparation**
    -   Create directory: `apiserver/src/handlers/`.
    -   Create `apiserver/src/handlers/mod.rs` to declare sub-modules.

2.  **Extract Common Utilities**
    -   Create `apiserver/src/handlers/common.rs`.
    -   Move `WorkspaceId` and `MemberContext` extractors there.

3.  **Extract Domain Handlers**
    -   Create `apiserver/src/handlers/auth.rs` (Login, Register, etc.).
    -   Create `apiserver/src/handlers/workspace.rs` (CRUD, Settings).
    -   Create `apiserver/src/handlers/members.rs` (Roles, Invitations).
    -   Create `apiserver/src/handlers/employees.rs` (CRUD, Profiles).
    -   Create `apiserver/src/handlers/leaves.rs` (Balances, Requests).
    -   Create `apiserver/src/handlers/agents.rs` (Agents, Devices).
    -   Create `apiserver/src/handlers/attendance.rs` (Logs, Sync).

4.  **Integration**
    -   Update `apiserver/src/lib.rs` (or `main.rs`) to point to the new module structure.
    -   Verify all imports in the new files.

### Phase 2: Frontend API (`webdashboard/src/lib/api.ts`)

1.  **Preparation**
    -   Create directory: `webdashboard/src/lib/api/`.

2.  **Extract Core**
    -   Create `webdashboard/src/lib/api/config.ts` (API_BASE_URL logic).
    -   Create `webdashboard/src/lib/api/types.ts` (Move all interfaces here).

3.  **Extract Domain Functions**
    -   Create `webdashboard/src/lib/api/auth.ts`.
    -   Create `webdashboard/src/lib/api/workspaces.ts`.
    -   Create `webdashboard/src/lib/api/employees.ts`.
    -   Create `webdashboard/src/lib/api/leaves.ts`.
    -   Create `webdashboard/src/lib/api/attendance.ts`.
    -   Create `webdashboard/src/lib/api/agents.ts`.

4.  **Backward Compatibility**
    -   Create `webdashboard/src/lib/api/index.ts`.
    -   Re-export everything: `export * from './types'; export * from './auth'; ...`
    -   This ensures we don't have to update 100+ files immediately.

### Phase 3: Frontend UI (`webdashboard/src/pages/ShiftsPage.tsx`)

1.  **Analysis**
    -   Identify distinct UI sections: Calendar View, Shift List, Edit Modal.

2.  **Extraction**
    -   Create directory: `webdashboard/src/components/shifts/`.
    -   Extract `ShiftCalendar.tsx`.
    -   Extract `ShiftFormModal.tsx`.
    -   Extract `ShiftFilterBar.tsx`.

3.  **Refactor Page**
    -   Update `ShiftsPage.tsx` to import and compose these new components.

## Verification
-   **Backend:** Run `cargo check` and `cargo test` to ensure no broken modules.
-   **Frontend:** Run `npm run build` (or `tsc`) to verify types and imports.
-   **Runtime:** Start the app and verify critical paths (Login, Leave Page, Shifts Page).

## Timeline
| Phase | Duration |
|-------|----------|
| Backend Split | 45 min |
| API Split | 30 min |
| UI Split | 45 min |
| **Total** | **2 hours** |
