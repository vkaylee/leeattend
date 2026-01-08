# Implementation Plan: Workspace Management (Multi-Tenancy)

## Approach
Implement a strict SaaS-style multi-tenancy isolation. This ensures that every organizational unit (Workspace) has its own isolated data set. Users are managed centrally but must be members of a workspace to access its agents, devices, and logs. This approach provides the best security and scalability for professional environments.

### Data Migration Strategy
Since the current system has agents and devices without a workspace, the plan includes creating a "Default Workspace" during migration and assigning all existing entities to it to prevent data loss.

## Steps

### Phase 1: Database & Core Models (`apiserver`)

1. **Database Migrations**
   - Create migrations for:
     - `workspaces` (id: UUID, name: VARCHAR, created_at: TIMESTAMPTZ)
     - `users` (id: UUID, email: VARCHAR, password_hash: VARCHAR, created_at: TIMESTAMPTZ)
     - `workspace_members` (workspace_id: UUID, user_id: UUID, role: VARCHAR)
   - Add `workspace_id` (UUID) to `authorized_agents` and `devices`.
   - **Data Migration Script:** Populate a default workspace and update existing agents/devices.

2. **Update Models (`models.rs`)**
   - Define `Workspace`, `User`, `WorkspaceMember` structs.
   - Update `AuthorizedAgent` and `Device` to include `workspace_id`.
   - Create request DTOs for Workspace CRUD and Member management.

### Phase 2: Backend Logic & Isolation (`apiserver`)

1. **Workspace Context Extraction**
   - Implement an Axum middleware or extractor to identify the "Current Workspace" from a request header (e.g., `X-Workspace-Id`) or a JWT claim.

2. **Enforce Isolation in Handlers (`handlers.rs`)**
   - Update all `SELECT`, `INSERT`, `UPDATE`, `DELETE` queries for Agents and Devices to include `WHERE workspace_id = $current_workspace_id`.
   - Update `ws_handler` to ensure agents connect with tokens bound to a specific workspace.

3. **New Workspace & Member Handlers**
   - Implement `GET /workspaces` (list user's workspaces).
   - Implement `POST /workspaces` (create new).
   - Implement `GET /workspaces/:id/members` and `POST /workspaces/:id/members`.

### Phase 3: Frontend Implementation (`webdashboard`)

1. **API Client Updates (`api.ts`)**
   - Update the API client to include the active `workspaceId` in all request headers.
   - Add functions for workspace and member management.

2. **Workspace Switcher Component**
   - Create a dropdown in the Sidebar to list and switch between workspaces.
   - Store the active `workspaceId` in local storage or a global React state (Context API).

3. **Workspace Settings Page**
   - Create a new page for Managing Workspace name and Invite/Remove members.

4. **Navigation Guards**
   - Ensure the user is redirected to a workspace selection page if no active workspace is set.

## Timeline

| Phase | Tasks | Duration |
|-------|-------|----------|
| **1. Database** | Migrations + Models | 4 hours |
| **2. Backend** | Isolation + Context + New Handlers | 8 hours |
| **3. Frontend** | API Client + Switcher UI | 6 hours |
| **4. Settings** | Member Management UI | 4 hours |
| **5. Testing** | Multi-tenant leak tests | 4 hours |
| **Total** | | **26 hours** |

## Rollback Plan
1. **Database:** Execute "Down" migrations for workspaces and members. Remove `workspace_id` columns.
2. **Backend:** Revert handlers to global scope (removing `workspace_id` filters).
3. **Frontend:** Remove workspace switcher and header logic.

## Security Checklist
- [ ] **Cross-tenant Check:** Verify that providing a valid UUID of another workspace in the header doesn't allow access (must check membership).
- [ ] **Role Validation:** Ensure only `Owner` or `Admin` can invite/remove members.
- [ ] **Agent Handshake:** Tokens must be unique system-wide OR scoped per workspace.
- [ ] **Data Cleanup:** When a workspace is deleted, ensure all associated agents and devices are also handled (soft delete or cascade).
