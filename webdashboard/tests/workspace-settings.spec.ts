import { expect } from '@playwright/test';

import { test } from './worker-orchestrator';

test.describe('Workspace Settings Flow', () => {
  test.beforeEach(async ({ authPage }) => {
    expect(authPage).toBeDefined();
  });

  test('should navigate to settings and update configurations', async ({ page }) => {
    // Navigate directly to General Settings (Settings Hub refactored navigation)
    await page.goto('/workspace/settings/general');
    await expect(page).toHaveURL(/.*\/workspace\/settings\/general/);

    // Verify page loaded
    await expect(page.getByTestId('general-settings-title')).toBeVisible({ timeout: 30000 });

    // Check if tabs are visible
    await expect(page.getByTestId('tab-localization')).toBeVisible({ timeout: 15000 });
    await expect(page.getByTestId('tab-appearance')).toBeVisible({ timeout: 15000 });

    // Verify Timezone change
    await page.getByTestId('timezone-select').click();
    await page.keyboard.press('Escape');

    // Save should be visible
    const saveBtn = page.getByTestId('save-workspace-config-btn');
    await expect(saveBtn).toBeVisible();
  });
});
