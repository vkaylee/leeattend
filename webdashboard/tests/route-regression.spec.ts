import { test, expect } from './worker-orchestrator';

test.describe('Route & Sidebar Regression', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.getByTestId('fast-login-btn').click();
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible({ timeout: 30000 });
  });

  const criticalRoutes = [
    { name: 'Dashboard', path: '/', testId: null },
    { name: 'Employees', path: '/employees', testId: 'sidebar-link-employees', sectionId: 'sidebar-section-org' },
    { name: 'Time Tracking', path: '/time-tracking', testId: 'sidebar-link-time-tracking', sectionId: 'sidebar-section-time' },
    { name: 'Attendance Logs', path: '/attendance-logs', testId: 'sidebar-link-attendance-logs', sectionId: 'sidebar-section-time' },
    { name: 'Leave Requests', path: '/leave-requests', testId: 'sidebar-link-leave-requests', sectionId: 'sidebar-section-time' },
    { name: 'Shifts', path: '/shifts', testId: 'sidebar-link-shifts', sectionId: 'sidebar-section-payroll' },
    { name: 'Working Patterns', path: '/working-patterns', testId: 'sidebar-link-working-patterns', sectionId: 'sidebar-section-payroll' },
    { name: 'Assignment Rules', path: '/assignment-rules', testId: 'sidebar-link-assignment-rules', sectionId: 'sidebar-section-payroll' },
    { name: 'Payroll', path: '/payroll', testId: 'sidebar-link-payroll', sectionId: 'sidebar-section-payroll' },
    { name: 'General Settings', path: '/workspace/settings/general', testId: 'sidebar-link-settings-general', sectionId: 'sidebar-section-system' },
    { name: 'People Settings', path: '/workspace/settings/people', testId: 'sidebar-link-settings-people', sectionId: 'sidebar-section-system' },
  ];

  for (const route of criticalRoutes) {
    test(`should navigate to ${route.name} and render without blank page`, async ({ page }) => {
      console.log(`Testing route: ${route.path}`);
      
      if (route.testId) {
        // If it's a sidebar link, ensure section is expanded
        if (route.sectionId) {
          const section = page.getByTestId(route.sectionId);
          const link = page.getByTestId(route.testId);
          
          if (!await link.isVisible()) {
            await section.click();
            await expect(link).toBeVisible({ timeout: 5000 });
          }
          await link.click();
        } else {
          await page.getByTestId(route.testId).click();
        }
      } else {
        await page.goto(route.path);
      }

      // Verify URL
      await expect(page).toHaveURL(new RegExp(route.path.replace(/\//g, '\\/')));

      // Verify page content is NOT blank (look for headings or specific text)
      // Every page should have a heading (h1) or specific identifying content
      const heading = page.getByRole('heading', { level: 1 });
      await expect(heading).toBeVisible({ timeout: 15000 });
      
      // Ensure the heading is not empty
      const headingText = await heading.innerText();
      expect(headingText.length).toBeGreaterThan(0);
      
      console.log(`Successfully verified ${route.name} with heading: "${headingText}"`);
    });
  }
});
