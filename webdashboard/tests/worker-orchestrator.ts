/* eslint-disable react-hooks/rules-of-hooks */
import { test as base, expect } from '@playwright/test';

import { setupApiMocks } from './mocks/api-intercept';
import { MOCK_USER } from './mocks/mock-data';
import { ConnectivityPage } from './pom/connectivity-page';
import { DevicesPage } from './pom/devices-page';
import { EmployeesPage } from './pom/employees-page';
import { LoginPage } from './pom/login-page';
import { Navbar } from './pom/navbar';
import { Sidebar } from './pom/sidebar';

export { expect };

// Test-scoped fixtures type definition
interface TestFixtures {
  authPage: LoginPage;
  connectivityPage: ConnectivityPage;
  devicesPage: DevicesPage;
  employeesPage: EmployeesPage;
  loginPage: LoginPage;
  navbar: Navbar;
  sidebar: Sidebar; 
  workerWebUrl: string; // Giữ lại property này để không gãy các test cũ đang import nó
}

export const test = base.extend<TestFixtures>({
  // workerWebUrl giờ đơn giản lấy từ baseURL của config
  workerWebUrl: async ({ baseURL }, use) => {
    await use(baseURL ?? 'http://127.0.0.1:5173');
  },

  loginPage: async ({ page }, use) => {
    await use(new LoginPage(page));
  },
  sidebar: async ({ page }, use) => {
    await use(new Sidebar(page));
  },
  navbar: async ({ page }, use) => {
    await use(new Navbar(page));
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
  await loginPage.login(MOCK_USER.email, 'password123');

  // Chờ cho đến khi thoát khỏi trang login
  await page.waitForURL(url => !url.pathname.includes('/login'), { timeout: 90000 });
  
  await use(loginPage);
},  storageState: async ({}, use) => { // eslint-disable-line no-empty-pattern
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
  page.on('pageerror', error => { console.error('[BROWSER ERROR]', error); });
  page.on('console', msg => {
    // eslint-disable-next-line no-console
    console.log('[BROWSER LOG]', msg.type(), msg.text());
  });
  // eslint-disable-next-line no-console
  page.on('request', request => { console.log('>>', request.method(), request.url()); });
  // eslint-disable-next-line no-console
  page.on('response', response => { console.log('<<', response.status(), response.url()); });
  await setupApiMocks(page);
});
