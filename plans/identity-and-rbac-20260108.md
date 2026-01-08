# Implementation Plan: Global Identity & Secure RBAC

## Approach
This plan shifts user identity to a truly global level while enforcing strict membership and role checks within each workspace. We will introduce a `MemberContext` extractor that verifies not just the workspace ID, but also the specific user's permissions within that workspace.

### Key Concepts:
- **Global User:** Identity (Email, Profile) is unique across the whole system.
- **Workspace Member:** A link between a User and a Workspace with a specific `role`.
- **RBAC Extractor:** A reusable Rust extractor to guard routes based on roles (Owner, Admin, Member).

## Steps

### Phase 1: Global User Profile (`apiserver`)

1. **Migration: User Enrichment**
   - Add profile fields: `full_name`, `avatar_url`, `bio`.
   - **File:** `apiserver/migrations/[timestamp]_enrich_users_table.sql`

2. **User Models & Handlers**
   - Implement `GET /me` to return the current user's global profile.
   - Implement `PATCH /me` to update profile details.
   - **Files:** `models.rs`, `handlers.rs`.

### Phase 2: RBAC & Security Context (`apiserver`)

1. **Authentication Emulation (Dev Mode)**
   - Since full JWT is pending, implement an `X-User-Id` header check to identify the requester.
   
2. **MemberContext Extractor**
   - Create a new extractor that:
     - Gets `user_id` from `X-User-Id`.
     - Gets `workspace_id` from `X-Workspace-Id`.
     - Queries `workspace_members` to verify the user belongs to that workspace.
     - Rejects with `403 Forbidden` if not a member.
   - **File:** `handlers.rs`.

3. **Role Validation Logic**
   - Create a helper function/trait to check if `MemberContext.role` satisfies a requirement (e.g., `is_admin()`).

### Phase 3: Secure Invitation System (`apiserver`)

1. **Invite Member Logic (The "Real" Way)**
   - Update `POST /workspaces/:id/members`:
     - **Verify Requester:** Must be `Owner` or `Admin` of the target workspace.
     - **Lookup User:** Search for existing user by email.
     - **Action:** If user exists, add them to `workspace_members`. If not, return `404 User Not Found` (In a later phase, we would send an email invitation).
   - **File:** `handlers.rs`.

### Phase 4: Frontend Integration (`webdashboard`)

1. **User Profile UI**
   - Add a "Profile" settings section.
   - Show user avatar/name in the Sidebar.

2. **Member Management Upgrades**
   - Update the "Invite" form to handle errors (e.g., "User not found").
   - Display member roles and prevent non-admins from seeing management actions.

## Timeline

| Phase | Tasks | Duration |
|-------|-------|----------|
| **1. Identity** | Global Profile Migration + API | 4 hours |
| **2. RBAC** | MemberContext Extractor + Guarding routes | 6 hours |
| **3. Invites** | Secure Invitation Logic + Search | 4 hours |
| **4. Frontend**| Profile UI + Secure UI States | 6 hours |
| **Total** | | **20 hours** |

## Rollback Plan
1. **Revert Migrations:** Remove new profile columns.
2. **Revert Handlers:** Remove `MemberContext` requirement from routes.
3. **Frontend:** Remove profile section.

## Security Checklist
- [ ] **Identity Spoofing:** Ensure `X-User-Id` cannot be easily guessed (eventually replace with signed JWT).
- [ ] **Privilege Escalation:** Verify a `Member` cannot promote themselves to `Admin`.
- [ ] **Data Leakage:** Ensure `GET /workspaces` only returns workspaces where the user is a member.
- [ ] **Profile Privacy:** Ensure User A cannot `PATCH` the profile of User B.
