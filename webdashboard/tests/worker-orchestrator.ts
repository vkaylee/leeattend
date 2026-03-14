/* eslint-disable react-hooks/rules-of-hooks */
import { test as base, expect } from '@playwright/test';
import { LoginPage } from './pom/LoginPage';
import { Sidebar } from './pom/Sidebar';
import { EmployeesPage } from './pom/EmployeesPage';
import { ConnectivityPage } from './pom/ConnectivityPage';
import { DevicesPage } from './pom/DevicesPage';
import { setupApiMocks } from './mocks/api-intercept';

export { expect };

// Test-scoped fixtures type definition
type TestFixtures = {
  loginPage: LoginPage;
  sidebar: Sidebar;
  employeesPage: EmployeesPage;
  connectivityPage: ConnectivityPage;
  devicesPage: DevicesPage;
  authPage: LoginPage; 
  workerWebUrl: string; // Giữ lại property này để không gãy các test cũ đang import nó
};

export const test = base.extend<TestFixtures>({
  // workerWebUrl giờ đơn giản lấy từ baseURL của config
  workerWebUrl: async ({ baseURL }, use) => {
    await use(baseURL || 'http://127.0.0.1:5173');
  },

  loginPage: async ({ page }, use) => {
    await use(new LoginPage(page));
  },
  sidebar: async ({ page }, use) => {
    await use(new Sidebar(page));
  },
  employeesPage: async ({ page }, use) => {
    await use(new EmployeesPage(page));
  },
  connectivityPage: async ({ page }, use) => {
    await use(new ConnectivityPage(page));
  },
  devicesPage: async ({ page }, use) => {
    await use(new DevicesPage(page));
  },

  // Fixture that automatically logs in
  authPage: async ({ page, loginPage }, use) => {
    await loginPage.goto();
    // fastLogin triggers window.location.reload() after the API call.
    // We must wait for the reload to complete before the test proceeds,
    // otherwise subsequent page.goto() calls race against the reload → net::ERR_ABORTED.
    await Promise.all([
      page.waitForEvent('load'),
      loginPage.fastLogin(),
    ]);
    // Extra guard: wait for the app to settle after reload
    await page.waitForLoadState('networkidle');
    await use(loginPage);
  },
  
  storageState: async ({}, use) => { // eslint-disable-line no-empty-pattern
    await use({
      cookies: [],
      origins: [
        {
          origin: 'http://127.0.0.1',
          localStorage: [
            {
              name: 'i18nextLng',
              value: 'en',
            },
          ],
        },
      ],
    });
  },
});

// Chặn API toàn cục cho mọi page trong context
base.beforeEach(async ({ page }) => {
  await setupApiMocks(page);
});
