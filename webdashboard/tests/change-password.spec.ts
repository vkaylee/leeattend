/* eslint-disable @typescript-eslint/restrict-template-expressions, no-console */
import { expect } from "@playwright/test";

import { test } from "./worker-orchestrator";

test.describe.skip("Change Password Flow", () => {
  test.skip("should change password successfully", async ({ page, baseURL }) => {
    test.slow();
    // 1. Initial Login
    await page.goto(`${baseURL}/login`);
    await page.getByTestId('login-email-input').fill("admin@leeattend.com");

    let currentPwd = "admin123";
    await page.getByTestId('login-password-input').fill(currentPwd);
    await page.getByTestId('login-submit-btn').click();

    await page.waitForTimeout(3000);
    if (page.url().includes("/login")) {
      console.log("admin123 failed, trying newadmin123");
      currentPwd = "newadmin123";
      await page.getByTestId('login-password-input').fill(currentPwd);
      await page.getByTestId('login-submit-btn').click();
    }

    await expect(page).toHaveURL("/", { timeout: 15000 });

    await page.goto("/profile");
    // Wait for SECURITY button and click it
    const securityTab = page.getByRole('button', { name: /SECURITY|Bảo mật/i });
    await expect(securityTab).toBeVisible({ timeout: 15000 });
    await securityTab.click();

    const nextPwd = currentPwd === "admin123" ? "newadmin123" : "admin123";

    console.log(`Changing password from ${currentPwd} to ${nextPwd}`);

    // Fill the change password form
    const oldPassInput = page.getByTestId('current-password-input');
    const newPassInput = page.getByTestId('new-password-input');
    const confirmPassInput = page.getByTestId('confirm-password-input');
    
    await oldPassInput.fill(currentPwd);
    await newPassInput.fill(nextPwd);
    await confirmPassInput.fill(nextPwd);

    const updateButton = page.getByTestId('update-password-btn');
    await updateButton.click();

    // Wait for logout and redirect to login
    await expect(page).toHaveURL("/login", { timeout: 15000 });

    // Try login with new password
    await page.getByTestId('login-email-input').fill("admin@leeattend.com");
    await page.getByTestId('login-password-input').fill(nextPwd);
    await page.getByTestId('login-submit-btn').click();

    await expect(page).toHaveURL("/", { timeout: 15000 });
    
    // --- RESET PASSWORD BACK TO admin123 for other tests ---
    console.log("Resetting password back to admin123...");
    await page.goto("/profile");
    const securityTabReset = page.getByRole('button', { name: /SECURITY|Bảo mật/i });
    await expect(securityTabReset).toBeVisible({ timeout: 15000 });
    await securityTabReset.click();
    
    await page.getByTestId('current-password-input').fill(nextPwd);
    await page.getByTestId('new-password-input').fill("admin123");
    await page.getByTestId('confirm-password-input').fill("admin123");
    await page.getByTestId('update-password-btn').click();
    await expect(page).toHaveURL("/login", { timeout: 15000 });
    
    console.log("Test passed and password reset!");
  });
});
