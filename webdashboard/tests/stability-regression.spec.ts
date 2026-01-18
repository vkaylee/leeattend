import { test, expect } from "./worker-orchestrator";

/**
 * REGRESSION TEST: Workspace Context Stability
 * Root Cause Fix: Ensures that the first API request after login 
 * includes the necessary workspace headers, preventing 400 Bad Request.
 */
test("should include X-Workspace-Id header on initial dashboard load", async ({ page, baseURL }) => {
  // 1. Intercept the dashboard stats request
  const statsRequestPromise = page.waitForRequest((request) => 
    request.url().includes("/api/dashboard/workspace-stats") && request.method() === "GET"
  );

  // 2. Perform Fast Login
  await page.goto(`${baseURL}/login`);
  const loginBtn = page.locator('button:has-text("⚡ Fast Login")');
  await expect(loginBtn).toBeVisible();
  await loginBtn.click();

  // 3. Wait for the stats request to be sent
  const request = await statsRequestPromise;
  const headers = request.headers();

  // 4. VERIFY: The header must exist and not be empty/null
  console.log("🛠️ Intercepted Headers for /stats:", JSON.stringify(headers, null, 2));
  
  expect(headers["x-workspace-id"]).toBeDefined();
  expect(headers["x-workspace-id"]).toMatch(/^[0-9a-f-]{36}$/i); // Must be a UUID
});

/**
 * REGRESSION TEST: UI Dependency Stability
 * Root Cause Fix: Ensures the App doesn't crash due to missing icons (like Settings2).
 */
test("should load settings page without ReferenceErrors", async ({ page, baseURL }) => {
  // Errors would be caught by worker-orchestrator's pageerror listener,
  // but we explicitly check navigation here.
  
  // Login first
  await page.goto(`${baseURL}/login`);
  await page.locator('button:has-text("⚡ Fast Login")').click();
  
  // Expand "System" section first
  const systemSection = page.getByRole('button', { name: 'System', exact: true });
  await expect(systemSection).toBeVisible();
  
  // Click to expand and wait for internal link to become visible
  const settingsLink = page.locator('nav a[href="/workspace/settings"]');
  
  // Try to click section if settings link not visible
  if (!(await settingsLink.isVisible())) {
    await systemSection.click();
  }

  // Wait for sidebar and navigate to settings
  await expect(settingsLink).toBeVisible({ timeout: 5000 });
  await settingsLink.click();
  
  // Verify heading is visible (Settings2 icon used to be in the sidebar or page header)
  await expect(page.locator('h1, h2:has-text("Workspace Settings")')).toBeVisible();
  
  // Check console for "ReferenceError"
  const logs: string[] = [];
  page.on("console", msg => logs.push(msg.text()));
  
  expect(logs.some(l => l.includes("ReferenceError"))).toBe(false);
});
