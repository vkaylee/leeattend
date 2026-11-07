const fs = require('node:fs');
const { chromium } = require('/usr/local/lib/node_modules/@playwright/mcp/node_modules/playwright');

const secretFile = process.env.PLAYWRIGHT_LOGIN_FILE || '/run/secrets/playwright-login.env';
const secretValues = Object.fromEntries(
  fs.readFileSync(secretFile, 'utf8')
    .split(/\r?\n/)
    .filter((line) => line && !line.startsWith('#'))
    .map((line) => {
      const separator = line.indexOf('=');
      return [line.slice(0, separator), line.slice(separator + 1)];
    }),
);
const email = secretValues.PLAYWRIGHT_LOGIN_EMAIL;
const password = secretValues.PLAYWRIGHT_LOGIN_PASSWORD;
const baseURL = process.env.PLAYWRIGHT_BASE_URL || 'http://caddy';
const outputPath = process.env.PLAYWRIGHT_AUTH_STATE || '/run/secrets/playwright-auth.json';

if (!email || !password) {
  throw new Error('Development Playwright credentials are required in the mounted secret file');
}

(async () => {
  const browser = await chromium.launch({
    channel: 'chrome',
    headless: true,
    args: ['--no-sandbox'],
  });
  const page = await browser.newPage();

  try {
    await page.goto(`${baseURL}/login`, { waitUntil: 'domcontentloaded' });
    await page.getByTestId('login-email-input').fill(email);
    await page.getByTestId('login-password-input').fill(password);
    await page.getByTestId('login-submit-btn').click();
    await page.waitForTimeout(2_000);
    if (page.url().endsWith('/login')) {
      const error = await page.getByRole('alert').textContent().catch(() => null);
      throw new Error(`Development login did not complete${error ? `: ${error}` : ''}`);
    }
    await page.context().storageState({ path: outputPath });
    process.stdout.write('Playwright auth state created.\n');
  } finally {
    await browser.close();
  }
})().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : 'Playwright auth setup failed'}\n`);
  process.exitCode = 1;
});
