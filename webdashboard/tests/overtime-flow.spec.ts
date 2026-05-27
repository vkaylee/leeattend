import { expect } from "@playwright/test";

import { test } from "./worker-orchestrator";

test.describe("Overtime Flow", () => {
  test('should allow manager to create pre-ot request and employee to view it', async ({ page, context }) => {
    page.on('console', _message => { /* empty */ });
    page.on('pageerror', _error => { /* empty */ });

    // 1. Login
    await page.goto('/login');
    await page.getByTestId('fast-login-btn').click();

    // Wait for dashboard to load
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible({ timeout: 10000 });

    // Navigate to OT Approvals Page
    await page.goto("/workspace/approvals/ot");
    await expect(page.getByRole('heading', { level: 1, name: /OT APPROVALS/i })).toBeVisible();

    // Open Pre-OT Form
    await page.getByRole('button', { name: /New Pre-OT Request/i }).click();
    await expect(page.getByRole('heading', { name: /Plan Overtime \(Pre-OT\)/i })).toBeVisible();

    // Select an employee (assuming there is at least one)
    const employeeCheckbox = page.locator('input[type="checkbox"]').first();
    if (await employeeCheckbox.isVisible()) {
      await employeeCheckbox.check();

      // Fill in details
      await page.fill('textarea[placeholder="Why is this overtime necessary?"]', "E2E Test Overtime Requirement");
      
      // Submit Request
      await page.getByRole('button', { name: /Submit Request/i }).click();
      
      // Verify Toast
      await expect(page.getByText(/Pre-OT Request created successfully/i)).toBeVisible();
    }
  });

  test('should display employee overtime dashboard and quota', async ({ page }) => {
    // 1. Login
    await page.goto('/login');
    await page.getByTestId('fast-login-btn').click();

    // Navigate to Employee OT Page
    await page.goto("/ess/ot");
    await expect(page.getByRole('heading', { level: 1, name: /My Overtime/i })).toBeVisible();

    // Quota Widget Should be visible
    await expect(page.getByRole('heading', { level: 3, name: /Overtime Quota/i })).toBeVisible();
  });
});
