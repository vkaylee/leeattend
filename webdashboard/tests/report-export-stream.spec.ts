import { test, expect } from './worker-orchestrator';

/**
 * E2E Tests for Report Export Streaming
 */

test.describe('Report Export Streaming E2E', () => {
  test.setTimeout(120000); // 2 minutes for export operations

  test.beforeEach(async ({ page }) => {
    // Enable browser console logging for debugging
    page.on('console', message => { console.log(`[BROWSER] ${message.type().toUpperCase()}: ${message.text()}`); });
    page.on('pageerror', error => { console.log(`[BROWSER ERROR] ${error.message}`); });
    
    page.on('response', async response => {
      if (response.url().includes('/reports/export')) {
        const body = await response.text();
        console.log(`[NETWORK RESPONSE] ${response.url()}: ${response.status()} ${body}`);
      }
    });

    // Login first
    await page.goto('/login');
    await page.getByTestId('login-email-input').fill('admin@leeattend.com');
    await page.getByTestId('login-password-input').fill('admin123');
    await page.getByTestId('login-submit-btn').click();
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible({ timeout: 60000 });

    // Verify console capture is working
    await page.evaluate(() => { console.error("INJECTED_TEST_LOG_FOR_DEBUGGING"); });
  });

  test('should export report with progress tracking', async ({ page }) => {
    // Navigate to Reports page
    await page.goto('/reports');
    await expect(page.getByTestId('reports-page')).toBeVisible({ timeout: 10000 });

    // Open export dialog
    await page.getByTestId('export-report-btn').click();
    await expect(page.getByTestId('report-export-dialog')).toBeVisible({ timeout: 5000 });

    // Verify export configuration is displayed
    await expect(page.getByTestId('report-export-info')).toBeVisible();

    // Start export
    await page.getByTestId('report-export-start-button').click();

    // Wait for completion status
    const statusText = page.getByTestId('report-export-status-text');
    await expect(statusText).toContainText(/completed|success|Sẵn sàng/i, { timeout: 60000 });

    // Verify download button appears
    const downloadButton = page.getByTestId('report-export-download-button');
    await expect(downloadButton).toBeVisible({ timeout: 5000 });

    // Trigger download and verify
    const downloadPromise = page.waitForEvent('download', { timeout: 30000 });
    await downloadButton.click();
    const download = await downloadPromise;

    // Verify download filename
    expect(download.suggestedFilename()).toMatch(/attendance-report-.*\.xlsx$/);

    // Dialog doesn't automatically close after download in current implementation, it closes via onClose() in handleDownload
    await expect(page.getByTestId('report-export-dialog')).not.toBeVisible({ timeout: 10000 });
  });

  test('should allow running export in background', async ({ page }) => {
    await page.goto('/reports');
    await expect(page.getByTestId('reports-page')).toBeVisible({ timeout: 10000 });

    // Open export dialog
    await page.getByTestId('export-report-btn').click();

    // Start export
    await page.getByTestId('report-export-start-button').click();

    // Click "Run in Background" button
    const backgroundButton = page.getByTestId('report-export-background-button');
    await expect(backgroundButton).toBeVisible({ timeout: 10000 });
    await backgroundButton.click({ force: true });

    // Verify dialog closes
    await expect(page.getByTestId('report-export-dialog')).not.toBeVisible({ timeout: 5000 });
  });
});
