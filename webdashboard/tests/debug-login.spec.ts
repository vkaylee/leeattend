import { expect } from '@playwright/test';
import { test } from './worker-orchestrator';

test.describe('Debug Login Flow', () => {
  test('should verify the Debug Login button text', async ({ page, workerWebUrl }) => {
    await page.goto(`${workerWebUrl}/login`);
    const debugButton = page.locator('text=⚡ Fast Login');
    await expect(debugButton).toBeVisible();
    await expect(debugButton).toBeEnabled();
  });

  test("should successfully login and redirect to dashboard when clicking Debug Login", async ({ page, workerWebUrl }) => {
    // 1. Navigate to login page
    await page.goto(`${workerWebUrl}/login`);

    // 2. Setup listener for login API request to verify payload
    const loginRequestPromise = page.waitForRequest(request => 
      request.url().includes('/api/auth/login') && request.method() === 'POST'
    );

    // 3. Click the debug login button
    await page.click('text=⚡ Fast Login');

    // 4. Verify request payload matches known dev credentials
    const request = await loginRequestPromise;
    const postData = request.postDataJSON();
    expect(postData.email).toBe('admin@leeattend.com');
    expect(postData.password).toBe('admin123');

    // 5. Verify successful redirection to Dashboard
    // We expect the URL to end with '/' (root)
    await page.waitForURL(/\/$/, { timeout: 15000 });
    await expect(page).toHaveURL(/\/$/);

    // 6. Verify Dashboard content is visible (e.g. Sidebar or Welcome message)
    // Checking for a common element like the sidebar or main heading
    await expect(page.locator('text=LeeAttend')).toBeVisible();
  });
});
