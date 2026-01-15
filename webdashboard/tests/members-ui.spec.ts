import { test } from "./worker-orchestrator";
import { expect } from "@playwright/test";

test.describe("Members UI", () => {
  test("should navigate to members page and open invite dialog", async ({ page, baseURL }) => {
    // Since we are running against real dev server, we can use the UI login flow.

    await page.goto(`${baseURL}/login`);

    // Check if we are already logged in (redirected to /)
    if (page.url().endsWith("/login")) {
        // Button text includes emoji in LoginPage.tsx
        await page.click('button:has-text("⚡ Fast Login")');
        await page.waitForURL("**/", { timeout: 30000 });
    }

    // 2. Expand Sidebar Section if needed
    const sectionHeader = page.locator('button:has-text("Organization"), button:has-text("Tổ chức")');
    await sectionHeader.click();

    // 3. Navigate to Members Page
    await page.click('a[href="/members"]');
    await expect(page).toHaveURL(`${baseURL}/members`);
    await expect(page.locator("h1")).toContainText(/Members & Permissions|Thành viên & Quyền hạn/);

    // 3. Open Invite Dialog
    await page.click('button:has-text("Invite Member"), button:has-text("Mời thành viên")');
    await expect(page.locator('h2:has-text("Invite Member"), h2:has-text("Mời thành viên")')).toBeVisible();

    // 4. Close Dialog
    // Use a more specific selector for the modal close button to avoid mobile sidebar close button
    await page.locator('div[role="dialog"] button:has(svg.lucide-x), .modal-content button:has(svg.lucide-x)').first().click();
    await expect(
      page.locator('h2:has-text("Invite Member")'),
    ).not.toBeVisible();
  });
});
