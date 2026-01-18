import { test } from "./worker-orchestrator";
import { expect } from "@playwright/test";

test.describe("Attendance Logs UI", () => {
  test('should navigate to Attendance Logs and see table headers', async ({ page }) => {
    try {
      // 1. Login
      await page.goto('/login');
      await page.getByTestId('fast-login-btn').click();

      // 2. Expand Sidebar Section if needed
      // Using testid for stability
      await page.getByTestId('sidebar-section-time').click();

      // 3. Click Logs link
      const logsLink = page.getByTestId('sidebar-link-attendance-logs');
      await logsLink.click();

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