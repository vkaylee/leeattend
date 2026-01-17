import { test, expect } from './worker-orchestrator';

test.describe('UI Z-Index Stacking Verification', () => {
  test('Combobox dropdown should have higher z-index than Modal backdrop', async ({ page }) => {
    // 1. Setup: Use Debug Login
    await page.goto('/login');
    const fastLoginBtn = page.locator('button').filter({ hasText: /Fast Login|⚡ Fast Login/i });
    await expect(fastLoginBtn).toBeVisible({ timeout: 15000 });
    await fastLoginBtn.click();
    await expect(page.locator('h1').filter({ hasText: /Dashboard/i })).toBeVisible({ timeout: 30000 });

    // 2. Navigate to Leave Requests via Sidebar
    console.log('Navigating to Leave Requests via Sidebar...');
    const timeLeaveSection = page.getByRole('button', { name: /Thời gian & Nghỉ phép|Time & Leave/i });
    await expect(timeLeaveSection).toBeVisible({ timeout: 15000 });
    // It's expanded by default, but let's make sure
    
    const leaveRequestsLink = page.getByRole('link', { name: /Nghỉ phép|Leave Requests/i }).first();
    await expect(leaveRequestsLink).toBeVisible({ timeout: 15000 });
    await leaveRequestsLink.click();
    
    await expect(page.locator('h1')).toHaveText(/Yêu cầu nghỉ phép|Leave Requests/i, { timeout: 15000 });
    
    // Switch to Policies tab
    console.log('Switching to Policies tab...');
    const policiesTab = page.locator('button').filter({ hasText: /Chính sách|Policies/i }).first();
    await expect(policiesTab).toBeVisible({ timeout: 15000 });
    await policiesTab.click();

    // Open Assignment Modal
    console.log('Opening Assignment Modal...');
    const assignBtn = page.getByRole('button', { name: /Gán chính sách|Assign Policy/i });
    await expect(assignBtn).toBeVisible({ timeout: 15000 });
    await assignBtn.click();

    // 3. Verify Modal Backdrop is present and get its z-index
    const backdrop = page.locator('.modal-backdrop');
    await expect(backdrop).toBeVisible();
    
    const backdropZIndex = await backdrop.evaluate((el) => {
      return window.getComputedStyle(el).zIndex;
    });
    console.log(`Modal Backdrop Z-Index: ${backdropZIndex}`);

    // 4. Open a Combobox inside the modal
    const comboboxTrigger = page.locator('button[id="assignee-type"]');
    await expect(comboboxTrigger).toBeVisible();
    await comboboxTrigger.click();

    // 5. Verify Combobox Content (Popover) is visible
    // Radix Popovers usually render in a portal. We look for the content div.
    // Based on our fix, it should have z-index 10000.
    const popoverContent = page.locator('[role="dialog"], [cmdk-root]').first();
    await expect(popoverContent).toBeVisible();

    const popoverZIndex = await popoverContent.evaluate((el) => {
      // Traverse up to find the closest element with a z-index if the target doesn't have it directly
      let current: HTMLElement | null = el as HTMLElement;
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
