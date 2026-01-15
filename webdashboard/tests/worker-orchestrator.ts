/* eslint-disable react-hooks/rules-of-hooks */
import { test as base, expect } from '@playwright/test';
import { spawn, ChildProcess } from 'child_process';
import path from 'path';
import fs from 'fs';

export { expect };

export const test = base.extend<{ workerWebUrl: string }>({
  // Override baseURL to use the dynamic one
  baseURL: async ({ workerWebUrl }, use) => {
    await use(workerWebUrl);
  },

  workerWebUrl: [async ({}, use, workerInfo) => { // eslint-disable-line no-empty-pattern
    const workerIndex = workerInfo.workerIndex;
    const apiPortFile = `.api_port_${workerIndex}`;
    const webPortFile = `.web_port_${workerIndex}`;

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
    const apiPortPath = path.resolve(__dirname, `../../apiserver/${apiPortFile}`);
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
    const webPortPath = path.resolve(__dirname, `../${webPortFile}`);
    const webPort = await waitForFile(webPortPath);

    const workerWebUrl = `http://127.0.0.1:${webPort}`;
    console.log(`👷 Worker ${workerIndex}: Environment ready at ${workerWebUrl}`);

    await use(workerWebUrl);

    // Cleanup
    console.log(`👷 Worker ${workerIndex}: Cleaning up...`);
    await killProcess(backend);
    await killProcess(frontend);
    
    // Safety delay to allow OS to release ports
    await new Promise(r => setTimeout(r, 1000));

    if (fs.existsSync(apiPortPath)) fs.unlinkSync(apiPortPath);
    if (fs.existsSync(webPortPath)) fs.unlinkSync(webPortPath);
  }, { scope: 'test', auto: true }],
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