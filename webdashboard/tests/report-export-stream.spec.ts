import { test, expect } from './worker-orchestrator';

/**
 * E2E Tests for Report Export Streaming
 *
 * Current Implementation: Polling-based export (ReportExportDialog.tsx)
 * Future Implementation: WebSocket-based streaming (use-report-stream.ts)
 *
 * Test Coverage:
 * - Happy path export flow
 * - Progress tracking
 * - Error handling and retry
 * - Download verification
 * - WebSocket fallback (when implemented)
 */

test.describe.skip('Report Export Streaming E2E', () => {
  test.setTimeout(120000); // 2 minutes for export operations

  test.beforeEach(async ({ page }) => {
    // Enable browser console logging for debugging
    page.on('console', msg => console.log(`[BROWSER] ${msg.type().toUpperCase()}: ${msg.text()}`));
    page.on('pageerror', err => console.log(`[BROWSER ERROR] ${err.message}`));

    // Login first
    await page.goto('/login');
    await page.getByTestId('fast-login-btn').click();
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible({ timeout: 60000 });
  });

  // TODO: Skipped until Reports page is implemented with export button
  test('should export report with progress tracking via polling', async ({ page }) => {
    // Navigate to Reports page
    await page.goto('/reports');
    await expect(page.getByTestId('reports-page')).toBeVisible({ timeout: 10000 });

    // Open export dialog
    await page.getByTestId('export-report-btn').click();
    await expect(page.getByTestId('export-dialog')).toBeVisible({ timeout: 5000 });

    // Verify export configuration is displayed
    await expect(page.getByTestId('export-date-range')).toBeVisible();
    await expect(page.getByTestId('export-report-type')).toBeVisible();

    // Start export
    await page.getByTestId('start-export-btn').click();

    // Verify progress bar appears
    const progressBar = page.locator('.h-4.w-full.bg-slate-100'); // Progress container
    await expect(progressBar).toBeVisible({ timeout: 5000 });

    // Verify status changes to "processing"
    await expect(page.locator('text=/processing|preparing/i')).toBeVisible({ timeout: 5000 });

    // Wait for completion status
    await expect(page.locator('text=/completed|success/i')).toBeVisible({ timeout: 60000 });

    // Verify download button appears
    const downloadBtn = page.getByTestId('download-report-btn');
    await expect(downloadBtn).toBeVisible({ timeout: 5000 });

    // Trigger download and verify
    const downloadPromise = page.waitForEvent('download', { timeout: 30000 });
    await downloadBtn.click();
    const download = await downloadPromise;

    // Verify download filename
    expect(download.suggestedFilename()).toMatch(/attendance-report-.*\.xlsx$/);

    // Verify dialog closes after download
    await expect(page.getByTestId('export-dialog')).not.toBeVisible({ timeout: 10000 });
  });

  // TODO: Skipped until Reports page is implemented
  test('should display batch progress during export', async ({ page }) => {
    await page.goto('/reports');
    await expect(page.getByTestId('reports-page')).toBeVisible({ timeout: 10000 });

    // Open export dialog
    await page.getByTestId('export-report-btn').click();

    // Start export
    await page.getByTestId('start-export-btn').click();

    // Wait for progress to appear
    await expect(page.locator('text=/processing/i')).toBeVisible({ timeout: 5000 });

    // Verify batch progress text appears
    // Format: "Batches completed: X / Y" or similar
    const batchProgress = page.locator('text=/batch|completed|total/i');

    // Check if batch progress updates (at least visible)
    await expect(batchProgress).toBeVisible({ timeout: 10000 });

    // Verify progress bar percentage increases
    const progressBar = page.locator('.h-4.w-full.bg-slate-100 > div').first();
    const initialWidth = await progressBar.evaluate(el => el.style.width);

    // Wait a moment for progress update
    await page.waitForTimeout(2000);

    const updatedWidth = await progressBar.evaluate(el => el.style.width);

    // Progress should increase (comparing width percentages)
    expect(updatedWidth).not.toBe(initialWidth);
  });

  // TODO: Skipped until Reports page is implemented
  test('should handle export errors and show retry option', async ({ page }) => {
    // Intercept export job creation to simulate error
    await page.route('**/api/v1/workspaces/*/reports/export/jobs', route => {
      route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'Simulated export error' }),
      });
    });

    await page.goto('/reports');
    await expect(page.getByTestId('reports-page')).toBeVisible({ timeout: 10000 });

    // Open export dialog
    await page.getByTestId('export-report-btn').click();

    // Start export
    await page.getByTestId('start-export-btn').click();

    // Verify error state appears
    await expect(page.locator('text=/failed|error/i')).toBeVisible({ timeout: 15000 });

    // Verify error message is displayed
    const errorMessage = page.locator('.bg-rose-50, .text-rose-700, [class*="error"]');
    await expect(errorMessage).toBeVisible({ timeout: 5000 });

    // Verify retry button is available
    const retryBtn = page.getByTestId('retry-export-btn').or(page.locator('button:has-text("Retry")'));
    await expect(retryBtn).toBeVisible({ timeout: 5000 });

    // Verify close button is available
    const closeBtn = page.getByTestId('close-dialog-btn').or(page.locator('button:has-text("Close")'));
    await expect(closeBtn).toBeVisible({ timeout: 5000 });
  });

  // TODO: Skipped until Reports page is implemented
  test('should allow running export in background', async ({ page }) => {
    await page.goto('/reports');
    await expect(page.getByTestId('reports-page')).toBeVisible({ timeout: 10000 });

    // Open export dialog
    await page.getByTestId('export-report-btn').click();

    // Start export
    await page.getByTestId('start-export-btn').click();

    // Wait for processing to start
    await expect(page.locator('text=/processing|preparing/i')).toBeVisible({ timeout: 5000 });

    // Click "Run in Background" button
    const backgroundBtn = page.locator('button:has-text(/run.*background/i)');
    await expect(backgroundBtn).toBeVisible({ timeout: 5000 });
    await backgroundBtn.click();

    // Verify dialog closes
    await expect(page.getByTestId('export-dialog')).not.toBeVisible({ timeout: 5000 });

    // TODO: Verify background job notification or status indicator
    // This would require additional UI components for job tracking
  });

  // TODO: Skipped until WebSocket streaming is implemented in UI
  test('should stream report data via WebSocket (future implementation)', async ({ page }) => {
    // This test covers the use-report-stream.ts hook when integrated

    await page.goto('/reports');
    await expect(page.getByTestId('reports-page')).toBeVisible({ timeout: 10000 });

    // Listen for WebSocket connections
    const wsConnections: unknown[] = [];
    page.on('websocket', ws => {
      console.log(`[WS] WebSocket connection: ${ws.url()}`);
      wsConnections.push(ws);

      ws.on('framereceived', event => {
        console.log(`[WS] Received: ${event.payload}`);
      });

      ws.on('framesent', event => {
        console.log(`[WS] Sent: ${event.payload}`);
      });
    });

    // Open export dialog
    await page.getByTestId('export-report-btn').click();

    // Start export with streaming
    await page.getByTestId('start-export-btn').click();

    // Verify WebSocket connection is established
    await page.waitForTimeout(2000);
    expect(wsConnections.length).toBeGreaterThan(0);

    // Verify WebSocket URL pattern
    const wsUrl = (wsConnections[0] as { url: () => string }).url();
    expect(wsUrl).toMatch(/ws:\/\/.*\/api\/ws\/reports\/.+/);

    // Verify streaming status indicator
    await expect(page.getByTestId('streaming-status')).toContainText(/streaming|connected/i);

    // Verify chunk progress updates
    const progressText = page.getByTestId('streaming-progress');
    await expect(progressText).toBeVisible({ timeout: 5000 });

    // Wait for completion
    await expect(page.locator('text=/completed|done/i')).toBeVisible({ timeout: 60000 });

    // Verify download becomes available
    await expect(page.getByTestId('download-report-btn')).toBeVisible({ timeout: 5000 });
  });

  // TODO: Skipped until WebSocket streaming is implemented
  test('should fallback to polling when WebSocket fails', async ({ page }) => {
    // Block WebSocket connections
    await page.route('**/api/ws/**', route => {
      route.abort('failed');
    });

    await page.goto('/reports');
    await expect(page.getByTestId('reports-page')).toBeVisible({ timeout: 10000 });

    // Open export dialog
    await page.getByTestId('export-report-btn').click();

    // Start export
    await page.getByTestId('start-export-btn').click();

    // Verify fallback to polling (no WebSocket connection indicator)
    await expect(page.locator('text=/processing|preparing/i')).toBeVisible({ timeout: 10000 });

    // Verify polling-based progress still works
    const progressBar = page.locator('.h-4.w-full.bg-slate-100 > div').first();
    await expect(progressBar).toBeVisible({ timeout: 5000 });

    // Verify completion works via polling
    await expect(page.locator('text=/completed|success/i')).toBeVisible({ timeout: 60000 });
  });

  // TODO: Skipped until WebSocket streaming is implemented
  test('should handle WebSocket reconnection after disconnect', async ({ page }) => {
    let wsConnection: unknown;

    page.on('websocket', ws => {
      if (ws.url().includes('/api/ws/reports/')) {
        wsConnection = ws;
      }
    });

    await page.goto('/reports');
    await expect(page.getByTestId('reports-page')).toBeVisible({ timeout: 10000 });

    // Open export dialog
    await page.getByTestId('export-report-btn').click();

    // Start export
    await page.getByTestId('start-export-btn').click();

    // Wait for WebSocket connection
    await page.waitForTimeout(2000);
    expect(wsConnection).toBeDefined();

    // Simulate disconnect
    await page.evaluate(() => {
      // Close WebSocket connection from client side
      const ws = (window as unknown as { __reportWebSocket?: { close: () => void } }).__reportWebSocket;
      if (ws) ws.close();
    });

    // Verify reconnection attempt indicator
    await expect(page.locator('text=/reconnect|connecting/i')).toBeVisible({ timeout: 5000 });

    // Verify stream continues after reconnect
    await expect(page.locator('text=/streaming|processing/i')).toBeVisible({ timeout: 15000 });
  });

  // TODO: Skipped until Reports page is implemented
  test('should cancel export job on dialog close during processing', async ({ page }) => {
    await page.goto('/reports');
    await expect(page.getByTestId('reports-page')).toBeVisible({ timeout: 10000 });

    // Track DELETE requests for job cancellation
    const deleteRequests: string[] = [];
    page.on('request', request => {
      if (request.method() === 'DELETE' && request.url().includes('/export/jobs/')) {
        deleteRequests.push(request.url());
      }
    });

    // Open export dialog
    await page.getByTestId('export-report-btn').click();

    // Start export
    await page.getByTestId('start-export-btn').click();

    // Wait for processing to start
    await expect(page.locator('text=/processing/i')).toBeVisible({ timeout: 5000 });

    // Close dialog while processing
    const closeBtn = page.locator('button:has-text(/close|cancel/i)').first();
    await closeBtn.click();

    // Verify dialog closes
    await expect(page.getByTestId('export-dialog')).not.toBeVisible({ timeout: 5000 });

    // TODO: Verify job cancellation request was sent
    // This depends on whether the backend supports job cancellation
    // expect(deleteRequests.length).toBeGreaterThan(0);
  });

  // TODO: Skipped until UI components have data-testid attributes
  test('should display correct report metadata in export dialog', async ({ page }) => {
    await page.goto('/reports');
    await expect(page.getByTestId('reports-page')).toBeVisible({ timeout: 10000 });

    // Set filters before opening export
    const startDate = '2026-01-01';
    const endDate = '2026-01-31';

    // Fill date filters (assuming date pickers exist)
    await page.getByTestId('filter-start-date').fill(startDate);
    await page.getByTestId('filter-end-date').fill(endDate);

    // Select report type
    await page.getByTestId('report-type-summary').click();

    // Open export dialog
    await page.getByTestId('export-report-btn').click();

    // Verify dialog displays correct metadata
    await expect(page.getByTestId('export-date-range')).toContainText(startDate);
    await expect(page.getByTestId('export-date-range')).toContainText(endDate);
    await expect(page.getByTestId('export-report-type')).toContainText('summary');
  });

  // TODO: Skipped until employee filter UI is implemented
  test('should export filtered employee subset', async ({ page }) => {
    await page.goto('/reports');
    await expect(page.getByTestId('reports-page')).toBeVisible({ timeout: 10000 });

    // Select specific employees
    await page.getByTestId('employee-filter-dropdown').click();
    await page.getByTestId('employee-filter-option-1').click();
    await page.getByTestId('employee-filter-option-2').click();
    await page.getByTestId('employee-filter-apply').click();

    // Open export dialog
    await page.getByTestId('export-report-btn').click();

    // Verify employee count in dialog
    await expect(page.getByTestId('export-employee-count')).toContainText('2');

    // Start export
    await page.getByTestId('start-export-btn').click();

    // Verify export completes
    await expect(page.locator('text=/completed/i')).toBeVisible({ timeout: 60000 });

    // Download and verify
    const downloadPromise = page.waitForEvent('download', { timeout: 30000 });
    await page.getByTestId('download-report-btn').click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/attendance-.*\.xlsx$/);
  });
});

