import { test, expect } from '@playwright/test';

test.describe('Members UI', () => {
  test('should navigate to members page and open invite dialog', async ({ page }) => {
    // 1. Login (Mock or Real)
    // We assume the dev environment allows fast login.
    // If we rely on real backend, we need to ensure we are logged in.
    // Since we are running against real dev server, we can use the UI login flow.

    await page.goto('http://localhost:5173/login');
    
    // Check if we are already logged in (redirected to /)
    if (page.url() !== 'http://localhost:5173/') {
        await page.click('button:has-text("Đăng nhập nhanh")');
        await page.waitForURL('http://localhost:5173/');
    }

    // 2. Navigate to Members Page
    await page.click('a[href="/members"]');
    await expect(page).toHaveURL('http://localhost:5173/members');
    await expect(page.locator('h1')).toContainText('Members & Roles');

    // 3. Open Invite Dialog
    await page.click('button:has-text("Invite Member")');
    await expect(page.locator('h2:has-text("Invite Member")')).toBeVisible();

    // 4. Close Dialog
    await page.locator('button:has(svg.lucide-x)').click();
    await expect(page.locator('h2:has-text("Invite Member")')).not.toBeVisible();
  });
});
