/* eslint-disable react-hooks/rules-of-hooks */
import { test as base, expect } from '@playwright/test';

import { setupApiMocks } from './mocks/api-intercept';
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
  authPage: async ({ page, loginPage }, use) => {
    await setupApiMocks(page);
    await page.goto('/');
    // Ensure we successfully navigated out of login page
    await expect(page).not.toHaveURL(/.*\/login/, { timeout: 15000 });
    await use(loginPage);
  },
});

// Chặn API toàn cục cho mọi page trong context
base.beforeEach(async ({ page }) => {
  page.on('pageerror', error => { console.error('[BROWSER ERROR]', error); });
  page.on('console', msg => {
     
    console.log('[BROWSER LOG]', msg.type(), msg.text());
  });
   
  page.on('request', request => { console.log('>>', request.method(), request.url()); });
   
  page.on('response', response => { console.log('<<', response.status(), response.url()); });
  await setupApiMocks(page);
});
