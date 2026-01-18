# Implementation Plan: Migrate to Hybrid Locators with `data-testid`

### ## Approach
- **Why this solution:** Decouples E2E tests from the UI copy (translations). Using `data-testid` for translated or complex components ensures tests don't break when text changes in Vietnamese, English, or Chinese. Standard attributes (`name`, `type`, `placeholder`) and semantic roles are still used where they are stable and descriptive.
- **Alternatives considered:** 
    - **Current Regex Approach:** Too brittle and high-maintenance for multi-language support.
    - **CSS Classes:** Rejected due to Tailwind's utility-first nature where classes change frequently.

### ## Steps

1. **Component Tagging** (30 min)
   Add `data-testid` to key interactive elements in the React components.
   - `webdashboard/src/pages/Auth/LoginPage.tsx`: Add `data-testid="login-submit"`.
   - `webdashboard/src/components/Layout/Sidebar.tsx`: Add `data-testid` to section buttons and navigation links (e.g., `sidebar-section-time`, `sidebar-link-attendance`).
   - `webdashboard/src/components/Common/Button.tsx`: (If applicable) Ensure it forwards `data-testid` to the underlying button.

2. **Test Migration (Auth & Navigation)** (30 min)
   Update core test files to use `getByTestId`.
   - `webdashboard/tests/login-success.spec.ts`
   - `webdashboard/tests/sidebar-navigation.spec.ts`
   - `webdashboard/tests/worker-orchestrator.ts` (if it contains helpers like `login`)

3. **Pattern Standardization** (15 min)
   Establish the hierarchy for selecting elements:
   1. `getByRole('button', { name: 'submit' })` (Standard/Stable roles)
   2. `page.locator('input[name="email"]')` (Semantic attributes)
   3. `page.getByTestId('...')` (Translated/Custom UI)
   4. **Avoid** `getByText(/Regex|Regex/i)`

4. **Verification** (15 min)
   - Run tests to ensure no regressions.
   - Verify that tests pass regardless of the active language (if language switching is enabled).

### ## Timeline
| Phase | Duration |
|-------|----------|
| Component Tagging | 30 min |
| Test Migration | 30 min |
| Standardization | 15 min |
| Verification | 15 min |
| **Total** | **1.5 hours** |

### ## Rollback Plan
- Use `git checkout src/ tests/` to revert changes if the new IDs conflict with existing styles or if the test runner fails to recognize `data-testid`.

### ## Security Checklist
- [x] Ensure `data-testid` values are generic and don't leak internal state or data.
- [x] Verify that production builds don't strip these attributes if they are needed for automated production monitoring (standard practice is to keep them or use a transform to remove them if bundle size is critical).
