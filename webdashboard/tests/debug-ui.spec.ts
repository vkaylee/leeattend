import { test, expect } from "./worker-orchestrator";

test("debug fast login button click", async ({ page, baseURL }) => {
  page.on("console", (msg) => console.log(`[BROWSER] ${msg.text()}`));
  page.on("pageerror", (err) => console.log(`[BROWSER_ERROR] ${err.message}`));

  await page.goto(`${baseURL}/login`);

  console.log("📍 Current page:", page.url());
  await page.screenshot({ path: "debug-before-click.png" });

  const loginBtn = page.locator('button:has-text("Đăng nhập nhanh")');

  if (await loginBtn.isVisible()) {
    console.log("🖱️ Clicking Fast Login button...");

    // Lắng nghe request gọi tới API
    const requestPromise = page.waitForRequest((request) =>
      request.url().includes("/dev/super-user"),
    );

    await loginBtn.click();

    try {
      const request = await requestPromise;
      console.log("📡 API Request sent to:", request.url());

      const response = await page.waitForResponse((response) =>
        response.url().includes("/dev/super-user"),
      );
      console.log("📥 API Response status:", response.status());
      const body = await response.text();
      console.log("📄 API Response body:", body);
    } catch (e) {
      console.log("❌ No API request detected or it timed out.");
    }

    await page.waitForTimeout(2000);
    await page.screenshot({ path: "debug-after-click.png" });
    console.log("📍 Page after click:", page.url());
  } else {
    console.log("❌ Fast Login button NOT FOUND!");
  }
});
