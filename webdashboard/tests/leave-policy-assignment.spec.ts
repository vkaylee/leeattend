import { test, expect } from './worker-orchestrator';

test.describe('Leave Policy Assignment Integration', () => {
  test('should populate assignment modal with employees and departments', async ({ page }) => {
    test.setTimeout(120000);
    
    // Capture console messages and errors
    page.on('console', msg => console.log(`[BROWSER] ${msg.type().toUpperCase()}: ${msg.text()}`));
    page.on('pageerror', err => console.log(`[BROWSER ERROR] ${err.message}`));

    console.log('--- Starting Leave Policy Assignment Integration Test ---');
    
    // 1. Setup: Use Debug Login
    await page.goto('/login');
    
    console.log('Looking for Fast Login button...');
    const fastLoginBtn = page.locator('button:has-text("Fast Login")');
    await expect(fastLoginBtn).toBeVisible({ timeout: 15000 });
    
    console.log('Clicking Fast Login button...');
    await fastLoginBtn.click();
    
    console.log('Waiting for Dashboard heading...');
    await expect(page.locator('h1:has-text("Dashboard")')).toBeVisible({ timeout: 15000 });

    // 2. Create a Department via UI
    console.log('Navigating to /departments...');
    await page.goto('/departments');
    const addDeptBtn = page.getByRole('button', { name: /Add|Thêm/i });
    await expect(addDeptBtn).toBeVisible({ timeout: 15000 });
    await addDeptBtn.click();
    
    await page.fill('input[name="name"]', 'Engineering Team');
    await page.fill('textarea[name="description"]', 'Test Department');
    await page.click('button[type="submit"]');
    await expect(page.locator('text=Engineering Team').first()).toBeVisible();
    console.log('Department created.');

    // 3. Create an Employee via UI
    console.log('Navigating to /employees...');
    await page.goto('/employees');
    const addEmpBtn = page.getByRole('button', { name: /Add|Thêm/i });
    await expect(addEmpBtn).toBeVisible({ timeout: 15000 });
    await addEmpBtn.click();
    
    const uniqueId = `INT-${Date.now()}`;
    await page.fill('input[name="first_name"]', 'Alice');
    await page.fill('input[name="last_name"]', 'Integrator');
    await page.fill('input[name="employee_id_number"]', uniqueId);
    await page.fill('input[name="hire_date"]', '2026-01-01');
    await page.click('button[type="submit"]');
    // Specifically look for the employee in the table body to avoid hidden <option> elements in the search box
    await expect(page.locator('tbody').locator('text=Alice Integrator').first()).toBeVisible();
    console.log('Employee created.');

    // 4. Create a Policy via UI
    console.log('Navigating to Leave Requests via Sidebar...');
    const timeLeaveSection = page.getByRole('button', { name: /Thời gian & Nghỉ phép|Time & Leave/i });
    await timeLeaveSection.click();
    
    const leaveRequestsLink = page.getByRole('link', { name: 'Nghỉ phép', exact: true });
    await expect(leaveRequestsLink).toBeVisible({ timeout: 15000 });
    await leaveRequestsLink.click();
    
    await expect(page.locator('h1')).toHaveText(/Yêu cầu nghỉ phép|Leave Requests/i, { timeout: 15000 });
    
    console.log('Switching to Policies tab...');
    const policiesTab = page.locator('div.flex.gap-2 button').filter({ hasText: /^Chính sách$|^Policies$/i });
    await policiesTab.click();
    
    console.log('Creating a policy...');
    const addPolicyBtn = page.getByRole('button', { name: /Thêm chính sách|Add Policy/i });
    await expect(addPolicyBtn).toBeVisible({ timeout: 10000 });
    await addPolicyBtn.click();
    
    await page.fill('input[id="policy-name"]', 'Integration Test Policy');
    await page.fill('input[id="annual-entitlement"]', '18');
    await page.click('button[type="submit"]');
    await expect(page.locator('text=Integration Test Policy').first()).toBeVisible();
    console.log('Policy created.');

    // 5. Open Assignment Modal
    console.log('Opening assignment modal...');
    const assignBtn = page.getByRole('button', { name: /Gán chính sách|Assign Policy/i });
    await assignBtn.click();
    
    // 6. Verify Employee Selection (Individual)
    console.log('Checking individual assignment...');
    await page.locator('button[id="assign-emp"]').click();
    await page.keyboard.type('Alice Integrator');
    await page.getByRole('option', { name: 'Alice Integrator' }).first().click();

    await page.locator('button[id="assign-policy"]').click();
    await page.keyboard.type('Integration Test Policy');
    await page.getByRole('option', { name: 'Integration Test Policy' }).first().click();

    await page.getByRole('button', { name: /Gán ngay|Assign Now/i }).click();
    // Use a more specific locator for the success toast to avoid matching previous ones
    await expect(page.locator('div[role="status"]').filter({ hasText: /thành công|successfully/i }).last()).toBeVisible();
    console.log('Individual assignment confirmed.');

    // WAIT FOR MODAL TO CLOSE COMPLETELY AND TOAST TO FADE
    await page.waitForTimeout(3000);

    // 7. Verify Department Selection
    console.log('Re-opening for Department assignment...');
    await assignBtn.click({ force: true });
    
    const assigneeTypeBtn = page.locator('button[id="assignee-type"]');
    await expect(assigneeTypeBtn).toBeVisible({ timeout: 5000 });
    await assigneeTypeBtn.click();
    
    // Select Department type
    await page.getByRole('option').filter({ hasText: /Phòng ban|Department/i }).first().click();
    
    const assignDeptBtn = page.locator('button[id="assign-dept"]');
    await expect(assignDeptBtn).toBeVisible({ timeout: 5000 });
    await assignDeptBtn.click();
    
    await page.keyboard.type('Engineering Team');
    await page.getByRole('option', { name: 'Engineering Team' }).first().click();

    await page.locator('button[id="assign-policy"]').click();
    await page.keyboard.type('Integration Test Policy');
    await page.getByRole('option', { name: 'Integration Test Policy' }).first().click();

    // Use a different effective date to avoid potential unique constraints or 422s
    await page.fill('input[id="assign-date"]', '2026-02-01');

    await page.getByRole('button', { name: /Gán ngay|Assign Now/i }).click();
    await expect(page.locator('div[role="status"]').filter({ hasText: /thành công|successfully/i }).last()).toBeVisible();
    
    console.log('--- Test Completed Successfully ---');
  });
});