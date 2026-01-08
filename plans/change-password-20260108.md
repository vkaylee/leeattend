# Implementation Plan: Change Password Feature

## Goal
Enhance security by allowing users to change their passwords from their profile settings.

## Backend Implementation
1.  **Models (`apiserver/src/models.rs`)**:
    - Add `ChangePasswordRequest` struct: `old_password`, `new_password`.
2.  **Handlers (`apiserver/src/handlers.rs`)**:
    - Implement `change_password` function.
    - Requires `UserId` extractor (protected route).
    - Fetch current `password_hash` from DB.
    - Verify `old_password`.
    - Hash `new_password`.
    - Update `password_hash` in `users` table.
    - Revoke existing sessions in `sessions` table (optional but recommended).
3.  **Routing (`apiserver/src/lib.rs`)**:
    - Add `.route("/auth/change-password", post(handlers::change_password))` to the authenticated router.

## Frontend Implementation
1.  **API (`webdashboard/src/lib/api.ts`)**:
    - Add `changePassword(data: ChangePasswordRequest)` function.
2.  **UI (`webdashboard/src/pages/ProfilePage.tsx`)**:
    - Add a "Security" section.
    - Form: Current Password, New Password, Confirm New Password.
    - Success/Error notifications.

## Testing
- **E2E Test (`webdashboard/tests/change-password.spec.ts`)**:
    - Login as test user.
    - Go to Profile.
    - Change password.
    - Logout.
    - Try login with OLD password (should fail).
    - Try login with NEW password (should succeed).
