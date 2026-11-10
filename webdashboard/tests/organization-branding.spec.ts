import { test, expect } from './worker-orchestrator';

test.describe('Organization Branding', () => {
  test.beforeEach(async ({ authPage }) => {
    expect(authPage).toBeDefined();
  });

  test('should update organization logo and tagline', async ({ page }) => {
    // Navigate directly to General Settings (Settings Hub refactored navigation)
    await page.goto('/settings/general');
    await expect(page).toHaveURL(/.*\/settings\/general/);
    await expect(page.getByTestId('general-settings-title')).toBeVisible({ timeout: 30000 });

    // Go to Appearance tab
    await page.getByTestId('tab-appearance').click();

    const testLogoUrl = new URL('/favicon.ico', page.url()).href;
    const testTagline = 'Synthetic organization branding';
    const editLogo = page.getByTestId('edit-logo-btn');
    if (await editLogo.count()) {
      await editLogo.click();
    } else {
      await page.getByTestId('upload-logo-btn').click();
    }
    const logoInput = page.locator('input[type="url"]');
    await logoInput.fill(testLogoUrl);
    await logoInput.press('Enter');

    // Set tagline
    await page.getByTestId('organization-tagline-input').fill(testTagline);

    // Save
    const saveBtn = page.getByTestId('save-organization-config-btn');
    await saveBtn.click();

    // Wait for mutation to complete by checking button state
    await expect(saveBtn).toBeEnabled({ timeout: 10000 });

    // Verify in settings appearance preview
    const logo = page.getByRole('img', { name: /Organization Logo|Logo tổ chức|组织标志/ });
    await expect(logo).toBeVisible({ timeout: 10000 });
    await expect(logo).toHaveCSS('background-image', `url("${testLogoUrl}")`);

    // Tagline input retains updated value
    await expect(page.getByTestId('organization-tagline-input')).toHaveValue(testTagline);
  });
});
