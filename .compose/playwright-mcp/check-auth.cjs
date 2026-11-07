const fs = require('node:fs');
const { request } = require('/usr/local/lib/node_modules/@playwright/mcp/node_modules/playwright');

const statePath = process.env.PLAYWRIGHT_AUTH_STATE;
const baseURL = process.env.PLAYWRIGHT_BASE_URL || 'http://caddy';

async function main() {
  if (!statePath || !fs.existsSync(statePath)) {
    return 1;
  }

  const context = await request.newContext({ storageState: statePath });
  try {
    const response = await context.get(`${baseURL}/api/v1/me`, { timeout: 10_000 });
    return response.ok() ? 0 : 1;
  } catch {
    return 1;
  } finally {
    await context.dispose();
  }
}

main().then((status) => { process.exitCode = status; });
