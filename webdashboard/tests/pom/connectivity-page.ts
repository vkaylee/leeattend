/* eslint-disable @typescript-eslint/no-useless-constructor */
import { Page, expect, Locator } from '@playwright/test';

import { BasePage } from './base-page';

export class ConnectivityPage extends BasePage {
  constructor(page: Page) {
    super(page);
  }

  async goto() {
    await this.page.goto('/workspace/settings/connectivity');
    await this.page.waitForURL('**/settings/connectivity', { timeout: 30000 });
    await this.waitForLoad();
  }

  async waitForLoad() {
    
    await expect(this.page.getByTestId('connectivity-topology')).toBeVisible({ timeout: 30000 });
    // Wait for either the agent list or the empty state to be visible (meaning data has finished loading)
    await expect(this.page.locator('[data-testid="agents-list"], [data-testid="empty-agents-state"]')).toBeVisible({ timeout: 30000 });
  }

  // Agent operations
  async clickAddAgent() {
    await this.page.getByTestId('add-agent-btn').click();
    await expect(this.page.getByTestId('add-agent-form')).toBeVisible();
  }


  async clickAddDeviceForAgent(agentId: string) {
    await this.page.getByTestId(`add-device-btn-${agentId}`).click();
    await expect(this.page.getByTestId('add-device-form')).toBeVisible();
  }

  async fillAgentForm(id: string, name: string, token: string) {
    await this.page.getByTestId('agent-id-input').fill(id);
    await this.page.getByTestId('agent-name-input').fill(name);
    await this.page.getByTestId('agent-token-input').fill(token);
  }

  async submitAgentForm() {
    await this.page.getByTestId('agent-form-submit').click();
  }

  async cancelAgentForm() {
    await this.page.getByTestId('agent-form-cancel').click();
  }

  getAgentCard(agentId: string): Locator {
    return this.page.getByTestId(`agent-card-${agentId}`);
  }

  async expandAgent(agentId: string) {
    await this.page.getByTestId(`agent-header-${agentId}`).click();
  }

  async deleteAgent(agentId: string) {
    await this.page.getByTestId(`agent-delete-${agentId}`).click();
  }

  async toggleAgentTrust(agentId: string) {
    await this.page.getByTestId(`agent-trust-toggle-${agentId}`).click();
  }

  // Device operations
  getDeviceItem(deviceId: string): Locator {
    return this.page.getByTestId(`device-item-${deviceId}`);
  }

  async toggleDevice(deviceId: string) {
    await this.page.getByTestId(`device-toggle-${deviceId}`).click();
  }

  async openDeviceMenu(deviceId: string) {
    // Hover first to reveal the menu button
    await this.getDeviceItem(deviceId).hover();
    await this.page.getByTestId(`device-menu-${deviceId}`).click();
  }

  async clickEditDevice(deviceId: string) {
    await this.openDeviceMenu(deviceId);
    await this.page.getByTestId(`device-edit-${deviceId}`).click();
    await expect(this.page.getByTestId('edit-device-form')).toBeVisible();
  }

  async clickDeleteDevice(deviceId: string) {
    await this.openDeviceMenu(deviceId);
    await this.page.getByTestId(`device-delete-${deviceId}`).click();
  }

  async fillDeviceForm(data: { name?: string; ip?: string; port?: string; password?: string; timezone?: string }) {
    if (data.name) {
      await this.page.getByTestId('device-name-input').fill(data.name);
    }
    if (data.ip) {
      await this.page.getByTestId('device-ip-input').fill(data.ip);
    }
    if (data.port) {
      await this.page.getByTestId('device-port-input').fill(data.port);
    }
    if (data.password !== undefined) {
      await this.page.getByTestId('device-password-input').fill(data.password);
    }
    if (data.timezone) {
      // Timezone is a Combobox — click to open, then select the option
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

  async getDeviceStatus(deviceId: string): Promise<string> {
    const status = this.page.getByTestId(`device-status-${deviceId}`);
    return await status.textContent() ?? '';
  }

  async getDeviceName(deviceId: string): Promise<string> {
    const name = this.page.getByTestId(`device-name-${deviceId}`);
    return await name.textContent() ?? '';
  }

  // Utility
  async expandAll() {
    await this.page.getByTestId('expand-all-btn').click();
  }

  async collapseAll() {
    await this.page.getByTestId('collapse-all-btn').click();
  }

  async toggleInactiveFilter() {
    await this.page.getByTestId('toggle-inactive-btn').click();
  }
}
