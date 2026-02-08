import { expect } from '@playwright/test';
import { test } from './worker-orchestrator';

test.describe.skip('Login Success Flow', () => {
  test('should successfully login with standard form and redirect to home', async ({ loginPage }) => {
    await loginPage.goto();
    await loginPage.login('admin@leeattend.com', 'admin123');

    // Verify successful redirection to Home
    await expect(loginPage['page']).toHaveURL(/\/$/);
    await loginPage.verifyTitle('dashboard-title');
  });

  test('should show error message with invalid credentials', async ({ loginPage }) => {
    await loginPage.goto();
    await loginPage.login('wrong@example.com', 'wrongpass');

    // Should show error message
    await loginPage.verifyLoginError();
  });
});