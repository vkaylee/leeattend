/* eslint-disable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access */
import { expect } from '@playwright/test';

import { test } from './worker-orchestrator';

test.describe.skip('Debug Login Flow', () => {
  test('should verify the Debug Login button text', async ({ page, workerWebUrl }) => {
    await page.goto(`${workerWebUrl}/login`);
    const debugButton = page.getByRole('button', { name: /Fast Login/i });
    await expect(debugButton).toBeVisible();
    await expect(debugButton).toBeEnabled();
  });

  test.skip("should successfully login and redirect to dashboard when clicking Debug Login", async ({ page, workerWebUrl }) => {
    // 1. Navigate to login page
    await page.goto(`${workerWebUrl}/login`);

    // 2. Setup listener for login API request to verify payload
    const loginRequestPromise = page.waitForRequest(request => 
      request.url().includes('/api/auth/login') && request.method() === 'POST'
    );

    // 3. Click the debug login button
    await page.getByRole('button', { name: /Fast Login/i }).click();

    // 4. Verify request payload matches known dev credentials
    const request = await loginRequestPromise;
    const postData = request.postDataJSON();
    expect(postData.email).toBe('admin@leeattend.com');
    expect(postData.password).toBe('admin123');

    // 5. Verify successful redirection to Dashboard
    // We expect the URL to end with '/' (root)
    await page.waitForURL(url => url.pathname === '/' || url.pathname === '', { timeout: 10000 });
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveURL(/\/$/);

    // 6. Verify Dashboard content is visible (e.g. Sidebar or Welcome message)
    // Checking for a common element like the sidebar or main heading
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible({ timeout: 15000 });
  });
});
