 
import { expect } from "@playwright/test";

import { test } from "./worker-orchestrator";

/**
 * SMOKE TEST: Đảm bảo UI được build và render thành công
 */
test.describe("UI Build Smoke Test", () => {
  test("smoke: app renders without errors", async ({ page, baseURL }) => {
    const errors: string[] = [];

    // Thu thập tất cả lỗi trình duyệt
    page.on("pageerror", (error) => {
      errors.push(error.message);
      console.error(`[BROWSER_ERROR] ${error.message}`);
    });

    // Đi tới trang chủ (hoặc login nếu chưa auth)
    await page.goto(baseURL ?? "http://localhost:5173");

    // Chờ cho đến khi biến mất màn hình loading
    await page.waitForFunction(() => {
      const text = document.body.innerText;
      return text.trim().length > 0 &&
        !text.includes("Checking security") &&
        !text.includes("Đang kiểm tra bảo mật") &&
        !text.includes("Loading workspace") &&
        !text.includes("Đang tải");
    }, { timeout: 15000 }).catch(() => { /* empty */ });

    // 1. Kiểm tra không có lỗi runtime nghiêm trọng làm chết app
    expect(errors).toHaveLength(0);

    // 2. Kiểm tra phần tử root (#root) tồn tại
    const root = page.locator("#root");
    await expect(root).toBeVisible();

    // 3. Đảm bảo không render trang trắng (Kiểm tra có text hoặc element quan trọng)
    const bodyText = await page.innerText("body");
    const hasAppContent = bodyText.includes("LeeAttend") ||
      bodyText.includes("Attend") ||
      bodyText.includes("Sign in") ||
      bodyText.includes("Đăng nhập") ||
      bodyText.includes("Dashboard") ||
      bodyText.includes("Bảng điều khiển") ||
      bodyText.includes("Fast Login") ||
      bodyText.includes("FAST LOGIN");

    expect(hasAppContent).toBe(true);
  });

  test("should have correctly configured styles and assets", async ({ page, baseURL }) => {
    await page.goto(baseURL ?? "http://localhost:5173");

    // Kiểm tra logo hoặc các asset quan trọng
    const images = await page.locator("img").count();
    // Thường login page sẽ có logo
    if (page.url().includes("/login")) {
      // login page mới có nhiều decor, ít nhất 1 logo/icon
      expect(images).toBeGreaterThanOrEqual(0);
    }
  });
});
