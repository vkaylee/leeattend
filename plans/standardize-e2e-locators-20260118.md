# Implementation Plan: Standardize E2E Locators

### ## Approach
- **Why this solution:** Standardizing on `getByRole` with localized regex ensures tests are resilient to UI copy changes and styling refactors (Tailwind) while natively supporting our multi-language requirement (vi, en, zh).
- **Alternatives considered:** 
    - `data-testid`: Good for complex components, but doesn't validate if the user can actually "see" the button.
    - CSS Classes: Rejected as too brittle for Tailwind.

### ## Steps

1. **Audit & Identification** (10 min)
   Scan all `.spec.ts` files for legacy `locator()`, `getByText()`, or class-based selectors.

2. **Core Implementation** (30 min)
   Update locators to use the hybrid Role + Regex pattern.
   
   **Pattern Mapping:**
   | Element | Legacy | Standardized |
   |---------|--------|--------------|
   | **Login Button** | `getByText('Đăng nhập')` | `getByRole('button', { name: /Login|Đăng nhập|登录/ })` |
   | **Sidebar Item** | `locator('.sidebar-item')` | `getByRole('link', { name: /Employees|Nhân viên|员工/ })` |
   | **Save Button** | `getByText('Lưu')` | `getByRole('button', { name: /Save|Lưu|保存/ })` |

3. **Integration & Helper Refactor** (15 min)
   Update the `login` helper and common navigation flows in `tests/utils.ts` (if applicable) to use the new standard.

4. **Verification (The "Locale Sweep")** (20 min)
   Run tests in at least two different languages to ensure regex matching works.

### ## Rollback Plan
- Use `git checkout webdashboard/tests/` to revert specific test files if regex matching causes unexpected false negatives.

### ## Security Checklist
- [x] No sensitive data in locators.
- [x] Ensure locators don't select `aria-hidden` elements.
