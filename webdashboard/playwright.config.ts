import { defineConfig, devices } from "@playwright/test";

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: "./tests",
  timeout: 60000,
  expect: {
    timeout: 10000,
  },
  /* Run tests in files in parallel */
  fullyParallel: true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 1 : 0,
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
      // Orchestrator will override this per worker.
      baseURL: process.env.PLAYWRIGHT_BASE_URL || `http://127.0.0.1:5173`,
  
      /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
      trace: "on-first-retry",
    },
  
    projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});