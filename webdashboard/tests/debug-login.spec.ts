import { test, expect } from '@playwright/test';

test.describe('Debug Login Flow', () => {
  test('should verify the Debug Login button text', async ({ page }) => {
    await page.goto('/login');
    const debugButton = page.locator('text=⚡ Đăng nhập nhanh (Dev Mode)');
    await expect(debugButton).toBeVisible();
    await expect(debugButton).toBeEnabled();
  });

  test('should successfully login and redirect to dashboard when clicking Debug Login', async ({ page }) => {
    // 1. Navigate to login page
    await page.goto('/login');

    // 2. Setup listener for login API request to verify payload
    const loginRequestPromise = page.waitForRequest(request => 
      request.url().includes('/api/auth/login') && request.method() === 'POST'
    );

    // 3. Click the debug login button
    await page.click('text=⚡ Đăng nhập nhanh (Dev Mode)');

    // 4. Verify request payload matches known dev credentials
    const request = await loginRequestPromise;
    const postData = request.postDataJSON();
    expect(postData.email).toBe('admin@leeattend.com');
    expect(postData.password).toBe('admin123');

    // 5. Verify successful redirection to Dashboard
    // We expect the URL to end with '/' (root)
    await expect(page).toHaveURL(/\/$/);

    // 6. Verify Dashboard content is visible (e.g. Sidebar or Welcome message)
    // Checking for a common element like the sidebar or main heading
    await expect(page.locator('text=LeeAttend')).toBeVisible();
  });
});
