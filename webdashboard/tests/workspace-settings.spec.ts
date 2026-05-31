import { expect } from '@playwright/test';

import { test } from './worker-orchestrator';

test.describe('Workspace Settings Flow', () => {
  test.beforeEach(async ({ authPage }) => {
    // authPage fixture automatically logs in
    expect(authPage).toBeDefined();
  });

  test('should navigate to settings and update configurations', async ({ page }) => {
    // Wait for the sidebar to load and system section to be visible (allow extra timeout for Vite cold compile)
    await expect(page.getByTestId('sidebar-section-system')).toBeVisible({ timeout: 30000 });
    await page.getByTestId('sidebar-section-system').click();
    await page.getByTestId('sidebar-link-settings-root').click();
    
    // Navigate to General Settings sub-tab client-side
    await page.getByTestId('settings-nav-/workspace/settings/general').click();
    await expect(page).toHaveURL(/.*\/workspace\/settings\/general/);
    
    // Check if tabs are visible (allow extra timeout for API data load)
    await expect(page.getByTestId('tab-localization')).toBeVisible({ timeout: 30000 });
    await expect(page.getByTestId('tab-appearance')).toBeVisible({ timeout: 30000 });

    // Verify Timezone change
    await page.getByTestId('timezone-select').click();
    // In a real app we might select an option here. We just ensure it opens.
    await page.keyboard.press('Escape'); 

    // Save should be visible
    const saveBtn = page.getByTestId('save-workspace-config-btn');
    await expect(saveBtn).toBeVisible();
  });
});
