import { test } from "./worker-orchestrator";
import { expect } from "@playwright/test";

test.describe("Leave Policies CRUD", () => {
  test('should create, edit, and delete a leave policy', async ({ page }) => {
    page.on('console', msg => console.log(`[BROWSER] ${msg.type().toUpperCase()}: ${msg.text()}`));
    page.on('pageerror', err => console.log(`[BROWSER ERROR] ${err.message}`));

    // 1. Login and Navigate
    await page.goto('/login');
    await page.getByTestId('fast-login-btn').click();

    // Wait for dashboard to load
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible({ timeout: 30000 });

    // Navigate to Leave Settings
    await page.goto("/workspace/settings/leave");
    await expect(page.getByRole('heading', { level: 2, name: /Leave Settings|Cài đặt nghỉ phép/i })).toBeVisible();

    // Switch to Policies tab
    await page.getByTestId('leave-tab-policies').click();

    // Add new policy
    await page.getByTestId('add-policy-btn').click();
    await expect(page.getByRole('heading', { name: /Create Leave Policy/i })).toBeVisible();

    const policyName = `E2E Policy ${Date.now()}`;
    await page.fill('input[id="policy-name"]', policyName);
    await page.fill('input[id="annual-entitlement"]', '15');

    await page.getByTestId('save-policy-btn').click();
    
    // Wait for modal to close and policy to appear
    await expect(page.getByRole('heading', { name: /Create Leave Policy/i })).not.toBeVisible();
    await expect(page.getByRole('heading', { name: policyName })).toBeVisible();

    // 5. Edit the Policy
    const card = page.locator('div.group.relative').filter({ has: page.getByRole('heading', { name: policyName }) });
    await card.hover();
    await card.getByTestId('edit-policy-btn').click({ force: true });
    await expect(page.getByRole('heading', { name: /Edit Policy/i })).toBeVisible();

    const updatedName = `${policyName} Updated`;
    await page.fill('input[id="policy-name"]', updatedName);
    await page.getByTestId('save-policy-btn').click();

    // Wait specifically for modal to disappear
    await expect(page.getByRole('heading', { name: /Edit Policy/i })).not.toBeVisible({ timeout: 10000 });
    await expect(page.getByRole('heading', { name: updatedName })).toBeVisible();

    // 6. Delete the Policy
    page.on('dialog', dialog => dialog.accept()); // Handle confirm()
    const updatedCard = page.locator('div.group.relative').filter({ has: page.getByRole('heading', { name: updatedName }) });
    await updatedCard.hover();
    await updatedCard.getByTestId('delete-policy-btn').click({ force: true });
    
    await expect(page.getByRole('heading', { name: updatedName })).not.toBeVisible();
  });
});