/**
 * Integration Test Notes:
 *
 * CURRENT IMPLEMENTATION (Polling):
 * - ReportExportDialog uses createReportExportJob() to start export
 * - Polls fetchExportJobStatus() every 1 second
 * - Shows progress.percentage and progress.completed_batches
 * - Downloads blob via downloadExportReport() on completion
 *
 * FUTURE IMPLEMENTATION (WebSocket Streaming):
 * - use-report-stream.ts hook establishes WebSocket connection
 * - Receives CHUNK messages with batch_index/total_batches
 * - Calculates progress percentage in real-time
 * - Uses exponential backoff for reconnection
 * - Accumulates chunks and triggers onComplete callback
 *
 * REQUIRED UI COMPONENTS (Not Yet Implemented):
 * - Reports page with export button
 * - Date range filters
 * - Report type selector (summary/detail)
 * - Employee filter dropdown
 * - Progress indicators with data-testid attributes
 * - WebSocket connection status indicator
 *
 * REQUIRED data-testid ATTRIBUTES:
 * - reports-page: Main reports container
 * - export-report-btn: Button to open export dialog
 * - export-dialog: Modal container
 * - export-date-range: Date range display
 * - export-report-type: Report type display
 * - export-employee-count: Selected employee count
 * - start-export-btn: Button to start export
 * - download-report-btn: Button to download completed report
 * - retry-export-btn: Button to retry failed export
 * - close-dialog-btn: Button to close dialog
 * - streaming-status: WebSocket connection status
 * - streaming-progress: Chunk/batch progress text
 *
 * UNRESOLVED QUESTIONS:
 * 1. Should we support job cancellation via DELETE endpoint?
 * 2. Should background jobs show persistent notification/status?
 * 3. What's the UX for viewing completed background exports?
 * 4. Should we implement client-side Excel generation (excel-builder.ts)?
 * 5. When will WebSocket streaming replace polling in the UI?
 */
