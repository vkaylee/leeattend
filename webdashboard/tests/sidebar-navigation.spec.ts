import { test, expect } from './worker-orchestrator';

test.describe.skip('Sidebar Navigation', () => {
  test('smoke: time section expanded by default', async ({ page }) => {
    // 1. Go to login page directly
    await page.goto('/login');
    
    // 2. Wait for login page to be ready
    const fastLoginButton = page.getByTestId('fast-login-btn');
    await expect(fastLoginButton).toBeVisible({ timeout: 30000 });
    
    // 3. Click Fast Login
    await fastLoginButton.click();

    // 4. Wait for dashboard to load
    await expect(page.getByTestId('dashboard-title')).toBeVisible({ timeout: 30000 });
    await expect(page).toHaveURL('/');
    
    // 5. Check if "Time & Leave" section is visible and expanded
    const sectionTitle = page.getByTestId('sidebar-section-time');
    await expect(sectionTitle).toBeVisible();

    // The links inside this section should be visible
    const timeTrackingLink = page.getByTestId('sidebar-link-attendance-logs');
    await expect(timeTrackingLink).toBeVisible();
    
    // 6. Verify "Organization" section is NOT expanded by default (max-h-0)
    const employeesLink = page.getByTestId('sidebar-link-employees');
    await expect(employeesLink).not.toBeVisible();
  });

  test('should expand one section and collapse others when clicked', async ({ page }) => {
    await page.goto('/login');
    
    // Wait for login page to be ready
    const fastLoginButton = page.getByTestId('fast-login-btn');
    await expect(fastLoginButton).toBeVisible({ timeout: 30000 });
    
    await fastLoginButton.click();
    
    // Wait for dashboard to load properly
    await expect(page.getByTestId('dashboard-title')).toBeVisible({ timeout: 30000 });
    await expect(page).toHaveURL('/');

    // Initially "Time & Leave" is expanded, "Organization" is collapsed
    const timeTrackingLink = page.getByTestId('sidebar-link-attendance-logs');
    const employeesLink = page.getByTestId('sidebar-link-employees');
    
    await expect(timeTrackingLink).toBeVisible();
    await expect(employeesLink).not.toBeVisible();

    // Click "Organization" section title
    await page.getByTestId('sidebar-section-org').click();

    // Now "Organization" should be expanded, "Time & Leave" should be collapsed
    await expect(employeesLink).toBeVisible();
    await expect(timeTrackingLink).not.toBeVisible();
  });
});
