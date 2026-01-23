import { test } from "./worker-orchestrator";
import { expect } from "@playwright/test";

test.describe("Members UI", () => {
  test('should navigate to Members page and show Invite Modal', async ({ page }) => {
    // 1. Login
    await page.goto('/login');
    await page.getByTestId('fast-login-btn').click();
    await expect(page.getByRole('heading', { level: 1, name: /Dashboard|Bảng điều khiển/i })).toBeVisible({ timeout: 30000 });

    // 2. Expand Sidebar Section if needed
    const peopleLink = page.getByTestId('sidebar-link-settings-people');
    const settingsSection = page.getByTestId('sidebar-section-system');
    
    // Check if link is visible, if not, click section to expand
    if (!await peopleLink.isVisible()) {
      await settingsSection.click();
      await expect(peopleLink).toBeVisible({ timeout: 10000 });
    }

    // 3. Click People link
    await peopleLink.click();
    await expect(page).toHaveURL(/\/workspace\/settings\/people/i, { timeout: 30000 });

    await expect(page.getByTestId('people-settings-title')).toBeVisible();
    await page.getByTestId('tab-members').click();

    // 4. Verify Invite Member Section is visible
    await expect(page.getByTestId('invite-member-heading')).toBeVisible();
    
    // 5. Check if the form elements are there
    await expect(page.getByPlaceholder('dong-nghiep@congty.com')).toBeVisible();
    await expect(page.getByTestId('invite-member-submit-btn')).toBeVisible();

    // 6. Verify Members List is also present
    await expect(page.getByTestId('members-list-heading')).toBeVisible();
  });
});
