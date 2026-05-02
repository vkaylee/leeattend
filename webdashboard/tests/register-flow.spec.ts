/* eslint-disable @typescript-eslint/restrict-template-expressions */
import { expect } from "@playwright/test";

import { test } from "./worker-orchestrator";

test.describe.skip("Registration Flow", () => {
  test.skip("should register a new user and login", async ({ page, baseURL }) => {
    // 1. Go to Login page
    await page.goto(`${baseURL}/login`);

    // 2. Click Sign up
    const registerLink = page.locator('a[href="/register"]');
    await expect(registerLink).toBeVisible({ timeout: 30000 });
    await registerLink.click();
    await expect(page).toHaveURL("/register");
    await expect(page.locator("h2")).toContainText("Create new account");

    // 3. Fill Form
    const uniqueEmail = `testuser_${Date.now()}@example.com`;
    await page.fill('input[name="full_name"]', "Test User");
    await page.fill('input[name="email"]', uniqueEmail);
    await page.fill('input[name="password"]', "password123");
    await page.fill('input[name="confirm_password"]', "password123");

    // 4. Submit
    await page.click('button[type="submit"]');

    // 5. Expect Redirect to Onboarding (or Home if onboarding skipped logic exists)
    // The current RegisterPage redirects to /onboarding
    await page.waitForURL("**/onboarding");

    // Check local storage for token
    const token = await page.evaluate(() =>
      localStorage.getItem("access_token"),
    );
    expect(token).toBeTruthy();
  });
});
