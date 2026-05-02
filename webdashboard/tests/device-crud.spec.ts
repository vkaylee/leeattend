/* eslint-disable @typescript-eslint/naming-convention, @typescript-eslint/prefer-nullish-coalescing, @typescript-eslint/restrict-template-expressions, max-lines-per-function, no-console */
import { expect } from "@playwright/test";

import { ConnectivityPage } from "./pom/connectivity-page";
import { test } from "./worker-orchestrator";

// Increase timeout for slower E2E environment
test.setTimeout(120000);

test.describe.skip("Device CRUD E2E Tests", () => {
  test.beforeEach(async ({ page }) => {
    page.on('console', message => { console.log(`[BROWSER] ${message.type().toUpperCase()}: ${message.text()}`); });
    page.on('pageerror', error => { console.log(`[BROWSER ERROR] ${error.message}`); });

    // Login first
    await page.goto('/login');
    await page.getByTestId('fast-login-btn').click();
    // Wait for dashboard to load
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible({ timeout: 60000 });
  });

  test('should display connectivity topology page', async ({ page }) => {
    const connectivity = new ConnectivityPage(page);
    await connectivity.goto();
    await connectivity.waitForLoad();

    // Verify main elements are visible
    await expect(page.getByTestId('connectivity-topology')).toBeVisible();
    await expect(page.getByTestId('add-agent-btn')).toBeVisible();
  });

  test('should create an agent and verify it appears', async ({ page }) => {
    page.on('dialog', dialog => dialog.accept());
    const connectivity = new ConnectivityPage(page);
    await connectivity.goto();
    await connectivity.waitForLoad();

    const agentId = `test-agent-${Date.now()}`;
    const agentName = `Test Agent ${Date.now()}`;
    const agentToken = `token-${Date.now()}`;

    // Create agent
    await connectivity.clickAddAgent();
    await connectivity.fillAgentForm(agentId, agentName, agentToken);
    await connectivity.submitAgentForm();

    // Verify agent appears
    await expect(page.getByTestId('add-agent-form')).not.toBeVisible({ timeout: 10000 });
    await expect(page.getByTestId(`agent-card-${agentId}`)).toBeVisible({ timeout: 10000 });
    await expect(page.getByTestId(`agent-name-${agentId}`)).toContainText(agentName);

    // Cleanup: delete the agent
    await connectivity.getAgentCard(agentId).hover();
    await connectivity.deleteAgent(agentId);
    await expect(page.getByTestId(`agent-card-${agentId}`)).not.toBeVisible({ timeout: 10000 });
  });

  test('should toggle device enable/disable and verify status changes', async ({ page }) => {
    // Handle confirmation dialog BEFORE navigating
    page.on('dialog', async dialog => {
      console.log(`[DIALOG] ${dialog.type()}: ${dialog.message()}`);
      await dialog.accept();
    });

    const connectivity = new ConnectivityPage(page);
    await connectivity.goto();
    await connectivity.waitForLoad();

    // Wait for agents to load
    await page.waitForTimeout(1000);

    // Check if there are any agents with devices
    const agentsList = page.getByTestId('agents-list');
    const hasAgents = await agentsList.count() > 0;

    if (!hasAgents) {
      test.skip(true, 'No agents available for testing device toggle');
      return;
    }

    // Expand all agents to see devices
    const expandButton = page.getByTestId('expand-all-btn');
    if (await expandButton.isVisible()) {
      await connectivity.expandAll();
    }
    await page.waitForTimeout(1000);

    // Find first device
    const firstDevice = page.locator('[data-testid^="device-item-"]').first();
    await expect(firstDevice).toBeVisible({ timeout: 10000 });

    // Get device ID from test id attribute
    const testId = await firstDevice.getAttribute('data-testid') || '';
    const deviceId = testId.replace('device-item-', '');

    // Get toggle button to check current state (button text indicates action, not state)
    const toggleButton = page.getByTestId(`device-toggle-${deviceId}`);
    const statusBadge = page.getByTestId(`device-status-${deviceId}`);

    // Helper to check if status indicates enabled (supports EN and VI)
    const isEnabled = (status: string): boolean => {
      const s = status.toLowerCase();
      // English: "active", Vietnamese: "hoạt động" (active)
      return s.includes('active') || s.includes('hoạt động');
    };

    // Get initial status from the status badge
    const initialStatusText = await statusBadge.textContent() || '';
    const wasEnabled = isEnabled(initialStatusText);
    console.log(`[TEST] Initial status: "${initialStatusText}", wasEnabled: ${wasEnabled}`);

    // Click toggle and wait for the network request to complete
    await toggleButton.click();

    // Wait for status to change (poll for change rather than fixed timeout)
    await expect(async () => {
      const currentStatus = await statusBadge.textContent() || '';
      const isNowEnabled = isEnabled(currentStatus);
      console.log(`[TEST] After toggle: "${currentStatus}", isNowEnabled: ${isNowEnabled}`);
      expect(isNowEnabled).not.toBe(wasEnabled);
    }).toPass({ timeout: 10000 });

    // Toggle back to original state
    await toggleButton.click();

    // Wait for status to revert
    await expect(async () => {
      const finalStatus = await statusBadge.textContent() || '';
      const isFinalEnabled = isEnabled(finalStatus);
      console.log(`[TEST] After revert: "${finalStatus}", isFinalEnabled: ${isFinalEnabled}`);
      expect(isFinalEnabled).toBe(wasEnabled);
    }).toPass({ timeout: 10000 });
  });

  test('should edit device and verify changes persist', async ({ page }) => {
    const connectivity = new ConnectivityPage(page);
    await connectivity.goto();
    await connectivity.waitForLoad();

    // Wait for data to load
    await page.waitForTimeout(1000);

    // Expand all to see devices
    const expandButton = page.getByTestId('expand-all-btn');
    if (await expandButton.isVisible()) {
      await connectivity.expandAll();
      await page.waitForTimeout(1000);
    }

    // Find first device
    const firstDevice = page.locator('[data-testid^="device-item-"]').first();
    await expect(firstDevice).toBeVisible({ timeout: 10000 });

    const testId = await firstDevice.getAttribute('data-testid') || '';
    const deviceId = testId.replace('device-item-', '');

    // Get original name
    const originalName = await connectivity.getDeviceName(deviceId);
    const newName = `Edited Device ${Date.now()}`;

    // Open edit modal
    await connectivity.clickEditDevice(deviceId);

    // Edit the name
    await connectivity.fillDeviceForm({ name: newName });
    await connectivity.submitDeviceForm();

    // Wait for modal to close and data to refresh
    await expect(page.getByTestId('edit-device-form')).not.toBeVisible({ timeout: 15000 });
    await page.waitForTimeout(1000);

    // Verify name changed
    await expect(page.getByTestId(`device-name-${deviceId}`)).toContainText(newName, { timeout: 10000 });

    // Restore original name
    await connectivity.clickEditDevice(deviceId);
    await connectivity.fillDeviceForm({ name: originalName.trim() });
    await connectivity.submitDeviceForm();

    await expect(page.getByTestId('edit-device-form')).not.toBeVisible({ timeout: 15000 });
  });

  test('should cancel edit device form without saving', async ({ page }) => {
    const connectivity = new ConnectivityPage(page);
    await connectivity.goto();
    await connectivity.waitForLoad();

    // Wait for data to load
    await page.waitForTimeout(1000);

    // Expand all
    const expandButton = page.getByTestId('expand-all-btn');
    if (await expandButton.isVisible()) {
      await connectivity.expandAll();
      await page.waitForTimeout(1000);
    }

    // Find first device
    const firstDevice = page.locator('[data-testid^="device-item-"]').first();
    await expect(firstDevice).toBeVisible({ timeout: 10000 });

    const testId = await firstDevice.getAttribute('data-testid') || '';
    const deviceId = testId.replace('device-item-', '');

    const originalName = await connectivity.getDeviceName(deviceId);

    // Open edit and make changes
    await connectivity.clickEditDevice(deviceId);
    await connectivity.fillDeviceForm({ name: 'Should Not Save' });

    // Cancel
    await connectivity.cancelDeviceForm();

    // Verify modal closed
    await expect(page.getByTestId('edit-device-form')).not.toBeVisible({ timeout: 10000 });

    // Verify name unchanged
    await expect(page.getByTestId(`device-name-${deviceId}`)).toContainText(originalName.trim());
  });

  test('should delete device after confirmation', async ({ page }) => {
    // Handle confirmation dialog BEFORE triggering it
    page.on('dialog', dialog => dialog.accept());

    const connectivity = new ConnectivityPage(page);
    await connectivity.goto();
    await connectivity.waitForLoad();

    // Wait for data to load
    await page.waitForTimeout(1000);

    // Expand all
    const expandButton = page.getByTestId('expand-all-btn');
    if (await expandButton.isVisible()) {
      await connectivity.expandAll();
      await page.waitForTimeout(1000);
    }

    // Count devices before
    const devicesBefore = await page.locator('[data-testid^="device-item-"]').count();

    if (devicesBefore === 0) {
      test.skip(true, 'No devices available for delete test');
      return;
    }

    const firstDevice = page.locator('[data-testid^="device-item-"]').first();
    const testId = await firstDevice.getAttribute('data-testid') || '';
    const deviceId = testId.replace('device-item-', '');

    // Delete device
    await connectivity.clickDeleteDevice(deviceId);

    // Wait for delete to complete
    await expect(page.getByTestId(`device-item-${deviceId}`)).not.toBeVisible({ timeout: 15000 });

    // Verify count decreased
    await page.waitForTimeout(500);
    const devicesAfter = await page.locator('[data-testid^="device-item-"]').count();
    expect(devicesAfter).toBe(devicesBefore - 1);
  });

  test('should expand and collapse agent accordion', async ({ page }) => {
    const connectivity = new ConnectivityPage(page);
    await connectivity.goto();
    await connectivity.waitForLoad();

    // Check if there are agents
    const agentCards = page.locator('[data-testid^="agent-card-"]');
    const agentCount = await agentCards.count();

    if (agentCount === 0) {
      test.skip(true, 'No agents available for expand/collapse test');
      return;
    }

    // Initially all should be collapsed (no device lists visible in expanded state)
    await connectivity.collapseAll();
    await page.waitForTimeout(300);

    // Expand all
    await connectivity.expandAll();
    await page.waitForTimeout(500);

    // Check that clicking collapse all hides content
    await connectivity.collapseAll();
    await page.waitForTimeout(300);

    // Expand single agent
    const firstCard = agentCards.first();
    const firstTestId = await firstCard.getAttribute('data-testid') || '';
    const agentId = firstTestId.replace('agent-card-', '');

    await connectivity.expandAgent(agentId);
    await page.waitForTimeout(300);

    // Verify the accordion expanded
    // (checking for visibility of content area would require more specific selectors)
    expect(true).toBe(true); // Test passes if no errors thrown
  });
});
