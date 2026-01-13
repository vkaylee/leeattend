import { test } from "./worker-orchestrator";
import { expect } from "@playwright/test";

test.describe("Members UI", () => {
  test("should navigate to members page and open invite dialog", async ({ page, baseURL }) => {
    // Since we are running against real dev server, we can use the UI login flow.

    await page.goto(`${baseURL}/login`);

    // Check if we are already logged in (redirected to /)
    if (page.url().endsWith("/login")) {
        // Button text includes emoji in LoginPage.tsx
        await page.click('button:has-text("Đăng nhập nhanh")');
        await page.waitForURL("**/", { timeout: 30000 });
    }

    // 2. Navigate to Members Page
    await page.click('a[href="/members"]');
    await expect(page).toHaveURL(`${baseURL}/members`);
    await expect(page.locator("h1")).toContainText("Members & Roles");

    // 3. Open Invite Dialog
    await page.click('button:has-text("Invite Member")');
    await expect(page.locator('h2:has-text("Invite Member")')).toBeVisible();

    // 4. Close Dialog
    await page.locator("button:has(svg.lucide-x)").click();
    await expect(
      page.locator('h2:has-text("Invite Member")'),
    ).not.toBeVisible();
  });
});
