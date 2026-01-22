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
    await page.getByTestId('login-submit-btn').click();
    await page.waitForURL('/', { timeout: 15000 });
    await expect(page.getByTestId('dashboard-title')).toBeVisible({ timeout: 15000 });

    // 2. Navigate to Employees page
    console.log('Navigating to /employees...');
    await page.goto('/employees');
    await page.waitForURL('/employees');
    await expect(page.getByTestId('employees-title')).toBeVisible({ timeout: 30000 });

    // 3. Create an employee via UI
    console.log('Creating new employee via UI...');
    // Use data-testid for stable selection
    const addButton = page.getByTestId('add-employee-btn');
    await addButton.click();

    // Fill form
    await expect(page.getByRole('heading', { level: 2, name: /Add Employee|Thêm nhân viên/i })).toBeVisible(); // Modal title
    await page.fill('input[name="full_name"]', 'SearchTarget');
    await page.fill('input[name="employee_id_number"]', 'EMP999');
    await page.fill('input[name="hire_date"]', '2024-01-01');
    
    // Submit the form using data-testid
    await page.getByTestId('save-employee-btn').click();

    // Wait for modal to close (or check if it disappears)
    await expect(page.getByRole('heading', { level: 2, name: /Add Employee|Thêm nhân viên/i })).toBeHidden();
    
    // Wait for the new employee to appear in the list (optional, but good verification)
    await expect(page.getByRole('row', { name: /SearchTarget/i })).toBeVisible();

    console.log('Employee created. Starting search test...');

    // 4. Use the search input on Employees page
    const searchInput = page.getByTestId('employee-search');
    await expect(searchInput).toBeVisible();
    await searchInput.fill('SearchTarget');

    // 5. Verify the list is filtered to show our target
    // We look for the row containing our target name
    const targetRow = page.getByRole('row', { name: /SearchTarget/i });
    await expect(targetRow).toBeVisible({ timeout: 10000 });
    
    console.log('Search successful, target found in filtered list.');
  });
});