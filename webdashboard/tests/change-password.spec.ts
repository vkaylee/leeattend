import { test } from "./worker-orchestrator";
import { expect } from "@playwright/test";

test.describe("Change Password Flow", () => {
  test("should change password successfully", async ({ page, baseURL }) => {
    test.slow(); // Increases timeout for this test
    // 1. Initial Login
    await page.goto(`${baseURL}/login`);
    await page.fill(
      'input[name="email"]',
      "admin@leeattend.com",
    );

    let currentPwd = "admin123";
    await page.fill('input[name="password"]', currentPwd);
    await page.getByTestId('login-submit-btn').click();

    await page.waitForTimeout(3000);
    if (page.url().includes("/login")) {
      console.log("admin123 failed, trying newadmin123");
      currentPwd = "newadmin123";
      await page.fill('input[name="password"]', currentPwd);
      await page.getByTestId('login-submit-btn').click();
    }

    await expect(page).toHaveURL("/", { timeout: 15000 });

    await page.goto("/profile");
    const nextPwd = currentPwd === "admin123" ? "newadmin123" : "admin123";

    console.log(`Changing password from ${currentPwd} to ${nextPwd}`);

    // Fill form in Profile
    // 3. Fill the change password form
    // Using testids for stability
    const oldPassInput = page.getByTestId('current-password-input');
    const newPassInput = page.getByTestId('new-password-input');
    const confirmPassInput = page.getByTestId('confirm-password-input');
    
    await oldPassInput.fill('admin123');
    await newPassInput.fill('newadmin123');
    await confirmPassInput.fill('newadmin123');

    const updateBtn = page.getByTestId('update-password-btn');
    await updateBtn.click();

    // Đợi logout và redirect về login
    await expect(page).toHaveURL("/login", { timeout: 15000 });

    // Thử login với mật khẩu mới
    await page.fill(
      'input[name="email"]',
      "admin@leeattend.com",
    );
    await page.fill('input[name="password"]', nextPwd);
    await page.getByTestId('login-submit-btn').click();

    await expect(page).toHaveURL("/", { timeout: 15000 });
    
    // --- RESET PASSWORD BACK TO admin123 for other tests ---
    console.log("Resetting password back to admin123...");
    await page.goto("/profile");
    await page.getByTestId('current-password-input').fill('newadmin123');
    await page.getByTestId('new-password-input').fill('admin123');
    await page.getByTestId('confirm-password-input').fill('admin123');
    await page.getByTestId('update-password-btn').click();
    await expect(page).toHaveURL("/login", { timeout: 15000 });
    
    console.log("Test passed and password reset!");
  });
});
