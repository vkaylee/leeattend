/* eslint-disable react-hooks/rules-of-hooks */
import { test as base, expect, WorkerInfo } from '@playwright/test';
import { spawn, ChildProcess } from 'child_process';
import path from 'path';
import fs from 'fs';
import { LoginPage } from './pom/LoginPage';
import { Sidebar } from './pom/Sidebar';
import { EmployeesPage } from './pom/EmployeesPage';
import { ConnectivityPage } from './pom/ConnectivityPage';

export { expect };

// 1. Extend base with worker-scoped fixtures
// Worker fixtures type definition
type WorkerFixtures = {
  workerWebUrl: string;
};

// Test-scoped fixtures type definition
type TestFixtures = {
  loginPage: LoginPage;
  sidebar: Sidebar;
  employeesPage: EmployeesPage;
  connectivityPage: ConnectivityPage;
  authPage: LoginPage; // Auto-authenticated page
};

const workerTest = base.extend<{}, WorkerFixtures>({
  workerWebUrl: [async ({}, use: (r: string) => Promise<void>, workerInfo: WorkerInfo) => { // eslint-disable-line no-empty-pattern
    const workerIndex = workerInfo.parallelIndex;
    const apiPortFile = `.api_port_${workerIndex}`;
    const webPortFile = `.web_port_${workerIndex}`;

    const apiPortPath = path.resolve(__dirname, `../../apiserver/${apiPortFile}`);
    const webPortPath = path.resolve(__dirname, `../${webPortFile}`);

    // Pre-cleanup stale files
    if (fs.existsSync(apiPortPath)) fs.unlinkSync(apiPortPath);
    if (fs.existsSync(webPortPath)) fs.unlinkSync(webPortPath);

    console.log(`👷 Worker ${workerIndex}: Starting isolated environment...`);

    // 1. Start Backend
    // Calculate isolated database URL for this worker
    const isolatedDbUrl = `postgres://test_user:test_password@localhost:5433/test_database_e2e_${workerIndex}`;
    const binaryPath = path.resolve(__dirname, '../../apiserver/target/debug/apiserver');
    
    const backend = spawn(binaryPath, [], {
      cwd: path.resolve(__dirname, '../../apiserver'),
      env: {
        ...process.env,
        PORT_FILE_NAME: apiPortFile,
        DATABASE_URL: isolatedDbUrl,
        API_PORT: '0', 
      },
      detached: true,
      stdio: 'pipe',
    });

    backend.stdout?.on('data', (data) => console.log(`[Backend W${workerIndex}] ${data}`));
    backend.stderr?.on('data', (data) => console.error(`[Backend W${workerIndex}] ${data}`));

    // 2. Wait for Backend port file
    const apiPort = await waitForFile(apiPortPath);

    // 3. Start Frontend (Vite)
    const frontend = spawn('npm', ['run', 'dev'], {
      cwd: path.resolve(__dirname, '../../webdashboard'),
      env: {
        ...process.env,
        VITE_BACKEND_PORT_FILE: apiPortFile,
        VITE_WEB_PORT_FILE: webPortFile,
        VITE_DEV_SERVER_HOST: '127.0.0.1',
      },
      detached: true,
      stdio: 'pipe',
    });

    frontend.stdout?.on('data', (data) => console.log(`[Frontend W${workerIndex}] ${data}`));
    frontend.stderr?.on('data', (data) => console.error(`[Frontend W${workerIndex}] ${data}`));

    // 4. Wait for Frontend port file
    const webPort = await waitForFile(webPortPath);

    const workerWebUrl = `http://127.0.0.1:${webPort}`;
    console.log(`👷 Worker ${workerIndex}: Environment ready at ${workerWebUrl}`);

    // Optional: wait a bit more for Vite to be fully ready
    await new Promise(r => setTimeout(r, 2000));

    await use(workerWebUrl);

    // Cleanup
    console.log(`👷 Worker ${workerIndex}: Cleaning up...`);
    await killProcess(backend);
    await killProcess(frontend);
    
    // Safety delay to allow OS to release ports
    await new Promise(r => setTimeout(r, 1000));

    if (fs.existsSync(apiPortPath)) fs.unlinkSync(apiPortPath);
    if (fs.existsSync(webPortPath)) fs.unlinkSync(webPortPath);
  }, { scope: 'worker' }],
});

// 2. Extend with test-scoped fixtures that depend on worker fixtures
export const test = workerTest.extend<TestFixtures>({
  // Override baseURL to use the dynamic one from worker scope
  baseURL: async ({ workerWebUrl }, use) => {
    await use(workerWebUrl);
  },

  // Page Objects
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

  // Fixture that automatically logs in
  authPage: async ({ page, loginPage }, use) => {
    await loginPage.goto();
    await loginPage.fastLogin();
    await expect(page.getByTestId('dashboard-title')).toBeVisible({ timeout: 30000 });
    await use(loginPage);
  },
  
      // Force English via storageState
  
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

async function waitForFile(filePath: string, timeout = 60000): Promise<string> {
  const start = Date.now();
  while (Date.now() - start < timeout) {
    if (fs.existsSync(filePath)) {
      const content = fs.readFileSync(filePath, 'utf8').trim();
      if (content && content !== '0') return content;
    }
    await new Promise(resolve => setTimeout(resolve, 500));
  }
  throw new Error(`Timeout waiting for file: ${filePath}`);
}

async function killProcess(child: ChildProcess) {
  if (child.pid) {
    try {
      // Kill the entire process group
      process.kill(-child.pid, 'SIGTERM');
    } catch (e) {
      child.kill('SIGTERM');
    }
  }
}
