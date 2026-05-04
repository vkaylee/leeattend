import { test, expect } from './worker-orchestrator';

test.describe('Navigation SaaS Refactor', () => {
  test.beforeEach(async ({ authPage }) => {
    await authPage.goto();
  });

  test('should not have logout button in sidebar', async ({ page }) => {
    await expect(page.getByTestId('sidebar-btn-logout')).not.toBeVisible();
  });

  test('should have user profile menu in navbar', async ({ navbar, page }) => {
    await expect(page.getByTestId('navbar-user-menu-trigger')).toBeVisible();
    
    // Open menu
    await page.getByTestId('navbar-user-menu-trigger').click();
    
    // Verify menu items
    await expect(page.getByTestId('user-menu-profile-btn')).toBeVisible();
    await expect(page.getByTestId('user-menu-settings-btn')).toBeVisible();
    await expect(page.getByTestId('user-menu-logout-btn')).toBeVisible();
  });

  test('should be able to logout from navbar', async ({ navbar, page }) => {
    await navbar.logout();
    await expect(page).toHaveURL(/\/login/);
  });
});
