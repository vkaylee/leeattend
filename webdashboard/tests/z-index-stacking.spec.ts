import { test, expect } from './worker-orchestrator';

test.describe.skip('UI Z-Index Stacking Verification', () => {
  test('Combobox dropdown should have higher z-index than Modal backdrop', async ({ page }) => {
    // 1. Setup: Use Debug Login
    await page.goto('/login');
    const fastLoginButton = page.getByTestId('fast-login-btn');
    await expect(fastLoginButton).toBeVisible({ timeout: 15000 });
    await fastLoginButton.click();
    await expect(page.locator('h1').filter({ hasText: /Dashboard/i })).toBeVisible({ timeout: 30000 });

    // 2. Navigate to Leave Settings
    console.log('Navigating to Leave Settings...');
    await page.goto('/workspace/settings/leave');
    
    await expect(page.getByRole('heading', { level: 2, name: /Leave Settings|Cài đặt nghỉ phép/i })).toBeVisible({ timeout: 15000 });
    
    // Switch to Assignments tab
    console.log('Switching to Assignments tab...');
    const assignmentsTab = page.getByTestId('leave-tab-assignments');
    await expect(assignmentsTab).toBeVisible({ timeout: 15000 });
    await assignmentsTab.click();

    // Open Assignment Modal
    console.log('Opening Assignment Modal...');
    const assignButton = page.getByTestId('assign-policy-btn');
    await expect(assignButton).toBeVisible({ timeout: 15000 });
    await assignButton.click();

    // 3. Verify Modal Backdrop is present and get its z-index
    const backdrop = page.locator('.modal-backdrop');
    await expect(backdrop).toBeVisible();
    
    const backdropZIndex = await backdrop.evaluate((element) => {
      return window.getComputedStyle(element).zIndex;
    });
    console.log(`Modal Backdrop Z-Index: ${backdropZIndex}`);

    // 4. Open a Combobox inside the modal
    const comboboxTrigger = page.locator('button[id="assignee-type"]');
    await expect(comboboxTrigger).toBeVisible();
    await comboboxTrigger.click();

    // 5. Verify Combobox Content (Popover) is visible
    const popoverContent = page.locator('[cmdk-root]').first();
    await expect(popoverContent).toBeVisible();

    const popoverZIndex = await popoverContent.evaluate((element) => {
      let current: HTMLElement | null = element as HTMLElement;
      while (current) {
        const z = window.getComputedStyle(current).zIndex;
        if (z !== 'auto') return z;
        current = current.parentElement;
      }
      return 'auto';
    });
    console.log(`Combobox Popover Z-Index: ${popoverZIndex}`);

    // 6. Assertions
    const bZ = parseInt(backdropZIndex, 10);
    const pZ = parseInt(popoverZIndex, 10);

    expect(bZ).toBe(9999);
    expect(pZ).toBe(10000);
    expect(pZ).toBeGreaterThan(bZ);
    
    console.log('✅ Z-Index stacking verified: Popover (10000) > Backdrop (9999)');
  });
});
