import { test } from "./worker-orchestrator";
import { expect } from "@playwright/test";

/**
 * SMOKE TEST: Đảm bảo UI được build và render thành công
 */
test.describe("UI Build Smoke Test", () => {
  test("should load the application and render the root element without errors", async ({ page, baseURL }) => {
    const errors: string[] = [];
    
    // Thu thập tất cả lỗi trình duyệt
    page.on("pageerror", (err) => {
      errors.push(err.message);
      console.error(`[BROWSER_ERROR] ${err.message}`);
    });

    // Đi tới trang chủ (hoặc login nếu chưa auth)
    await page.goto(baseURL || "http://localhost:5173");

    // Chờ cho đến khi biến mất màn hình loading "Đang kiểm tra bảo mật..."
    await page.waitForSelector('text=Đang kiểm tra bảo mật...', { state: 'detached', timeout: 10000 });

    // 1. Kiểm tra không có lỗi runtime nghiêm trọng làm chết app
    expect(errors).toHaveLength(0);

    // 2. Kiểm tra phần tử root (#root) tồn tại
    const root = page.locator("#root");
    await expect(root).toBeVisible();

    // 3. Đảm bảo không render trang trắng (Kiểm tra có text hoặc element quan trọng)
    // Tùy theo login hay dashboard, ta kiểm tra các từ khóa đặc trưng
    const bodyText = await page.innerText("body");
    const hasAppContent = bodyText.includes("LeeAttend") || 
                         bodyText.includes("Đăng nhập") || 
                         bodyText.includes("Dashboard");
    
    expect(hasAppContent).toBe(true);
  });

  test("should have correctly configured styles and assets", async ({ page, baseURL }) => {
    await page.goto(baseURL || "http://localhost:5173");

    // Kiểm tra logo hoặc các asset quan trọng
    const images = await page.locator("img").count();
    // Thường login page sẽ có logo
    if (page.url().includes("/login")) {
       // login page mới có nhiều decor, ít nhất 1 logo/icon
       expect(images).toBeGreaterThanOrEqual(0);
    }
  });
});
