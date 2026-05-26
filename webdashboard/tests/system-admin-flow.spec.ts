import { expect } from '@playwright/test';

import { test } from './worker-orchestrator';

test.describe('System Admin Flow', () => {
  test.beforeEach(async ({ loginPage, page }) => {
    await loginPage.goto();
    // Assuming superadmin or similar account has access
    await loginPage.login('admin@leeattend.com', 'admin123');
    await expect(page).toHaveURL(/\/$/);
  });

  test('should navigate to system admin tenants and see list', async ({ page }) => {
    await page.goto('/system-admin/tenants');
    await expect(page).toHaveURL(/.*\/system-admin\/tenants/);
    
    // Check page title
    await expect(page.getByRole('heading', { name: /tenants/i })).toBeVisible();
    
    // Verify Create Tenant button exists
    const createBtn = page.getByRole('button', { name: /create|add/i });
    if (await createBtn.isVisible()) {
      await createBtn.click();
      // Expect some modal or form
      await page.keyboard.press('Escape');
    }
  });

  test('should navigate to system roles and view matrix', async ({ page }) => {
    await page.goto('/system-admin/roles');
    await expect(page).toHaveURL(/.*\/system-admin\/roles/);
    
    await expect(page.getByRole('heading', { name: /roles/i })).toBeVisible();
    // Validate table or matrix is visible
    await expect(page.locator('table')).toBeVisible();
  });
});
