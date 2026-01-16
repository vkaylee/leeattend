import { test } from "./worker-orchestrator";
import { expect } from "@playwright/test";

test.describe("Leave Policies CRUD", () => {
  test("should create, edit, and delete leave policies", async ({ page, baseURL }) => {
    // 1. Login
    await page.goto(`${baseURL}/login`);
    if (page.url().endsWith("/login")) {
        await page.click('button:has-text("⚡ Fast Login"), button:has-text("⚡ Đăng nhập nhanh")');
        await page.waitForURL("**/", { timeout: 30000 });
    }

    await expect(page.locator("text=Đang kiểm tra bảo mật...")).not.toBeVisible({ timeout: 60000 });
    await expect(page.locator("text=Đang tải không gian làm việc...")).not.toBeVisible({ timeout: 60000 });

    // 2. Navigate to Leave Requests page
    const sectionHeader = page.locator('button:has-text("Time & Leave"), button:has-text("Thời gian & Nghỉ phép")');
    if (await sectionHeader.isVisible()) {
        await sectionHeader.click();
    }
    await page.click('a[href="/leave-requests"]');
    await expect(page.locator("h1")).toContainText(/Leave Requests|Yêu cầu nghỉ phép/);

    // 3. Switch to Policies Tab
    await page.click('button:has-text("Policies"), button:has-text("Chính sách")');
    
    // 4. Create a New Policy
    await page.click('button:has-text("Add Policy"), button:has-text("Thêm chính sách")');
    await expect(page.locator('h2:has-text("Leave Policies"), h2:has-text("Chính sách nghỉ phép")')).toBeVisible();
    
    const policyName = "E2E Test Policy " + Date.now();
    await page.fill('input[id="policy-name"]', policyName);
    await page.fill('input[id="annual-entitlement"]', "15");
    await page.click('button:has-text("Save Policy"), button:has-text("Lưu chính sách")');
    
    // Wait for modal to close and policy to appear in table
    await expect(page.locator('h2:has-text("Leave Policies"), h2:has-text("Chính sách nghỉ phép")')).not.toBeVisible();
    await expect(page.locator(`td:has-text("${policyName}")`)).toBeVisible();

    // 5. Edit the Policy
    const row = page.locator(`tr:has-text("${policyName}")`);
    // First button in actions is Edit
    await row.locator('button').first().click();
    
    await expect(page.locator('h2:has-text("Edit Policy"), h2:has-text("Sửa chính sách")')).toBeVisible();
    const updatedName = policyName + " Updated";
    await page.fill('input[id="policy-name"]', updatedName);
    await page.click('button:has-text("Save changes"), button:has-text("Lưu thay đổi")');
    
    await expect(page.locator('h2:has-text("Edit Policy"), h2:has-text("Sửa chính sách")')).not.toBeVisible();
    await expect(page.locator(`td:has-text("${updatedName}")`)).toBeVisible();

    // 6. Delete the Policy
    page.on('dialog', dialog => dialog.accept()); // Handle confirm()
    await row.locator('button').nth(1).click(); // Second button is Delete
    
    await expect(page.locator(`td:has-text("${updatedName}")`)).not.toBeVisible();
  });
});
