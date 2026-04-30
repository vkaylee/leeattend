import { defineConfig, _devices } from "@playwright/test";

import config from "./playwright.config";

export default defineConfig({
    ...config,
    webServer: undefined,
    use: {
        ...config.use,
        baseURL: process.env.PLAYWRIGHT_BASE_URL || "http://localhost:8080",
    },
    reporter: [["list"]],
    outputDir: "test-results-new",
});
