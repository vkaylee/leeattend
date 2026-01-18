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
    await page.getByRole('button', { name: /Login|Đăng nhập/i }).click();
    await page.waitForURL('/', { timeout: 15000 });
    await expect(page.getByRole('heading', { level: 1, name: /Dashboard/i })).toBeVisible({ timeout: 15000 });

    // 2. Navigate to Employees page
    console.log('Navigating to /employees...');
    await page.goto('/employees');
    await expect(page.getByRole('heading', { level: 1, name: /Employees|Nhân sự|Nhân viên/i })).toBeVisible({ timeout: 15000 });

    // 3. Create an employee via UI
    console.log('Creating new employee via UI...');
    // Use data-testid for stable selection
    const addButton = page.getByTestId('add-employee-btn');
    await addButton.click();

    // Fill form
    await expect(page.getByRole('heading', { level: 2, name: /Add Employee|Thêm nhân viên/i })).toBeVisible(); // Modal title
    await page.fill('input[name="last_name"]', 'SearchTarget');
    await page.fill('input[name="first_name"]', 'Test');
    await page.fill('input[name="employee_id_number"]', 'EMP999');
    await page.fill('input[name="hire_date"]', '2024-01-01');
    
    // Submit the form using data-testid
    await page.getByTestId('save-employee-btn').click();

    // Wait for modal to close (or check if it disappears)
    await expect(page.getByRole('heading', { level: 2, name: /Add Employee|Thêm nhân viên/i })).toBeHidden();
    
    // Wait for the new employee to appear in the list (optional, but good verification)
    await expect(page.getByRole('row', { name: /SearchTarget/i })).toBeVisible();

    console.log('Employee created. Starting search test...');

    // 4. Find the EmployeeSearch component (header area)
    const searchTrigger = page.getByTestId('employee-search');
    await expect(searchTrigger).toBeVisible();
    await searchTrigger.click();
    await page.waitForTimeout(500); // Wait for popover animation

    // 5. Type search term in Combobox input
    // The input is inside the popover triggered by getByTestId('employee-search')
    const comboboxInput = page.getByPlaceholder(/Search|Tìm/i);
    await comboboxInput.fill('SearchTarget');

    // 6. Wait for the server-side result to appear in the list
    // Combobox generates data-testid for options as `${testId}-option-${option.value}`
    // Since we don't know the exact ID here (EMP999 is just a number), we can still use role option
    // but now we have a more stable trigger.
    const option = page.getByRole('option', { name: /SearchTarget/i });
    await expect(option).toBeVisible({ timeout: 10000 });
    await option.click();

    // 7. Verify selection is reflected in the trigger
    await expect(page.getByRole('button', { name: /SearchTarget/i })).toBeVisible();
  });
});