import { defineConfig, devices } from "@playwright/test";
import fs from "fs";
import path from "path";

/**
 * Read the web port from the discovery file or fallback to 5173
 * Includes a retry logic to wait for the file if Vite is still starting.
 */
function getWebPort() {
  const portFileName = process.env.PLAYWRIGHT_WEB_PORT_FILE || ".web_port";
  const portFile = path.resolve(__dirname, portFileName);
  const maxAttempts = 20; // Thử trong 10 giây (20 * 500ms)
  
  for (let attempt = 0; attempt < maxAttempts; attempt++) {
    try {
      if (fs.existsSync(portFile)) {
        const port = fs.readFileSync(portFile, "utf8").trim();
        if (port) {
          console.log(`📡 Playwright: Discovered Web Dashboard on port ${port} (Attempt ${attempt + 1})`);
          return port;
        }
      }
    } catch (e) {
      // Ignore error and retry
    }
    
    // Nếu chưa thấy file, đợi 500ms trước khi thử lại (Synchronous wait for config phase)
    if (attempt < maxAttempts - 1) {
      const waitTill = new Date(new Date().getTime() + 500);
      while (waitTill > new Date()) {
        // Busy wait
      }
    }
  }

  console.warn("📡 Playwright: Could not discover .web_port, falling back to 5173");
  return "5173";
}

const WEB_PORT = process.env.VITE_WEB_PORT || getWebPort();

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: "./tests",
  timeout: 120000,
  /* Run tests in files in parallel */
  fullyParallel: true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  /* Opt out of parallel tests on CI. */
  workers: process.env.CI ? 1 : undefined,
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [
    ['html', { open: 'never' }],
    ['line']
  ],
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
    use: {
      /* Base URL to use in actions like `await page.goto('/')`. */
      baseURL: process.env.PLAYWRIGHT_BASE_URL || `http://127.0.0.1:${WEB_PORT}`,
  
      /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
      trace: "on-first-retry",
    },
  
    // webServer: [
    //   {
    //     command: `PORT_FILE_NAME=.api_port_${process.env.TEST_WORKER_INDEX || '0'} cargo run`,
    //     cwd: '../apiserver',
    //     port: 0, // dynamic
    //     reuseExistingServer: !process.env.CI,
    //   },
    //   {
    //     command: `VITE_BACKEND_PORT_FILE=.api_port_${process.env.TEST_WORKER_INDEX || '0'} VITE_WEB_PORT_FILE=.web_port_${process.env.TEST_WORKER_INDEX || '0'} npm run dev`,
    //     port: 0, // dynamic
    //     reuseExistingServer: !process.env.CI,
    //   }
    // ],
  
    projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
