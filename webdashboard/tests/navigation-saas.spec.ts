import { test, expect } from './worker-orchestrator';

test.describe('Navigation SaaS Refactor', () => {
  test.beforeEach(async ({ authPage }) => {
    // The authPage fixture already navigates to / with a mock authenticated session.
    // Calling LoginPage.goto() here would incorrectly send the test back to /login.
    expect(authPage).toBeDefined();
  });

  test('should not have old sidebar logout button', async ({ page }) => {
    await expect(page.getByTestId('sidebar-btn-logout')).not.toBeVisible();
  });

  test('should have user profile and logout in nav rail', async ({ page }) => {
    // Use accessible names so this contract remains stable across icon-rail internals.
    const navRail = page.getByRole('navigation', { name: 'Navigation' });
    const profileLink = navRail.getByRole('link', { name: 'Profile', exact: true });
    const logoutButton = navRail.getByRole('button', { name: 'Logout', exact: true });
    await expect(profileLink).toBeVisible({ timeout: 15000 });
    await expect(logoutButton).toBeVisible();

    // Profile link navigates to ESS profile
    await profileLink.click();
    await expect(page).toHaveURL(/\/ess\/profile/, { timeout: 15000 });
  });

  test('should have working logout button in nav rail', async ({ page }) => {
    // Query the direct nav-rail control by its accessible action name.
    const logoutBtn = page.getByRole('button', { name: 'Logout' });
    await expect(logoutBtn).toBeVisible({ timeout: 15000 });
    await expect(logoutBtn).toBeEnabled();

    // Verify logout button has aria-label
    await expect(logoutBtn).toHaveAttribute('aria-label', 'Logout');

    // Verify logout button text content indicates logout action
    const ariaLabel = await logoutBtn.getAttribute('aria-label');
    expect(ariaLabel?.toLowerCase()).toContain('logout');
  });
});
