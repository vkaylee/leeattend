/* eslint-disable @typescript-eslint/no-useless-constructor */
import { Page, expect, Locator } from '@playwright/test';

import { BasePage } from './base-page';

export class DevicesPage extends BasePage {
  constructor(page: Page) {
    super(page);
  }
  async goto() {
    await this.page.goto('/devices');
    await this.page.waitForURL('**/devices', { timeout: 30000 });
    await this.waitForLoad();
  }

  async waitForLoad() {
    await expect(this.page.getByTestId('devices-settings')).toBeVisible({ timeout: 30000 });
    await expect(this.page.getByTestId('device-item-device-mock-1')).toBeVisible({ timeout: 20000 });
  }

  // ---------- Add/Edit form ----------

  async clickAddDevice() {
    await this.page.getByTestId('devices-add-button').click();
    await expect(this.page.getByTestId('add-device-form')).toBeVisible();
  }


  async fillDeviceForm(data: {
    name?: string;
    ip?: string;
    port?: string;
    password?: string;
    brand?: string;
    timezone?: string;
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
    
    if (data.timezone !== undefined) {
      await this.page.getByTestId('device-timezone-select').click();
      await this.page.getByRole('option', { name: data.timezone }).click();
    }
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
    const btn = this.page.getByTestId(`device-edit-${deviceId}`);
    await btn.waitFor({ state: 'visible', timeout: 5000 });
    await btn.click();
    await expect(this.page.getByTestId('edit-device-form')).toBeVisible();
  }

  async clickDeleteDevice(deviceId: string) {
    const btn = this.page.getByTestId(`device-delete-${deviceId}`);
    await btn.waitFor({ state: 'visible', timeout: 5000 });
    await btn.click();
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

