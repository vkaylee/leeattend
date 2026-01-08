# Implementation Plan: User Registration

## Goal
Allow new users to sign up for an account. This enables the "Invite Member" flow to work smoothly (Admin tells user to sign up, then invites them) and allows general onboarding.

## Backend Implementation
1.  **Models (`apiserver/src/models.rs`)**:
    - Add `RegisterRequest` struct.
2.  **Handlers (`apiserver/src/handlers.rs`)**:
    - Implement `register` function.
    - Check if email exists (return 409 Conflict).
    - Hash password using `AuthService`.
    - Insert new user into `users` table.
    - Generate Access/Refresh tokens.
    - Return `AuthResponse`.
3.  **Routing (`apiserver/src/lib.rs`)**:
    - Add `.route("/auth/register", post(handlers::register))` to the open routes (non-authenticated).

## Frontend Implementation
1.  **API (`webdashboard/src/lib/api.ts`)**:
    - Add `register(data: RegisterRequest)` function.
2.  **UI (`webdashboard/src/pages/RegisterPage.tsx`)**:
    - Create a modern registration form (matching Login style).
    - Fields: Full Name, Email, Password, Confirm Password.
    - Client-side validation (matching passwords).
3.  **Navigation (`webdashboard/src/App.tsx` & `LoginPage.tsx`)**:
    - Add `/register` route.
    - Add "Create an account" link on Login page.
    - Add "Already have an account?" link on Register page.

## Testing
- **E2E Test (`webdashboard/tests/auth-flow.spec.ts`)**:
    - Navigate to Register.
    - Create account.
    - Verify redirection to Dashboard (or Onboarding).
    - Logout.
    - Login with new credentials.

## Security
- Password hashing is already handled by `AuthService` (Bcrypt).
- Input validation (Email format) should be checked.
