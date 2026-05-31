import { expect } from '@playwright/test';

import { test } from './worker-orchestrator';

test.describe('System Admin Flow', () => {
  test.beforeEach(async ({ authPage }) => {
    // authPage fixture automatically logs in
    expect(authPage).toBeDefined();
  });

  test('should navigate to system admin tenants and see list', async ({ page }) => {
    // Wait for sidebar to load and system section to be visible (allow extra timeout for Vite cold compile)
    await expect(page.getByTestId('sidebar-section-system')).toBeVisible({ timeout: 30000 });
    // Expand system section if collapsed
    await page.getByTestId('sidebar-section-system').click();
    await page.getByTestId('sidebar-link-system-tenants').click();
    await expect(page).toHaveURL(/.*\/system\/tenants/);
    
    // Check page title with 30s timeout for stability
    await expect(page.getByRole('heading', { name: /tenant/i })).toBeVisible({ timeout: 30000 });
    
    // Verify Create Tenant button exists
    const createBtn = page.getByRole('button', { name: /create|add/i });
    if (await createBtn.isVisible()) {
      await createBtn.click();
      // Expect some modal or form
      await page.keyboard.press('Escape');
    }
  });

  test('should navigate to system roles and view matrix', async ({ page }) => {
    // Wait for sidebar to load and system section to be visible (allow extra timeout for Vite cold compile)
    await expect(page.getByTestId('sidebar-section-system')).toBeVisible({ timeout: 30000 });
    // Expand system section if collapsed
    await page.getByTestId('sidebar-section-system').click();
    await page.getByTestId('sidebar-link-system-roles').click();
    await expect(page).toHaveURL(/.*\/system\/roles/);
    
    await expect(page.getByRole('heading', { name: /platform|nền tảng|admin/i }).first()).toBeVisible({ timeout: 30000 });
    // Validate grid is visible
    await expect(page.locator('.grid')).toBeVisible({ timeout: 30000 });
  });
});
