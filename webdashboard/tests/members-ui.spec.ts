import { test } from "./worker-orchestrator";
import { expect } from "@playwright/test";

test.describe("Members UI", () => {
  test('should navigate to Members page and show Invite Modal', async ({ page }) => {
    // 1. Login
    await page.goto('/login');
    await page.getByTestId('fast-login-btn').click();

    // 2. Expand Sidebar Section if needed
    await page.getByTestId('sidebar-section-org').click();

    // 3. Click Members link
    await page.getByTestId('sidebar-link-members').click();

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
