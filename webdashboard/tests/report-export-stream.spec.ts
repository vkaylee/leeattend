import { test, expect } from './worker-orchestrator';

test.describe('Synchronous Report Export E2E', () => {
  test.setTimeout(120000);

  test.beforeEach(async ({ authPage, page }) => {
    expect(authPage).toBeDefined();
    page.on('console', _message => { /* empty */ });
    page.on('pageerror', _error => { /* empty */ });
  });

  test('downloads XLSX directly from export request', async ({ page }) => {
    await page.goto('/reports');
    await expect(page.getByTestId('reports-page')).toBeVisible({ timeout: 10000 });
    await page.getByTestId('export-report-btn').click();
    await expect(page.getByTestId('report-export-dialog')).toBeVisible({ timeout: 5000 });
    await expect(page.getByTestId('report-export-info')).toBeVisible();

    const downloadPromise = page.waitForEvent('download', { timeout: 60000 });
    await page.getByTestId('report-export-start-button').click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/attendance-report-.*\.xlsx$/);
    await expect(page.getByTestId('report-export-dialog')).not.toBeVisible({ timeout: 10000 });
  });
});
