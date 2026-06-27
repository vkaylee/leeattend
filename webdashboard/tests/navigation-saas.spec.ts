import { test, expect } from './worker-orchestrator';

test.describe('Navigation SaaS Refactor', () => {
  test.beforeEach(async ({ authPage }) => {
    await authPage.goto();
  });

  test('should not have old sidebar logout button', async ({ page }) => {
    await expect(page.getByTestId('sidebar-btn-logout')).not.toBeVisible();
  });

  test('should have user profile and logout in nav rail', async ({ page }) => {
    // DSM App Shell icon rail shows profile link and logout button directly
    await expect(page.getByTestId('nav-rail-user-avatar')).toBeVisible({ timeout: 15000 });
    await expect(page.getByTestId('nav-rail-logout')).toBeVisible();

    // Profile link navigates to ESS profile
    await page.getByTestId('nav-rail-user-avatar').click();
    await expect(page).toHaveURL(/\/ess\/profile/, { timeout: 15000 });
  });

  test('should have working logout button in nav rail', async ({ page }) => {
    // DSM App Shell: direct logout button in icon rail
    const logoutBtn = page.getByTestId('nav-rail-logout');
    await expect(logoutBtn).toBeVisible({ timeout: 15000 });
    await expect(logoutBtn).toBeEnabled();

    // Verify logout button has aria-label
    await expect(logoutBtn).toHaveAttribute('aria-label', 'Logout');

    // Verify logout button text content indicates logout action
    const ariaLabel = await logoutBtn.getAttribute('aria-label');
    expect(ariaLabel?.toLowerCase()).toContain('logout');
  });
});
