// @ts-nocheck
import { createRequire } from "node:module";

import { defineConfig, devices } from "@playwright/test";

const require = createRequire(import.meta.url);
const viewportManifest = require("../config/ui-viewports.json");

const viewportByName = new Map(
  viewportManifest.supported.map((viewport) => [viewport.name, viewport]),
);
const getViewport = (name: string) => {
  const viewport = viewportByName.get(name);
  if (!viewport) {
    throw new Error(`Unknown viewport: ${name}`);
  }
  return viewport;
};
const baselineViewport = getViewport(viewportManifest.baseline);
const viewportSize = (name: string) => {
  const { width, height } = getViewport(name);
  return { width, height };
};

const chromiumDevice = {
  ...devices["Desktop Chrome"],
  viewport: viewportSize(baselineViewport.name),
};
const responsiveProjects = viewportManifest.supported.map((viewport) => ({
  name: `responsive-${viewport.name}`,
  testMatch: /responsive\.spec\.ts/,
  use: { ...chromiumDevice, viewport: viewportSize(viewport.name) },
  metadata: { viewport: viewport.name, category: viewport.category },
}));

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
      command: 'bun run build:test && bun run preview --port 5173 --host 127.0.0.1',
      url: 'http://127.0.0.1:5173',
      reuseExistingServer: false,
      stdout: 'pipe',
      stderr: 'pipe',
      timeout: 180 * 1000, // 3 minutes max to start dev server
    },
  
    projects: [
    {
      name: "chromium",
      testIgnore: /dashboard\.spec\.ts|responsive\.spec\.ts/,
      use: chromiumDevice,
      metadata: { viewport: baselineViewport.name, category: baselineViewport.category },
    },
    {
      name: "dashboard-desktop",
      testMatch: /dashboard\.spec\.ts/,
      use: chromiumDevice,
      metadata: { viewport: baselineViewport.name, category: baselineViewport.category },
    },
    {
      name: "dashboard-tablet",
      testMatch: /dashboard\.spec\.ts/,
      use: { ...chromiumDevice, viewport: viewportSize("tablet-portrait") },
      metadata: { viewport: "tablet-portrait", category: "tablet" },
    },
    {
      name: "dashboard-mobile",
      testMatch: /dashboard\.spec\.ts/,
      use: { ...devices["Pixel 7"], viewport: viewportSize("mobile") },
      metadata: { viewport: "mobile", category: "mobile" },
    },
    ...responsiveProjects,
  ],
});