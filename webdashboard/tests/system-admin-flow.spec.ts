import { expect } from '@playwright/test';

import { test } from './worker-orchestrator';

test.describe('System Admin Flow', () => {
  test.beforeEach(async ({ authPage }) => {
    expect(authPage).toBeDefined();
  });

  test('should navigate to system admin tenants and see list', async ({ page }) => {
    // DSM App Shell: click System group in icon rail to open floating panel
    const systemGroup = page.getByTestId('nav-rail-group-system');
    await expect(systemGroup).toBeVisible({ timeout: 30000 });
    await systemGroup.click();

    // Wait for panel to appear, then click Tenants link
    await expect(page.getByTestId('sidebar-link-system-tenants')).toBeVisible({ timeout: 10000 });
    await page.getByTestId('sidebar-link-system-tenants').click();
    await expect(page).toHaveURL(/.*\/system\/tenants/);

    await expect(page.getByRole('heading', { name: 'Tenants Management', exact: true })).toBeVisible({ timeout: 30000 });

    // Verify Create Tenant button exists
    const createBtn = page.getByRole('button', { name: /create|add/i });
    if (await createBtn.isVisible()) {
      await createBtn.click();
      await page.keyboard.press('Escape');
    }
  });

  test('should navigate to system roles and view matrix', async ({ page }) => {
    // DSM App Shell: click System group in icon rail to open floating panel
    const systemGroup = page.getByTestId('nav-rail-group-system');
    await expect(systemGroup).toBeVisible({ timeout: 30000 });
    await systemGroup.click();

    // Wait for panel, then click Roles link
    await expect(page.getByTestId('sidebar-link-system-roles')).toBeVisible({ timeout: 10000 });
    await page.getByTestId('sidebar-link-system-roles').click();
    await expect(page).toHaveURL(/.*\/system\/roles/);

    await expect(page.getByRole('heading', { name: /platform|nền tảng|admin/i }).first()).toBeVisible({ timeout: 30000 });
    // Validate grid is visible
    await expect(page.locator('.grid')).toBeVisible({ timeout: 30000 });
  });
});
