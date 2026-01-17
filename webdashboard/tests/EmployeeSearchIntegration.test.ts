import { expect } from '@playwright/test';
import { test } from './worker-orchestrator';

test.describe('Employee Search Integration', () => {
  test('should find and select an employee by name', async ({ page }) => {
    test.setTimeout(60000);

    // Capture console messages and errors
    page.on('console', msg => console.log(`[BROWSER] ${msg.type().toUpperCase()}: ${msg.text()}`));
    page.on('pageerror', err => console.log(`[BROWSER ERROR] ${err.message}`));

    console.log('--- Starting Employee Search Integration Test ---');

    // 1. Login
    await page.goto('/login');
    await page.fill('input[name="email"]', 'admin@leeattend.com');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/', { timeout: 15000 });
    await expect(page.locator('h1').filter({ hasText: /Dashboard/i })).toBeVisible({ timeout: 15000 });

    // 2. Navigate to Employees page
    console.log('Navigating to /employees...');
    await page.goto('/employees');
    await expect(page.locator('h1').filter({ hasText: /Employees|Nhân sự|Nhân viên/i })).toBeVisible({ timeout: 15000 });

    // 3. Create an employee via UI
    console.log('Creating new employee via UI...');
    // Click "Add Employee" button (matches "Add Employee" or "Thêm nhân viên")
    const addButton = page.locator('button').filter({ hasText: /Add Employee|Thêm nhân viên/i });
    await addButton.click();

    // Fill form
    await expect(page.locator('h2').filter({ hasText: /Add Employee|Thêm nhân viên/i })).toBeVisible(); // Modal title
    await page.fill('input[name="last_name"]', 'SearchTarget');
    await page.fill('input[name="first_name"]', 'Test');
    await page.fill('input[name="employee_id_number"]', 'EMP999');
    await page.fill('input[name="hire_date"]', '2024-01-01');
    
    // Submit
    const saveButton = page.locator('button[type="submit"]').filter({ hasText: /Save|Lưu/i });
    await saveButton.click();

    // Wait for modal to close (or check if it disappears)
    await expect(page.locator('h2').filter({ hasText: /Add Employee|Thêm nhân viên/i })).not.toBeVisible();
    
    // Wait for the new employee to appear in the list (optional, but good verification)
    await expect(page.locator('table').filter({ hasText: /SearchTarget/i })).toBeVisible();

    console.log('Employee created. Starting search test...');

    // 4. Find the EmployeeSearch component (header area)
    const searchTrigger = page.locator('button[type="button"]').filter({ hasText: /Search|Tìm/i });
    await expect(searchTrigger).toBeVisible();
    await searchTrigger.click();
    await page.waitForTimeout(500); // Wait for popover animation

    // 5. Type search term in Combobox input
    const comboboxInput = page.locator('input[placeholder*="Search"], input[placeholder*="Tìm"]');
    await comboboxInput.fill('SearchTarget');

    // 6. Wait for the server-side result to appear in the list
    const option = page.locator('[role="option"]').filter({ hasText: /SearchTarget/i });
    await expect(option).toBeVisible({ timeout: 10000 });
    await option.click();

    // 7. Verify selection is reflected in the trigger
    await expect(page.locator('button[type="button"]').filter({ hasText: /SearchTarget/i })).toBeVisible();
  });
});