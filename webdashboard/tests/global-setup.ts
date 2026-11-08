import { chromium, FullConfig } from '@playwright/test';

import { MOCK_USER } from './mocks/mock-data';

async function globalSetup(config: FullConfig) {
  const { baseURL } = config.projects[0].use;
  const browser = await chromium.launch();
  const page = await browser.newPage();
  
  // Set up local storage mock session
  await page.goto(baseURL ?? 'http://127.0.0.1:5173/login');
  await page.evaluate(({ user }) => {
    localStorage.setItem('access_token', 'mock-access-token');
    localStorage.setItem('refresh_token', 'mock-refresh-token');
    localStorage.setItem('active_user_id', user.id);
    localStorage.setItem('currentUser', JSON.stringify(user));
    localStorage.setItem('e2e_test_mode', 'true');
    localStorage.setItem('i18nextLng', 'en');
  }, { user: MOCK_USER });
  
  // Save storage state to a global file
  await page.context().storageState({ path: 'playwright/.auth/user.json' });
  await browser.close();
}

export default globalSetup;
