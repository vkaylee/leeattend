/* eslint-disable @typescript-eslint/no-useless-constructor */
import { Page, expect, Locator } from '@playwright/test';

import { BasePage } from './base-page';

export class DevicesPage extends BasePage {
  constructor(page: Page) {
    super(page);
  }
  async goto() {
    await this.page.goto('/workspace/settings/connectivity');
    await this.page.waitForURL('**/settings/connectivity', { timeout: 10000 });
    await this.waitForLoad();
  }

  async waitForLoad() {
    // 1. Wait for the main container
    const container = this.page.getByTestId('connectivity-topology');
    await expect(container).toBeVisible({ timeout: 10000 });
    
    // 2. Wait for loading to finish (either agents list or empty state appears)
    await expect(this.page.locator('[data-testid="agents-list"], [data-testid="empty-agents-state"]')).toBeVisible({ timeout: 20000 });


    // 4. Ensure accordions are expanded so devices are visible
    // We can rely on it being present if agents-list is present
    const expandButton = this.page.getByTestId('expand-all-btn');
    if (await this.page.getByTestId('agents-list').isVisible()) {
      await expandButton.click();
    }
  }

  // ---------- Add/Edit form ----------

  async clickAddDevice() {
    const agentCards = this.page.locator('[data-testid^="agent-card-"]');
    const firstCard = agentCards.first();
    const testId = await firstCard.getAttribute('data-testid') ?? '';
    const agentId = testId.replace('agent-card-', '');
    await this.page.getByTestId(`agent-header-${agentId}`).click(); // Expand first
    await this.page.getByTestId(`add-device-btn-${agentId}`).click();
    await this.page.waitForTimeout(1000);
    await this.page.screenshot({ path: 'add-device-fail.png', fullPage: true });
    await expect(this.page.getByTestId('add-device-form')).toBeVisible();
  }

  async fillDeviceForm(data: {
    name?: string;
    ip?: string;
    port?: string;
    password?: string;
    agentId?: string; // Note: In current UI, agent is often pre-selected or read-only in Add form
    brand?: string;
    timezone?: string; // Note: Timezone removed from current UI
  }) {
    if (data.name !== undefined) {
      await this.page.getByTestId('device-name-input').fill(data.name);
    }
    if (data.ip !== undefined) {
      await this.page.getByTestId('device-ip-input').fill(data.ip);
    }
    if (data.port !== undefined) {
      await this.page.getByTestId('device-port-input').fill(data.port);
    }
    if (data.password !== undefined) {
      await this.page.getByTestId('device-password-input').fill(data.password);
    }
    
    // In current UI, brand is a Combobox
    if (data.brand !== undefined) {
      await this.page.getByTestId('device-brand-select').click();
      await this.page.getByTestId(`device-brand-select-option-${data.brand}`).click();
    }
    
    // Note: agentId and timezone are currently skipped as they are either pre-selected
    // or removed from the UI. If needed in the future, add them here.
  }

  async submitDeviceForm() {
    await this.page.getByTestId('device-form-submit').click();
  }

  async cancelDeviceForm() {
    await this.page.getByTestId('device-form-cancel').click();
  }

  // ---------- Device list interactions ----------

  getDeviceItem(deviceId: string): Locator {
    return this.page.getByTestId(`device-item-${deviceId}`);
  }

  /**
   * Open the dropdown menu for a device.
   */
  async openDeviceMenu(deviceId: string) {
    // Hover first to reveal any hidden elements
    await this.getDeviceItem(deviceId).hover();
    // Click the dropdown trigger
    await this.page.getByTestId(`device-menu-${deviceId}`).click();
  }

  async clickEditDevice(deviceId: string) {
    await this.page.getByTestId(`device-edit-${deviceId}`).click();
    await expect(this.page.getByTestId('edit-device-form')).toBeVisible();
  }

  async clickDeleteDevice(deviceId: string) {
    await this.page.getByTestId(`device-delete-${deviceId}`).click();
    // The delete confirmation modal should appear
    await expect(this.page.getByTestId(`delete-modal-${deviceId}`)).toBeVisible();
  }

  async confirmDelete(deviceId: string) {
    await this.page.getByTestId(`delete-confirm-${deviceId}`).click();
  }

  async cancelDelete(deviceId: string) {
    await this.page.getByTestId(`delete-cancel-${deviceId}`).click();
  }

  // ---------- Toggle enable/disable ----------

  async clickToggleDevice(deviceId: string) {
    await this.page.getByTestId(`device-toggle-${deviceId}`).click();
    await expect(this.page.getByTestId(`toggle-modal-${deviceId}`)).toBeVisible();
  }

  async confirmToggle(deviceId: string) {
    await this.page.getByTestId(`toggle-confirm-${deviceId}`).click();
  }

  async cancelToggle(deviceId: string) {
    await this.page.getByTestId(`toggle-cancel-${deviceId}`).click();
  }
}

