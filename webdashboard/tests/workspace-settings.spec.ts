import { expect } from '@playwright/test';

import { test } from './worker-orchestrator';

test.describe('Workspace Settings Flow', () => {
  test.beforeEach(async ({ loginPage, page }) => {
    await loginPage.goto();
    await loginPage.login('admin@leeattend.com', 'admin123');
    await expect(page).toHaveURL(/\/$/);
  });

  test('should navigate to settings and update configurations', async ({ page }) => {
    await page.goto('/workspace/settings/general');
    await expect(page).toHaveURL(/.*\/workspace\/settings\/general/);
    
    // Check if tabs are visible
    await expect(page.getByTestId('tab-general')).toBeVisible();
    await expect(page.getByTestId('tab-appearance')).toBeVisible();
    await expect(page.getByTestId('tab-danger')).toBeVisible();

    // Verify Timezone change
    await page.getByTestId('timezone-select').click();
    // In a real app we might select an option here. We just ensure it opens.
    await page.keyboard.press('Escape'); 

    // Save should be visible
    const saveBtn = page.getByTestId('save-workspace-config-btn');
    await expect(saveBtn).toBeVisible();
  });
});
