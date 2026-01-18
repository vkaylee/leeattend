import { test, expect } from './worker-orchestrator';

test.describe('Sidebar Navigation', () => {
  test('should have "Time & Leave" section expanded by default', async ({ page }) => {
    // 1. Go to login page
    await page.goto('/');
    
    // 2. Click Fast Login to bypass auth quickly
    await page.getByTestId('fast-login-btn').click();

    // 3. Wait for dashboard to load
    await expect(page).toHaveURL('/');
    
    // 4. Check if "Time & Leave" section is visible and expanded
    const sectionTitle = page.getByTestId('sidebar-section-time');
    await expect(sectionTitle).toBeVisible();

    // The links inside this section should be visible
    const timeTrackingLink = page.getByTestId('sidebar-link-time-tracking');
    await expect(timeTrackingLink).toBeVisible();
    
    // 5. Verify "Organization" section is NOT expanded by default (max-h-0)
    const membersLink = page.getByTestId('sidebar-link-members');
    await expect(membersLink).not.toBeVisible();
  });

  test('should expand one section and collapse others when clicked', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('fast-login-btn').click();
    await expect(page).toHaveURL('/');

    // Initially "Time & Leave" is expanded, "Organization" is collapsed
    const timeTrackingLink = page.getByTestId('sidebar-link-time-tracking');
    const membersLink = page.getByTestId('sidebar-link-members');
    
    await expect(timeTrackingLink).toBeVisible();
    await expect(membersLink).not.toBeVisible();

    // Click "Organization" section title
    await page.getByTestId('sidebar-section-org').click();

    // Now "Organization" should be expanded, "Time & Leave" should be collapsed
    await expect(membersLink).toBeVisible();
    await expect(timeTrackingLink).not.toBeVisible();
  });
});
