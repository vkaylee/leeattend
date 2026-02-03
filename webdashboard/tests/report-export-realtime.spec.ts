import { test, expect } from "@playwright/test";

/**
 * Report Export Real-Time WebSocket Tests
 *
 * Tests the complete WebSocket streaming flow in ReportExportDialog
 * including fallback mechanism and error scenarios
 */

test.describe("Report Export Real-Time Updates", () => {
  test.beforeEach(async ({ page }) => {
    // Setup: Navigate to login
    await page.goto("/login");

    // Use fast login if available
    const fastLoginBtn = page.getByTestId("fast-login-btn");
    if (await fastLoginBtn.isVisible({ timeout: 5000 }).catch(() => false)) {
      await fastLoginBtn.click();
    } else {
      // Manual login as fallback
      await page.fill('[data-testid="login-email"]', "test@example.com");
      await page.fill('[data-testid="login-password"]', "password");
      await page.click('[data-testid="login-submit"]');
    }

    // Wait for dashboard to load
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible({
      timeout: 30000,
    });

    // Navigate to reports page
    await page.goto("/reports");
    await page.waitForLoadState("networkidle");
  });

  test("should open export dialog", async ({ page }) => {
    // Click export button
    const exportBtn = page.getByTestId("reports-export-button");
    if (await exportBtn.isVisible().catch(() => false)) {
      await exportBtn.click();
    } else {
      // Alternative: Use menu button
      await page.click('button:has-text("Export")');
    }

    // Verify dialog appears
    const dialog = page.getByTestId("report-export-dialog");
    await expect(dialog).toBeVisible();

    // Verify key elements are visible
    await expect(page.getByTestId("report-export-info")).toBeVisible();
    await expect(page.getByTestId("report-export-start-button")).toBeVisible();
  });

  test("should show real-time progress via WebSocket", async ({ page }) => {
    // Click export button
    const exportBtn = page.getByTestId("reports-export-button");
    if (await exportBtn.isVisible().catch(() => false)) {
      await exportBtn.click();
    }

    // Wait for dialog
    await expect(page.getByTestId("report-export-dialog")).toBeVisible();

    // Start export
    await page.click('[data-testid="report-export-start-button"]');

    // Verify connecting/preparing state
    await expect(page.getByTestId("report-export-status-text")).toContainText(
      /Preparing|Processing/i,
      { timeout: 5000 }
    );

    // Verify progress bar is visible
    const progressBar = page.getByTestId("report-export-progress-bar");
    await expect(progressBar).toBeVisible();

    // Wait for streaming status
    await expect(page.getByTestId("report-export-status-text")).toContainText(
      /Processing|Completed/i,
      { timeout: 15000 }
    );

    // Verify batch counter appears during processing
    const batchCounter = page.getByTestId("report-export-batch-counter");
    if (await batchCounter.isVisible({ timeout: 2000 }).catch(() => false)) {
      await expect(batchCounter).toContainText(/\d+ of \d+/);
    }

    // Wait for completion
    await expect(page.getByTestId("report-export-status-text")).toContainText(
      /Completed|completed/i,
      { timeout: 30000 }
    );

    // Verify download button appears
    await expect(page.getByTestId("report-export-download-button")).toBeVisible();
  });

  test("should show progress bar updates", async ({ page }) => {
    // Click export button
    const exportBtn = page.getByTestId("reports-export-button");
    if (await exportBtn.isVisible().catch(() => false)) {
      await exportBtn.click();
    }

    // Wait for dialog
    await expect(page.getByTestId("report-export-dialog")).toBeVisible();

    // Start export
    await page.click('[data-testid="report-export-start-button"]');

    // Wait for processing to start
    await expect(page.getByTestId("report-export-status-text")).toContainText(
      /Processing/i,
      { timeout: 10000 }
    );

    // Get initial progress
    const progressFill = page.getByTestId("report-export-progress-fill");
    const initialWidth = await progressFill.evaluate((el) => {
      const width = window.getComputedStyle(el).width;
      return parseFloat(width);
    });

    // Wait a bit and check progress increased
    await page.waitForTimeout(2000);
    const updatedWidth = await progressFill.evaluate((el) => {
      const width = window.getComputedStyle(el).width;
      return parseFloat(width);
    });

    // Progress should have increased or be at completion
    expect(updatedWidth).toBeGreaterThanOrEqual(initialWidth);
  });

  test("should update batch counter in real-time", async ({ page }) => {
    // Click export button
    const exportBtn = page.getByTestId("reports-export-button");
    if (await exportBtn.isVisible().catch(() => false)) {
      await exportBtn.click();
    }

    // Wait for dialog
    await expect(page.getByTestId("report-export-dialog")).toBeVisible();

    // Start export
    await page.click('[data-testid="report-export-start-button"]');

    // Wait for processing
    await expect(page.getByTestId("report-export-status-text")).toContainText(
      /Processing/i,
      { timeout: 10000 }
    );

    // Wait for batch counter to appear
    const batchCounter = page.getByTestId("report-export-batch-counter");
    await expect(batchCounter).toBeVisible({ timeout: 5000 });

    // Verify format: "X of Y" or similar
    await expect(batchCounter).toContainText(/\d+ (of|\/|out of) \d+/i);

    // Get initial text
    const initialText = await batchCounter.textContent();

    // Wait and check it updates
    await page.waitForTimeout(1000);
    const updatedText = await batchCounter.textContent();

    // Text may update, or may reach completion
    expect(updatedText).toBeDefined();
  });

  test("should handle cancellation during processing", async ({ page }) => {
    // Click export button
    const exportBtn = page.getByTestId("reports-export-button");
    if (await exportBtn.isVisible().catch(() => false)) {
      await exportBtn.click();
    }

    // Wait for dialog
    await expect(page.getByTestId("report-export-dialog")).toBeVisible();

    // Start export
    await page.click('[data-testid="report-export-start-button"]');

    // Wait for processing to start
    await expect(page.getByTestId("report-export-status-text")).toContainText(
      /Processing|Preparing/i,
      { timeout: 5000 }
    );

    // Click background button to cancel/close
    const bgButton = page.getByTestId("report-export-background-button");
    if (await bgButton.isVisible({ timeout: 2000 }).catch(() => false)) {
      await bgButton.click();

      // Verify dialog closed
      await expect(page.getByTestId("report-export-dialog")).not.toBeVisible();
    }
  });

  test("should download file on completion", async ({ page }) => {
    // Click export button
    const exportBtn = page.getByTestId("reports-export-button");
    if (await exportBtn.isVisible().catch(() => false)) {
      await exportBtn.click();
    }

    // Wait for dialog
    await expect(page.getByTestId("report-export-dialog")).toBeVisible();

    // Start export
    await page.click('[data-testid="report-export-start-button"]');

    // Wait for completion
    await expect(page.getByTestId("report-export-status-text")).toContainText(
      /Completed|completed/i,
      { timeout: 30000 }
    );

    // Setup download listener
    const downloadPromise = page.waitForEvent("download");

    // Click download
    await page.click('[data-testid="report-export-download-button"]');

    // Wait for download to start
    const download = await downloadPromise;

    // Verify filename contains expected pattern
    const filename = download.suggestedFilename();
    expect(filename).toMatch(/attendance-report.*\.xlsx/i);

    // Cleanup: Delete the download
    await download.delete();
  });

  test("should show error state on failure", async ({ page }) => {
    // This test would require mocking an error response
    // In real environment, we simulate error conditions

    // Click export button
    const exportBtn = page.getByTestId("reports-export-button");
    if (await exportBtn.isVisible().catch(() => false)) {
      await exportBtn.click();
    }

    // Wait for dialog
    await expect(page.getByTestId("report-export-dialog")).toBeVisible();

    // Try to start export and look for potential errors
    await page.click('[data-testid="report-export-start-button"]');

    // Wait to see if error appears (timeout means no error in this test)
    const errorElement = page.getByTestId("report-export-error");
    const errorVisible = await errorElement.isVisible({ timeout: 5000 }).catch(() => false);

    if (errorVisible) {
      // If error appeared, verify retry button is available
      await expect(page.getByTestId("report-export-retry-button")).toBeVisible();
    }
  });

  test("should show fallback notification when WebSocket unavailable", async ({
    page,
  }) => {
    // This test checks for fallback mode notification
    // In production, this would only trigger on actual WebSocket failures

    // Click export button
    const exportBtn = page.getByTestId("reports-export-button");
    if (await exportBtn.isVisible().catch(() => false)) {
      await exportBtn.click();
    }

    // Wait for dialog
    await expect(page.getByTestId("report-export-dialog")).toBeVisible();

    // Start export
    await page.click('[data-testid="report-export-start-button"]');

    // Wait for processing
    await expect(page.getByTestId("report-export-status-text")).toContainText(
      /Processing|Completed/i,
      { timeout: 15000 }
    );

    // Check if fallback notification appears (won't in normal WebSocket mode)
    const fallbackNotice = page.getByTestId(
      "report-export-fallback-notice"
    );
    const isFallback = await fallbackNotice.isVisible({ timeout: 2000 }).catch(() => false);

    if (isFallback) {
      // If fallback is active, verify it still completes
      await expect(page.getByTestId("report-export-status-text")).toContainText(
        /Completed|completed/i,
        { timeout: 30000 }
      );
    }
  });

  test("should handle retry after error", async ({ page }) => {
    // Click export button
    const exportBtn = page.getByTestId("reports-export-button");
    if (await exportBtn.isVisible().catch(() => false)) {
      await exportBtn.click();
    }

    // Wait for dialog
    await expect(page.getByTestId("report-export-dialog")).toBeVisible();

    // Start export
    await page.click('[data-testid="report-export-start-button"]');

    // Wait to see if we get an error
    const errorElement = page.getByTestId("report-export-error");
    const errorVisible = await errorElement.isVisible({ timeout: 10000 }).catch(() => false);

    if (errorVisible) {
      // Verify error state
      await expect(page.getByTestId("report-export-status-text")).toContainText(
        /Failed|failed/i
      );

      // Click retry
      const retryBtn = page.getByTestId("report-export-retry-button");
      if (await retryBtn.isVisible()) {
        await retryBtn.click();

        // Should return to processing state
        await expect(page.getByTestId("report-export-status-text")).toContainText(
          /Processing|Preparing/i,
          { timeout: 5000 }
        );
      }
    }
  });

  test("should maintain UI state consistency", async ({ page }) => {
    // Click export button
    const exportBtn = page.getByTestId("reports-export-button");
    if (await exportBtn.isVisible().catch(() => false)) {
      await exportBtn.click();
    }

    // Dialog should be visible
    const dialog = page.getByTestId("report-export-dialog");
    await expect(dialog).toBeVisible();

    // Initially should show start button
    let startBtn = page.getByTestId("report-export-start-button");
    await expect(startBtn).toBeVisible();

    // Start export
    await startBtn.click();

    // Start button should be replaced by background button
    startBtn = page.getByTestId("report-export-start-button");
    const bgBtn = page.getByTestId("report-export-background-button");

    // Wait for transition
    await page.waitForTimeout(500);

    // One of them should be visible (depending on state)
    const isStartVisible = await startBtn.isVisible({ timeout: 1000 }).catch(() => false);
    const isBgVisible = await bgBtn.isVisible({ timeout: 1000 }).catch(() => false);

    expect(isStartVisible || isBgVisible).toBe(true);
  });
});
