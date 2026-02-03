# Phase 6: E2E Tests

**Status**: pending
**Priority**: P2
**Effort**: 1.5h

---

## Context

Write end-to-end tests using Playwright to validate full user flow including WebSocket streaming and fallback.

---

## Requirements

### Functional
- Test complete export flow with WebSocket
- Test fallback to polling on WebSocket failure
- Test progress updates and completion
- Test error scenarios
- Test download functionality

### Non-Functional
- Tests run in CI/CD pipeline
- No flaky tests
- Clear failure messages
- Fast execution (< 30s)

---

## Related Files

**Create/Modify**:
- `webdashboard/tests/report-export-realtime.spec.ts`

**Reference**:
- Existing E2E tests in `webdashboard/tests/`
- Test IDs from Phase 4

---

## Test Structure

```typescript
// webdashboard/tests/report-export-realtime.spec.ts

import { test, expect } from "@playwright/test";

test.describe("Report Export Real-Time Updates", () => {
  test.beforeEach(async ({ page }) => {
    // Login and navigate to reports page
    await page.goto("/login");
    await page.fill('[data-testid="login-email"]', "test@example.com");
    await page.fill('[data-testid="login-password"]', "password");
    await page.click('[data-testid="login-submit"]');
    await expect(page).toHaveURL(/\/dashboard/);

    // Navigate to reports
    await page.goto("/reports");
  });

  // Test cases below
});
```

---

## Implementation Steps

### 1. Test: Open Export Dialog

```typescript
test("should open export dialog", async ({ page }) => {
  // Click export button
  await page.click('[data-testid="reports-export-button"]');

  // Verify dialog appears
  await expect(page.getByTestId("report-export-dialog")).toBeVisible();
  await expect(page.getByTestId("report-export-info")).toBeVisible();

  // Verify start button visible
  await expect(page.getByTestId("report-export-start-button")).toBeVisible();
});
```

### 2. Test: WebSocket Real-Time Updates

```typescript
test("should show real-time progress via WebSocket", async ({ page }) => {
  // Intercept WebSocket connection
  await page.route("**/api/ws/reports/**", (route) => {
    // Allow WebSocket upgrade
    route.continue();
  });

  // Open dialog and start export
  await page.click('[data-testid="reports-export-button"]');
  await page.click('[data-testid="report-export-start-button"]');

  // Verify connecting state
  await expect(page.getByTestId("report-export-status-text"))
    .toContainText("Preparing");

  // Wait for streaming state
  await expect(page.getByTestId("report-export-status-text"))
    .toContainText("Processing", { timeout: 5000 });

  // Verify progress bar updates
  const progressBar = page.getByTestId("report-export-progress-fill");
  await expect(progressBar).toHaveCSS("width", /[1-9]\d*%/);

  // Verify batch counter updates
  await expect(page.getByTestId("report-export-batch-counter"))
    .toBeVisible();

  // Wait for completion
  await expect(page.getByTestId("report-export-status-text"))
    .toContainText("Completed", { timeout: 30000 });

  // Verify download button appears
  await expect(page.getByTestId("report-export-download-button"))
    .toBeVisible();
});
```

### 3. Test: Fallback to Polling

```typescript
test("should fallback to polling when WebSocket fails", async ({ page }) => {
  // Block WebSocket connections
  await page.route("**/api/ws/reports/**", (route) => {
    route.abort("failed");
  });

  // Open dialog and start export
  await page.click('[data-testid="reports-export-button"]');
  await page.click('[data-testid="report-export-start-button"]');

  // Wait for fallback notification
  await expect(page.getByTestId("report-export-fallback-notice"))
    .toBeVisible({ timeout: 10000 });

  // Verify fallback message
  await expect(page.getByTestId("report-export-fallback-notice"))
    .toContainText("Real-time updates unavailable");

  // Verify polling continues (progress still updates)
  await expect(page.getByTestId("report-export-status-text"))
    .toContainText("Processing");

  // Wait for completion
  await expect(page.getByTestId("report-export-status-text"))
    .toContainText("Completed", { timeout: 30000 });
});
```

### 4. Test: Progress Bar Animation

```typescript
test("should animate progress bar smoothly", async ({ page }) => {
  await page.click('[data-testid="reports-export-button"]');
  await page.click('[data-testid="report-export-start-button"]');

  // Wait for processing
  await expect(page.getByTestId("report-export-status-text"))
    .toContainText("Processing");

  // Track progress updates
  const progressFill = page.getByTestId("report-export-progress-fill");
  const widths: string[] = [];

  // Sample width 5 times
  for (let i = 0; i < 5; i++) {
    const width = await progressFill.evaluate((el) =>
      window.getComputedStyle(el).width
    );
    widths.push(width);
    await page.waitForTimeout(500);
  }

  // Verify progress increases
  const numericWidths = widths.map((w) => parseFloat(w));
  expect(numericWidths[4]).toBeGreaterThan(numericWidths[0]);
});
```

### 5. Test: Batch Counter Updates

```typescript
test("should update batch counter in real-time", async ({ page }) => {
  await page.click('[data-testid="reports-export-button"]');
  await page.click('[data-testid="report-export-start-button"]');

  // Wait for processing
  await expect(page.getByTestId("report-export-status-text"))
    .toContainText("Processing");

  // Wait for batch counter
  const batchCounter = page.getByTestId("report-export-batch-counter");
  await expect(batchCounter).toBeVisible();

  // Verify format: "X of Y batches"
  await expect(batchCounter).toContainText(/\d+ of \d+/);

  // Wait a bit and verify it updates
  const initialText = await batchCounter.textContent();
  await page.waitForTimeout(2000);
  const updatedText = await batchCounter.textContent();

  expect(updatedText).not.toBe(initialText);
});
```

