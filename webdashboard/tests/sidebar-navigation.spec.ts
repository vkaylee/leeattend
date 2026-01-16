import { test, expect } from './worker-orchestrator';

test.describe('Sidebar Navigation', () => {
  test('should have "Time & Leave" section expanded by default', async ({ page }) => {
    // 1. Go to login page
    await page.goto('/');
    
    // 2. Click Fast Login to bypass auth quickly
    const fastLoginBtn = page.getByText(/Đăng nhập nhanh|Fast Login/i);
    await fastLoginBtn.click();

    // 3. Wait for dashboard to load
    await expect(page).toHaveURL('/');
    
    // 4. Check if "Time & Leave" section is visible and expanded
    // Section title from i18n: "Thời gian & Nghỉ phép"
    const sectionTitle = page.getByText(/Thời gian & Nghỉ phép|Time & Leave/i);
    await expect(sectionTitle).toBeVisible();

    // The links inside this section should be visible
    // "Chấm công" (Time Tracking) should be visible because section is expanded
    const timeTrackingLink = page.getByRole('link', { name: /Chấm công|Time Tracking/i });
    await expect(timeTrackingLink).toBeVisible();
    
    // 5. Verify "Organization" section is NOT expanded by default (max-h-0)
    const membersLink = page.getByRole('link', { name: /Thành viên|Members/i });
    await expect(membersLink).not.toBeVisible();
  });

  test('should expand one section and collapse others when clicked', async ({ page }) => {
    await page.goto('/');
    await page.getByText(/Đăng nhập nhanh|Fast Login/i).click();
    await expect(page).toHaveURL('/');

    // Initially "Time & Leave" is expanded, "Organization" is collapsed
    const timeTrackingLink = page.getByRole('link', { name: /Chấm công|Time Tracking/i });
    const membersLink = page.getByRole('link', { name: /Thành viên|Members/i });
    
    await expect(timeTrackingLink).toBeVisible();
    await expect(membersLink).not.toBeVisible();

    // Click "Organization" section title
    await page.getByText(/Tổ chức|Organization/i).click();

    // Now "Organization" should be expanded, "Time & Leave" should be collapsed
    await expect(membersLink).toBeVisible();
    await expect(timeTrackingLink).not.toBeVisible();
  });
});
