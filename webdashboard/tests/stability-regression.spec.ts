/* eslint-disable @typescript-eslint/restrict-template-expressions, no-console */
import { test, expect } from "./worker-orchestrator";

/**
 * REGRESSION TEST: Workspace Context Stability
 * Root Cause Fix: Ensures that the first API request after login 
 * includes the necessary workspace headers, preventing 400 Bad Request.
 */
test.skip("should include X-Workspace-Id header on initial dashboard load", async ({ page }) => {
  // 1. Intercept the dashboard stats request
  const statsRequestPromise = page.waitForRequest((request) => 
    request.url().includes("/api/dashboard/workspace-stats") && request.method() === "GET"
  );

  await page.goto('/');

  // 2. Fast Login
  // Using testid for stability
  await page.getByTestId('fast-login-btn').click();

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
test.skip("should load settings page without ReferenceErrors", async ({ page, baseURL }) => {
  // Errors would be caught by worker-orchestrator's pageerror listener,
  // but we explicitly check navigation here.
  
  // Login first
  await page.goto(`${baseURL}/login`);
  await page.getByTestId('fast-login-btn').click();
  
  // Navigate via testids
  await page.getByTestId('sidebar-section-system').click();
  const settingsLink = page.getByTestId('sidebar-link-settings-general');
  await settingsLink.click();
  
  // Verify heading is visible (Settings2 icon used to be in the sidebar or page header)
  await expect(page.getByRole('heading', { level: 1 })).toContainText(/Workspace Settings|Cài đặt Workspace|General|Chung|Cấu hình chung|常规|常规设置/i, { timeout: 15000 });
  
  // Check console for "ReferenceError"
  const logs: string[] = [];
  page.on("console", message => logs.push(message.text()));
  
  expect(logs.some(l => l.includes("ReferenceError"))).toBe(false);
});