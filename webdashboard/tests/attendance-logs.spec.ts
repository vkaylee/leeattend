import { test } from "./worker-orchestrator";
import { expect } from "@playwright/test";

test.describe("Attendance Logs UI", () => {
  test("should navigate to attendance logs page and see the logs table", async ({ page, baseURL }) => {
    // 1. Login
    console.log(`Navigating to ${baseURL}/login...`);
    await page.goto(`${baseURL}/login`);
    
    try {
    // If redirected to login, perform fast login
    if (page.url().endsWith("/login")) {
        await page.click('button:has-text("⚡ Fast Login")');
        await page.waitForURL("**/", { timeout: 30000 });
    }

      console.log("Waiting for loading screen to disappear...");
      await expect(page.locator("text=Đang kiểm tra bảo mật...")).not.toBeVisible({ timeout: 60000 });
      await expect(page.locator("text=Đang tải không gian làm việc...")).not.toBeVisible({ timeout: 60000 });

      console.log("Current URL before sidebar check:", page.url());
      
      // 2. Navigate to Attendance Logs
      console.log("Waiting for sidebar link...");
      try {
        await page.waitForSelector('a[href="/attendance-logs"]', { timeout: 30000 });
      } catch (e) {
        console.error("Sidebar link not found. Page HTML snippet:");
        const content = await page.content();
        console.log(content.substring(0, 1000)); // Log first 1000 chars of HTML
        throw e;
      }
      console.log("Sidebar link found, clicking...");
      // 2. Expand Sidebar Section if needed
      // Look for "Time & Leave" or "Thời gian & Nghỉ phép"
      const sectionHeader = page.locator('button:has-text("Time & Leave"), button:has-text("Thời gian & Nghỉ phép")');
      await sectionHeader.click();

      // 3. Navigate to Attendance Logs Page
      await page.click('a[href="/attendance-logs"]');

      // The page title in H1 is "Attendance Logs" or "Nhật ký quẹt thẻ"
      console.log("Checking page content...");
      await expect(page.locator("h1")).toContainText(/Attendance Logs|Nhật ký quẹt thẻ/, { timeout: 30000 });

      // 3. Check if table headers are present
      await expect(page.locator("th:has-text('Time'), th:has-text('Thời gian')")).toBeVisible();
      await expect(page.locator("th:has-text('Employee'), th:has-text('Nhân viên')")).toBeVisible();
      
      console.log("Test passed!");
    } catch (e) {
      console.error("Test failed, taking screenshot...", e);
      await page.screenshot({ path: "test-failure.png" });
      throw e;
    }
  });
});
