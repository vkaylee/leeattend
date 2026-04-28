import { test } from "./worker-orchestrator";
import { expect } from "@playwright/test";

test.describe.skip("Attendance Logs UI", () => {
  test('smoke: navigate to attendance logs', async ({ page }) => {
    try {
      // 1. Login
      await page.goto('/login');
      await page.getByTestId('fast-login-btn').click();

      // Wait for dashboard to load
      await expect(page.getByRole('heading', { level: 1 })).toBeVisible({ timeout: 30000 });

      // 2. Expand Sidebar Section if needed
      const logsLink = page.getByTestId('sidebar-link-attendance-logs');
      if (!await logsLink.isVisible()) {
        await page.getByTestId('sidebar-section-time').click({ force: true });
      }

      // 3. Click Logs link
      await logsLink.click({ force: true });

      // The page title in H1 is "Attendance Logs" or "Nhật ký quẹt thẻ"
      console.log("Checking page content...");
      await expect(page.getByRole('heading', { level: 1 })).toContainText(/Attendance Logs|Nhật ký quẹt thẻ/, { timeout: 30000 });

      // 3. Check if table headers are present
      await expect(page.getByRole('columnheader', { name: /Time|Thời gian/i })).toBeVisible();
      await expect(page.getByRole('columnheader', { name: /Employee|Nhân viên/i })).toBeVisible();
      
      console.log("Test passed!");
    } catch (e) {
      console.error("Test failed, taking screenshot...", e);
      await page.screenshot({ path: "test-failure.png" });
      throw e;
    }
  });
});