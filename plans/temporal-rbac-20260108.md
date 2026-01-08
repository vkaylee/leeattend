# Implementation Plan: Temporal RBAC (Multi-Role with Expiry)

## Approach
Implement a "Primary + Secondary" role model. The primary role (permanent) remains in the `workspace_members` table. A new table `member_extra_roles` will store secondary roles with an `expires_at` timestamp. The `MemberContext` extractor will be refactored to fetch both and merge them into an active "Role Set" for permission checks.

## Steps

### Phase 1: Database & Core Models (`apiserver`)

1. **Database Migration**
   - Create table `member_extra_roles`:
     - `id`: UUID (Primary Key)
     - `workspace_id`: UUID (FK)
     - `user_id`: UUID (FK)
     - `role`: VARCHAR(50)
     - `assigned_at`: TIMESTAMPTZ (Default NOW)
     - `expires_at`: TIMESTAMPTZ (NOT NULL)
   - **File:** `apiserver/migrations/[timestamp]_create_member_extra_roles.sql`

2. **Update Models (`models.rs`)**
   - Define `MemberExtraRole` struct.
   - Define `AssignRoleRequest` DTO (role, expires_at).
   - Update `MemberContext` to use `roles: Vec<String>` instead of a single `role`.

### Phase 2: Refactor Security Context (`apiserver`)

1. **Refactor `MemberContext` Extractor**
   - Update `from_request_parts` in `handlers.rs`:
     - Fetch primary role from `workspace_members`.
     - Fetch active secondary roles from `member_extra_roles` where `expires_at > NOW()`.
     - Combine both into `Vec<String>`.
   - Add helper methods to `MemberContext` like `.has_role(role_name)` or `.is_admin()`.

2. **Temporal Roles Logic**
   - Create a helper to verify if any of the active roles satisfy a required permission level.

### Phase 3: Role Management API (`apiserver`)

1. **New Handlers in `handlers.rs`**
   - `GET /workspaces/:id/members/:user_id/roles`: List all roles (primary + extra).
   - `POST /workspaces/:id/members/:user_id/roles`: Assign a temporary extra role.
   - `DELETE /workspaces/:id/members/:user_id/roles/:role_id`: Manually revoke an extra role.

2. **Register Routes in `lib.rs`**

### Phase 4: Frontend Implementation (`webdashboard`)

1. **API Client Updates (`api.ts`)**
   - Add functions: `fetchMemberRoles`, `assignExtraRole`, `revokeExtraRole`.

2. **UI: Workspace Settings Upgrades**
   - In the member list, add a "Manage Roles" action.
   - Create a sub-component or Modal to:
     - Show the primary role.
     - List temporary roles with a "Time Remaining" countdown.
     - Form to add a new temporary role with a date/time picker.

## Timeline

| Phase | Tasks | Duration |
|-------|-------|----------|
| **1. Database** | Migration + Models | 3 hours |
| **2. Backend Logic**| Refactor Extractor + Role Merge | 5 hours |
| **3. API Layer** | CRUD for Extra Roles | 4 hours |
| **4. Frontend UI** | Role Management Interface | 6 hours |
| **5. Testing** | Expiry & Multi-role tests | 4 hours |
| **Total** | | **22 hours** |

## Rollback Plan
1. **Database:** Execute "Down" migration for `member_extra_roles`.
2. **Backend:** Revert `MemberContext` to fetch only from `workspace_members`.
3. **Frontend:** Revert changes in Member Management UI.

## Security Checklist
- [ ] **Expiry Enforcement:** Ensure the DB query strictly filters by `expires_at > NOW()` to avoid "stale" permissions.
- [ ] **Role Validation:** Only `Owner` or `Admin` can assign extra roles.
- [ ] **No Self-Promotion:** Prevent a user from assigning themselves a higher-level extra role.
- [ ] **Input Validation:** Ensure `expires_at` is always in the future.
