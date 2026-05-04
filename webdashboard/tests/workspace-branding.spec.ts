import { type Dialog } from '@playwright/test';

import { test, expect } from './worker-orchestrator';

test.describe('Workspace Branding', () => {
  test.beforeEach(async ({ loginPage, page }) => {
    await loginPage.goto();
    // Using real login for better reliability in E2E
    await loginPage.login('admin@leeattend.com', 'admin123');
    await expect(page).toHaveURL(/\/$/);
  });

  test('should update workspace logo and tagline', async ({ page }) => {
    // Navigate directly to general settings
    await page.goto('/workspace/settings/general');
    
    await expect(page).toHaveURL(/.*\/workspace\/settings\/general/);
    await expect(page.getByTestId('general-settings-title')).toBeVisible();

    // Go to Appearance tab
    await page.getByTestId('tab-appearance').click();

    const testLogoUrl = 'https://raw.githubusercontent.com/lucide-react/lucide/main/icons/building-2.svg';
    const testTagline = 'Premium HR Management';

    // Handle the prompt for logo URL
    const dialogHandler = async (dialog: Dialog) => {
      if (dialog.type() === 'prompt') {
        await dialog.accept(testLogoUrl);
      } else {
        await dialog.dismiss();
      }
    };
    page.on('dialog', dialogHandler);

    // Set logo
    await page.getByTestId('upload-logo-btn').click();
    
    // Set tagline
    await page.getByTestId('workspace-tagline-input').fill(testTagline);

    // Save
    const saveBtn = page.getByTestId('save-workspace-config-btn');
    await saveBtn.click();
    
    // Wait for mutation to complete by checking button state
    await expect(saveBtn).toBeEnabled({ timeout: 10000 });
    
    // Wait for a bit for React Query to propagate changes
    await page.waitForTimeout(2000);

    // Verify in Sidebar
    // Logo should be visible in sidebar
    const logo = page.getByTestId('sidebar-workspace-logo');
    await expect(logo).toBeVisible({ timeout: 10000 });
    await expect(logo).toHaveCSS('background-image', new RegExp(testLogoUrl));

    // Tagline should be visible in sidebar
    const tagline = page.getByTestId('sidebar-workspace-tagline').first();
    await expect(tagline).toBeVisible();
    await expect(tagline).toHaveText(testTagline);

    // Cleanup: remove dialog listener
    page.off('dialog', dialogHandler);
  });
});
