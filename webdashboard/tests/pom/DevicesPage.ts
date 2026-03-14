import { Page, expect, Locator } from '@playwright/test';
import { BasePage } from './BasePage';

export class DevicesPage extends BasePage {
  constructor(page: Page) {
    super(page);
  }

  async goto() {
    await super.goto('/devices');
  }

  async waitForLoad() {
    // Wait for the "Add Device" button which is always visible on DevicesPage
    await expect(this.page.getByTestId('add-device-btn')).toBeVisible({ timeout: 30000 });
  }

  // ---------- Add/Edit form ----------

  async clickAddDevice() {
    await this.page.getByTestId('add-device-btn').click();
    await expect(this.page.getByTestId('add-device-form')).toBeVisible();
  }

  async fillDeviceForm(data: {
    name?: string;
    ip?: string;
    port?: string;
    password?: string;
    agentId?: string;
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
    if (data.agentId !== undefined) {
      // Combobox — click trigger then select option
      await this.page.getByTestId('device-agent-select').click();
      await this.page.getByTestId(`device-agent-select-option-${data.agentId}`).click();
    }
    if (data.brand !== undefined) {
      await this.page.getByTestId('device-brand-select').click();
      await this.page.getByTestId(`device-brand-select-option-${data.brand}`).click();
    }
    if (data.timezone !== undefined) {
      await this.page.getByTestId('device-timezone-select').click();
      await this.page.getByTestId(`device-timezone-select-option-${data.timezone}`).click();
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
   * Open the three-dot dropdown menu for a device (via DeviceListItem).
   * The DropdownMenu trigger is inside the device card header.
   */
  async openDeviceMenu(deviceId: string) {
    // Hover first to reveal any hidden elements
    await this.getDeviceItem(deviceId).hover();
    // Click the dropdown trigger — it's inside the device-item card
    const item = this.getDeviceItem(deviceId);
    const trigger = item.locator('[data-radix-collection-item]').first();
    await trigger.click();
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
