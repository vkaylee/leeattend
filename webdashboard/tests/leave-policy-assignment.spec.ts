import { test, expect } from './worker-orchestrator';

test.describe('Leave Policy Assignment Integration', () => {
  test('should populate assignment modal with employees and departments', async ({ page }) => {
    test.setTimeout(120000);
    
    // Capture console messages and errors
    page.on('console', msg => console.log(`[BROWSER] ${msg.type().toUpperCase()}: ${msg.text()}`));
    page.on('pageerror', err => console.log(`[BROWSER ERROR] ${err.message}`));

    console.log('--- Starting Leave Policy Assignment Integration Test ---');
    
    // 1. Login
    await page.goto('/login');
    const fastLoginBtn = page.getByTestId('fast-login-btn');
    await fastLoginBtn.click();
    
    console.log('Waiting for Dashboard heading...');
    await expect(page.getByRole('heading', { level: 1, name: /Dashboard/i })).toBeVisible({ timeout: 15000 });

    // 2. Create a Department via UI
    console.log('Navigating to /departments...');
    await page.goto('/departments');
    await page.waitForLoadState('networkidle');
    const addDeptBtn = page.getByTestId('add-dept-btn');
    await expect(addDeptBtn).toBeVisible({ timeout: 30000 });
    await addDeptBtn.click();
    
    await page.fill('input[name="name"]', 'Engineering Team');
    await page.fill('textarea[name="description"]', 'Test Department');
    await page.getByTestId('save-dept-btn').click();
    await expect(page.getByText('Engineering Team').first()).toBeVisible();
    console.log('Department created.');

    // 3. Create an Employee via UI
    console.log('Navigating to /employees...');
    await page.goto('/employees');
    const addEmpBtn = page.getByTestId('add-employee-btn');
    await expect(addEmpBtn).toBeVisible({ timeout: 15000 });
    await addEmpBtn.click();
    
    const uniqueId = `INT-${Date.now()}`;
    await page.fill('input[name="first_name"]', 'Alice');
    await page.fill('input[name="last_name"]', 'Integrator');
    await page.fill('input[name="employee_id_number"]', uniqueId);
    await page.fill('input[name="hire_date"]', '2026-01-01');
    await page.getByTestId('save-employee-btn').click();
    // Specifically look for the employee in the table body to avoid hidden <option> elements in the search box
    await expect(page.getByRole('row', { name: 'Alice Integrator' })).toBeVisible();
    console.log('Employee created.');

    // 2. Navigate to Leave Requests via Sidebar
    console.log('Navigating to Leave Requests via Sidebar...');
    const leaveRequestsLink = page.getByTestId('sidebar-link-leave-requests');
    if (!await leaveRequestsLink.isVisible()) {
      await page.getByTestId('sidebar-section-time').click();
    }
    await leaveRequestsLink.click({ force: true });

    await expect(page.getByRole('heading', { level: 1 })).toHaveText(/Yêu cầu nghỉ phép|Leave Requests/i, { timeout: 15000 });

    // 3. Switch to Policies Tab
    const policiesTab = page.getByTestId('leave-tab-policies');
    await policiesTab.click();

    // 4. Create a Policy for assignment
    const addPolicyBtn = page.getByTestId('add-policy-btn');
    await addPolicyBtn.click();

    await page.fill('input[id="policy-name"]', 'Integration Test Policy');
    await page.getByTestId('save-policy-btn').click();
    await expect(page.getByRole('cell', { name: 'Integration Test Policy' }).first()).toBeVisible();

    // 5. Assign to Employee
    const assignBtn = page.getByTestId('assign-policy-btn');
    await assignBtn.click();
    
    // 6. Verify Employee Selection (Individual)
    console.log('Checking individual assignment...');
    await page.locator('#assign-emp').click();
    await page.keyboard.type('Alice Integrator');
    await page.getByRole('option', { name: 'Alice Integrator' }).first().click();

    await page.locator('#assign-policy').click();
    await page.keyboard.type('Integration Test Policy');
    await page.getByRole('option', { name: 'Integration Test Policy' }).first().click();

    await page.getByTestId('submit-assign-policy-btn').click();
    // Use a more specific locator for the success toast to avoid matching previous ones
    await expect(page.getByRole('status').filter({ hasText: /thành công|successfully/i }).last()).toBeVisible();
    console.log('Individual assignment confirmed.');

    // WAIT FOR MODAL TO CLOSE COMPLETELY AND TOAST TO FADE
    await page.waitForTimeout(3000);

    // 7. Verify Department Selection
    console.log('Re-opening for Department assignment...');
    await assignBtn.click({ force: true });
    
    const assigneeTypeBtn = page.locator('#assignee-type');
    await expect(assigneeTypeBtn).toBeVisible({ timeout: 5000 });
    await assigneeTypeBtn.click();
    
    // Select Department type
    await page.getByRole('option').filter({ hasText: /Phòng ban|Department/i }).first().click();
    
    const assignDeptBtn = page.locator('#assign-dept');
    await expect(assignDeptBtn).toBeVisible({ timeout: 5000 });
    await assignDeptBtn.click();
    
    await page.keyboard.type('Engineering Team');
    await page.getByRole('option', { name: 'Engineering Team' }).first().click();

    await page.locator('#assign-policy').click();
    await page.keyboard.type('Integration Test Policy');
    await page.getByRole('option', { name: 'Integration Test Policy' }).first().click();

    // Use a different effective date to avoid potential unique constraints or 422s
    await page.fill('input[id="assign-date"]', '2026-02-01');

    await page.getByTestId('submit-assign-policy-btn').click();
    await expect(page.getByRole('status').filter({ hasText: /thành công|successfully/i }).last()).toBeVisible();
    
    console.log('--- Test Completed Successfully ---');
  });
});