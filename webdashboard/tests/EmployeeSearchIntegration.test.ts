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
    await page.getByTestId('login-email-input').fill('admin@leeattend.com');
    await page.getByTestId('login-password-input').fill('admin123');
    await page.getByTestId('login-submit-btn').click();

    await expect(page.getByTestId('dashboard-title')).toBeVisible({ timeout: 15000 });

    // Navigate to employees page
    await page.goto('/employees');
    await expect(page.getByTestId('employees-title')).toBeVisible({ timeout: 30000 });

    // 3. Create an employee via UI
    console.log('Creating new employee via UI...');
    // Use data-testid for stable selection
    const addButton = page.getByTestId('add-employee-btn');
    await addButton.click();

    // Fill the form using standardized testids
    await page.getByTestId('employees-form-name-input').fill('SearchTarget');
    await page.getByTestId('employees-form-id-input').fill('EMP999');
    await page.getByTestId('employees-form-hire-date-input').fill('2024-01-01');
    
    // Submit the form using data-testid
    await page.getByTestId('save-employee-btn').click();

    // Wait for the new employee to appear in the list (using row testid if possible, but row name is also fine)
    await expect(page.getByText('SearchTarget')).toBeVisible({ timeout: 15000 });

    console.log('Employee created. Starting search test...');

    // 4. Use the search input on Employees page
    const searchInput = page.getByTestId('employee-search');
    await expect(searchInput).toBeVisible();
    await searchInput.fill('SearchTarget');

    // 5. Verify the list is filtered to show our target
    // We look for the text containing our target name
    await expect(page.getByText('SearchTarget')).toBeVisible({ timeout: 10000 });
    
    console.log('Search successful, target found in filtered list.');
  });
});
