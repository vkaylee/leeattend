import { test } from "./worker-orchestrator";
import { expect } from "@playwright/test";

test.describe("Change Password Flow", () => {
  test("should change password successfully", async ({ page, baseURL }) => {
    // 1. Initial Login
    await page.goto(`${baseURL}/login`);
    await page.fill(
      'input[name="email"]',
      "admin@leeattend.com",
    );

    let currentPwd = "admin123";
    await page.fill('input[name="password"]', currentPwd);
    await page.click('button[type="submit"]');

    await page.waitForTimeout(3000);
    if (page.url().includes("/login")) {
      console.log("admin123 failed, trying newadmin123");
      currentPwd = "newadmin123";
      await page.fill('input[name="password"]', currentPwd);
      await page.click('button[type="submit"]');
    }

    await expect(page).toHaveURL("/", { timeout: 15000 });

    await page.goto("/profile");
    const nextPwd = currentPwd === "admin123" ? "newadmin123" : "admin123";

    console.log(`Changing password from ${currentPwd} to ${nextPwd}`);

    // Fill form in Profile
    const inputs = page.locator('input[type="password"]');
    await inputs.nth(0).fill(currentPwd);
    await inputs.nth(1).fill(nextPwd);
    await inputs.nth(2).fill(nextPwd);

    // Try both languages for the update button
    const updateBtn = page.locator('button:has-text("Update Password"), button:has-text("Cập nhật mật khẩu")');
    await updateBtn.click();

    // Đợi logout và redirect về login
    await expect(page).toHaveURL("/login", { timeout: 15000 });

    // Thử login với mật khẩu mới
    await page.fill(
      'input[name="email"]',
      "admin@leeattend.com",
    );
    await page.fill('input[name="password"]', nextPwd);
    await page.click('button[type="submit"]');

    await expect(page).toHaveURL("/", { timeout: 15000 });
    console.log("Test passed!");
  });
});
