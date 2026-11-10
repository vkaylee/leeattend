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

  test('should expose profile from top navigation', async ({ page }) => {
    const topbar = page.getByRole('banner', { name: 'Top Navigation' });
    const profileLink = topbar.getByRole('link', { name: 'Profile', exact: true });
    await expect(profileLink).toBeVisible({ timeout: 15000 });

    await profileLink.click();
    await expect(page).toHaveURL(/\/ess\/profile/, { timeout: 15000 });
  });

  test('should not render logout in the workspace shell', async ({ page }) => {
    await expect(page.getByRole('button', { name: 'Logout', exact: true })).toHaveCount(0);
  });
});
