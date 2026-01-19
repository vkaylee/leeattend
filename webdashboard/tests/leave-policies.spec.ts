import { test } from "./worker-orchestrator";
import { expect } from "@playwright/test";

test.describe("Leave Policies CRUD", () => {
  test('should create, edit, and delete a leave policy', async ({ page }) => {
    // 1. Login and Navigate
    await page.goto('/login');
    await page.getByTestId('fast-login-btn').click();

    // Navigate to Leave Requests
    const leaveRequestsLink = page.getByTestId('sidebar-link-leave-requests');
    if (!await leaveRequestsLink.isVisible()) {
      await page.getByTestId('sidebar-section-time').click();
    }
    await leaveRequestsLink.click({ force: true });
    await expect(page.getByRole('heading', { level: 1 })).toContainText(/Leave Requests|Yêu cầu nghỉ phép/);

    // Switch to Policies tab
    await page.getByTestId('leave-tab-policies').click();

    // Add new policy
    await page.getByTestId('add-policy-btn').click();
    await expect(page.getByRole('heading', { level: 2, name: /Leave Policies|Chính sách nghỉ phép/i })).toBeVisible();

    const policyName = `E2E Policy ${Date.now()}`;
    await page.fill('input[id="policy-name"]', policyName);
    await page.fill('input[id="annual-entitlement"]', '15');

    await page.getByTestId('save-policy-btn').click();
    
    // Wait for modal to close and policy to appear in table
    await expect(page.getByRole('heading', { level: 2, name: /Leave Policies|Chính sách nghỉ phép/i })).not.toBeVisible();
    await expect(page.getByRole('cell', { name: policyName })).toBeVisible();

    // 5. Edit the Policy
    const row = page.getByRole('row', { name: policyName });
    // First button in actions is Edit
        await row.getByRole('button').first().click();
        await expect(page.getByRole('heading', { level: 2, name: /Edit Policy|Sửa chính sách/i })).toBeVisible();
    
        const updatedName = `${policyName} Updated`;
        await page.fill('input[id="policy-name"]', updatedName);
        await page.getByTestId('save-policy-btn').click();
    
    await expect(page.getByRole('heading', { level: 2, name: /Edit Policy|Sửa chính sách/i })).not.toBeVisible();
    await expect(page.getByRole('cell', { name: updatedName })).toBeVisible();

    // 6. Delete the Policy
    page.on('dialog', dialog => dialog.accept()); // Handle confirm()
    await page.getByRole('row', { name: updatedName }).getByRole('button').nth(1).click(); // Second button is Delete
    
    await expect(page.getByRole('cell', { name: updatedName })).not.toBeVisible();
  });
});
