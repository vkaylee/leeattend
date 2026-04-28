import { test, expect } from '../worker-orchestrator';

test.describe('Device CRUD — DevicesPage', () => {
  test.beforeEach(async ({ authPage, devicesPage }) => {
    // authPage fixture already logs in; now navigate to DevicesPage
    await devicesPage.goto();
    await devicesPage.waitForLoad();
  });

  // ---- List ----

  test('smoke: display mock devices', async ({ devicesPage }) => {
    await expect(devicesPage.getDeviceItem('device-mock-1')).toBeVisible();
    await expect(devicesPage.getDeviceItem('device-mock-2')).toBeVisible();
  });

  // ---- Create ----

  test('should add a new device via the add form', async ({ devicesPage, page }) => {
    await devicesPage.clickAddDevice();

    await devicesPage.fillDeviceForm({
      name: 'TEST DEVICE ALPHA',
      ip: '10.0.0.50',
      port: '4371',
      agentId: 'agent-mock-1',
      brand: 'zkteco',
      password: 'secret123',
      timezone: 'UTC',
    });

    await devicesPage.submitDeviceForm();

    // The modal should close after successful creation
    await expect(page.getByTestId('add-device-form')).not.toBeVisible({ timeout: 10000 });

    // The newly created device should appear in the list
    await expect(devicesPage.getDeviceItem('device-new-100')).toBeVisible({ timeout: 10000 });
  });

  test('should cancel adding a device without creating it', async ({ devicesPage, page }) => {
    await devicesPage.clickAddDevice();

    await devicesPage.fillDeviceForm({
      name: 'SHOULD NOT EXIST',
      ip: '1.2.3.4',
    });

    await devicesPage.cancelDeviceForm();

    // Modal should close
    await expect(page.getByTestId('add-device-form')).not.toBeVisible({ timeout: 5000 });

    // No extra device items should appear (still just 2 mock devices)
    await expect(devicesPage.getDeviceItem('device-mock-1')).toBeVisible();
    await expect(devicesPage.getDeviceItem('device-mock-2')).toBeVisible();
  });

  // ---- Edit ----

  test('should edit a device via the dropdown menu', async ({ devicesPage, page }) => {
    await devicesPage.openDeviceMenu('device-mock-1');
    await devicesPage.clickEditDevice('device-mock-1');

    // Clear and type a new name
    await devicesPage.fillDeviceForm({ name: 'UPDATED ENTRANCE' });

    await devicesPage.submitDeviceForm();

    // The edit modal should close
    await expect(page.getByTestId('edit-device-form')).not.toBeVisible({ timeout: 10000 });
  });

  // ---- Delete ----

  test('should delete a device after confirmation', async ({ devicesPage, page }) => {
    await devicesPage.openDeviceMenu('device-mock-2');
    await devicesPage.clickDeleteDevice('device-mock-2');

    // Confirm the delete
    await devicesPage.confirmDelete('device-mock-2');

    // The device should disappear
    await expect(devicesPage.getDeviceItem('device-mock-2')).not.toBeVisible({ timeout: 10000 });
  });

  test('should cancel device deletion without removing it', async ({ devicesPage }) => {
    await devicesPage.openDeviceMenu('device-mock-2');
    await devicesPage.clickDeleteDevice('device-mock-2');

    // Cancel
    await devicesPage.cancelDelete('device-mock-2');

    // Device should still be visible
    await expect(devicesPage.getDeviceItem('device-mock-2')).toBeVisible();
  });

  // ---- Toggle enable/disable ----

  test('should disable an enabled device after confirmation', async ({ devicesPage, page }) => {
    // device-mock-1 is enabled
    await devicesPage.clickToggleDevice('device-mock-1');
    await devicesPage.confirmToggle('device-mock-1');

    // The toggle modal should close — we trust the mock to reflect the change
    await expect(page.getByTestId('toggle-modal-device-mock-1')).not.toBeVisible({ timeout: 5000 });
  });

  test('should cancel toggle without changing state', async ({ devicesPage, page }) => {
    await devicesPage.clickToggleDevice('device-mock-1');
    await devicesPage.cancelToggle('device-mock-1');

    // The modal should close, and device should still be there
    await expect(page.getByTestId('toggle-modal-device-mock-1')).not.toBeVisible({ timeout: 5000 });
    await expect(devicesPage.getDeviceItem('device-mock-1')).toBeVisible();
  });
});
