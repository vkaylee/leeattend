# ADR 0001: Navigation Refactor - SaaS Pattern Migration

## Status
Accepted

## Context
The legacy navigation architecture placed session management (Profile, Settings, Logout) in the Sidebar. This deviated from modern SaaS design patterns where the Sidebar is reserved for primary content navigation and the Header (Navbar) manages user-specific context and actions.

## Decision
We moved the `Logout`, `Profile`, and `Settings` actions from the `Sidebar` to a new `UserProfileMenu` component located in the `Navbar`.

### Key Changes:
- Removed `Logout` button from `sidebar.tsx`.
- Created `UserProfileMenu` component in `navbar.tsx`.
- Updated `navbar.tsx` to include `ThemeToggle`, `LanguageSwitcher`, and `UserProfileMenu`.
- Updated Playwright Page Object Model (POM) to include `Navbar` actions.
- Added a new E2E test suite `navigation-saas.spec.ts` to verify the new navigation flow.

## Consequences
- **Positive:** Improved UI/UX consistency, adhering to SaaS best practices.
- **Positive:** Dedicated space for user actions in the Header, freeing up Sidebar space.
- **Negative:** Required updates to E2E tests and POMs that relied on Sidebar-based session management.
- **Neutral:** Increased complexity in `Navbar` component (mitigated by extracting sub-components).

## Verification
- Unit tests for `webdashboard` passed.
- Playwright E2E tests (`navigation-saas.spec.ts`) passed.
- Manual verification of UI aesthetics and responsiveness.
