// eslint-disable-next-line import-x/no-named-as-default
import AxeBuilder from '@axe-core/playwright';
import { expect } from '@playwright/test';

import { test } from './worker-orchestrator';

test.describe('Accessibility Contrast Rules', () => {
  // Clear global storage state so we stay on the login page
  test.use({ storageState: { cookies: [], origins: [] } });

  test('login page buttons and elements should have sufficient color contrast', async ({ page }) => {
    await page.goto('/login');
    await page.waitForLoadState('networkidle');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withRules(['color-contrast'])
      .include('[data-component="Button"]')
      .analyze();

    await test.info().attach('accessibility-scan-results', {
      body: JSON.stringify(accessibilityScanResults, null, 2),
      contentType: 'application/json'
    });

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('should catch low contrast button (intentional failure test)', async ({ page }) => {
    await page.goto('/login');
    await page.waitForLoadState('networkidle');

    // Inject a button with terrible contrast (yellow text on white background)
    await page.evaluate(() => {
      const badBtn = document.createElement('button');
      badBtn.innerText = 'Bad Contrast Button';
      badBtn.style.backgroundColor = '#ffffff';
      badBtn.style.color = '#ffff00'; // yellow on white
      badBtn.style.padding = '10px';
      document.body.appendChild(badBtn);
    });

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withRules(['color-contrast'])
      .include('button')
      .analyze();

    // We EXPECT violations to exist
    expect(accessibilityScanResults.violations.length).toBeGreaterThan(0);
    // Specifically verify it's a color-contrast violation
    expect(accessibilityScanResults.violations[0].id).toBe('color-contrast');
  });


});
