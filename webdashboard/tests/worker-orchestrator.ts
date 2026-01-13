import { test as base } from '@playwright/test';
import { spawn, ChildProcess } from 'child_process';
import path from 'path';
import fs from 'fs';

export const test = base.extend<{ workerInstance: void, workerWebUrl: string }>({
  workerInstance: [async ({}, use, workerInfo) => {
    const workerIndex = workerInfo.workerIndex;
    const apiPortFile = `.api_port_${workerIndex}`;
    const webPortFile = `.web_port_${workerIndex}`;
    const dbName = `leeattend_test_${workerIndex}`;

    console.log(`👷 Worker ${workerIndex}: Starting isolated environment...`);

    // 1. Start Backend
    const backend = spawn('cargo', ['run'], {
      cwd: path.resolve(__dirname, '../../apiserver'),
      env: {
        ...process.env,
        PORT_FILE_NAME: apiPortFile,
        // Use environment DATABASE_URL or fallback to the one found in .env
        DATABASE_URL: process.env.DATABASE_URL || 'postgres://leeatt:leeattp@192.168.2.5:5434/leeattzkt',
        API_PORT: '0', // Let it find any port
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
    
    if (fs.existsSync(apiPortPath)) fs.unlinkSync(apiPortPath);
    if (fs.existsSync(webPortPath)) fs.unlinkSync(webPortPath);
  }, { scope: 'worker', auto: true }],

  workerWebUrl: async ({ workerInstance }, use) => {
    // This looks redundant but it's how we pass the value from workerInstance fixture to the test
    // Actually, workerInstance is 'void' in 'base.extend', let's fix the type above.
    await use(workerInstance as any);
  }
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
