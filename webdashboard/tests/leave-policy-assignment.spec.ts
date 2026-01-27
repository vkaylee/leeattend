import { test, expect } from './worker-orchestrator';

test.describe('Leave Policy Assignment Integration', () => {
  test('should populate assignment modal with employees and departments', async ({ page }) => {
    test.setTimeout(180000);
    
    // Capture console messages and errors
    page.on('console', msg => console.log(`[BROWSER] ${msg.type().toUpperCase()}: ${msg.text()}`));
    page.on('pageerror', err => console.log(`[BROWSER ERROR] ${err.message}`));

    console.log('--- Starting Leave Policy Assignment Integration Test ---');
    
    // 1. Login
    await page.goto('/login');
    await page.getByTestId('fast-login-btn').click();
    
    console.log('Waiting for Dashboard heading...');
    await expect(page.getByRole('heading', { level: 1, name: /Dashboard|Bảng điều khiển/i })).toBeVisible({ timeout: 30000 });

    // 2. Create an Employee
    console.log('Navigating to Employees...');
    await page.goto('/employees');
    await expect(page.getByTestId('employees-title')).toBeVisible({ timeout: 15000 });
    
    const addEmpBtn = page.getByTestId('add-employee-btn');
    await expect(addEmpBtn).toBeEnabled({ timeout: 15000 });
    await addEmpBtn.click({ force: true });
    
    // Wait for form input directly
    console.log('Waiting for Add Employee form...');
    const nameInput = page.getByTestId('employees-form-name-input');
    await expect(nameInput).toBeVisible({ timeout: 30000 });

    const uniqueId = `INT-${Date.now()}`;
    const fullName = 'Alice Doe';
    await nameInput.fill('Alice Doe');
    
    await page.getByTestId('employees-form-id-input').fill(uniqueId);
    await page.getByTestId('employees-form-hire-date-input').fill('2026-01-01');
    await page.getByTestId('save-employee-btn').click();
    
    // Wait for row to appear
    await expect(page.getByRole('row', { name: fullName })).toBeVisible({ timeout: 30000 });
    console.log('Employee created.');

    // 3. Navigate to Leave Settings
    console.log('Navigating to Leave Settings...');
    await page.goto('/workspace/settings/leave');
    await expect(page.getByRole('heading', { level: 2, name: /Leave Settings|Cài đặt nghỉ phép/i })).toBeVisible({ timeout: 15000 });

    // 4. Switch to Policies Tab and create a policy
    await page.getByTestId('leave-tab-policies').click();
    const addPolicyBtn = page.getByTestId('add-policy-btn');
    await expect(addPolicyBtn).toBeVisible({ timeout: 15000 });
    await addPolicyBtn.click();

    const policyName = `Policy-${Date.now()}`;
    await page.fill('input[id="policy-name"]', policyName);
    await page.getByTestId('save-policy-btn').click();
    await expect(page.getByRole('heading', { name: policyName })).toBeVisible({ timeout: 15000 });
    console.log('Policy created.');

    // 5. Switch to Assignments Tab and assign
    await page.getByTestId('leave-tab-assignments').click();
    const assignBtn = page.getByTestId('assign-policy-btn');
    await expect(assignBtn).toBeVisible({ timeout: 15000 });
    await assignBtn.click();
    
    console.log('Filling assignment modal...');
    await page.getByTestId('assign-type-select').click();
    await page.getByRole('option', { name: /Individual|Cá nhân/i }).click();

    await page.getByTestId('assign-emp-select').click();
    await page.keyboard.type(fullName);
    await page.getByRole('option', { name: fullName }).first().click();

    await page.getByTestId('assign-policy-select').click();
    await page.keyboard.type(policyName);
    await page.getByRole('option', { name: policyName }).first().click();

    await page.getByTestId('submit-assign-policy-btn').click();
    
    // Verify assignment exists in table
    await expect(page.getByRole('cell', { name: policyName })).toBeVisible({ timeout: 15000 });
    console.log('Assignment successful.');
  });
});
