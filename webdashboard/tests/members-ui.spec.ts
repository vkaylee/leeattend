import { test } from "./worker-orchestrator";
import { expect } from "@playwright/test";

test.describe("Members UI", () => {
  test('should navigate to Members page and show Invite Modal', async ({ page }) => {
    // 1. Login
    await page.goto('/login');
    await page.getByTestId('fast-login-btn').click();
    await expect(page.getByRole('heading', { level: 1, name: /Dashboard|Bảng điều khiển/i })).toBeVisible({ timeout: 30000 });

    // 2. Expand Sidebar Section if needed
    const membersLink = page.getByTestId('sidebar-link-members');
    const orgSection = page.getByTestId('sidebar-section-org');
    
    // Check if link is visible, if not, click section to expand
    if (!await membersLink.isVisible()) {
      await orgSection.click();
      await expect(membersLink).toBeVisible({ timeout: 10000 });
    }

    // 3. Click Members link
    await membersLink.click();
    await expect(page).toHaveURL(/\/members/i, { timeout: 30000 });

    await expect(page.getByRole('heading', { level: 1 })).toContainText(/Members & Permissions|Thành viên & Quyền hạn/);

    // 4. Click Invite Member button
    await page.getByTestId('invite-member-trigger').click();
    await expect(page.getByRole('heading', { level: 2, name: /Invite Member|Mời thành viên/i })).toBeVisible();

    // 5. Close the dialog using X icon (which should be findable by role)
    await page.getByRole('dialog').getByRole('button').filter({ has: page.locator('svg.lucide-x') }).first().click();

    // 6. Verify modal is closed
    await expect(
      page.getByRole('heading', { level: 2, name: /Invite Member/i }),
    ).not.toBeVisible();
  });
});
