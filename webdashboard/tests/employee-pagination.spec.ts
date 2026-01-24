import { test } from './worker-orchestrator';
import { expect } from '@playwright/test';

test.describe('Employees Pagination UI', () => {
  test('should show pagination controls at top and bottom of employees list', async ({ page }) => {
    // 1. Login
    await page.goto('/login');
    await page.getByTestId('fast-login-btn').click();
    await expect(page.getByRole('heading', { level: 1, name: /Dashboard|Bảng điều khiển/i })).toBeVisible({ timeout: 30000 });

    // 2. Expand Sidebar Section if needed
    const employeesLink = page.getByTestId('sidebar-link-employees');
    const orgSection = page.getByTestId('sidebar-section-org');
    
    if (!await employeesLink.isVisible()) {
      await orgSection.click();
      await expect(employeesLink).toBeVisible({ timeout: 10000 });
    }

    // 3. Click Employees link
    await employeesLink.click();
    await expect(page).toHaveURL(/\/employees/i, { timeout: 30000 });

    // 4. Verify top pagination is visible
    const paginationControls = page.locator('nav[aria-label="Pagination"]');
    await expect(paginationControls.first()).toBeVisible();

    // 5. Verify bottom pagination is visible
    await expect(paginationControls.last()).toBeVisible();

    // 6. Verify Search resetting page (smoke test)
    await page.getByTestId('employee-search').fill('TestSearchTerm');
    await expect(paginationControls.first()).toBeVisible();
  });
});