// @ts-nocheck
import { defineConfig, devices } from "@playwright/test";

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: "./tests",
  timeout: 120 * 1000,
  globalTimeout: 10 * 60 * 1000,
  globalSetup: './tests/global-setup',
  expect: {
    timeout: 5000,
  },

  /* Run tests in files in parallel */
  fullyParallel: true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry for stability */
  retries: 1,
  /* Opt out of parallel tests on CI. */
  workers: 2,
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [
    ['html', { open: 'never' }],
    ['line']
  ],
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
    use: {
      /* Base URL to use in actions like `await page.goto('/')`. */
      baseURL: process.env.PLAYWRIGHT_BASE_URL ?? `http://127.0.0.1:5173`,
  
      /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
      trace: "on-first-retry",
      storageState: 'playwright/.auth/user.json',
    },

    /* Run your local dev server before starting the tests */
    webServer: {
      command: 'bun run build --mode test && bun run preview --port 5173 --host 127.0.0.1 --mode test',
      url: 'http://127.0.0.1:5173',
      reuseExistingServer: true,
      stdout: 'pipe',
      stderr: 'pipe',
      timeout: 180 * 1000, // 3 minutes max to start dev server
    },
  
    projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});