import { expect } from "@playwright/test";
import { test } from "./worker-orchestrator";

test.describe("Shifts Flow and Navigation", () => {
  test.beforeEach(async ({ authPage }) => {
    test.setTimeout(150000);
    // authPage fixture automatically logs in
    expect(authPage).toBeDefined();
  });

  test("should navigate to shifts list and to the dedicated create shift page", async ({ page }) => {
    // Navigate to /shifts
    await page.goto("/shifts");
    
    // Check that we can see the shifts directory list title
    await expect(page.getByTestId("shifts-title")).toBeVisible({ timeout: 10000 });

    // Find the Create Shift link/button
    const createButton = page.locator('a[href="/shifts/new"]');
    await expect(createButton).toBeVisible();

    // Click it to navigate
    await createButton.click();

    // Verify URL is now /shifts/new
    await expect(page).toHaveURL(/\/shifts\/new/);

    // Verify that the Shift Setup page is rendered
    const basicInfoHeading = page.locator('h3').first();
    await expect(basicInfoHeading).toBeVisible();

    // Fill in shift name
    const nameInput = page.locator('input[type="text"]').first();
    await expect(nameInput).toBeVisible();
    await nameInput.fill("E2E Test Shift Name");

    // Click Cancel to navigate back
    const cancelButton = page.getByTestId("shift-cancel-btn");
    await expect(cancelButton).toBeVisible();
    await cancelButton.click();

    // Verify we are back on the shifts list page
    await expect(page).toHaveURL(/\/shifts/);
  });
});
