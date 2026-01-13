import { test, expect } from "@playwright/test";

test.describe("Attendance Logs UI", () => {
  test("should navigate to attendance logs page and see the logs table", async ({
    page,
  }) => {
    // Increase timeout for slow environments
    test.setTimeout(120000);

    // 1. Login
    console.log("Navigating to /login...");
    await page.goto("/login");
    
    try {
      if (page.url().endsWith("/login")) {
        console.log("On login page, performing fast login...");
        // Button text includes emoji in LoginPage.tsx
        await page.click('button:has-text("Đăng nhập nhanh")');
        console.log("Waiting for URL change from /login...");
        await page.waitForURL("**/", { timeout: 30000 });
        console.log("Login successful! Current URL:", page.url());
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
      await page.click('a[href="/attendance-logs"]');
      
      console.log("Waiting for URL change...");
      await expect(page).toHaveURL(/.*attendance-logs/, { timeout: 30000 });
      
      // The page title in H1 is "Attendance Logs" (from AttendanceLogsPage.tsx)
      console.log("Checking page content...");
      await expect(page.locator("h1")).toContainText("Attendance Logs", { timeout: 30000 });

      // 3. Check if table headers are present
      await expect(page.locator("th:has-text('Date')")).toBeVisible();
      await expect(page.locator("th:has-text('Employee')")).toBeVisible();
      
      console.log("Test passed!");
    } catch (e) {
      console.error("Test failed, taking screenshot...", e);
      await page.screenshot({ path: "test-failure.png" });
      throw e;
    }
  });
});
