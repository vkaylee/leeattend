import { expect } from '@playwright/test';
import { test } from './worker-orchestrator';

test.describe('Login Success Flow', () => {
  test('should successfully login with standard form and redirect to home', async ({ page, workerWebUrl }) => {
    // 1. Navigate to login page
    await page.goto(`${workerWebUrl}/login`);

    // 2. Fill in the login form
    await page.fill('input[name="email"]', 'admin@leeattend.com');
    await page.fill('input[name="password"]', 'admin123');

    // 3. Click the login button
    // The button has text "Đăng nhập" in Vietnamese (from i18n)
    // We use data-testid for stability
    await page.getByTestId('login-submit-btn').click();

    // 4. Verify successful redirection to Home
    await page.waitForURL(`${workerWebUrl}/`, { timeout: 15000 });
    // We expect the URL to be the root or contains /dashboard if it redirects there
    // LoginPage.tsx does navigate("/", { replace: true });
    await expect(page).toHaveURL(new RegExp(`${workerWebUrl}/?$`));

    // 5. Verify Sidebar or some home page element is visible
    await expect(page.getByText('LeeAttend')).toBeVisible();
    
    // Check if we are actually logged in by looking for a logout button or profile info
    // Usually there's a sidebar with menu items
    await expect(page.locator('aside nav')).toBeVisible();
  });

  test('should show error message with invalid credentials', async ({ page, workerWebUrl }) => {
    await page.goto(`${workerWebUrl}/login`);

    await page.fill('input[name="email"]', 'wrong@example.com');
    await page.fill('input[name="password"]', 'wrongpass');

    await page.getByTestId('login-submit-btn').click();

    // Should stay on login page
    await expect(page).toHaveURL(`${workerWebUrl}/login`);

    // Should show error message
    // LoginPage.tsx shows error in a div with role="alert" or similar
    await expect(page.getByRole('alert')).toBeVisible();
  });
});
