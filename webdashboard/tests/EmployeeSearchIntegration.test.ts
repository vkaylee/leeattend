import { expect } from '@playwright/test';
import { test } from './worker-orchestrator';

test.describe('Employee Search Integration', () => {
  test('should find and select an employee by name', async ({ page, workerWebUrl }) => {
    // 1. Login
    await page.goto(`${workerWebUrl}/login`);
    await page.fill('input[name="email"]', 'admin@leeattend.com');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL(`${workerWebUrl}/`, { timeout: 15000 });

    // 2. Navigate to Employees page
    // Clicking the "Nhân sự" or looking for the link
    await page.goto(`${workerWebUrl}/employees`);
    await expect(page.locator('h1')).toContainText(/Employees|Nhân sự/i);

    // 3. Find the EmployeeSearch component (it's inside the header area)
    const searchTrigger = page.locator('button[type="button"]').filter({ hasText: /Search employees|Tìm nhân viên/i });
    await expect(searchTrigger).toBeVisible();
    await searchTrigger.click();

    // 4. Type search term in Combobox input
    const comboboxInput = page.locator('input[placeholder*="Search"]');
    await comboboxInput.fill('Admin');

    // 5. Wait for the server-side result to appear in the list
    // The default admin usually has "Admin" in name
    const option = page.locator('[role="option"]').filter({ hasText: /Admin/i });
    await expect(option).toBeVisible();
    await option.click();

    // 6. Verify selection is reflected in the trigger
    await expect(searchTrigger).toContainText(/Admin/i);
  });
});
