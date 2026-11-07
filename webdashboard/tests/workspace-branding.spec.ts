import { type Dialog } from '@playwright/test';

import { test, expect } from './worker-orchestrator';

test.describe('Workspace Branding', () => {
  test.beforeEach(async ({ authPage }) => {
    expect(authPage).toBeDefined();
  });

  test('should update workspace logo and tagline', async ({ page }) => {
    // Navigate directly to General Settings (Settings Hub refactored navigation)
    await page.goto('/workspace/settings/general');
    await expect(page).toHaveURL(/.*\/workspace\/settings\/general/);
    await expect(page.getByTestId('general-settings-title')).toBeVisible({ timeout: 30000 });

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

    // Verify in settings appearance preview
    const logo = page.getByRole('img', { name: 'Workspace Logo' });
    await expect(logo).toBeVisible({ timeout: 10000 });
    await expect(logo).toHaveCSS('background-image', new RegExp(testLogoUrl));

    // Tagline input retains updated value
    await expect(page.getByTestId('workspace-tagline-input')).toHaveValue(testTagline);
    // Cleanup: remove dialog listener
    page.off('dialog', dialogHandler);
  });
});
