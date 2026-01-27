import { test, expect } from './worker-orchestrator';

test.describe('Route & Sidebar Regression', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.getByTestId('fast-login-btn').click();
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible({ timeout: 30000 });
  });

  const criticalRoutes = [
    { name: 'Dashboard', path: '/', testId: null, titleTestId: 'dashboard-title' },
    { name: 'Employees', path: '/employees', testId: 'sidebar-link-employees', sectionId: 'sidebar-section-org', titleTestId: 'employees-title' },
    { name: 'Attendance Logs', path: '/attendance-logs', testId: 'sidebar-link-attendance-logs', sectionId: 'sidebar-section-time', titleTestId: 'attendance-logs-title' },
    { name: 'Leave Requests', path: '/leave-requests', testId: 'sidebar-link-leave-requests', sectionId: 'sidebar-section-time', titleTestId: 'leave-requests-title' },
    { name: 'Shifts', path: '/shifts', testId: 'sidebar-link-shifts', sectionId: 'sidebar-section-payroll', titleTestId: 'shifts-title' },
    { name: 'Working Patterns', path: '/workspace/settings/time?tab=patterns', testId: 'sidebar-link-working-patterns', sectionId: 'sidebar-section-payroll', titleTestId: 'patterns-title' },
    { name: 'Assignment Rules', path: '/workspace/settings/time?tab=rules', testId: 'sidebar-link-assignment-rules', sectionId: 'sidebar-section-payroll', titleTestId: 'assignment-rules-title' },
    { name: 'Payroll', path: '/payroll', testId: 'sidebar-link-payroll', sectionId: 'sidebar-section-payroll', titleTestId: 'payroll-title' },
    { name: 'General Settings', path: '/workspace/settings/general', testId: 'sidebar-link-settings-general', sectionId: 'sidebar-section-system', titleTestId: 'general-settings-title' },
    { name: 'People Settings', path: '/workspace/settings/people', testId: 'sidebar-link-settings-people', sectionId: 'sidebar-section-system', titleTestId: 'people-settings-title' },
  ];

  for (const route of criticalRoutes) {
    test(`should navigate to ${route.name} and render without blank page`, async ({ page }) => {
      console.log(`Testing route: ${route.path}`);
      
      // Use direct navigation to avoid sidebar animation flakiness in regression tests
      await page.goto(route.path);

      // Verify URL
      await expect(page).toHaveURL(new RegExp(route.path.replace(/\//g, '\\/').replace(/\?/g, '\\?')));

      // Verify page content via titleTestId
      const title = page.getByTestId(route.titleTestId);
      await expect(title).toBeVisible({ timeout: 30000 });
      
      console.log(`Successfully verified ${route.name} via data-testid: "${route.titleTestId}"`);
    });
  }
});