### 6. Test: Download File

```typescript
test("should download report file on completion", async ({ page }) => {
  await page.click('[data-testid="reports-export-button"]');
  await page.click('[data-testid="report-export-start-button"]');

  // Wait for completion
  await expect(page.getByTestId("report-export-status-text"))
    .toContainText("Completed", { timeout: 30000 });

  // Setup download listener
  const downloadPromise = page.waitForEvent("download");

  // Click download button
  await page.click('[data-testid="report-export-download-button"]');

  // Verify download started
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toMatch(/attendance-report-.*\.xlsx/);

  // Verify dialog closed
  await expect(page.getByTestId("report-export-dialog"))
    .not.toBeVisible();
});
```

### 7. Test: Cancel Export

```typescript
test("should allow canceling export", async ({ page }) => {
  await page.click('[data-testid="reports-export-button"]');
  await page.click('[data-testid="report-export-start-button"]');

  // Wait for processing
  await expect(page.getByTestId("report-export-status-text"))
    .toContainText("Processing");

  // Click background button (cancel)
  await page.click('[data-testid="report-export-background-button"]');

  // Verify dialog closed
  await expect(page.getByTestId("report-export-dialog"))
    .not.toBeVisible();
});
```

### 8. Test: Error Handling

```typescript
test("should handle export errors gracefully", async ({ page }) => {
  // Mock error response
  await page.route("**/api/workspaces/*/reports/export", (route) => {
    route.fulfill({
      status: 500,
      contentType: "application/json",
      body: JSON.stringify({ error: "Database connection failed" }),
    });
  });

  await page.click('[data-testid="reports-export-button"]');
  await page.click('[data-testid="report-export-start-button"]');

  // Verify error state
  await expect(page.getByTestId("report-export-status-text"))
    .toContainText("Failed");

  // Verify error message
  await expect(page.getByTestId("report-export-error"))
    .toBeVisible();
  await expect(page.getByTestId("report-export-error"))
    .toContainText("Database connection failed");

  // Verify retry button
  await expect(page.getByTestId("report-export-retry-button"))
    .toBeVisible();
});
```

### 9. Test: Retry After Failure

```typescript
test("should allow retry after failure", async ({ page }) => {
  let callCount = 0;

  // First call fails, second succeeds
  await page.route("**/api/workspaces/*/reports/export", (route) => {
    callCount++;
    if (callCount === 1) {
      route.fulfill({ status: 500 });
    } else {
      route.continue();
    }
  });

  await page.click('[data-testid="reports-export-button"]');
  await page.click('[data-testid="report-export-start-button"]');

  // Wait for error
  await expect(page.getByTestId("report-export-status-text"))
    .toContainText("Failed");

  // Click retry
  await page.click('[data-testid="report-export-retry-button"]');

  // Verify it retries
  await expect(page.getByTestId("report-export-status-text"))
    .toContainText("Processing");
});
```

### 10. Test: Background Processing

```typescript
test("should continue processing in background", async ({ page }) => {
  await page.click('[data-testid="reports-export-button"]');
  await page.click('[data-testid="report-export-start-button"]');

  // Wait for processing
  await expect(page.getByTestId("report-export-status-text"))
    .toContainText("Processing");

  // Close dialog
  await page.click('[data-testid="report-export-background-button"]');

  // Verify dialog closed
  await expect(page.getByTestId("report-export-dialog"))
    .not.toBeVisible();

  // Note: Background job continues on backend
  // Could verify via polling job status API if needed
});
```

---

## Test Data Setup

**Prerequisites**:
- Test workspace with employees
- Date range with attendance data
- Valid user session

**Mock Backend** (if needed):
```typescript
test.beforeEach(async ({ page }) => {
  // Mock successful job creation
  await page.route("**/api/workspaces/*/reports/export", (route) => {
    route.fulfill({
      status: 202,
      contentType: "application/json",
      body: JSON.stringify({
        job_id: "test-job-123",
        total_batches: 4,
      }),
    });
  });
});
```

---

## Todo List

- [ ] Create E2E test file
- [ ] Setup test authentication
- [ ] Test: Open dialog
- [ ] Test: WebSocket real-time updates
- [ ] Test: Fallback to polling
- [ ] Test: Progress bar animation
- [ ] Test: Batch counter updates
- [ ] Test: Download file
- [ ] Test: Cancel export
- [ ] Test: Error handling
- [ ] Test: Retry after failure
- [ ] Test: Background processing
- [ ] Run tests: `npm run test:e2e -- report-export-realtime`
- [ ] Verify all tests pass

---

## Success Criteria

✅ All E2E tests pass
✅ WebSocket streaming validated
✅ Fallback mechanism validated
✅ No flaky tests
✅ Tests run in < 30s

---

## Running Tests

```bash
# Run E2E tests
npm run test:e2e -- report-export-realtime.spec.ts

# Run with UI mode (debug)
npm run test:e2e -- --ui report-export-realtime.spec.ts

# Run specific test
npm run test:e2e -- -g "should show real-time progress"

# Run in headed mode (see browser)
npm run test:e2e -- --headed report-export-realtime.spec.ts
```

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| WebSocket timing issues | Medium | Use proper timeout values |
| Flaky progress checks | Medium | Sample multiple times |
| Test data dependencies | Low | Mock backend responses |
| CI/CD environment issues | Medium | Test locally first |

---

## CI/CD Integration

Add to GitHub Actions workflow:
```yaml
- name: Run E2E tests
  run: npm run test:e2e -- report-export-realtime.spec.ts
  env:
    CI: true
```

---

## Security Considerations

- Use test credentials (not production)
- Clean up test data after runs
- No sensitive data in test fixtures

---

## Next Steps

→ Phase 7: Validation & code review
